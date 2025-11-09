//! CSV Backend for NQL
//!
//! Provides DataSource implementation for CSV files with automatic
//! delimiter detection and type inference.

use crate::datasource::{ColumnInfo, CsvOptions, DataSource, SourceType, TableInfo};
use crate::error::{NoctraError, Result};
use crate::types::{Column, Parameters, ResultSet, Row, Value};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};

/// CSV Data Source
#[derive(Debug)]
pub struct CsvDataSource {
    /// Path to the CSV file
    path: PathBuf,

    /// Name/alias for this source
    name: String,

    /// CSV options
    options: CsvOptions,

    /// Parsed schema (columns)
    schema: Vec<ColumnInfo>,

    /// Cached data rows
    data: Vec<Vec<Value>>,
}

impl CsvDataSource {
    /// Create a new CSV data source
    pub fn new<P: AsRef<Path>>(path: P, name: String, options: CsvOptions) -> Result<Self> {
        let path = path.as_ref().to_path_buf();

        // Verify file exists
        if !path.exists() {
            return Err(NoctraError::Internal(format!(
                "CSV file not found: {}",
                path.display()
            )));
        }

        // Auto-detect delimiter if not specified
        let delimiter = if let Some(d) = options.delimiter {
            d
        } else {
            Self::detect_delimiter(&path)?
        };

        // Create options with detected delimiter
        let mut opts = options.clone();
        opts.delimiter = Some(delimiter);

        // Parse the CSV file
        let (schema, data) = Self::parse_csv(&path, &opts)?;

        Ok(Self {
            path,
            name,
            options: opts,
            schema,
            data,
        })
    }

    /// Detect the delimiter by sampling the first few rows
    fn detect_delimiter(path: &Path) -> Result<char> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);

        let mut lines: Vec<String> = reader
            .lines()
            .take(5)
            .collect::<std::io::Result<Vec<_>>>()?;

        if lines.is_empty() {
            return Err(NoctraError::Internal("Empty CSV file".to_string()));
        }

        // Common delimiters to test
        let delimiters = [',', ';', '\t', '|'];
        let mut delimiter_counts: HashMap<char, Vec<usize>> = HashMap::new();

        for delim in &delimiters {
            let counts: Vec<usize> = lines
                .iter()
                .map(|line| line.matches(*delim).count())
                .collect();
            delimiter_counts.insert(*delim, counts);
        }

        // Find the delimiter with the most consistent count across lines
        let best_delimiter = delimiters
            .iter()
            .max_by_key(|delim| {
                let counts = &delimiter_counts[delim];
                if counts.is_empty() {
                    return 0;
                }

                // Check if counts are consistent (same count on each line)
                let first = counts[0];
                if first == 0 {
                    return 0;
                }

                let all_same = counts.iter().all(|&c| c == first);
                if all_same {
                    first
                } else {
                    0
                }
            })
            .copied()
            .unwrap_or(',');

        Ok(best_delimiter)
    }

    /// Parse the CSV file and infer types
    fn parse_csv(path: &Path, options: &CsvOptions) -> Result<(Vec<ColumnInfo>, Vec<Vec<Value>>)> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let delimiter = options.delimiter.unwrap_or(',');

        let mut lines = reader.lines();
        let mut all_rows: Vec<Vec<String>> = Vec::new();

        // Skip rows if specified
        for _ in 0..options.skip_rows {
            lines.next();
        }

        // Read all lines
        for line in lines {
            let line = line?;
            if line.trim().is_empty() {
                continue;
            }

            let fields: Vec<String> = Self::parse_csv_line(&line, delimiter, options.quote);
            all_rows.push(fields);
        }

        if all_rows.is_empty() {
            return Err(NoctraError::Internal("No data in CSV file".to_string()));
        }

        // Determine column names
        let column_names = if options.has_header {
            let header = all_rows.remove(0);
            header
        } else {
            // Generate column names: col1, col2, ...
            (0..all_rows[0].len())
                .map(|i| format!("col{}", i + 1))
                .collect()
        };

        // Infer column types from data
        let schema = Self::infer_schema(&column_names, &all_rows);

        // Convert string data to typed values
        let data = all_rows
            .into_iter()
            .map(|row| Self::convert_row(&row, &schema))
            .collect();

        Ok((schema, data))
    }

    /// Parse a single CSV line considering quotes
    fn parse_csv_line(line: &str, delimiter: char, quote: char) -> Vec<String> {
        let mut fields = Vec::new();
        let mut current_field = String::new();
        let mut in_quotes = false;
        let mut chars = line.chars().peekable();

        while let Some(ch) = chars.next() {
            if ch == quote {
                in_quotes = !in_quotes;
            } else if ch == delimiter && !in_quotes {
                fields.push(current_field.trim().to_string());
                current_field.clear();
            } else {
                current_field.push(ch);
            }
        }

        // Don't forget the last field
        fields.push(current_field.trim().to_string());

        fields
    }

    /// Infer column types from data samples
    fn infer_schema(column_names: &[String], data: &[Vec<String>]) -> Vec<ColumnInfo> {
        column_names
            .iter()
            .enumerate()
            .map(|(idx, name)| {
                let data_type = Self::infer_column_type(data, idx);
                ColumnInfo {
                    name: name.clone(),
                    data_type,
                    nullable: true, // CSV columns can always be null
                    default_value: None,
                }
            })
            .collect()
    }

    /// Infer type for a single column
    fn infer_column_type(data: &[Vec<String>], col_idx: usize) -> String {
        // Sample up to 100 rows
        let sample_size = data.len().min(100);

        let mut all_integers = true;
        let mut all_floats = true;
        let mut all_booleans = true;

        for row in data.iter().take(sample_size) {
            if let Some(value) = row.get(col_idx) {
                if value.is_empty() {
                    continue; // Skip empty values
                }

                // Try integer
                if value.parse::<i64>().is_err() {
                    all_integers = false;
                }

                // Try float
                if value.parse::<f64>().is_err() {
                    all_floats = false;
                }

                // Try boolean
                let lower = value.to_lowercase();
                if !matches!(lower.as_str(), "true" | "false" | "t" | "f" | "1" | "0" | "yes" | "no") {
                    all_booleans = false;
                }
            }
        }

        if all_booleans {
            "BOOLEAN".to_string()
        } else if all_integers {
            "INTEGER".to_string()
        } else if all_floats {
            "REAL".to_string()
        } else {
            "TEXT".to_string()
        }
    }

    /// Convert a string row to typed values
    fn convert_row(row: &[String], schema: &[ColumnInfo]) -> Vec<Value> {
        row.iter()
            .enumerate()
            .map(|(idx, value_str)| {
                if value_str.is_empty() {
                    return Value::Null;
                }

                let col_type = schema.get(idx).map(|c| c.data_type.as_str()).unwrap_or("TEXT");

                match col_type {
                    "INTEGER" => value_str
                        .parse::<i64>()
                        .map(Value::Integer)
                        .unwrap_or_else(|_| Value::Text(value_str.clone())),
                    "REAL" => value_str
                        .parse::<f64>()
                        .map(Value::Float)
                        .unwrap_or_else(|_| Value::Text(value_str.clone())),
                    "BOOLEAN" => {
                        let lower = value_str.to_lowercase();
                        let bool_val = matches!(lower.as_str(), "true" | "t" | "1" | "yes");
                        Value::Boolean(bool_val)
                    }
                    _ => Value::Text(value_str.clone()),
                }
            })
            .collect()
    }

    /// Get the table name (derived from filename)
    fn table_name(&self) -> String {
        self.path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("csv_table")
            .to_string()
    }
}

