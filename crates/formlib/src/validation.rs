//! Validación de formularios FDL2
//!
//! Sistema de validación para campos de formularios, incluyendo
//! validaciones de tipos, rangos, patrones y reglas de negocio.

use chrono::{NaiveDate, NaiveDateTime};
use regex::Regex;
use std::collections::HashMap;
use thiserror::Error;

use crate::forms::{FieldType, FieldValidations, Form, FormField};

/// Error de validación
#[derive(Error, Debug)]
pub enum ValidationError {
    /// Campo requerido faltante
    #[error("Campo requerido '{0}' faltante")]
    RequiredField(String),

    /// Tipo de dato inválido
    #[error("Tipo de dato inválido para campo '{0}': {1}")]
    InvalidType(String, String),

    /// Valor fuera de rango
    #[error("Valor fuera de rango para campo '{0}': {1}")]
    OutOfRange(String, String),

    /// Patrón no cumplido
    #[error("Patrón no cumplido para campo '{0}': {1}")]
    PatternMismatch(String, String),

    /// Longitud inválida
    #[error("Longitud inválida para campo '{0}': {1}")]
    InvalidLength(String, String),

    /// Valor no permitido
    #[error("Valor '{0}' no permitido en campo '{1}'")]
    ValueNotAllowed(String, String),
}

/// Resultado de validación
pub type ValidationResult = Result<(), ValidationError>;

/// Validador principal
pub struct FormValidator;

impl Default for FormValidator {
    fn default() -> Self {
        Self::new()
    }
}

impl FormValidator {
    /// Crear nuevo validador
    pub fn new() -> Self {
        Self
    }

