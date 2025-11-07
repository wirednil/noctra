//! Loader de formularios FDL2
//!
//! Módulo para cargar formularios desde archivos TOML/JSON,
//! procesar configuraciones y preparar formularios para ejecución.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use thiserror::Error;

use crate::forms::{ActionType, FieldType, Form, FormAction, FormField, ParamType};

/// Error de carga de formulario
#[derive(Error, Debug)]
pub enum LoadError {
    /// Archivo no encontrado
    #[error("Archivo no encontrado: {0}")]
    FileNotFound(String),

    /// Error de parseo de TOML/JSON
    #[error("Error de parseo en {0}: {1}")]
    ParseError(String, String),

    /// Error de validación de esquema
    #[error("Error de validación: {0}")]
    ValidationError(String),

    /// Error de IO
    #[error("Error de IO: {0}")]
    IoError(String),
}

/// Resultado de carga
pub type LoadResult<T> = Result<T, LoadError>;

/// Configuración global del loader
#[derive(Debug, Clone)]
pub struct LoaderConfig {
    /// Directorio base de formularios
    pub base_path: Option<String>,

    /// Extensiones soportadas
    pub supported_extensions: Vec<String>,

    /// Validación estricta de esquemas
    pub strict_validation: bool,

    /// Auto-detectar tipos de campos
    pub auto_detect_types: bool,
}

impl Default for LoaderConfig {
    fn default() -> Self {
        Self {
            base_path: None,
            supported_extensions: vec!["toml".to_string(), "json".to_string()],
            strict_validation: true,
            auto_detect_types: false,
        }
    }
}

/// Loader principal de formularios
pub struct FormLoader {
    config: LoaderConfig,
}

impl FormLoader {
    /// Crear nuevo loader
    pub fn new(config: LoaderConfig) -> Self {
        Self { config }
    }

    /// Crear loader con configuración por defecto
    pub fn default() -> Self {
        Self::new(LoaderConfig::default())
    }

    /// Cargar formulario desde path
    pub fn load_from_path(&self, path: &Path) -> LoadResult<Form> {
        let path_str = path.to_string_lossy().to_string();

        if !path.exists() {
            return Err(LoadError::FileNotFound(path_str));
        }

        let content = fs::read_to_string(path).map_err(|e| LoadError::IoError(e.to_string()))?;

        self.load_from_string(&content, &path_str)
    }

    /// Cargar formulario desde string (TOML/JSON)
    pub fn load_from_string(&self, content: &str, source: &str) -> LoadResult<Form> {
        let extension = Path::new(source)
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("");

        match extension {
            "toml" => self.load_from_toml(content, source),
            "json" => self.load_from_json(content, source),
            _ => Err(LoadError::ParseError(
                source.to_string(),
                "Formato no soportado. Use TOML o JSON".to_string(),
            )),
        }
    }

    /// Cargar desde TOML
    fn load_from_toml(&self, content: &str, source: &str) -> LoadResult<Form> {
        let form: TomlForm = toml::from_str(content)
            .map_err(|e| LoadError::ParseError(source.to_string(), e.to_string()))?;

        self.convert_and_validate(form.into(), source)
    }

    /// Cargar desde JSON
    fn load_from_json(&self, content: &str, source: &str) -> LoadResult<Form> {
        let form: JsonForm = serde_json::from_str(content)
            .map_err(|e| LoadError::ParseError(source.to_string(), e.to_string()))?;

        self.convert_and_validate(form.into(), source)
    }

    /// Convertir y validar formulario
    fn convert_and_validate(&self, mut form: Form, source: &str) -> LoadResult<Form> {
        // Auto-detectar tipos si está habilitado
        if self.config.auto_detect_types {
            self.auto_detect_field_types(&mut form);
        }

        // Validar esquema
        if self.config.strict_validation {
            self.validate_form_schema(&form, source)?;
        }

        Ok(form)
    }