impl DataSource for CsvDataSource {
    fn query(&self, sql: &str, _parameters: &Parameters) -> Result<ResultSet> {
        // For MVP, we'll do simple filtering in memory
        // Full SQL support would require embedding SQLite or similar

        // For now, just return all data if it's a SELECT *
        if sql.trim().to_uppercase().starts_with("SELECT * FROM") {
            let columns: Vec<Column> = self.schema
                .iter()
                .enumerate()
                .map(|(idx, col)| Column {
                    name: col.name.clone(),
                    data_type: col.data_type.clone(),
                    ordinal: idx,
                })
                .collect();

            let rows: Vec<Row> = self.data
                .iter()
                .map(|values| Row {
                    values: values.clone(),
                })
                .collect();

            Ok(ResultSet {
                columns,
                rows,
                rows_affected: None,
                last_insert_rowid: None,
            })
        } else {
            Err(NoctraError::Internal(
                "CSV source only supports 'SELECT * FROM <table>' queries for now".to_string()
            ))
        }
    }

    fn schema(&self) -> Result<Vec<TableInfo>> {
        Ok(vec![TableInfo {
            name: self.table_name(),
            columns: self.schema.clone(),
            row_count: Some(self.data.len()),
        }])
    }

    fn source_type(&self) -> SourceType {
        SourceType::CSV {
            path: self.path.display().to_string(),
            delimiter: self.options.delimiter.unwrap_or(','),
            has_header: self.options.has_header,
            encoding: self.options.encoding.clone().unwrap_or_else(|| "utf-8".to_string()),
        }
    }

    fn name(&self) -> &str {
        &self.name
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_delimiter_detection() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "name,age,city").unwrap();
        writeln!(file, "Alice,30,NYC").unwrap();
        writeln!(file, "Bob,25,LA").unwrap();
        file.flush().unwrap();

        let delimiter = CsvDataSource::detect_delimiter(file.path()).unwrap();
        assert_eq!(delimiter, ',');
    }

    #[test]
    fn test_parse_csv_basic() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "name,age").unwrap();
        writeln!(file, "Alice,30").unwrap();
        writeln!(file, "Bob,25").unwrap();
        file.flush().unwrap();

        let source = CsvDataSource::new(
            file.path(),
            "test".to_string(),
            CsvOptions::default(),
        )
        .unwrap();

        assert_eq!(source.schema.len(), 2);
        assert_eq!(source.schema[0].name, "name");
        assert_eq!(source.schema[1].name, "age");
        assert_eq!(source.data.len(), 2);
    }

    #[test]
    fn test_type_inference() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "id,price,active").unwrap();
        writeln!(file, "1,19.99,true").unwrap();
        writeln!(file, "2,29.99,false").unwrap();
        file.flush().unwrap();

        let source = CsvDataSource::new(
            file.path(),
            "test".to_string(),
            CsvOptions::default(),
        )
        .unwrap();

        assert_eq!(source.schema[0].data_type, "INTEGER");
        assert_eq!(source.schema[1].data_type, "REAL");
        assert_eq!(source.schema[2].data_type, "BOOLEAN");
    }
}
