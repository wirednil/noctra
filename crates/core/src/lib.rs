//! Noctra Core Runtime
//!
//! El núcleo del sistema Noctra que proporciona tipos base,
//! execution engine y adaptadores de backend.

pub mod csv_backend;
pub mod datasource;
pub mod error;
pub mod executor;
pub mod session;
pub mod types;

pub use csv_backend::CsvDataSource;
pub use datasource::{
    ColumnInfo, CsvOptions, DataSource, SourceMetadata, SourceRegistry, SourceType, TableInfo,
};
pub use error::{NoctraError, Result};
pub use executor::{Backend, Executor, RqlQuery, SqliteBackend};
pub use session::{Session, SessionManager};
pub use types::{Column, ResultSet, Row, Value};
