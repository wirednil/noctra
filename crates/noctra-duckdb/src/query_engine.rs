//! Query Engine with Hybrid Backend Routing
//!
//! Provides intelligent query routing between DuckDB (file-native) and future backends.
//!
//! ## Architecture
//!
//! The QueryEngine wraps DuckDBSource and provides:
//! - Automatic routing logic (future: multi-backend)
//! - Unified query interface
//! - File registration and attachment management
//!
//! ## Example
//!
//! ```rust,no_run
//! use noctra_duckdb::{DuckDBSource, QueryEngine, RoutingStrategy};
//! use noctra_core::datasource::DataSource;
//! use noctra_core::types::Parameters;
//!
//! // Create DuckDB backend
//! let source = DuckDBSource::new_in_memory()?;
//!
//! // Create engine with auto-routing
//! let mut engine = QueryEngine::new(source);
//!
//! // Register a CSV file
//! engine.register_file("sales.csv", "sales")?;
//!
//! // Query the file
//! let result = engine.query("SELECT * FROM sales WHERE amount > 100", &Parameters::new())?;
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

use crate::error::Result;
use crate::source::DuckDBSource;
use noctra_core::datasource::DataSource;
use noctra_core::types::{Parameters, ResultSet};

/// Query routing strategy
///
/// Determines how queries are routed to backends in multi-backend scenarios.
/// Currently only Auto is used, but this is designed for future extensibility.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RoutingStrategy {
    /// Automatic detection based on query context
    ///
    /// The engine analyzes registered sources and query structure to determine
    /// the optimal backend. Currently always routes to DuckDB.
    Auto,

    /// Force all queries to file-native backend (DuckDB)
    ///
    /// Useful when you want to ensure consistent analytical query performance.
    ForceFile,

    /// Future: Force all queries to database backend (SQLite, PostgreSQL, etc.)
    ForceDatabase,
}

impl Default for RoutingStrategy {
    fn default() -> Self {
        RoutingStrategy::Auto
    }
}

/// Query Engine with intelligent backend routing
///
/// Wraps DuckDBSource and provides a unified query interface with future
/// extensibility for multi-backend scenarios.
pub struct QueryEngine {
    /// DuckDB backend for file-native queries
    duckdb: DuckDBSource,

    /// Routing strategy (currently always Auto)
    routing: RoutingStrategy,
}

impl QueryEngine {
    /// Create a new query engine with DuckDB backend
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use noctra_duckdb::{DuckDBSource, QueryEngine};
    ///
    /// let source = DuckDBSource::new_in_memory()?;
    /// let engine = QueryEngine::new(source);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn new(duckdb: DuckDBSource) -> Self {
        Self {
            duckdb,
            routing: RoutingStrategy::Auto,
        }
    }

    /// Create a new query engine with custom routing strategy
    ///
    /// Currently, all strategies route to DuckDB. Future versions will support
    /// multi-backend routing.
    pub fn with_strategy(duckdb: DuckDBSource, routing: RoutingStrategy) -> Self {
        Self { duckdb, routing }
    }

    /// Get the current routing strategy
    pub fn routing_strategy(&self) -> RoutingStrategy {
        self.routing
    }

    /// Register a file as a queryable data source
    ///
    /// The file will be registered as a virtual table in DuckDB.
    /// Supported formats: CSV, JSON, Parquet
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use noctra_duckdb::{DuckDBSource, QueryEngine};
    /// # let source = DuckDBSource::new_in_memory()?;
    /// let mut engine = QueryEngine::new(source);
    /// engine.register_file("data.csv", "my_data")?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn register_file(&mut self, file_path: &str, alias: &str) -> Result<()> {
        self.duckdb.register_file(file_path, alias)
    }

    /// Attach a SQLite database for cross-source queries
    ///
    /// The attached database can be queried alongside file-based sources.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use noctra_duckdb::{DuckDBSource, QueryEngine};
    /// # let source = DuckDBSource::new_in_memory()?;
    /// let mut engine = QueryEngine::new(source);
    /// engine.attach_sqlite("warehouse.db", "wh")?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn attach_sqlite(&mut self, db_path: &str, alias: &str) -> Result<()> {
        self.duckdb.attach_sqlite(db_path, alias)
    }

    /// Execute a SQL query
    ///
    /// The query is routed to DuckDB backend. Future versions will support
    /// intelligent routing to multiple backends.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use noctra_duckdb::{DuckDBSource, QueryEngine};
    /// # use noctra_core::types::Parameters;
    /// # let source = DuckDBSource::new_in_memory()?;
    /// let mut engine = QueryEngine::new(source);
    /// engine.register_file("sales.csv", "sales")?;
    ///
    /// let result = engine.query("SELECT * FROM sales", &Parameters::new())?;
    /// println!("Found {} rows", result.rows.len());
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn query(&self, sql: &str, params: &Parameters) -> noctra_core::error::Result<ResultSet> {
        // Currently all queries route to DuckDB
        // Future: Parse SQL and route based on strategy and source type
        self.duckdb.query(sql, params)
    }

    /// Get reference to the underlying DuckDB source
    ///
    /// This allows direct access to DuckDB-specific features if needed.
    pub fn duckdb(&self) -> &DuckDBSource {
        &self.duckdb
    }

    /// Get mutable reference to the underlying DuckDB source
    pub fn duckdb_mut(&mut self) -> &mut DuckDBSource {
        &mut self.duckdb
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn test_engine_creation() {
        let source = DuckDBSource::new_in_memory().unwrap();
        let engine = QueryEngine::new(source);

        assert_eq!(engine.routing_strategy(), RoutingStrategy::Auto);
    }

    #[test]
    fn test_engine_with_strategy() {
        let source = DuckDBSource::new_in_memory().unwrap();
        let engine = QueryEngine::with_strategy(source, RoutingStrategy::ForceFile);

        assert_eq!(engine.routing_strategy(), RoutingStrategy::ForceFile);
    }

    #[test]
    fn test_simple_query() {
        let source = DuckDBSource::new_in_memory().unwrap();
        let engine = QueryEngine::new(source);

        let result = engine.query("SELECT 42 as answer", &Parameters::new());
        assert!(result.is_ok());

        let result = result.unwrap();
        assert_eq!(result.rows.len(), 1);
        assert_eq!(result.columns.len(), 1);
        assert_eq!(result.columns[0].name, "answer");
    }

    #[test]
    fn test_file_registration() {
        let source = DuckDBSource::new_in_memory().unwrap();
        let mut engine = QueryEngine::new(source);

        // Create temp CSV
        let mut csv_file = tempfile::Builder::new().suffix(".csv").tempfile().unwrap();
        writeln!(csv_file, "id,name,value").unwrap();
        writeln!(csv_file, "1,Alice,100").unwrap();
        writeln!(csv_file, "2,Bob,200").unwrap();
        csv_file.flush().unwrap();

        // Register file
        let result = engine.register_file(csv_file.path().to_str().unwrap(), "test_data");
        assert!(result.is_ok());

        // Query the file
        let result = engine.query("SELECT * FROM test_data WHERE value > 150", &Parameters::new());
        assert!(result.is_ok());

        let result = result.unwrap();
        assert_eq!(result.rows.len(), 1);
    }

    #[test]
    fn test_duckdb_access() {
        let source = DuckDBSource::new_in_memory().unwrap();
        let engine = QueryEngine::new(source);

        // Should be able to access underlying DuckDB source
        let duckdb = engine.duckdb();
        assert!(duckdb.config().threads.is_some());
    }
}
