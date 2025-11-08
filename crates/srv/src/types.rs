//! Tipos de datos para el servidor Noctra
//!
//! Definiciones de tipos para peticiones, respuestas y estado del servidor.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use noctra_core::{Value, ResultSet};

/// Petición de query SQL/RQL
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryRequest {
    /// Query SQL o RQL a ejecutar
    pub query: String,

    /// Parámetros de la query
    #[serde(default)]
    pub parameters: HashMap<String, Value>,

    /// ID de sesión (opcional)
    pub session_id: Option<String>,

    /// Timeout en segundos (opcional)
    pub timeout: Option<u64>,
}

/// Respuesta de query
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryResponse {
    /// Resultado de la query
    pub result: ResultSet,

    /// Tiempo de ejecución en milisegundos
    pub execution_time_ms: u64,

    /// ID de sesión utilizado
    pub session_id: Option<String>,

    /// Metadata adicional
    #[serde(default)]
    pub metadata: HashMap<String, String>,
}

/// Petición de formulario FDL2
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormRequest {
    /// Nombre del formulario
    pub form_name: String,

    /// Acción a ejecutar
    pub action: String,

    /// Datos del formulario
    #[serde(default)]
    pub data: HashMap<String, Value>,

    /// ID de sesión (opcional)
    pub session_id: Option<String>,
}

/// Respuesta de formulario
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormResponse {
    /// Indica si la acción fue exitosa
    pub success: bool,

    /// Mensaje de resultado
    pub message: String,

    /// Datos de respuesta
    #[serde(default)]
    pub data: HashMap<String, Value>,

    /// Errores de validación (si los hay)
    #[serde(default)]
    pub validation_errors: Vec<ValidationError>,
}

/// Error de validación de formulario
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationError {
    /// Campo con error
    pub field: String,

    /// Mensaje de error
    pub message: String,

    /// Código de error
    pub code: String,
}

/// Estado del servidor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerStatus {
    /// Versión del servidor
    pub version: String,

    /// Tiempo de actividad en segundos
    pub uptime_seconds: u64,

    /// Número de sesiones activas
    pub active_sessions: usize,

    /// Número de queries ejecutadas
    pub queries_executed: u64,

    /// Backend de base de datos utilizado
    pub database_backend: String,

    /// Estado de salud
    pub health: HealthStatus,
}

/// Estado de salud del servidor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HealthStatus {
    /// Servidor operativo
    Healthy,

    /// Servidor con problemas
    Degraded,

    /// Servidor no operativo
    Unhealthy,
}

/// Error del servidor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerError {
    /// Código de error HTTP
    pub status_code: u16,

    /// Mensaje de error
    pub message: String,

    /// Detalles adicionales (para modo desarrollo)
    pub details: Option<String>,

    /// Timestamp del error
    pub timestamp: String,
}

impl ServerError {
    /// Crear error de petición inválida
    pub fn bad_request<T: Into<String>>(message: T) -> Self {
        Self {
            status_code: 400,
            message: message.into(),
            details: None,
            timestamp: chrono::Utc::now().to_rfc3339(),
        }
    }

    /// Crear error de servidor interno
    pub fn internal_error<T: Into<String>>(message: T) -> Self {
        Self {
            status_code: 500,
            message: message.into(),
            details: None,
            timestamp: chrono::Utc::now().to_rfc3339(),
        }
    }

    /// Crear error de no autorizado
    pub fn unauthorized<T: Into<String>>(message: T) -> Self {
        Self {
            status_code: 401,
            message: message.into(),
            details: None,
            timestamp: chrono::Utc::now().to_rfc3339(),
        }
    }

    /// Crear error de no encontrado
    pub fn not_found<T: Into<String>>(message: T) -> Self {
        Self {
            status_code: 404,
            message: message.into(),
            details: None,
            timestamp: chrono::Utc::now().to_rfc3339(),
        }
    }

    /// Agregar detalles al error
    pub fn with_details<T: Into<String>>(mut self, details: T) -> Self {
        self.details = Some(details.into());
        self
    }
}