    /// Auto-detectar tipos de campos basado en nombres
    fn auto_detect_field_types(&self, form: &mut Form) {
        for (name, field) in &mut form.fields {
            if matches!(field.field_type, FieldType::Text) {
                // Auto-detectar basado en nombre del campo
                match name.to_lowercase().as_str() {
                    name if name.contains("email") => field.field_type = FieldType::Email,
                    name if name.contains("date") && !name.contains("time") => {
                        field.field_type = FieldType::Date
                    }
                    name if name.contains("datetime") || name.contains("timestamp") => {
                        field.field_type = FieldType::DateTime
                    }
                    name if name.contains("count")
                        || name.contains("num")
                        || name.contains("id") =>
                    {
                        field.field_type = FieldType::Int
                    }
                    name if name.contains("price")
                        || name.contains("amount")
                        || name.contains("rate") =>
                    {
                        field.field_type = FieldType::Float
                    }
                    name if name.contains("active")
                        || name.contains("enabled")
                        || name.contains("visible") =>
                    {
                        field.field_type = FieldType::Boolean
                    }
                    _ => {}
                }
            }
        }
    }

    /// Validar esquema del formulario
    fn validate_form_schema(&self, form: &Form, _source: &str) -> LoadResult<()> {
        // Validar que tenga al menos un campo
        if form.fields.is_empty() {
            return Err(LoadError::ValidationError(
                "Formulario debe tener al menos un campo".to_string(),
            ));
        }

        // Validar que tenga al menos una acción
        if form.actions.is_empty() {
            return Err(LoadError::ValidationError(
                "Formulario debe tener al menos una acción".to_string(),
            ));
        }

        // Validar acciones
        for (action_name, action) in &form.actions {
            if let Some(sql) = &action.sql {
                if sql.trim().is_empty() {
                    return Err(LoadError::ValidationError(format!(
                        "Acción '{}' tiene SQL vacío",
                        action_name
                    )));
                }
            }
        }

        // Validar campos requeridos
        for (field_name, field) in &form.fields {
            if field.required && field.default.is_none() {
                // Campo requerido sin default - OK si viene del usuario
                continue;
            }

            // Validar tipos de selección
            if let FieldType::Select { options } = &field.field_type {
                if options.is_empty() {
                    return Err(LoadError::ValidationError(format!(
                        "Campo '{}' de tipo Select debe tener opciones",
                        field_name
                    )));
                }
            }

            if let FieldType::MultiSelect {
                options,
                max_selections,
            } = &field.field_type
            {
                if options.is_empty() {
                    return Err(LoadError::ValidationError(format!(
                        "Campo '{}' de tipo MultiSelect debe tener opciones",
                        field_name
                    )));
                }

                if let Some(max) = max_selections {
                    if *max == 0 {
                        return Err(LoadError::ValidationError(format!(
                            "Campo '{}' MultiSelect max_selections debe ser > 0",
                            field_name
                        )));
                    }
                }
            }
        }

        Ok(())
    }
}

/// Cargar formulario desde path (wrapper)
pub fn load_form_from_path(path: &Path) -> LoadResult<Form> {
    let loader = FormLoader::default();
    loader.load_from_path(path)
}

/// Cargar formulario desde string
pub fn load_form(content: &str, source: &str) -> LoadResult<Form> {
    let loader = FormLoader::default();
    loader.load_from_string(content, source)
}

