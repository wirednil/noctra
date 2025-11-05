//! Estructuras de formularios FDL2
//! 
//! Define los tipos de datos principales para representar formularios
//! declarativos en FDL2 (Form Definition Language).

use std::collections::HashMap;
use serde::{Deserialize, Serialize};

/// Un campo de formulario
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormField {
    /// Label visual del campo
    pub label: String,
    
    /// Tipo de datos del campo
    pub field_type: FieldType,
    
    /// Campo requerido
    pub required: bool,
    
    /// Ancho del campo (para UI)
    pub width: Option<usize>,
    
    /// Valor por defecto
    pub default: Option<String>,
    
    /// Validaciones específicas del campo
    pub validations: Option<FieldValidations>,
}

/// Tipo de campo
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FieldType {
    /// Campo de texto
    Text,
    
    /// Campo numérico entero
    Int,
    
    /// Campo numérico flotante
    Float,
    
    /// Campo booleano (checkbox/radio)
    Boolean,
    
    /// Campo de fecha
    Date,
    
    /// Campo de fecha y hora
    DateTime,
    
    /// Campo de email
    Email,
    
    /// Campo de contraseña
    Password,
    
    /// Campo de selección (dropdown)
    Select {
        options: Vec<String>,
    },
    
    /// Campo multi-selección
    MultiSelect {
        options: Vec<String>,
        max_selections: Option<usize>,
    },
    
    /// Campo de texto largo (textarea)
    TextArea {
        rows: usize,
    },
}

/// Validaciones específicas de campo
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldValidations {
    /// Valor mínimo (para números/fechas)
    pub min: Option<String>,
    
    /// Valor máximo (para números/fechas)
    pub max: Option<String>,
    
    /// Patrón regex para validación
    pub pattern: Option<String>,
    
    /// Longitud mínima
    pub min_length: Option<usize>,
    
    /// Longitud máxima
    pub max_length: Option<usize>,
    
    /// Lista de valores permitidos
    pub allowed_values: Option<Vec<String>>,
}

/// Una acción de formulario
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormAction {
    /// Tipo de acción
    pub action_type: ActionType,
    
    /// Consulta SQL asociada
    pub sql: Option<String>,
    
    /// Parámetros que usa esta acción
    pub params: Option<Vec<String>>,
    
    /// Tipo de parámetros
    pub param_type: ParamType,
}

/// Tipo de acción
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ActionType {
    /// Consulta SELECT (obtener datos)
    Query,
    
    /// Operación INSERT
    Insert,
    
    /// Operación UPDATE
    Update,
    
    /// Operación DELETE
    Delete,
    
    /// Script personalizado
    Script,
    
    /// Llamada a API externa
    ApiCall,
}

/// Tipo de parámetros
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ParamType {
    /// Parámetros posicionados ($1, $2, etc.)
    Positional,
    
    /// Parámetros nombrados (:name)
    Named,
}

/// Un formulario completo FDL2
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Form {
    /// Título del formulario
    pub title: String,
    
    /// Esquema/base de datos por defecto
    pub schema: Option<String>,
    
    /// Descripción del formulario
    pub description: Option<String>,
    
    /// Campos del formulario
    pub fields: HashMap<String, FormField>,
    
    /// Acciones disponibles
    pub actions: HashMap<String, FormAction>,
    
    /// Configuración de UI
    pub ui_config: Option<UiConfig>,
    
    /// Configuración de paginación
    pub pagination: Option<PaginationConfig>,
}

/// Configuración de interfaz de usuario
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiConfig {
    /// Ancho del formulario
    pub width: Option<usize>,
    
    /// Altura del formulario
    pub height: Option<usize>,
    
    /// Layout de campos (single/multi-column)
    pub layout: Option<LayoutType>,
    
    /// Estilo visual
    pub theme: Option<String>,
    
    /// Botones de acción
    pub buttons: Option<Vec<String>>,
}

/// Tipo de layout
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LayoutType {
    /// Layout de una columna
    Single,
    
    /// Layout de dos columnas
    Double,
    
    /// Layout flexible
    Flexible,
}

/// Configuración de paginación
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginationConfig {
    /// Número de filas por página
    pub page_size: Option<usize>,
    
    /// Campos para ordenar
    pub order_by: Option<Vec<String>>,
    
    /// Filtros por defecto
    pub default_filters: Option<HashMap<String, String>>,
}

/// Contexto de ejecución de formulario
#[derive(Debug, Clone)]
pub struct FormExecutionContext {
    /// Variables de sesión
    pub session_vars: HashMap<String, String>,
    
    /// Parámetros del formulario
    pub form_params: HashMap<String, String>,
    
    /// Conexión a base de datos
    pub database_url: Option<String>,
}

/// Resultado de ejecución de formulario
#[derive(Debug, Clone)]
pub struct FormExecutionResult {
    /// Éxito de la operación
    pub success: bool,
    
    /// Mensaje descriptivo
    pub message: String,
    
    /// Datos resultantes (si aplica)
    pub data: Option<noctra_core::ResultSet>,
    
    /// ID del registro insertado (si aplica)
    pub insert_id: Option<i64>,
    
    /// Número de filas afectadas (si aplica)
    pub affected_rows: Option<u64>,
}