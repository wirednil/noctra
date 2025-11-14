//! Noctra DuckDB Backend
//!
//! DuckDB-powered data source implementation for Noctra, providing
//! native file-native queries for CSV, JSON, and Parquet files.
//!
//! ## Example Usage
//!
//! ```rust,no_run
//! use noctra_duckdb::DuckDBSource;
//! use noctra_core::datasource::DataSource; // Required for query() method
//!
//! // Create in-memory DuckDB source
//! let mut source = DuckDBSource::new_in_memory()?;
//!
//! // Register a CSV file as a virtual table
//! source.register_file("data.csv", "my_table")?;
//!
//! // Query the data
//! let result = source.query("SELECT * FROM my_table WHERE age > 25", &noctra_core::types::Parameters::new())?;
//!
//! // The result is compatible with existing Noctra API
//! println!("Found {} rows", result.rows.len());
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

pub mod source;
pub mod engine;
pub mod query_engine;
pub mod extensions;
pub mod error;
pub mod config;
pub mod attachment;

pub use source::DuckDBSource;
pub use engine::DuckDBEngine;
pub use query_engine::{QueryEngine, RoutingStrategy};
pub use error::{DuckDBError, Result};
pub use config::DuckDBConfig;
pub use attachment::{AttachmentConfig, AttachmentRegistry};