//! Noctra Core Runtime
//!
//! El núcleo del sistema Noctra que proporciona tipos base,
//! execution engine y adaptadores de backend.

pub mod error;
pub mod executor;
pub mod session;
pub mod types;

pub use error::{NoctraError, Result};
pub use executor::{Backend, Executor, SqliteBackend};
pub use session::{Session, SessionManager};
pub use types::{Column, ResultSet, Row, Value};
