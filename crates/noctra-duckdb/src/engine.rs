//! Query execution engine for DuckDB backend

use crate::error::{DuckDBError, Result};
use crate::source::DuckDBSource;
use noctra_core::types::{Parameters, ResultSet};
use noctra_core::DataSource;

/// Query execution engine for DuckDB
pub struct DuckDBEngine {
    source: DuckDBSource,
}

impl DuckDBEngine {
    /// Create a new DuckDB engine with in-memory database
    pub fn new_in_memory() -> Result<Self> {
        let source = DuckDBSource::new_in_memory()?;
        Ok(Self { source })
    }

    /// Register a file for querying
    pub fn register_file(&mut self, file_path: &str, alias: &str) -> Result<()> {
        self.source.register_file(file_path, alias)
    }

    /// Execute a SQL query
    pub fn execute_query(&self, sql: &str, params: &Parameters) -> Result<ResultSet> {
        self.source.query(sql, params).map_err(|e| DuckDBError::QueryFailed(format!("Query execution failed: {}", e)))
    }

    /// Get the underlying DuckDB source
    pub fn source(&self) -> &DuckDBSource {
        &self.source
    }

    /// Get mutable access to the source
    pub fn source_mut(&mut self) -> &mut DuckDBSource {
        &mut self.source
    }
}