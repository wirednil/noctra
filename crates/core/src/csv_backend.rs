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

/// Parsed SQL query representation
#[derive(Debug)]
enum ParsedQuery {
    Select {
        columns: Vec<String>,
        where_clause: Option<String>,
        order_by: Option<Vec<OrderByColumn>>,
        limit: Option<usize>,
        offset: Option<usize>,
    },
    Aggregate {
        function: AggregateFunction,
        column: Option<String>,
        where_clause: Option<String>,
    },
}

/// Aggregate functions
#[derive(Debug)]
enum AggregateFunction {
    Count,
    Sum,
    Avg,
    Min,
    Max,
}

/// ORDER BY column specification
#[derive(Debug)]
struct OrderByColumn {
    column: String,
    direction: OrderDirection,
}

/// Sort direction
#[derive(Debug)]
enum OrderDirection {
    Asc,
    Desc,
}

impl CsvDataSource {
    /// Maximum rows allowed in CSV file (prevents DoS)
    const MAX_ROWS: usize = 1_000_000;

    /// Maximum file size in bytes (100MB)
    const MAX_FILE_SIZE: u64 = 100 * 1024 * 1024;

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

        // Validate file path (sandboxing)
        Self::validate_file_path(&path)?;

        // Check file size
        let metadata = std::fs::metadata(&path)?;
        if metadata.len() > Self::MAX_FILE_SIZE {
            return Err(NoctraError::Internal(format!(
                "CSV file too large: {} bytes (max: {} bytes)",
                metadata.len(),
                Self::MAX_FILE_SIZE
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

        // Read all lines with row limit
        for line in lines {
            let line = line?;
            if line.trim().is_empty() {
                continue;
            }

            // Check row limit
            if all_rows.len() >= Self::MAX_ROWS {
                return Err(NoctraError::Internal(format!(
                    "CSV file exceeds maximum row limit: {} rows",
                    Self::MAX_ROWS
                )));
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

    /// Validate file path to prevent directory traversal attacks
    fn validate_file_path(path: &Path) -> Result<()> {
        // Prevent absolute paths to system directories
        let path_str = path.to_string_lossy();

        // Blocked directories
        let blocked_dirs = [
            "/etc/",
            "/sys/",
            "/proc/",
            "/dev/",
            "/root/",
            "/boot/",
            "C:\\Windows\\",
            "C:\\Program Files\\",
        ];

        for blocked in &blocked_dirs {
            if path_str.starts_with(blocked) {
                return Err(NoctraError::Internal(format!(
                    "Access denied: Cannot read from system directory: {}",
                    path_str
                )));
            }
        }

        // Check for path traversal patterns
        if path_str.contains("..") {
            return Err(NoctraError::Internal(
                "Access denied: Path traversal not allowed".to_string(),
            ));
        }

        // Ensure it's a regular file (not a device or socket)
        if path.exists() {
            let metadata = std::fs::metadata(path)?;
            if !metadata.is_file() {
                return Err(NoctraError::Internal(
                    "Access denied: Path must be a regular file".to_string(),
                ));
            }
        }

        Ok(())
    }

    /// Sanitize column name to prevent SQL injection
    fn sanitize_column_name(name: &str) -> Result<String> {
        // Only allow alphanumeric, underscore, and hyphen
        if name
            .chars()
            .all(|c| c.is_alphanumeric() || c == '_' || c == '-')
        {
            Ok(name.to_string())
        } else {
            Err(NoctraError::Internal(format!(
                "Invalid column name: '{}' (only alphanumeric, _, - allowed)",
                name
            )))
        }
    }

    /// Parse SQL query into components
    fn parse_sql_query(&self, sql: &str) -> Result<ParsedQuery> {
        let sql_upper = sql.to_uppercase();

        // Check for aggregation functions
        if sql_upper.contains("COUNT(")
            || sql_upper.contains("SUM(")
            || sql_upper.contains("AVG(")
            || sql_upper.contains("MIN(")
            || sql_upper.contains("MAX(")
        {
            return self.parse_aggregate_query(sql);
        }

        // Parse SELECT query
        self.parse_select_query(sql)
    }

    /// Parse SELECT query
    fn parse_select_query(&self, sql: &str) -> Result<ParsedQuery> {
        let sql_upper = sql.to_uppercase();

        // Extract column list (for now, only support * or column names)
        let columns = if sql_upper.contains("SELECT *") {
            vec!["*".to_string()]
        } else {
            // Simple column extraction between SELECT and FROM
            let select_idx = sql_upper.find("SELECT ").ok_or_else(|| {
                NoctraError::Internal("Invalid SELECT query".to_string())
            })?;
            let from_idx = sql_upper.find(" FROM ").ok_or_else(|| {
                NoctraError::Internal("Missing FROM clause".to_string())
            })?;

            let cols_str = sql[select_idx + 7..from_idx].trim();
            cols_str
                .split(',')
                .map(|s| s.trim().to_string())
                .collect()
        };

        // Extract WHERE clause
        let where_clause = if let Some(where_idx) = sql_upper.find(" WHERE ") {
            let after_where = sql[where_idx + 7..].trim();
            // Find end of WHERE (before ORDER BY, LIMIT, or end of string)
            let end_idx = after_where
                .to_uppercase()
                .find(" ORDER BY ")
                .or_else(|| after_where.to_uppercase().find(" LIMIT "))
                .unwrap_or(after_where.len());
            Some(after_where[..end_idx].trim().to_string())
        } else {
            None
        };

        // Extract ORDER BY clause
        let order_by = if let Some(order_idx) = sql_upper.find(" ORDER BY ") {
            let after_order = sql[order_idx + 10..].trim();
            let end_idx = after_order
                .to_uppercase()
                .find(" LIMIT ")
                .unwrap_or(after_order.len());
            let order_str = after_order[..end_idx].trim();
            Some(self.parse_order_by(order_str)?)
        } else {
            None
        };

        // Extract LIMIT clause
        let limit = if let Some(limit_idx) = sql_upper.find(" LIMIT ") {
            let after_limit = sql[limit_idx + 7..].trim();
            let end_idx = after_limit
                .to_uppercase()
                .find(" OFFSET ")
                .unwrap_or(after_limit.len());
            let limit_str = after_limit[..end_idx].trim();
            Some(limit_str.parse::<usize>().map_err(|_| {
                NoctraError::Internal("Invalid LIMIT value".to_string())
            })?)
        } else {
            None
        };

        // Extract OFFSET clause
        let offset = if let Some(offset_idx) = sql_upper.find(" OFFSET ") {
            let after_offset = sql[offset_idx + 8..].trim();
            Some(after_offset.parse::<usize>().map_err(|_| {
                NoctraError::Internal("Invalid OFFSET value".to_string())
            })?)
        } else {
            None
        };

        Ok(ParsedQuery::Select {
            columns,
            where_clause,
            order_by,
            limit,
            offset,
        })
    }

    /// Parse ORDER BY clause
    fn parse_order_by(&self, order_str: &str) -> Result<Vec<OrderByColumn>> {
        let parts: Vec<&str> = order_str.split(',').collect();
        let mut order_columns = Vec::new();

        for part in parts {
            let tokens: Vec<&str> = part.trim().split_whitespace().collect();
            if tokens.is_empty() {
                continue;
            }

            let column_name = tokens[0].to_string();
            let direction = if tokens.len() > 1 && tokens[1].to_uppercase() == "DESC" {
                OrderDirection::Desc
            } else {
                OrderDirection::Asc
            };

            order_columns.push(OrderByColumn {
                column: column_name,
                direction,
            });
        }

        Ok(order_columns)
    }

    /// Parse aggregate query
    fn parse_aggregate_query(&self, sql: &str) -> Result<ParsedQuery> {
        let sql_upper = sql.to_uppercase();

        // Extract aggregate function
        let (function, column) = if let Some(count_idx) = sql_upper.find("COUNT(") {
            let after_count = &sql[count_idx + 6..];
            let close_paren = after_count.find(')').ok_or_else(|| {
                NoctraError::Internal("Invalid COUNT syntax".to_string())
            })?;
            let col = after_count[..close_paren].trim();
            let col_name = if col == "*" {
                None
            } else {
                Some(col.to_string())
            };
            (AggregateFunction::Count, col_name)
        } else if let Some(sum_idx) = sql_upper.find("SUM(") {
            let after_sum = &sql[sum_idx + 4..];
            let close_paren = after_sum.find(')').ok_or_else(|| {
                NoctraError::Internal("Invalid SUM syntax".to_string())
            })?;
            let col = after_sum[..close_paren].trim().to_string();
            (AggregateFunction::Sum, Some(col))
        } else if let Some(avg_idx) = sql_upper.find("AVG(") {
            let after_avg = &sql[avg_idx + 4..];
            let close_paren = after_avg.find(')').ok_or_else(|| {
                NoctraError::Internal("Invalid AVG syntax".to_string())
            })?;
            let col = after_avg[..close_paren].trim().to_string();
            (AggregateFunction::Avg, Some(col))
        } else if let Some(min_idx) = sql_upper.find("MIN(") {
            let after_min = &sql[min_idx + 4..];
            let close_paren = after_min.find(')').ok_or_else(|| {
                NoctraError::Internal("Invalid MIN syntax".to_string())
            })?;
            let col = after_min[..close_paren].trim().to_string();
            (AggregateFunction::Min, Some(col))
        } else if let Some(max_idx) = sql_upper.find("MAX(") {
            let after_max = &sql[max_idx + 4..];
            let close_paren = after_max.find(')').ok_or_else(|| {
                NoctraError::Internal("Invalid MAX syntax".to_string())
            })?;
            let col = after_max[..close_paren].trim().to_string();
            (AggregateFunction::Max, Some(col))
        } else {
            return Err(NoctraError::Internal(
                "Unknown aggregate function".to_string(),
            ));
        };

        // Extract WHERE clause
        let where_clause = if let Some(where_idx) = sql_upper.find(" WHERE ") {
            Some(sql[where_idx + 7..].trim().to_string())
        } else {
            None
        };

        Ok(ParsedQuery::Aggregate {
            function,
            column,
            where_clause,
        })
    }

    /// Execute SELECT query
    fn execute_select(
        &self,
        columns: &[String],
        where_clause: Option<String>,
        order_by: Option<Vec<OrderByColumn>>,
        limit: Option<usize>,
        offset: Option<usize>,
    ) -> Result<ResultSet> {
        // Start with all data
        let mut filtered_data: Vec<Vec<Value>> = self.data.clone();

        // Apply WHERE filter
        if let Some(where_expr) = where_clause {
            filtered_data = self.apply_where_filter(&filtered_data, &where_expr)?;
        }

        // Apply ORDER BY
        if let Some(order_cols) = order_by {
            self.apply_order_by(&mut filtered_data, &order_cols)?;
        }

        // Apply OFFSET
        let start_idx = offset.unwrap_or(0);
        if start_idx > 0 {
            filtered_data = filtered_data.into_iter().skip(start_idx).collect();
        }

        // Apply LIMIT
        if let Some(lim) = limit {
            filtered_data.truncate(lim);
        }

        // Build columns
        let result_columns: Vec<Column> = if columns.len() == 1 && columns[0] == "*" {
            self.schema
                .iter()
                .enumerate()
                .map(|(idx, col)| Column {
                    name: col.name.clone(),
                    data_type: col.data_type.clone(),
                    ordinal: idx,
                })
                .collect()
        } else {
            // Specific columns
            columns
                .iter()
                .enumerate()
                .filter_map(|(idx, col_name)| {
                    self.schema.iter().find(|c| &c.name == col_name).map(|col| Column {
                        name: col.name.clone(),
                        data_type: col.data_type.clone(),
                        ordinal: idx,
                    })
                })
                .collect()
        };

        let rows: Vec<Row> = filtered_data
            .into_iter()
            .map(|values| Row { values })
            .collect();

        Ok(ResultSet {
            columns: result_columns,
            rows,
            rows_affected: None,
            last_insert_rowid: None,
        })
    }

    /// Apply WHERE filter to data
    fn apply_where_filter(
        &self,
        data: &[Vec<Value>],
        where_expr: &str,
    ) -> Result<Vec<Vec<Value>>> {
        let mut result = Vec::new();

        for row in data {
            if self.evaluate_where_condition(row, where_expr)? {
                result.push(row.clone());
            }
        }

        Ok(result)
    }

    /// Evaluate WHERE condition for a single row
    fn evaluate_where_condition(&self, row: &[Value], condition: &str) -> Result<bool> {
        // Simple condition parser: column operator value
        // Supports: =, !=, <, >, <=, >=, AND, OR

        let condition = condition.trim();

        // Handle AND/OR
        if let Some(and_pos) = condition.to_uppercase().find(" AND ") {
            let left = &condition[..and_pos];
            let right = &condition[and_pos + 5..];
            return Ok(
                self.evaluate_where_condition(row, left)?
                    && self.evaluate_where_condition(row, right)?,
            );
        }

        if let Some(or_pos) = condition.to_uppercase().find(" OR ") {
            let left = &condition[..or_pos];
            let right = &condition[or_pos + 4..];
            return Ok(
                self.evaluate_where_condition(row, left)?
                    || self.evaluate_where_condition(row, right)?,
            );
        }

        // Parse simple condition
        let operators = [">=", "<=", "!=", "=", ">", "<"];
        for op in &operators {
            if let Some(op_pos) = condition.find(op) {
                let column_name = condition[..op_pos].trim();
                let value_str = condition[op_pos + op.len()..].trim();

                // Find column index
                let col_idx = self
                    .schema
                    .iter()
                    .position(|c| c.name == column_name)
                    .ok_or_else(|| {
                        NoctraError::Internal(format!("Column not found: {}", column_name))
                    })?;

                let row_value = &row[col_idx];
                let compare_value = self.parse_value(value_str, &self.schema[col_idx].data_type);

                return self.compare_values(row_value, &compare_value, op);
            }
        }

        Err(NoctraError::Internal(format!(
            "Invalid WHERE condition: {}",
            condition
        )))
    }

    /// Parse string value to typed Value
    fn parse_value(&self, value_str: &str, data_type: &str) -> Value {
        let cleaned = value_str.trim_matches(|c| c == '\'' || c == '"');

        match data_type {
            "INTEGER" => cleaned
                .parse::<i64>()
                .map(Value::Integer)
                .unwrap_or_else(|_| Value::Text(cleaned.to_string())),
            "REAL" => cleaned
                .parse::<f64>()
                .map(Value::Float)
                .unwrap_or_else(|_| Value::Text(cleaned.to_string())),
            "BOOLEAN" => {
                let lower = cleaned.to_lowercase();
                let bool_val = matches!(lower.as_str(), "true" | "t" | "1" | "yes");
                Value::Boolean(bool_val)
            }
            _ => Value::Text(cleaned.to_string()),
        }
    }

    /// Compare two values with an operator
    fn compare_values(&self, left: &Value, right: &Value, op: &str) -> Result<bool> {
        match (left, right) {
            (Value::Integer(l), Value::Integer(r)) => Ok(match op {
                "=" => l == r,
                "!=" => l != r,
                "<" => l < r,
                ">" => l > r,
                "<=" => l <= r,
                ">=" => l >= r,
                _ => false,
            }),
            (Value::Float(l), Value::Float(r)) => Ok(match op {
                "=" => (l - r).abs() < f64::EPSILON,
                "!=" => (l - r).abs() >= f64::EPSILON,
                "<" => l < r,
                ">" => l > r,
                "<=" => l <= r,
                ">=" => l >= r,
                _ => false,
            }),
            (Value::Text(l), Value::Text(r)) => Ok(match op {
                "=" => l == r,
                "!=" => l != r,
                "<" => l < r,
                ">" => l > r,
                "<=" => l <= r,
                ">=" => l >= r,
                _ => false,
            }),
            (Value::Boolean(l), Value::Boolean(r)) => Ok(match op {
                "=" => l == r,
                "!=" => l != r,
                _ => false,
            }),
            _ => Ok(false),
        }
    }

    /// Apply ORDER BY to data
    fn apply_order_by(
        &self,
        data: &mut [Vec<Value>],
        order_columns: &[OrderByColumn],
    ) -> Result<()> {
        if order_columns.is_empty() {
            return Ok(());
        }

        // Get column indices
        let col_indices: Vec<usize> = order_columns
            .iter()
            .map(|order_col| {
                self.schema
                    .iter()
                    .position(|c| c.name == order_col.column)
                    .ok_or_else(|| {
                        NoctraError::Internal(format!("Column not found: {}", order_col.column))
                    })
            })
            .collect::<Result<Vec<_>>>()?;

        data.sort_by(|a, b| {
            for (idx, order_col) in order_columns.iter().enumerate() {
                let col_idx = col_indices[idx];
                let cmp = self.compare_values_for_sort(&a[col_idx], &b[col_idx]);
                let cmp = match order_col.direction {
                    OrderDirection::Asc => cmp,
                    OrderDirection::Desc => cmp.reverse(),
                };
                if cmp != std::cmp::Ordering::Equal {
                    return cmp;
                }
            }
            std::cmp::Ordering::Equal
        });

        Ok(())
    }

    /// Compare values for sorting
    fn compare_values_for_sort(&self, left: &Value, right: &Value) -> std::cmp::Ordering {
        match (left, right) {
            (Value::Integer(l), Value::Integer(r)) => l.cmp(r),
            (Value::Float(l), Value::Float(r)) => {
                l.partial_cmp(r).unwrap_or(std::cmp::Ordering::Equal)
            }
            (Value::Text(l), Value::Text(r)) => l.cmp(r),
            (Value::Boolean(l), Value::Boolean(r)) => l.cmp(r),
            (Value::Null, Value::Null) => std::cmp::Ordering::Equal,
            (Value::Null, _) => std::cmp::Ordering::Less,
            (_, Value::Null) => std::cmp::Ordering::Greater,
            _ => std::cmp::Ordering::Equal,
        }
    }

    /// Execute aggregate query
    fn execute_aggregate(
        &self,
        function: &AggregateFunction,
        column: Option<&str>,
        where_clause: Option<String>,
    ) -> Result<ResultSet> {
        // Apply WHERE filter if present
        let mut data = self.data.clone();
        if let Some(where_expr) = where_clause {
            data = self.apply_where_filter(&data, &where_expr)?;
        }

        // Calculate aggregate
        let result_value = match function {
            AggregateFunction::Count => Value::Integer(data.len() as i64),
            AggregateFunction::Sum => {
                let col_name = column.ok_or_else(|| {
                    NoctraError::Internal("SUM requires a column name".to_string())
                })?;
                let col_idx = self
                    .schema
                    .iter()
                    .position(|c| c.name == col_name)
                    .ok_or_else(|| {
                        NoctraError::Internal(format!("Column not found: {}", col_name))
                    })?;

                let sum: f64 = data
                    .iter()
                    .filter_map(|row| match &row[col_idx] {
                        Value::Integer(i) => Some(*i as f64),
                        Value::Float(f) => Some(*f),
                        _ => None,
                    })
                    .sum();

                Value::Float(sum)
            }
            AggregateFunction::Avg => {
                let col_name = column.ok_or_else(|| {
                    NoctraError::Internal("AVG requires a column name".to_string())
                })?;
                let col_idx = self
                    .schema
                    .iter()
                    .position(|c| c.name == col_name)
                    .ok_or_else(|| {
                        NoctraError::Internal(format!("Column not found: {}", col_name))
                    })?;

                let values: Vec<f64> = data
                    .iter()
                    .filter_map(|row| match &row[col_idx] {
                        Value::Integer(i) => Some(*i as f64),
                        Value::Float(f) => Some(*f),
                        _ => None,
                    })
                    .collect();

                if values.is_empty() {
                    Value::Null
                } else {
                    let sum: f64 = values.iter().sum();
                    Value::Float(sum / values.len() as f64)
                }
            }
            AggregateFunction::Min => {
                let col_name = column.ok_or_else(|| {
                    NoctraError::Internal("MIN requires a column name".to_string())
                })?;
                let col_idx = self
                    .schema
                    .iter()
                    .position(|c| c.name == col_name)
                    .ok_or_else(|| {
                        NoctraError::Internal(format!("Column not found: {}", col_name))
                    })?;

                let min_val = data
                    .iter()
                    .filter_map(|row| match &row[col_idx] {
                        Value::Integer(i) => Some(*i as f64),
                        Value::Float(f) => Some(*f),
                        _ => None,
                    })
                    .min_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

                min_val.map(Value::Float).unwrap_or(Value::Null)
            }
            AggregateFunction::Max => {
                let col_name = column.ok_or_else(|| {
                    NoctraError::Internal("MAX requires a column name".to_string())
                })?;
                let col_idx = self
                    .schema
                    .iter()
                    .position(|c| c.name == col_name)
                    .ok_or_else(|| {
                        NoctraError::Internal(format!("Column not found: {}", col_name))
                    })?;

                let max_val = data
                    .iter()
                    .filter_map(|row| match &row[col_idx] {
                        Value::Integer(i) => Some(*i as f64),
                        Value::Float(f) => Some(*f),
                        _ => None,
                    })
                    .max_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

                max_val.map(Value::Float).unwrap_or(Value::Null)
            }
        };

        // Build result
        let function_name = match function {
            AggregateFunction::Count => "COUNT",
            AggregateFunction::Sum => "SUM",
            AggregateFunction::Avg => "AVG",
            AggregateFunction::Min => "MIN",
            AggregateFunction::Max => "MAX",
        };

        let column_name = if let Some(col) = column {
            format!("{}({})", function_name, col)
        } else {
            format!("{}(*)", function_name)
        };

        Ok(ResultSet {
            columns: vec![Column {
                name: column_name,
                data_type: "REAL".to_string(),
                ordinal: 0,
            }],
            rows: vec![Row {
                values: vec![result_value],
            }],
            rows_affected: None,
            last_insert_rowid: None,
        })
    }
}

impl DataSource for CsvDataSource {
    fn query(&self, sql: &str, _parameters: &Parameters) -> Result<ResultSet> {
        eprintln!("[DEBUG CSV] Query received: {}", sql);
        eprintln!("[DEBUG CSV] Table name: {}", self.table_name());
        eprintln!("[DEBUG CSV] Schema: {} columns", self.schema.len());

        // Parse the SQL query
        let parsed_query = self.parse_sql_query(sql)?;
        eprintln!("[DEBUG CSV] Parsed query: {:?}", parsed_query);

        // Execute based on query type
        match parsed_query {
            ParsedQuery::Select {
                columns,
                where_clause,
                order_by,
                limit,
                offset,
            } => {
                self.execute_select(&columns, where_clause, order_by, limit, offset)
            }
            ParsedQuery::Aggregate {
                function,
                column,
                where_clause,
            } => {
                self.execute_aggregate(&function, column.as_deref(), where_clause)
            }
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
