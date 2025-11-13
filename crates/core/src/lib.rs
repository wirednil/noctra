//! Noctra Core Runtime
//!
//! El núcleo del sistema Noctra que proporciona tipos base,
//! execution engine y adaptadores de backend.

pub mod datasource;
pub mod error;
pub mod executor;
pub mod session;
pub mod types;

pub use datasource::{
    ColumnInfo, CsvOptions, DataSource, SourceMetadata, SourceRegistry, SourceType, TableInfo,
};

#[deprecated(since = "0.6.0", note = "Use noctra-duckdb instead")]
pub mod csv_backend {
    //! Legacy CSV backend - DEPRECATED
    //!
    //! This module is deprecated as of v0.6.0. Use `noctra-duckdb` crate instead.
    //! The CSV backend has been replaced by DuckDB for better performance and features.
    //!
    //! Migration guide:
    //! - Replace `CsvDataSource` with `noctra_duckdb::DuckDBSource`
    //! - Use `USE 'file.csv' AS alias` instead of manual CSV loading
    //! - DuckDB provides automatic type inference and better performance
}
pub use error::{NoctraError, Result};
pub use executor::{Backend, Executor, RqlQuery, SqliteBackend};
pub use session::{Session, SessionManager};
pub use types::{Column, ResultSet, Row, Value};