    /// Validar formulario completo
    pub fn validate_form(
        &self,
        form: &Form,
        values: &HashMap<String, String>,
    ) -> Result<(), Vec<ValidationError>> {
        let mut errors = Vec::new();

        // Validar cada campo
        for (field_name, field) in &form.fields {
            let value = values.get(field_name);

            // Validación requerida
            if field.required && value.is_none() {
                errors.push(ValidationError::RequiredField(field_name.clone()));
                continue;
            }

            // Si hay valor, validar el contenido
            if let Some(val) = value {
                if let Err(error) = self.validate_field(field, val) {
                    errors.push(error);
                }
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    /// Validar campo individual
    pub fn validate_field(&self, field: &FormField, value: &str) -> ValidationResult {
        // Validar tipo
        self.validate_type(field, value)?;

        // Validaciones específicas del campo
        if let Some(validations) = &field.validations {
            self.validate_field_validations(field, value, validations)?;
        }

        Ok(())
    }

    /// Validar tipo de datos
    fn validate_type(&self, field: &FormField, value: &str) -> ValidationResult {
        match &field.field_type {
            FieldType::Text => {
                if !value
                    .chars()
                    .all(|c| c.is_ascii_graphic() || c.is_whitespace())
                {
                    Err(ValidationError::InvalidType(
                        field.label.clone(),
                        "Texto inválido".to_string(),
                    ))
                } else {
                    Ok(())
                }
            }

            FieldType::Int => match value.parse::<i64>() {
                Ok(_) => Ok(()),
                Err(_) => Err(ValidationError::InvalidType(
                    field.label.clone(),
                    "Entero inválido".to_string(),
                )),
            },

            FieldType::Float => match value.parse::<f64>() {
                Ok(_) => Ok(()),
                Err(_) => Err(ValidationError::InvalidType(
                    field.label.clone(),
                    "Flotante inválido".to_string(),
                )),
            },

            FieldType::Boolean => match value.to_lowercase().as_str() {
                "true" | "false" | "1" | "0" | "sí" | "no" | "si" | "on" | "off" => Ok(()),
                _ => Err(ValidationError::InvalidType(
                    field.label.clone(),
                    "Booleano inválido".to_string(),
                )),
            },

            FieldType::Email => {
                if value.contains('@') && value.contains('.') {
                    Ok(())
                } else {
                    Err(ValidationError::InvalidType(
                        field.label.clone(),
                        "Email inválido".to_string(),
                    ))
                }
            }

            FieldType::Date => match NaiveDate::parse_from_str(value, "%Y-%m-%d") {
                Ok(_) => Ok(()),
                Err(_) => Err(ValidationError::InvalidType(
                    field.label.clone(),
                    "Fecha inválida (YYYY-MM-DD)".to_string(),
                )),
            },

            FieldType::DateTime => {
                match NaiveDateTime::parse_from_str(value, "%Y-%m-%d %H:%M:%S") {
                    Ok(_) => Ok(()),
                    Err(_) => Err(ValidationError::InvalidType(
                        field.label.clone(),
                        "FechaHora inválida (YYYY-MM-DD HH:MM:SS)".to_string(),
                    )),
                }
            }

            FieldType::Select { options } => {
                if options.contains(&value.to_string()) {
                    Ok(())
                } else {
                    Err(ValidationError::ValueNotAllowed(
                        value.to_string(),
                        field.label.clone(),
                    ))
                }
            }

            FieldType::MultiSelect { options, .. } => {
                let values: Vec<&str> = value.split(',').map(|s| s.trim()).collect();
                for val in values {
                    if !options.contains(&val.to_string()) {
                        return Err(ValidationError::ValueNotAllowed(
                            val.to_string(),
                            field.label.clone(),
                        ));
                    }
                }
                Ok(())
            }

            _ => Ok(()), // Otros tipos sin validación específica
        }
    }

    /// Validar reglas específicas del campo
    fn validate_field_validations(
        &self,
        field: &FormField,
        value: &str,
        validations: &FieldValidations,
    ) -> ValidationResult {
        // Validar rango mínimo/máximo
        if let Some(min) = &validations.min {
            self.validate_min_value(field, value, min)?;
        }

        if let Some(max) = &validations.max {
            self.validate_max_value(field, value, max)?;
        }

        // Validar patrón regex
        if let Some(pattern) = &validations.pattern {
            self.validate_pattern(field, value, pattern)?;
        }

        // Validar longitud
        if let Some(min_length) = validations.min_length {
            if value.len() < min_length {
                return Err(ValidationError::InvalidLength(
                    field.label.clone(),
                    format!("Mínimo {} caracteres", min_length),
                ));
            }
        }

        if let Some(max_length) = validations.max_length {
            if value.len() > max_length {
                return Err(ValidationError::InvalidLength(
                    field.label.clone(),
                    format!("Máximo {} caracteres", max_length),
                ));
            }
        }

        // Validar valores permitidos
        if let Some(allowed_values) = &validations.allowed_values {
            if !allowed_values.contains(&value.to_string()) {
                return Err(ValidationError::ValueNotAllowed(
                    value.to_string(),
                    field.label.clone(),
                ));
            }
        }

        Ok(())
    }

    /// Validar valor mínimo
    fn validate_min_value(&self, field: &FormField, value: &str, min: &str) -> ValidationResult {
        match &field.field_type {
            FieldType::Int => {
                let val: i64 = value.parse().map_err(|_| {
                    ValidationError::InvalidType(field.label.clone(), "Entero inválido".to_string())
                })?;
                let min_val: i64 = min.parse().map_err(|_| {
                    ValidationError::InvalidType(
                        field.label.clone(),
                        "Valor mínimo inválido".to_string(),
                    )
                })?;

                if val < min_val {
                    Err(ValidationError::OutOfRange(
                        field.label.clone(),
                        format!("Mínimo {}", min_val),
                    ))
                } else {
                    Ok(())
                }
            }

            FieldType::Float => {
                let val: f64 = value.parse().map_err(|_| {
                    ValidationError::InvalidType(
                        field.label.clone(),
                        "Flotante inválido".to_string(),
                    )
                })?;
                let min_val: f64 = min.parse().map_err(|_| {
                    ValidationError::InvalidType(
                        field.label.clone(),
                        "Valor mínimo inválido".to_string(),
                    )
                })?;

                if val < min_val {
                    Err(ValidationError::OutOfRange(
                        field.label.clone(),
                        format!("Mínimo {}", min_val),
                    ))
                } else {
                    Ok(())
                }
            }

            _ => Ok(()),
        }
    }

    /// Validar valor máximo
    fn validate_max_value(&self, field: &FormField, value: &str, max: &str) -> ValidationResult {
        match &field.field_type {
            FieldType::Int => {
                let val: i64 = value.parse().map_err(|_| {
                    ValidationError::InvalidType(field.label.clone(), "Entero inválido".to_string())
                })?;
                let max_val: i64 = max.parse().map_err(|_| {
                    ValidationError::InvalidType(
                        field.label.clone(),
                        "Valor máximo inválido".to_string(),
                    )
                })?;

                if val > max_val {
                    Err(ValidationError::OutOfRange(
                        field.label.clone(),
                        format!("Máximo {}", max_val),
                    ))
                } else {
                    Ok(())
                }
            }

            FieldType::Float => {
                let val: f64 = value.parse().map_err(|_| {
                    ValidationError::InvalidType(
                        field.label.clone(),
                        "Flotante inválido".to_string(),
                    )
                })?;
                let max_val: f64 = max.parse().map_err(|_| {
                    ValidationError::InvalidType(
                        field.label.clone(),
                        "Valor máximo inválido".to_string(),
                    )
                })?;

                if val > max_val {
                    Err(ValidationError::OutOfRange(
                        field.label.clone(),
                        format!("Máximo {}", max_val),
                    ))
                } else {
                    Ok(())
                }
            }

            _ => Ok(()),
        }
    }

    /// Validar patrón regex
    fn validate_pattern(&self, field: &FormField, value: &str, pattern: &str) -> ValidationResult {
        match Regex::new(pattern) {
            Ok(regex) => {
                if regex.is_match(value) {
                    Ok(())
                } else {
                    Err(ValidationError::PatternMismatch(
                        field.label.clone(),
                        format!("No cumple el patrón: {}", pattern),
                    ))
                }
            }
            Err(_) => Err(ValidationError::InvalidType(
                field.label.clone(),
                "Patrón regex inválido".to_string(),
            )),
        }
    }
}
