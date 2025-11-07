//! Manejo de errores para Noctra Core

use std::fmt;
use thiserror::Error;

/// Tipos de errores en Noctra
#[derive(Error, Debug, Clone)]
pub enum NoctraError {
    #[error("Error de conexión a base de datos: {0}")]
    Database(String),

    #[error("Error de sintaxis SQL: {0}")]
    SqlSyntax(String),

    #[error("Error de ejecución SQL: {0}")]
    SqlExecution(String),

    #[error("Parámetro no encontrado: {0}")]
    ParameterNotFound(String),

    #[error("Variable de sesión no encontrada: {0}")]
    SessionVariableNotFound(String),

    #[error("Error de validación: {0}")]
    Validation(String),

    #[error("Error de configuración: {0}")]
    Configuration(String),

    #[error("Error de I/O: {0}")]
    Io(String),

    #[error("Error de serialización: {0}")]
    Serialization(String),

    #[error("Error interno: {0}")]
    Internal(String),
}

impl From<rusqlite::Error> for NoctraError {
    fn from(error: rusqlite::Error) -> Self {
        NoctraError::Database(format!("SQLite error: {}", error))
    }
}

impl NoctraError {
    /// Crear error de base de datos
    pub fn database<T: fmt::Display>(msg: T) -> Self {
        Self::Database(msg.to_string())
    }

    /// Crear error de sintaxis SQL
    pub fn sql_syntax<T: fmt::Display>(msg: T) -> Self {
        Self::SqlSyntax(msg.to_string())
    }

    /// Crear error de ejecución SQL
    pub fn sql_execution<T: fmt::Display>(msg: T) -> Self {
        Self::SqlExecution(msg.to_string())
    }

    /// Crear error de parámetro no encontrado
    pub fn parameter_not_found<T: fmt::Display>(param: T) -> Self {
        Self::ParameterNotFound(param.to_string())
    }

    /// Crear error de variable de sesión no encontrada
    pub fn session_variable_not_found<T: fmt::Display>(var: T) -> Self {
        Self::SessionVariableNotFound(var.to_string())
    }
}

/// Result type para operaciones de Noctra
pub type Result<T> = std::result::Result<T, NoctraError>;

/// Trait para conversión de errores
pub trait IntoNoctraError {
    fn into_noctra_error(self) -> NoctraError;
}
