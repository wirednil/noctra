//! Noctra Parser - RQL Extended SQL Parser
//!
//! Parser para RQL (Extended SQL) que extiende sqlparser con características
//! específicas de Noctra como parámetros posicionados/nombrados y comandos extendidos.

pub mod error;
pub mod parser;
pub mod rql_ast;
pub mod template;

pub use error::{ParserError, ParserResult};
pub use parser::{RqlParser, RqlProcessor};
pub use rql_ast::{ExportFormat, MapExpression, ParameterType, RqlAst, RqlParameter, RqlStatement};
pub use template::{TemplateEngine, TemplateProcessor};

#[cfg(test)]
mod tests;  // Include comprehensive tests from tests.rs
