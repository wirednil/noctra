//! RQL (Relational Query Language) Parser
//!
//! Extends standard SQL with data source management commands:
//! - `USE 'file.ext' AS alias` - Register file as queryable source
//! - `ATTACH 'db.sqlite' AS alias (TYPE sqlite)` - Attach database
//! - `SHOW SOURCES` - List registered sources
//! - `DETACH alias` - Unregister source
//!
//! ## Example
//!
//! ```rust
//! use noctra_duckdb::parser::{RqlParser, Statement, SourceType};
//!
//! // Parse USE command
//! let stmt = RqlParser::parse("USE 'data.csv' AS sales").unwrap();
//! match stmt {
//!     Statement::Use { source, alias, source_type } => {
//!         assert_eq!(source, "data.csv");
//!         assert_eq!(alias, "sales");
//!         assert_eq!(source_type, SourceType::CSV);
//!     }
//!     _ => panic!("Expected USE statement"),
//! }
//! ```

use crate::error::{DuckDBError, Result};

/// SQL statement type (RQL extensions + standard SQL)
#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    /// USE 'file.ext' AS alias
    ///
    /// Registers a file as a queryable data source.
    /// The file type is auto-detected from the extension.
    Use {
        source: String,
        alias: String,
        source_type: SourceType,
    },

    /// ATTACH 'db.path' AS alias (TYPE db_type)
    ///
    /// Attaches an external database for cross-source queries.
    Attach {
        path: String,
        alias: String,
        db_type: String,
    },

    /// DETACH alias
    ///
    /// Unregisters a previously attached source.
    Detach { alias: String },

    /// SHOW SOURCES
    ///
    /// Lists all registered data sources.
    ShowSources,

    /// Standard SQL query (SELECT, INSERT, UPDATE, DELETE, etc.)
    ///
    /// Passed through to the query engine without parsing.
    Query(String),
}

/// Data source type detected from file extension
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SourceType {
    /// CSV file (.csv)
    CSV,
    /// JSON file (.json, .jsonl, .ndjson)
    JSON,
    /// Parquet file (.parquet)
    Parquet,
    /// SQLite database (.db, .sqlite, .sqlite3)
    SQLite,
    /// Auto-detect from extension
    Auto,
}

impl SourceType {
    /// Detect source type from file extension
    pub fn from_extension(ext: &str) -> Self {
        match ext.to_lowercase().as_str() {
            "csv" => SourceType::CSV,
            "json" | "jsonl" | "ndjson" => SourceType::JSON,
            "parquet" => SourceType::Parquet,
            "db" | "sqlite" | "sqlite3" => SourceType::SQLite,
            _ => SourceType::Auto,
        }
    }

    /// Detect source type from full file path
    pub fn from_path(path: &str) -> Self {
        std::path::Path::new(path)
            .extension()
            .and_then(|e| e.to_str())
            .map(Self::from_extension)
            .unwrap_or(SourceType::Auto)
    }
}

/// Simple RQL parser for data source management commands
pub struct RqlParser;

impl RqlParser {
    /// Parse a SQL string into a Statement
    ///
    /// This parser recognizes RQL extensions (USE, ATTACH, etc.) and standard SQL.
    /// If the statement doesn't match any RQL command, it's treated as standard SQL.
    ///
    /// # Example
    ///
    /// ```rust
    /// use noctra_duckdb::parser::{RqlParser, Statement};
    ///
    /// // RQL command
    /// let stmt = RqlParser::parse("USE 'sales.csv' AS sales").unwrap();
    /// assert!(matches!(stmt, Statement::Use { .. }));
    ///
    /// // Standard SQL
    /// let stmt = RqlParser::parse("SELECT * FROM sales").unwrap();
    /// assert!(matches!(stmt, Statement::Query(_)));
    /// ```
    pub fn parse(sql: &str) -> Result<Statement> {
        let sql_upper = sql.trim().to_uppercase();

        // USE command
        if sql_upper.starts_with("USE") {
            return Self::parse_use(sql);
        }

        // ATTACH command
        if sql_upper.starts_with("ATTACH") {
            return Self::parse_attach(sql);
        }

        // DETACH command
        if sql_upper.starts_with("DETACH") {
            return Self::parse_detach(sql);
        }

        // SHOW SOURCES
        if sql_upper.starts_with("SHOW SOURCES") || sql_upper.starts_with("SHOW SOURCE") {
            return Ok(Statement::ShowSources);
        }

        // Default: Standard SQL query
        Ok(Statement::Query(sql.to_string()))
    }

