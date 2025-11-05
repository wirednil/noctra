//! Error handling para el parser de RQL

use thiserror::Error;

/// Errores del parser de RQL
#[derive(Error, Debug, Clone, PartialEq)]
pub enum ParserError {
    #[error("Error de sintaxis en línea {line}: {message}")]
    SyntaxError {
        line: usize,
        column: usize,
        message: String,
    },
    
    #[error("Token inesperado '{token}' en línea {line}")]
    UnexpectedToken {
        line: usize,
        column: usize,
        token: String,
    },
    
    #[error("Parámetro no válido: {0}")]
    InvalidParameter(String),
    
    #[error("Comando RQL no reconocido: {0}")]
    UnknownCommand(String),
    
    #[error("Error de template: {0}")]
    TemplateError(String),
    
    #[error("Error de conversión de tipos: {0}")]
    TypeConversionError(String),
    
    #[error("Error de parseo de sqlparser: {0}")]
    SqlParserError(String),
    
    #[error("Error interno del parser: {0}")]
    InternalError(String),
}

impl ParserError {
    /// Crear error de sintaxis
    pub fn syntax_error<T: Into<String>>(line: usize, column: usize, message: T) -> Self {
        Self::SyntaxError {
            line,
            column,
            message: message.into(),
        }
    }
    
    /// Crear error de token inesperado
    pub fn unexpected_token<T: Into<String>>(line: usize, column: usize, token: T) -> Self {
        Self::UnexpectedToken {
            line,
            column,
            token: token.into(),
        }
    }
    
    /// Crear error de parámetro inválido
    pub fn invalid_parameter<T: Into<String>>(param: T) -> Self {
        Self::InvalidParameter(param.into())
    }
    
    /// Crear error de comando desconocido
    pub fn unknown_command<T: Into<String>>(command: T) -> Self {
        self::ParserError::UnknownCommand(command.into())
    }
    
    /// Crear error de template
    pub fn template_error<T: Into<String>>(message: T) -> Self {
        Self::TemplateError(message.into())
    }
}

/// Result type para operaciones del parser
pub type ParserResult<T> = std::result::Result<T, ParserError>;

impl From<sqlparser::parser::ParserError> for ParserError {
    fn from(error: sqlparser::parser::ParserError) -> Self {
        Self::SqlParserError(error.to_string())
    }
}

impl From<regex::Error> for ParserError {
    fn from(error: regex::Error) -> Self {
        Self::TemplateError(format!("Regex error: {}", error))
    }
}

impl From<serde_json::Error> for ParserError {
    fn from(error: serde_json::Error) -> Self {
        Self::InternalError(format!("JSON serialization error: {}", error))
    }
}