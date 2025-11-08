//! Form Renderer - Renderizado de formularios FDL2 en TUI
//!
//! Widget para renderizar y manejar formularios declarativos
//! con validación en tiempo real.

use std::collections::HashMap;
use thiserror::Error;

use noctra_formlib::{FieldType, Form, FormField, ValidationError};
use noctra_formlib::validation::FormValidator;

/// Error del FormRenderer
#[derive(Error, Debug)]
pub enum FormRenderError {
    /// Campo no encontrado
    #[error("Campo '{0}' no encontrado")]
    FieldNotFound(String),

    /// Error de validación
    #[error("Errores de validación: {0:?}")]
    ValidationErrors(Vec<ValidationError>),

    /// Error de renderizado
    #[error("Error de renderizado: {0}")]
    RenderError(String),
}

/// Resultado de operaciones con FormRenderer
pub type FormRenderResult<T> = Result<T, FormRenderError>;

/// Estado de un campo de formulario
#[derive(Debug, Clone)]
pub struct FieldState {
    /// Valor actual del campo
    pub value: String,

    /// Campo enfocado
    pub focused: bool,

    /// Errores de validación
    pub errors: Vec<String>,

    /// Campo tocado (edited)
    pub touched: bool,

    /// Campo válido
    pub valid: bool,
}

impl Default for FieldState {
    fn default() -> Self {
        Self {
            value: String::new(),
            focused: false,
            errors: Vec::new(),
            touched: false,
            valid: true,
        }
    }
}

/// Renderer de formularios
pub struct FormRenderer {
    /// Formulario a renderizar
    pub form: Form,

    /// Estado de los campos
    field_states: HashMap<String, FieldState>,

    /// Orden de los campos (para tab navigation)
    field_order: Vec<String>,

    /// Índice del campo enfocado
    focused_field_index: usize,

    /// Validador
    validator: FormValidator,

    /// Ancho del formulario
    width: usize,

    /// Altura del formulario
    height: usize,

    /// Offset de scroll
    scroll_offset: usize,

    /// Modo de validación
    validate_on_change: bool,
}

impl FormRenderer {
    /// Crear nuevo renderer
    pub fn new(form: Form) -> Self {
        let field_order: Vec<String> = form.fields.keys().cloned().collect();
        let mut field_states = HashMap::new();

        // Inicializar estados de campos con valores por defecto
        for (name, field) in &form.fields {
            let mut state = FieldState::default();
            if let Some(default) = &field.default {
                state.value = default.clone();
            }
            field_states.insert(name.clone(), state);
        }

        // Enfocar primer campo
        if let Some(first_field) = field_order.first() {
            if let Some(state) = field_states.get_mut(first_field) {
                state.focused = true;
            }
        }

        Self {
            form,
            field_states,
            field_order,
            focused_field_index: 0,
            validator: FormValidator::new(),
            width: 80,
            height: 24,
            scroll_offset: 0,
            validate_on_change: true,
        }
    }

    /// Crear con dimensiones específicas
    pub fn with_size(mut self, width: usize, height: usize) -> Self {
        self.width = width;
        self.height = height;
        self
    }

    /// Habilitar/deshabilitar validación en tiempo real
    pub fn with_validate_on_change(mut self, validate: bool) -> Self {
        self.validate_on_change = validate;
        self
    }

    /// Obtener valor de un campo
    pub fn get_field_value(&self, field_name: &str) -> Option<&str> {
        self.field_states.get(field_name).map(|s| s.value.as_str())
    }

    /// Establecer valor de un campo
    pub fn set_field_value(&mut self, field_name: &str, value: String) -> FormRenderResult<()> {
        let state = self
            .field_states
            .get_mut(field_name)
            .ok_or_else(|| FormRenderError::FieldNotFound(field_name.to_string()))?;

        state.value = value;
        state.touched = true;

        // Validar si está habilitado
        if self.validate_on_change {
            self.validate_field(field_name)?;
        }

        Ok(())
    }

    /// Validar un campo específico
    pub fn validate_field(&mut self, field_name: &str) -> FormRenderResult<()> {
        let field = self
            .form
            .fields
            .get(field_name)
            .ok_or_else(|| FormRenderError::FieldNotFound(field_name.to_string()))?;

        let state = self
            .field_states
            .get_mut(field_name)
            .ok_or_else(|| FormRenderError::FieldNotFound(field_name.to_string()))?;

        // Limpiar errores anteriores
        state.errors.clear();
        state.valid = true;

        // Validar si tiene valor o es requerido
        if !state.value.is_empty() || field.required {
            if let Err(error) = self.validator.validate_field(field, &state.value) {
                state.errors.push(error.to_string());
                state.valid = false;
            }
        }

        Ok(())
    }