/// Representación intermedia de TOML
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename = "root")]
struct TomlForm {
    title: String,
    schema: Option<String>,
    description: Option<String>,
    fields: HashMap<String, TomlField>,
    actions: HashMap<String, TomlAction>,
    ui_config: Option<TomlUiConfig>,
    pagination: Option<TomlPaginationConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TomlField {
    label: String,
    #[serde(rename = "type")]
    field_type: String,
    required: Option<bool>,
    width: Option<usize>,
    default: Option<String>,
    validations: Option<TomlValidations>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TomlValidations {
    min: Option<String>,
    max: Option<String>,
    pattern: Option<String>,
    min_length: Option<usize>,
    max_length: Option<usize>,
    allowed_values: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TomlAction {
    action_type: String,
    sql: Option<String>,
    params: Option<Vec<String>>,
    param_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TomlUiConfig {
    width: Option<usize>,
    height: Option<usize>,
    layout: Option<String>,
    theme: Option<String>,
    buttons: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TomlPaginationConfig {
    page_size: Option<usize>,
    order_by: Option<Vec<String>>,
    default_filters: Option<HashMap<String, String>>,
}

/// Representación intermedia de JSON
#[derive(Debug, Clone, Serialize, Deserialize)]
struct JsonForm {
    title: String,
    schema: Option<String>,
    description: Option<String>,
    fields: HashMap<String, JsonField>,
    actions: HashMap<String, JsonAction>,
    ui_config: Option<JsonUiConfig>,
    pagination: Option<JsonPaginationConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct JsonField {
    label: String,
    field_type: String,
    required: Option<bool>,
    width: Option<usize>,
    default: Option<String>,
    validations: Option<JsonValidations>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct JsonValidations {
    min: Option<String>,
    max: Option<String>,
    pattern: Option<String>,
    min_length: Option<usize>,
    max_length: Option<usize>,
    allowed_values: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct JsonAction {
    action_type: String,
    sql: Option<String>,
    params: Option<Vec<String>>,
    param_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct JsonUiConfig {
    width: Option<usize>,
    height: Option<usize>,
    layout: Option<String>,
    theme: Option<String>,
    buttons: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct JsonPaginationConfig {
    page_size: Option<usize>,
    order_by: Option<Vec<String>>,
    default_filters: Option<HashMap<String, String>>,
}

/// Conversiones desde representaciones intermedias
impl From<TomlForm> for Form {
    fn from(toml_form: TomlForm) -> Self {
        let fields = toml_form
            .fields
            .into_iter()
            .map(|(name, field)| (name, field.into()))
            .collect();

        let actions = toml_form
            .actions
            .into_iter()
            .map(|(name, action)| (name, action.into()))
            .collect();

        Self {
            title: toml_form.title,
            schema: toml_form.schema,
            description: toml_form.description,
            fields,
            actions,
            ui_config: toml_form.ui_config.map(Into::into),
            pagination: toml_form.pagination.map(Into::into),
        }
    }
}

impl From<TomlField> for FormField {
    fn from(field: TomlField) -> Self {
        Self {
            label: field.label,
            field_type: parse_field_type(&field.field_type),
            required: field.required.unwrap_or(false),
            width: field.width,
            default: field.default,
            validations: field.validations.map(Into::into),
        }
    }
}

impl From<TomlValidations> for crate::forms::FieldValidations {
    fn from(validations: TomlValidations) -> Self {
        Self {
            min: validations.min,
            max: validations.max,
            pattern: validations.pattern,
            min_length: validations.min_length,
            max_length: validations.max_length,
            allowed_values: validations.allowed_values,
        }
    }
}

impl From<TomlAction> for FormAction {
    fn from(action: TomlAction) -> Self {
        Self {
            action_type: parse_action_type(&action.action_type),
            sql: action.sql,
            params: action.params,
            param_type: action
                .param_type
                .as_deref()
                .map(parse_param_type)
                .unwrap_or(ParamType::Named),
        }
    }
}

impl From<TomlUiConfig> for crate::forms::UiConfig {
    fn from(config: TomlUiConfig) -> Self {
        Self {
            width: config.width,
            height: config.height,
            layout: config.layout.as_deref().map(parse_layout_type),
            theme: config.theme,
            buttons: config.buttons,
        }
    }
}

impl From<TomlPaginationConfig> for crate::forms::PaginationConfig {
    fn from(config: TomlPaginationConfig) -> Self {
        Self {
            page_size: config.page_size,
            order_by: config.order_by,
            default_filters: config.default_filters,
        }
    }
}

impl From<JsonForm> for Form {
    fn from(json_form: JsonForm) -> Self {
        let fields = json_form
            .fields
            .into_iter()
            .map(|(name, field)| (name, field.into()))
            .collect();

        let actions = json_form
            .actions
            .into_iter()
            .map(|(name, action)| (name, action.into()))
            .collect();

        Self {
            title: json_form.title,
            schema: json_form.schema,
            description: json_form.description,
            fields,
            actions,
            ui_config: json_form.ui_config.map(Into::into),
            pagination: json_form.pagination.map(Into::into),
        }
    }
}

impl From<JsonField> for FormField {
    fn from(field: JsonField) -> Self {
        Self {
            label: field.label,
            field_type: parse_field_type(&field.field_type),
            required: field.required.unwrap_or(false),
            width: field.width,
            default: field.default,
            validations: field.validations.map(Into::into),
        }
    }
}

impl From<JsonValidations> for crate::forms::FieldValidations {
    fn from(validations: JsonValidations) -> Self {
        Self {
            min: validations.min,
            max: validations.max,
            pattern: validations.pattern,
            min_length: validations.min_length,
            max_length: validations.max_length,
            allowed_values: validations.allowed_values,
        }
    }
}

impl From<JsonAction> for FormAction {
    fn from(action: JsonAction) -> Self {
        Self {
            action_type: parse_action_type(&action.action_type),
            sql: action.sql,
            params: action.params,
            param_type: action
                .param_type
                .as_deref()
                .map(parse_param_type)
                .unwrap_or(ParamType::Named),
        }
    }
}

impl From<JsonUiConfig> for crate::forms::UiConfig {
    fn from(config: JsonUiConfig) -> Self {
        Self {
            width: config.width,
            height: config.height,
            layout: config.layout.as_deref().map(parse_layout_type),
            theme: config.theme,
            buttons: config.buttons,
        }
    }
}

impl From<JsonPaginationConfig> for crate::forms::PaginationConfig {
    fn from(config: JsonPaginationConfig) -> Self {
        Self {
            page_size: config.page_size,
            order_by: config.order_by,
            default_filters: config.default_filters,
        }
    }
}

/// Funciones de parseo auxiliares
fn parse_field_type(type_str: &str) -> FieldType {
    match type_str.to_lowercase().as_str() {
        "text" => FieldType::Text,
        "int" | "integer" => FieldType::Int,
        "float" | "number" => FieldType::Float,
        "bool" | "boolean" => FieldType::Boolean,
        "date" => FieldType::Date,
        "datetime" | "timestamp" => FieldType::DateTime,
        "email" => FieldType::Email,
        "password" => FieldType::Password,
        "textarea" => FieldType::TextArea { rows: 5 },
        _ => FieldType::Text,
    }
}

fn parse_action_type(type_str: &str) -> ActionType {
    match type_str.to_lowercase().as_str() {
        "query" | "select" => ActionType::Query,
        "insert" => ActionType::Insert,
        "update" => ActionType::Update,
        "delete" => ActionType::Delete,
        "script" => ActionType::Script,
        "api" | "apicall" => ActionType::ApiCall,
        _ => ActionType::Query,
    }
}

fn parse_param_type(type_str: &str) -> ParamType {
    match type_str.to_lowercase().as_str() {
        "positional" | "pos" => ParamType::Positional,
        "named" => ParamType::Named,
        _ => ParamType::Named,
    }
}

fn parse_layout_type(layout_str: &str) -> crate::forms::LayoutType {
    match layout_str.to_lowercase().as_str() {
        "single" => crate::forms::LayoutType::Single,
        "double" => crate::forms::LayoutType::Double,
        "flexible" => crate::forms::LayoutType::Flexible,
        _ => crate::forms::LayoutType::Single,
    }
}
