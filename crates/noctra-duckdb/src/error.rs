//! Error types for noctra-duckdb

use thiserror::Error;

/// Errors that can occur in the DuckDB backend
#[derive(Error, Debug)]
pub enum DuckDBError {
    #[error("DuckDB error: {0}")]
    DuckDB(#[from] duckdb::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Unsupported file type: {0}")]
    UnsupportedFileType(String),

    #[error("File not found: {0}")]
    FileNotFound(String),

    #[error("Query execution failed: {0}")]
    QueryFailed(String),

    #[error("Type conversion error: {0}")]
    TypeConversion(String),

    #[error("Schema introspection failed: {0}")]
    SchemaError(String),
}

/// Result type alias for DuckDB operations
pub type Result<T> = std::result::Result<T, DuckDBError>;