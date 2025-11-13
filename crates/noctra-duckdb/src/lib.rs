//! Noctra DuckDB Backend
//!
//! DuckDB-powered data source implementation for Noctra, providing
//! native file-native queries for CSV, JSON, and Parquet files.
//!
//! ## Example Usage
//!
//! ```rust
//! use noctra_duckdb::DuckDBSource;
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
pub mod extensions;
pub mod error;

pub use source::DuckDBSource;
pub use engine::DuckDBEngine;
pub use error::{DuckDBError, Result};