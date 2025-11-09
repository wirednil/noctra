//! Form Renderer - Renderizado de formularios FDL2 en TUI con Ratatui
//!
//! Widget para renderizar y manejar formularios declarativos
//! con validación en tiempo real usando Ratatui.

use std::collections::HashMap;
use thiserror::Error;

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
    Frame,
};

use noctra_formlib::validation::FormValidator;
use noctra_formlib::{FieldType, Form, ValidationError};

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

/// Renderer de formularios usando Ratatui
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
            scroll_offset: 0,
            validate_on_change: true,
        }
    }

    /// Mantener compatibilidad con código existente (no hace nada, ratatui se adapta solo)
    pub fn with_size(self, _width: usize, _height: usize) -> Self {
        self
    }

    /// Setear valor de campo
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

    /// Obtener valor de campo
    pub fn get_field_value(&self, field_name: &str) -> Option<&str> {
        self.field_states.get(field_name).map(|s| s.value.as_str())
    }

    /// Validar campo
    pub fn validate_field(&mut self, field_name: &str) -> FormRenderResult<()> {
        let field = self
            .form
            .fields
            .get(field_name)
            .ok_or_else(|| FormRenderError::FieldNotFound(field_name.to_string()))?
            .clone();

        let state = self
            .field_states
            .get_mut(field_name)
            .ok_or_else(|| FormRenderError::FieldNotFound(field_name.to_string()))?;

        // Validar con FormValidator
        match self.validator.validate_field(&field, &state.value) {
            Ok(_) => {
                state.valid = true;
                state.errors.clear();
            }
            Err(error) => {
                state.valid = false;
                state.errors = vec![error.to_string()];
            }
        }

        Ok(())
    }

    /// Validar todos los campos
    pub fn validate_all(&mut self) -> FormRenderResult<()> {
        let mut all_errors = Vec::new();

        for field_name in self.field_order.clone() {
            if let Err(FormRenderError::ValidationErrors(errors)) = self.validate_field(&field_name)
            {
                all_errors.extend(errors);
            }
        }

        if all_errors.is_empty() {
            Ok(())
        } else {
            Err(FormRenderError::ValidationErrors(all_errors))
        }
    }

    /// Obtener todos los valores
    pub fn get_values(&self) -> HashMap<String, String> {
        self.field_states
            .iter()
            .map(|(k, v)| (k.clone(), v.value.clone()))
            .collect()
    }

    /// Navegar al siguiente campo
    pub fn focus_next(&mut self) {
        // Desenfocar campo actual
        if let Some(field_name) = self.field_order.get(self.focused_field_index) {
            if let Some(state) = self.field_states.get_mut(field_name) {
                state.focused = false;
            }
        }

        // Mover índice
        self.focused_field_index = (self.focused_field_index + 1) % self.field_order.len();

        // Enfocar nuevo campo
        if let Some(field_name) = self.field_order.get(self.focused_field_index) {
            if let Some(state) = self.field_states.get_mut(field_name) {
                state.focused = true;
            }
        }
    }

    /// Navegar al campo anterior
    pub fn focus_previous(&mut self) {
        // Desenfocar campo actual
        if let Some(field_name) = self.field_order.get(self.focused_field_index) {
            if let Some(state) = self.field_states.get_mut(field_name) {
                state.focused = false;
            }
        }

        // Mover índice (wrap around)
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

    /// Renderizar formulario usando Ratatui (nuevo método)
    pub fn render(&self, frame: &mut Frame, area: Rect) {
        // Layout principal: header, fields, footer
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Header con título
                Constraint::Min(3),    // Campos
                Constraint::Length(3), // Botones
                Constraint::Length(1), // Ayuda
            ])
            .split(area);

        // Header
        self.render_header(frame, chunks[0]);

        // Campos
        self.render_fields(frame, chunks[1]);

        // Botones
        self.render_actions(frame, chunks[2]);

        // Ayuda
        self.render_help(frame, chunks[3]);
    }

    /// Renderizar header
    fn render_header(&self, frame: &mut Frame, area: Rect) {
        let title = &self.form.title;
        let desc = self.form.description.as_deref().unwrap_or("");

        let text = if !desc.is_empty() {
            Text::from(vec![
                Line::from(Span::styled(
                    title,
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                )),
                Line::from(Span::styled(desc, Style::default().fg(Color::Gray))),
            ])
        } else {
            Text::from(Line::from(Span::styled(
                title,
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )))
        };

        let block = Block::default()
            .borders(Borders::ALL)
            .style(Style::default());

        let paragraph = Paragraph::new(text).block(block).wrap(Wrap { trim: true });

        frame.render_widget(paragraph, area);
    }

    /// Renderizar campos
    fn render_fields(&self, frame: &mut Frame, area: Rect) {
        let mut items = Vec::new();

        for (i, field_name) in self.field_order.iter().enumerate() {
            if i < self.scroll_offset {
                continue;
            }

            if let (Some(field), Some(state)) = (
                self.form.fields.get(field_name),
                self.field_states.get(field_name),
            ) {
                // Línea del label
                let focus_marker = if state.focused { "▶" } else { " " };
                let required_marker = if field.required { "*" } else { " " };

                let label_style = if state.focused {
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                };

                let label_line = Line::from(vec![
                    Span::raw(format!("{} ", focus_marker)),
                    Span::styled(format!("{}{}: ", required_marker, field.label), label_style),
                ]);

                items.push(ListItem::new(label_line));

                // Línea del valor
                let value_display = if state.value.is_empty() {
                    "<empty>".to_string()
                } else {
                    match field.field_type {
                        FieldType::Password => "•".repeat(state.value.len()),
                        _ => state.value.clone(),
                    }
                };

                let value_style = if state.focused {
                    Style::default().fg(Color::Green)
                } else if !state.valid {
                    Style::default().fg(Color::Red)
                } else {
                    Style::default()
                };

                let value_prefix = if state.focused { "[" } else { " " };
                let value_suffix = if state.focused { "]" } else { " " };

                let value_line = Line::from(vec![
                    Span::raw("  "),
                    Span::raw(value_prefix),
                    Span::styled(value_display, value_style),
                    Span::raw(value_suffix),
                ]);

                items.push(ListItem::new(value_line));

                // Errores si existen
                for error in &state.errors {
                    let error_line = Line::from(vec![
                        Span::raw("  "),
                        Span::styled(format!("❌ {}", error), Style::default().fg(Color::Red)),
                    ]);
                    items.push(ListItem::new(error_line));
                }

                // Espacio entre campos
                items.push(ListItem::new(Line::from("")));
            }
        }

        let list = List::new(items).block(Block::default().borders(Borders::ALL).title("Fields"));

        frame.render_widget(list, area);
    }

    /// Renderizar botones de acción
    fn render_actions(&self, frame: &mut Frame, area: Rect) {
        let buttons: Vec<String> = self
            .form
            .actions
            .keys()
            .map(|name| format!("[ {} ]", name.to_uppercase()))
            .collect();

        let text = Text::from(Line::from(Span::styled(
            buttons.join("  "),
            Style::default().fg(Color::Cyan),
        )));

        let paragraph = Paragraph::new(text).block(Block::default().borders(Borders::ALL));

        frame.render_widget(paragraph, area);
    }

    /// Renderizar línea de ayuda
    fn render_help(&self, frame: &mut Frame, area: Rect) {
        let help_text = " TAB=Next | Shift+TAB=Prev | ENTER=Submit | ESC=Cancel";

        let text = Text::from(Line::from(Span::styled(
            help_text,
            Style::default().fg(Color::Gray),
        )));

        let paragraph = Paragraph::new(text);
        frame.render_widget(paragraph, area);
    }

    /// Renderizar como String para compatibilidad con código antiguo (preview mode)
    pub fn render_to_string(&self) -> String {
        let mut output = String::new();

        // Título
        output.push_str(&format!("┌─ {} ", self.form.title));
        output.push_str(&"─".repeat(60));
        output.push_str("┐\n");

        // Descripción
        if let Some(desc) = &self.form.description {
            output.push_str(&format!("│ {} ", desc));
            output.push_str(&" ".repeat(60));
            output.push_str("│\n");
            output.push_str(&format!("├{}┤\n", "─".repeat(78)));
        }

        // Campos
        for field_name in &self.field_order {
            if let (Some(field), Some(state)) = (
                self.form.fields.get(field_name),
                self.field_states.get(field_name),
            ) {
                let focus = if state.focused { "▶" } else { " " };
                let req = if field.required { "*" } else { " " };
                output.push_str(&format!("│ {}{} {}:\n", focus, req, field.label));

                let value = if state.value.is_empty() {
                    "<empty>".to_string()
                } else {
                    match field.field_type {
                        FieldType::Password => "•".repeat(state.value.len()),
                        _ => state.value.clone(),
                    }
                };

                let brackets = if state.focused {
                    ("[", "]")
                } else {
                    (" ", " ")
                };
                output.push_str(&format!("│  {}{}{}\n", brackets.0, value, brackets.1));

                for error in &state.errors {
                    output.push_str(&format!("│  ❌ {}\n", error));
                }
            }
        }

        // Botones
        output.push_str(&format!("├{}┤\n", "─".repeat(78)));
        let buttons: Vec<String> = self
            .form
            .actions
            .keys()
            .map(|k| format!("[ {} ]", k.to_uppercase()))
            .collect();
        output.push_str(&format!("│  {}\n", buttons.join("  ")));
        output.push_str(&format!("└{}┘\n", "─".repeat(78)));
        output.push_str(" TAB=Next Field | ENTER=Submit | ESC=Cancel\n");

        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use noctra_formlib::{FieldType, Form, FormField};

    fn create_test_form() -> Form {
        let mut fields = HashMap::new();
        fields.insert(
            "name".to_string(),
            FormField {
                label: "Name".to_string(),
                field_type: FieldType::Text,
                required: true,
                width: None,
                default: None,
                validations: None,
            },
        );
        fields.insert(
            "email".to_string(),
            FormField {
                label: "Email".to_string(),
                field_type: FieldType::Email,
                required: false,
                width: None,
                default: None,
                validations: None,
            },
        );

        Form {
            title: "Test Form".to_string(),
            schema: None,
            description: Some("A test form".to_string()),
            fields,
            actions: HashMap::new(),
            ui_config: None,
            pagination: None,
        }
    }

    #[test]
    fn test_form_renderer_creation() {
        let form = create_test_form();
        let renderer = FormRenderer::new(form);
        assert_eq!(renderer.field_order.len(), 2);
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

        let first_field = renderer.get_focused_field().unwrap().to_string();
        assert!(first_field == "name" || first_field == "email");

        renderer.focus_next();
        let second_field = renderer.get_focused_field().unwrap();
        assert_ne!(first_field, second_field);
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

    #[test]
    fn test_render_to_string() {
        let form = create_test_form();
        let renderer = FormRenderer::new(form);
        let output = renderer.render_to_string();
        assert!(output.contains("Test Form"));
        assert!(output.contains("Name"));
        assert!(output.contains("Email"));
    }
}
