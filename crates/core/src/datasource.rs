//! DataSource abstraction for multi-source support (NQL)
//!
//! This module provides the abstraction layer for working with multiple
//! data sources (SQLite, CSV, JSON, Memory) using a unified interface.

use crate::error::{NoctraError, Result};
use crate::types::{Parameters, ResultSet};
use std::fmt::Debug;
use std::path::PathBuf;

/// Trait for data sources in NQL
///
/// This trait abstracts different data sources (databases, files, memory)
/// to provide a unified query interface.
pub trait DataSource: Send + Sync + Debug {
    /// Execute a query against the data source
    fn query(&self, sql: &str, parameters: &Parameters) -> Result<ResultSet>;

    /// Get schema information (tables/columns)
    fn schema(&self) -> Result<Vec<TableInfo>>;

    /// Get the type of this data source
    fn source_type(&self) -> SourceType;

    /// Get the name/identifier of this source
    fn name(&self) -> &str;

    /// Get metadata about this source
    fn metadata(&self) -> SourceMetadata {
        SourceMetadata {
            name: self.name().to_string(),
            source_type: self.source_type(),
            tables: self.schema().unwrap_or_default(),
        }
    }

    /// Close the data source (optional)
    fn close(&mut self) -> Result<()> {
        Ok(())
    }
}

/// Type of data source
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SourceType {
    /// SQLite database
    SQLite {
        /// Path to database file (":memory:" for in-memory)
        path: String,
    },

    /// CSV file
    CSV {
        /// Path to CSV file
        path: String,
        /// Delimiter character
        delimiter: char,
        /// Whether the file has a header row
        has_header: bool,
        /// Encoding (e.g., "utf-8", "latin1")
        encoding: String,
    },

    /// JSON file
    JSON {
        /// Path to JSON file
        path: String,
    },

    /// In-memory dataset
    Memory {
        /// Capacity hint for the dataset
        capacity: usize,
    },
}

impl SourceType {
    /// Get a human-readable name for the source type
    pub fn type_name(&self) -> &str {
        match self {
            SourceType::SQLite { .. } => "sqlite",
            SourceType::CSV { .. } => "csv",
            SourceType::JSON { .. } => "json",
            SourceType::Memory { .. } => "memory",
        }
    }

    /// Get the display path/identifier
    pub fn display_path(&self) -> String {
        match self {
            SourceType::SQLite { path } => path.clone(),
            SourceType::CSV { path, .. } => path.clone(),
            SourceType::JSON { path } => path.clone(),
            SourceType::Memory { .. } => "(in-memory)".to_string(),
        }
    }
}

/// Information about a table in a data source
#[derive(Debug, Clone)]
pub struct TableInfo {
    /// Table name
    pub name: String,
    /// Columns in the table
    pub columns: Vec<ColumnInfo>,
    /// Number of rows (if known)
    pub row_count: Option<usize>,
}

/// Information about a column
#[derive(Debug, Clone)]
pub struct ColumnInfo {
    /// Column name
    pub name: String,
    /// Column type (e.g., "TEXT", "INTEGER", "REAL")
    pub data_type: String,
    /// Whether the column can be null
    pub nullable: bool,
    /// Default value (if any)
    pub default_value: Option<String>,
}

/// Metadata about a data source
#[derive(Debug, Clone)]
pub struct SourceMetadata {
    /// Name/alias of the source
    pub name: String,
    /// Type of source
    pub source_type: SourceType,
    /// Tables in the source
    pub tables: Vec<TableInfo>,
}

/// Options for CSV data sources
#[derive(Debug, Clone)]
pub struct CsvOptions {
    /// Delimiter character (None = auto-detect)
    pub delimiter: Option<char>,
    /// Whether the file has a header row
    pub has_header: bool,
    /// Encoding (None = auto-detect)
    pub encoding: Option<String>,
    /// Quote character
    pub quote: char,
    /// Skip N rows at the beginning
    pub skip_rows: usize,
}

impl Default for CsvOptions {
    fn default() -> Self {
        Self {
            delimiter: None, // Auto-detect
            has_header: true,
            encoding: None, // Auto-detect
            quote: '"',
            skip_rows: 0,
        }
    }
}

/// Registry of named data sources
#[derive(Debug, Default)]
pub struct SourceRegistry {
    sources: std::collections::HashMap<String, Box<dyn DataSource>>,
    active_source: Option<String>,
}

impl SourceRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a new data source with an alias
    pub fn register(&mut self, alias: String, source: Box<dyn DataSource>) -> Result<()> {
        // If this is the first source, make it active
        if self.active_source.is_none() {
            self.active_source = Some(alias.clone());
        }

        self.sources.insert(alias, source);
        Ok(())
    }

    /// Get a data source by alias
    pub fn get(&self, alias: &str) -> Option<&dyn DataSource> {
        self.sources.get(alias).map(|s| s.as_ref())
    }

    /// Get a mutable reference to a data source
    pub fn get_mut(&mut self, alias: &str) -> Option<&mut (dyn DataSource + '_)> {
        match self.sources.get_mut(alias) {
            Some(source) => Some(source.as_mut()),
            None => None,
        }
    }

    /// Get the active data source
    pub fn active(&self) -> Option<&dyn DataSource> {
        self.active_source
            .as_ref()
            .and_then(|alias| self.get(alias))
    }

    /// Set the active data source
    pub fn set_active(&mut self, alias: &str) -> Result<()> {
        if !self.sources.contains_key(alias) {
            return Err(NoctraError::Internal(format!(
                "Data source '{}' not found",
                alias
            )));
        }
        self.active_source = Some(alias.to_string());
        Ok(())
    }

    /// List all registered sources
    pub fn list_sources(&self) -> Vec<(String, SourceType)> {
        self.sources
            .iter()
            .map(|(alias, source)| (alias.clone(), source.source_type()))
            .collect()
    }

    /// Remove a data source
    pub fn remove(&mut self, alias: &str) -> Result<()> {
        self.sources
            .remove(alias)
            .ok_or_else(|| NoctraError::Internal(format!("Data source '{}' not found", alias)))?;

        // If we removed the active source, clear it
        if self.active_source.as_deref() == Some(alias) {
            self.active_source = None;
            // Set a new active source if any remain
            if let Some(first) = self.sources.keys().next() {
                self.active_source = Some(first.clone());
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_source_type_display() {
        let sqlite = SourceType::SQLite {
            path: "test.db".to_string(),
        };
        assert_eq!(sqlite.type_name(), "sqlite");
        assert_eq!(sqlite.display_path(), "test.db");

        let csv = SourceType::CSV {
            path: "data.csv".to_string(),
            delimiter: ',',
            has_header: true,
            encoding: "utf-8".to_string(),
        };
        assert_eq!(csv.type_name(), "csv");
        assert_eq!(csv.display_path(), "data.csv");
    }

    #[test]
    fn test_csv_options_default() {
        let opts = CsvOptions::default();
        assert!(opts.delimiter.is_none());
        assert!(opts.has_header);
        assert_eq!(opts.quote, '"');
        assert_eq!(opts.skip_rows, 0);
    }
}