    /// Parse USE 'file.ext' AS alias
    fn parse_use(sql: &str) -> Result<Statement> {
        // Pattern: USE 'path' AS alias
        // Support both single and double quotes

        let pattern = regex::Regex::new(r#"(?i)USE\s+['"]([^'"]+)['"]\s+AS\s+(\w+)"#)
            .map_err(|e| DuckDBError::QueryFailed(format!("Regex error: {}", e)))?;

        if let Some(caps) = pattern.captures(sql) {
            let source = caps.get(1).unwrap().as_str().to_string();
            let alias = caps.get(2).unwrap().as_str().to_string();

            // Detect source type from file extension
            let source_type = SourceType::from_path(&source);

            Ok(Statement::Use {
                source,
                alias,
                source_type,
            })
        } else {
            Err(DuckDBError::QueryFailed(
                "Invalid USE syntax. Expected: USE 'file.ext' AS alias".to_string(),
            ))
        }
    }

    /// Parse ATTACH 'db.path' AS alias (TYPE db_type)
    fn parse_attach(sql: &str) -> Result<Statement> {
        // Pattern: ATTACH 'path' AS alias (TYPE type_name)
        // TYPE clause is optional, defaults to 'sqlite'

        let pattern =
            regex::Regex::new(r#"(?i)ATTACH\s+['"]([^'"]+)['"]\s+AS\s+(\w+)(?:\s+\(TYPE\s+(\w+)\))?"#)
                .map_err(|e| DuckDBError::QueryFailed(format!("Regex error: {}", e)))?;

        if let Some(caps) = pattern.captures(sql) {
            let path = caps.get(1).unwrap().as_str().to_string();
            let alias = caps.get(2).unwrap().as_str().to_string();
            let db_type = caps
                .get(3)
                .map(|m| m.as_str().to_string())
                .unwrap_or_else(|| "sqlite".to_string());

            Ok(Statement::Attach { path, alias, db_type })
        } else {
            Err(DuckDBError::QueryFailed(
                "Invalid ATTACH syntax. Expected: ATTACH 'path' AS alias (TYPE type)".to_string(),
            ))
        }
    }

    /// Parse DETACH alias
    fn parse_detach(sql: &str) -> Result<Statement> {
        let pattern = regex::Regex::new(r"(?i)DETACH\s+(\w+)")
            .map_err(|e| DuckDBError::QueryFailed(format!("Regex error: {}", e)))?;

        if let Some(caps) = pattern.captures(sql) {
            let alias = caps.get(1).unwrap().as_str().to_string();
            Ok(Statement::Detach { alias })
        } else {
            Err(DuckDBError::QueryFailed(
                "Invalid DETACH syntax. Expected: DETACH alias".to_string(),
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_use_csv() {
        let stmt = RqlParser::parse("USE 'data.csv' AS sales").unwrap();
        assert_eq!(
            stmt,
            Statement::Use {
                source: "data.csv".to_string(),
                alias: "sales".to_string(),
                source_type: SourceType::CSV,
            }
        );
    }

    #[test]
    fn test_parse_use_json() {
        let stmt = RqlParser::parse("USE 'events.json' AS events").unwrap();
        match stmt {
            Statement::Use {
                source,
                alias,
                source_type,
            } => {
                assert_eq!(source, "events.json");
                assert_eq!(alias, "events");
                assert_eq!(source_type, SourceType::JSON);
            }
            _ => panic!("Expected USE statement"),
        }
    }

    #[test]
    fn test_parse_use_parquet() {
        let stmt = RqlParser::parse("USE \"large_data.parquet\" AS data").unwrap();
        match stmt {
            Statement::Use { source_type, .. } => {
                assert_eq!(source_type, SourceType::Parquet);
            }
            _ => panic!("Expected USE statement"),
        }
    }

    #[test]
    fn test_parse_use_case_insensitive() {
        let stmt = RqlParser::parse("use 'file.csv' as my_table").unwrap();
        assert!(matches!(stmt, Statement::Use { .. }));
    }

    #[test]
    fn test_parse_attach_with_type() {
        let stmt = RqlParser::parse("ATTACH 'warehouse.db' AS wh (TYPE sqlite)").unwrap();
        assert_eq!(
            stmt,
            Statement::Attach {
                path: "warehouse.db".to_string(),
                alias: "wh".to_string(),
                db_type: "sqlite".to_string(),
            }
        );
    }

    #[test]
    fn test_parse_attach_without_type() {
        let stmt = RqlParser::parse("ATTACH 'data.db' AS mydb").unwrap();
        match stmt {
            Statement::Attach { db_type, .. } => {
                assert_eq!(db_type, "sqlite"); // Default
            }
            _ => panic!("Expected ATTACH statement"),
        }
    }

    #[test]
    fn test_parse_detach() {
        let stmt = RqlParser::parse("DETACH sales").unwrap();
        assert_eq!(
            stmt,
            Statement::Detach {
                alias: "sales".to_string(),
            }
        );
    }

    #[test]
    fn test_parse_show_sources() {
        let stmt = RqlParser::parse("SHOW SOURCES").unwrap();
        assert_eq!(stmt, Statement::ShowSources);
    }

    #[test]
    fn test_parse_standard_sql() {
        let sql = "SELECT * FROM sales WHERE amount > 100";
        let stmt = RqlParser::parse(sql).unwrap();
        assert_eq!(stmt, Statement::Query(sql.to_string()));
    }

    #[test]
    fn test_source_type_from_extension() {
        assert_eq!(SourceType::from_extension("csv"), SourceType::CSV);
        assert_eq!(SourceType::from_extension("CSV"), SourceType::CSV);
        assert_eq!(SourceType::from_extension("json"), SourceType::JSON);
        assert_eq!(SourceType::from_extension("parquet"), SourceType::Parquet);
        assert_eq!(SourceType::from_extension("sqlite"), SourceType::SQLite);
        assert_eq!(SourceType::from_extension("unknown"), SourceType::Auto);
    }

    #[test]
    fn test_source_type_from_path() {
        assert_eq!(
            SourceType::from_path("/path/to/data.csv"),
            SourceType::CSV
        );
        assert_eq!(
            SourceType::from_path("sales.parquet"),
            SourceType::Parquet
        );
        assert_eq!(SourceType::from_path("db.sqlite"), SourceType::SQLite);
    }

    #[test]
    fn test_parse_use_invalid() {
        let result = RqlParser::parse("USE data.csv sales");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_attach_invalid() {
        let result = RqlParser::parse("ATTACH db.sqlite wh");
        assert!(result.is_err());
    }
}