    /// Validar formulario completo
    pub fn validate_form(&mut self) -> FormRenderResult<()> {
        let values: HashMap<String, String> = self
            .field_states
            .iter()
            .map(|(name, state)| (name.clone(), state.value.clone()))
            .collect();

        match self.validator.validate_form(&self.form, &values) {
            Ok(()) => {
                // Marcar todos los campos como válidos
                for state in self.field_states.values_mut() {
                    state.valid = true;
                    state.errors.clear();
                }
                Ok(())
            }
            Err(errors) => {
                // Marcar campos con errores
                for error in &errors {
                    // Extraer nombre del campo del mensaje de error
                    // Por ahora marcar como inválido
                    for state in self.field_states.values_mut() {
                        if !state.value.is_empty() {
                            state.errors.push(error.to_string());
                            state.valid = false;
                        }
                    }
                }
                Err(FormRenderError::ValidationErrors(errors))
            }
        }
    }

    /// Obtener todos los valores del formulario
    pub fn get_values(&self) -> HashMap<String, String> {
        self.field_states
            .iter()
            .map(|(name, state)| (name.clone(), state.value.clone()))
            .collect()
    }

    /// Verificar si el formulario es válido
    pub fn is_valid(&self) -> bool {
        self.field_states.values().all(|state| state.valid)
    }

    /// Mover foco al siguiente campo
    pub fn focus_next(&mut self) {
        // Desenfocar campo actual
        if let Some(field_name) = self.field_order.get(self.focused_field_index) {
            if let Some(state) = self.field_states.get_mut(field_name) {
                state.focused = false;
            }
        }

        // Mover al siguiente
        self.focused_field_index = (self.focused_field_index + 1) % self.field_order.len();

        // Enfocar nuevo campo
        if let Some(field_name) = self.field_order.get(self.focused_field_index) {
            if let Some(state) = self.field_states.get_mut(field_name) {
                state.focused = true;
            }
        }
    }

    /// Mover foco al campo anterior
    pub fn focus_previous(&mut self) {
        // Desenfocar campo actual
        if let Some(field_name) = self.field_order.get(self.focused_field_index) {
            if let Some(state) = self.field_states.get_mut(field_name) {
                state.focused = false;
            }
        }

        // Mover al anterior
        if self.focused_field_index == 0 {
            self.focused_field_index = self.field_order.len() - 1;
        } else {
            self.focused_field_index -= 1;
        }

        // Enfocar nuevo campo
        if let Some(field_name) = self.field_order.get(self.focused_field_index) {
            if let Some(state) = self.field_states.get_mut(field_name) {
                state.focused = true;
            }
        }
    }

    /// Obtener campo enfocado
    pub fn get_focused_field(&self) -> Option<&str> {
        self.field_order
            .get(self.focused_field_index)
            .map(|s| s.as_str())
    }

    /// Renderizar formulario
    pub fn render(&self) -> String {
        let mut output = String::new();

        // Título del formulario
        let title_line = format!("┌─ {} ", self.form.title);
        let padding = self.width.saturating_sub(title_line.len() + 1);
        output.push_str(&title_line);
        output.push_str(&"─".repeat(padding));
        output.push_str("┐\n");

        // Descripción si existe
        if let Some(desc) = &self.form.description {
            output.push_str(&format!("│ {} ", desc));
            let padding = self.width.saturating_sub(desc.len() + 4);
            output.push_str(&" ".repeat(padding));
            output.push_str("│\n");
            output.push_str(&format!("├{}┤\n", "─".repeat(self.width - 2)));
        }

        // Renderizar campos
        for (i, field_name) in self.field_order.iter().enumerate() {
            if i < self.scroll_offset {
                continue;
            }

            if let (Some(field), Some(state)) = (
                self.form.fields.get(field_name),
                self.field_states.get(field_name),
            ) {
                output.push_str(&self.render_field(field_name, field, state));
            }
        }

        // Separador antes de botones
        output.push_str(&format!("├{}┤\n", "─".repeat(self.width - 2)));

        // Botones de acción
        output.push_str(&self.render_actions());

        // Footer
        output.push_str(&format!("└{}┘\n", "─".repeat(self.width - 2)));

        // Ayuda
        output.push_str(&format!(
            " TAB=Next Field | ENTER=Submit | ESC=Cancel\n"
        ));

        output
    }

    /// Renderizar un campo
    fn render_field(&self, field_name: &str, field: &FormField, state: &FieldState) -> String {
        let mut output = String::new();

        // Label del campo
        let required_marker = if field.required { "*" } else { " " };
        let focus_marker = if state.focused { "▶" } else { " " };

        let label_line = format!(
            "│ {}{} {}:",
            focus_marker, required_marker, field.label
        );
        output.push_str(&label_line);

        let label_padding = self.width.saturating_sub(label_line.len() + 1);
        output.push_str(&" ".repeat(label_padding));
        output.push_str("│\n");

        // Input del campo
        let input_line = self.render_field_input(field, state);
        output.push_str(&format!("│  {}", input_line));

        let input_padding = self.width.saturating_sub(input_line.len() + 4);
        output.push_str(&" ".repeat(input_padding));
        output.push_str("│\n");

        // Errores de validación
        if !state.errors.is_empty() {
            for error in &state.errors {
                let error_line = format!("│  ❌ {}", error);
                output.push_str(&error_line);
                let error_padding = self.width.saturating_sub(error_line.len() + 1);
                output.push_str(&" ".repeat(error_padding));
                output.push_str("│\n");
            }
        }

        output
    }

    /// Renderizar input de un campo según su tipo
    fn render_field_input(&self, field: &FormField, state: &FieldState) -> String {
        let value_display = if state.value.is_empty() {
            "<empty>".to_string()
        } else {
            // Enmascarar passwords
            match field.field_type {
                FieldType::Password => "•".repeat(state.value.len()),
                _ => state.value.clone(),
            }
        };

        let width = field.width.unwrap_or(30);

        if state.focused {
            format!("[{}]", self.pad_or_truncate(&value_display, width))
        } else {
            format!(" {} ", self.pad_or_truncate(&value_display, width))
        }
    }

    /// Renderizar botones de acción
    fn render_actions(&self) -> String {
        let mut output = String::new();

        let buttons: Vec<String> = self
            .form
            .actions
            .keys()
            .map(|name| format!("[ {} ]", name.to_uppercase()))
            .collect();

        let buttons_line = format!("│  {}", buttons.join("  "));
        output.push_str(&buttons_line);

        let padding = self.width.saturating_sub(buttons_line.len() + 1);
        output.push_str(&" ".repeat(padding));
        output.push_str("│\n");

        output
    }

    /// Pad or truncate string to exact width
    fn pad_or_truncate(&self, s: &str, width: usize) -> String {
        if s.len() > width {
            s[..width].to_string()
        } else {
            format!("{}{}", s, " ".repeat(width - s.len()))
        }
    }

    /// Resetear formulario
    pub fn reset(&mut self) {
        for (name, field) in &self.form.fields {
            if let Some(state) = self.field_states.get_mut(name) {
                state.value = field.default.clone().unwrap_or_default();
                state.touched = false;
                state.errors.clear();
                state.valid = true;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn create_test_form() -> Form {
        let mut fields = HashMap::new();
        fields.insert(
            "name".to_string(),
            FormField {
                label: "Name".to_string(),
                field_type: FieldType::Text,
                required: true,
                width: Some(30),
                default: None,
                validations: None,
            },
        );
        fields.insert(
            "email".to_string(),
            FormField {
                label: "Email".to_string(),
                field_type: FieldType::Email,
                required: true,
                width: Some(30),
                default: None,
                validations: None,
            },
        );

        let mut actions = HashMap::new();
        actions.insert(
            "submit".to_string(),
            noctra_formlib::FormAction {
                action_type: noctra_formlib::ActionType::Query,
                sql: Some("SELECT 1".to_string()),
                params: None,
                param_type: noctra_formlib::ParamType::Named,
            },
        );

        Form {
            title: "Test Form".to_string(),
            schema: None,
            description: Some("Test description".to_string()),
            fields,
            actions,
            ui_config: None,
            pagination: None,
        }
    }

    #[test]
    fn test_form_renderer_creation() {
        let form = create_test_form();
        let renderer = FormRenderer::new(form);

        assert_eq!(renderer.field_order.len(), 2);
        assert_eq!(renderer.focused_field_index, 0);
        assert!(renderer.is_valid());
    }

    #[test]
    fn test_set_field_value() {
        let form = create_test_form();
        let mut renderer = FormRenderer::new(form);

        renderer
            .set_field_value("name", "John Doe".to_string())
            .unwrap();
        assert_eq!(renderer.get_field_value("name"), Some("John Doe"));
    }

    #[test]
    fn test_focus_navigation() {
        let form = create_test_form();
        let mut renderer = FormRenderer::new(form);

        // Guardar campo inicial (puede ser "name" o "email" dependiendo del orden del HashMap)
        let first_field = renderer.get_focused_field().unwrap().to_string();
        assert!(first_field == "name" || first_field == "email");

        // Navegar al siguiente
        renderer.focus_next();
        let second_field = renderer.get_focused_field().unwrap().to_string();
        assert!(second_field == "name" || second_field == "email");
        assert_ne!(first_field, second_field); // Debe ser diferente

        // Volver al anterior
        renderer.focus_previous();
        assert_eq!(renderer.get_focused_field(), Some(first_field.as_str()));
    }

    #[test]
    fn test_render() {
        let form = create_test_form();
        let renderer = FormRenderer::new(form);

        let output = renderer.render();
        assert!(output.contains("Test Form"));
        assert!(output.contains("Name"));
        assert!(output.contains("Email"));
        assert!(output.contains("SUBMIT"));
    }

    #[test]
    fn test_get_values() {
        let form = create_test_form();
        let mut renderer = FormRenderer::new(form);

        renderer
            .set_field_value("name", "John".to_string())
            .unwrap();
        renderer
            .set_field_value("email", "john@example.com".to_string())
            .unwrap();

        let values = renderer.get_values();
        assert_eq!(values.get("name"), Some(&"John".to_string()));
        assert_eq!(values.get("email"), Some(&"john@example.com".to_string()));
    }
}
