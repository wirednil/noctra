//! Componentes TUI principales
//! 
//! Componentes reutilizables para formularios, tablas y navegación
//! en la interfaz de usuario terminal.

use noctra_core::{ResultSet, Value};
use noctra_formlib::{Form, FormField};
use std::collections::HashMap;

/// Evento de componente TUI
#[derive(Debug, Clone)]
pub enum ComponentEvent {
    /// Activación de componente
    Activate,
    
    /// Escape/ cancelar
    Cancel,
    
    /// Cambio de valor
    ValueChanged(String),
    
    /// Cambio de campo
    FieldChanged(String),
    
    /// Acción personalizada
    Custom(String),
}

/// Resultado de interacción con componente
#[derive(Debug, Clone)]
pub struct ComponentResult {
    /// Evento generado
    pub event: ComponentEvent,
    
    /// Datos resultantes
    pub data: Option<HashMap<String, String>>,
    
    /// Flag de completion
    pub completed: bool,
}

/// Trait base para todos los componentes TUI
pub trait Component {
    /// Obtener dimensiones del componente
    fn get_size(&self) -> (usize, usize);
    
    /// Renderizar componente
    fn render(&self) -> String;
    
    /// Procesar evento
    fn handle_event(&mut self, event: crossterm::event::Event) -> ComponentResult;
    
    /// Verificar si está activo
    fn is_focused(&self) -> bool;
    
    /// Establecer focus
    fn set_focused(&mut self, focused: bool);
}

/// Componente de tabla para mostrar resultados SQL
pub struct TableComponent {
    /// Result set a mostrar
    result_set: ResultSet,
    
    /// Posición actual (fila, columna)
    cursor_position: (usize, usize),
    
    /// Offset de scroll
    scroll_offset: (usize, usize),
    
    /// Componente enfocado
    focused: bool,
    
    /// Altura máxima para paginación
    max_height: usize,
    
    /// Ancho máximo para paginación
    max_width: usize,
}

impl TableComponent {
    /// Crear nueva tabla
    pub fn new(result_set: ResultSet, max_height: usize, max_width: usize) -> Self {
        Self {
            result_set,
            cursor_position: (0, 0),
            scroll_offset: (0, 0),
            focused: false,
            max_height,
            max_width,
        }
    }
    
    /// Obtener fila actual
    pub fn get_current_row(&self) -> Option<&Row> {
        self.result_set.rows.get(self.cursor_position.0)
    }
    
    /// Mover cursor
    pub fn move_cursor(&mut self, delta_row: isize, delta_col: isize) {
        let new_row = self.cursor_position.0.saturating_add_signed(delta_row);
        let new_col = self.cursor_position.1.saturating_add_signed(delta_col);
        
        if new_row < self.result_set.row_count() {
            self.cursor_position.0 = new_row;
        }
        
        if new_col < self.result_set.column_count() {
            self.cursor_position.1 = new_col;
        }
    }
    
    /// Hacer scroll
    pub fn scroll(&mut self, delta_row: isize, delta_col: isize) {
        let new_scroll_row = self.scroll_offset.0.saturating_add_signed(delta_row);
        let new_scroll_col = self.scroll_offset.1.saturating_add_signed(delta_col);
        
        self.scroll_offset.0 = new_scroll_row;
        self.scroll_offset.1 = new_scroll_col;
    }
    
    /// Obtener datos de la fila seleccionada
    pub fn get_selected_row_data(&self) -> Option<HashMap<String, String>> {
        self.get_current_row().map(|row| {
            let mut data = HashMap::new();
            
            for (i, column) in self.result_set.columns.iter().enumerate() {
                if let Some(value) = row.values.get(i) {
                    data.insert(column.name.clone(), value.to_string());
                }
            }
            
            data
        })
    }
}

impl Component for TableComponent {
    fn get_size(&self) -> (usize, usize) {
        (self.max_height, self.max_width)
    }
    
    fn render(&self) -> String {
        let mut output = String::new();
        
        // Header con columnas
        if !self.result_set.columns.is_empty() {
            output.push_str("┌");
            
            let column_widths: Vec<usize> = self.result_set.columns.iter()
                .map(|col| {
                    let max_content_width = self.result_set.rows.iter()
                        .filter_map(|row| row.get(col.ordinal))
                        .map(|v| v.to_string().len())
                        .max()
                        .unwrap_or(col.name.len());
                    
                    col.name.len().max(max_content_width).min(20)
                })
                .collect();
            
            for (i, width) in column_widths.iter().enumerate() {
                if i > 0 {
                    output.push_str("┬");
                }
                output.push_str(&"─".repeat(*width + 2));
            }
            output.push_str("┐\n");
            
            // Headers
            output.push_str("│");
            for (i, (col, width)) in self.result_set.columns.iter().zip(column_widths.iter()).enumerate() {
                if i > 0 {
                    output.push_str("│");
                }
                let display_text = &col.name[..col.name.len().min(*width)];
                output.push_str(&format!(" {:width$} ", display_text, width = *width));
            }
            output.push_str("│\n");
            
            // Separator después del header
            output.push_str("├");
            for (i, width) in column_widths.iter().enumerate() {
                if i > 0 {
                    output.push_str("┼");
                }
                output.push_str(&"─".repeat(*width + 2));
            }
            output.push_str("┤\n");
            
            // Filas de datos
            let visible_rows = self.result_set.rows.iter()
                .skip(self.scroll_offset.0)
                .take(self.max_height.saturating_sub(3)) // Reservar espacio para header y footer
                .collect::<Vec<_>>();
            
            for row in visible_rows {
                output.push_str("│");
                for (i, (col, width)) in self.result_set.columns.iter().zip(column_widths.iter()).enumerate() {
                    if i > 0 {
                        output.push_str("│");
                    }
                    
                    let value_str = row.get(col.ordinal)
                        .map(|v| v.to_string())
                        .unwrap_or_else(|| "NULL".to_string());
                    
                    let display_text = &value_str[..value_str.len().min(*width)];
                    output.push_str(&format!(" {:width$} ", display_text, width = *width));
                }
                output.push_str("│\n");
            }
            
            // Footer
            output.push_str("└");
            for (i, width) in column_widths.iter().enumerate() {
                if i > 0 {
                    output.push_str("┴");
                }
                output.push_str(&"─".repeat(*width + 2));
            }
            output.push_str("┘\n");
        } else {
            output.push_str("No hay datos para mostrar\n");
        }
        
        // Info de status
        if self.focused {
            output.push_str(&format!("Fila: {}/{} | Col: {}/{} | Renglón: {}", 
                self.cursor_position.0 + 1, 
                self.result_set.row_count(),
                self.cursor_position.1 + 1,
                self.result_set.column_count(),
                self.scroll_offset.0 + 1));
        } else {
            output.push_str(&format!("({} filas, {} columnas)", 
                self.result_set.row_count(),
                self.result_set.column_count()));
        }
        
        output
    }
    
    fn handle_event(&mut self, event: crossterm::event::Event) -> ComponentResult {
        match event {
            crossterm::event::Event::Key(key_event) => match key_event.code {
                crossterm::event::KeyCode::Up => {
                    self.move_cursor(-1, 0);
                    ComponentResult {
                        event: ComponentEvent::ValueChanged(format!("{},{}", self.cursor_position.0, self.cursor_position.1)),
                        data: self.get_selected_row_data(),
                        completed: false,
                    }
                }
                crossterm::event::KeyCode::Down => {
                    self.move_cursor(1, 0);
                    ComponentResult {
                        event: ComponentEvent::ValueChanged(format!("{},{}", self.cursor_position.0, self.cursor_position.1)),
                        data: self.get_selected_row_data(),
                        completed: false,
                    }
                }
                crossterm::event::KeyCode::Left => {
                    self.move_cursor(0, -1);
                    ComponentResult {
                        event: ComponentEvent::ValueChanged(format!("{},{}", self.cursor_position.0, self.cursor_position.1)),
                        data: None,
                        completed: false,
                    }
                }
                crossterm::event::KeyCode::Right => {
                    self.move_cursor(0, 1);
                    ComponentResult {
                        event: ComponentEvent::ValueChanged(format!("{},{}", self.cursor_position.0, self.cursor_position.1)),
                        data: None,
                        completed: false,
                    }
                }
                crossterm::event::KeyCode::Enter => {
                    ComponentResult {
                        event: ComponentEvent::Activate,
                        data: self.get_selected_row_data(),
                        completed: true,
                    }
                }
                crossterm::event::KeyCode::Esc => {
                    ComponentResult {
                        event: ComponentEvent::Cancel,
                        data: None,
                        completed: true,
                    }
                }
                _ => ComponentResult {
                    event: ComponentEvent::Custom("key_pressed".to_string()),
                    data: None,
                    completed: false,
                }
            },
            _ => ComponentResult {
                event: ComponentEvent::Custom("unknown_event".to_string()),
                data: None,
                completed: false,
            }
        }
    }
    
    fn is_focused(&self) -> bool {
        self.focused
    }
    
    fn set_focused(&mut self, focused: bool) {
        self.focused = focused;
    }
}

/// Componente de formulario
pub struct FormComponent {
    /// Formulario a mostrar
    form: Form,
    
    /// Valores actuales de campos
    field_values: HashMap<String, String>,
    
    /// Campo activo
    active_field: Option<String>,
    
    /// Componente enfocado
    focused: bool,
    
    /// Posición del cursor en el campo activo
    cursor_position: usize,
}

impl FormComponent {
    /// Crear nuevo formulario
    pub fn new(form: Form) -> Self {
        Self {
            form,
            field_values: HashMap::new(),
            active_field: None,
            focused: false,
            cursor_position: 0,
        }
    }
    
    /// Establecer valor de campo
    pub fn set_field_value(&mut self, field_name: &str, value: String) {
        self.field_values.insert(field_name.to_string(), value);
    }
    
    /// Obtener valor de campo
    pub fn get_field_value(&self, field_name: &str) -> Option<&String> {
        self.field_values.get(field_name)
    }
    
    /// Obtener siguiente campo navegable
    pub fn get_next_field(&self, current_field: Option<&str>) -> Option<&String> {
        let fields: Vec<&String> = self.form.fields.keys().collect();
        
        if let Some(current) = current_field {
            if let Some(current_index) = fields.iter().position(|&f| f == current) {
                let next_index = (current_index + 1) % fields.len();
                return Some(fields[next_index]);
            }
        }
        
        fields.first().cloned()
    }
    
    /// Obtener campos requeridos con valor faltante
    pub fn get_missing_required_fields(&self) -> Vec<String> {
        self.form.fields.iter()
            .filter(|(name, field)| field.required && !self.field_values.contains_key(*name))
            .map(|(name, _)| name.clone())
            .collect()
    }
    
    /// Validar y obtener datos del formulario
    pub fn get_form_data(&self) -> Result<HashMap<String, String>, Vec<String>> {
        let missing_fields = self.get_missing_required_fields();
        
        if !missing_fields.is_empty() {
            Err(missing_fields)
        } else {
            Ok(self.field_values.clone())
        }
    }
}

impl Component for FormComponent {
    fn get_size(&self) -> (usize, usize) {
        // Estimar tamaño basado en número de campos
        let height = self.form.fields.len().saturating_add(5);
        let width = 80; // Ancho fijo para formularios
        
        (height, width)
    }
    
    fn render(&self) -> String {
        let mut output = String::new();
        
        // Título del formulario
        output.push_str(&format!("=== {} ===\n", self.form.title));
        
        if let Some(description) = &self.form.description {
            output.push_str(&format!("{}\n", description));
            output.push_str(&"=".repeat(self.form.title.len().max(description.len()).saturating_add(4)));
            output.push('\n');
        }
        
        // Campos del formulario
        for (field_name, field) in &self.form.fields {
            let is_active = self.active_field.as_ref() == Some(field_name);
            let value = self.field_values.get(field_name).unwrap_or(&"".to_string());
            
            // Label del campo
            let label = format!("{}:", field.label);
            let required_mark = if field.required { " *" } else { "" };
            output.push_str(&format!("{}{}: ", label, required_mark));
            
            // Campo de entrada
            if is_active && self.focused {
                output.push_str(&format!("[{}]", value));
                
                // Mostrar cursor (representado como underscore)
                let cursor = " ".repeat(self.cursor_position.min(value.len()));
                let underscore = "_";
                output.push_str(&format!("{}{}", cursor, underscore));
            } else {
                output.push_str(value);
            }
            
            output.push('\n');
            
            // Mostrar validaciones fallidas
            if field.required && value.is_empty() {
                output.push_str("  ⚠ Campo requerido\n");
            }
        }
        
        // Info de ayuda
        output.push_str("\nControles:\n");
        output.push_str("  Tab/Enter: Siguiente campo\n");
        output.push_str("  Esc: Cancelar\n");
        output.push_str("  Ctrl+S: Enviar formulario\n");
        
        if self.focused {
            output.push_str("\n⚡ Formulario activo");
        }
        
        output
    }
    
    fn handle_event(&mut self, event: crossterm::event::Event) -> ComponentResult {
        match event {
            crossterm::event::Event::Key(key_event) => match key_event.code {
                crossterm::event::KeyCode::Tab => {
                    // Cambiar al siguiente campo
                    let next_field = self.get_next_field(self.active_field.as_deref());
                    if let Some(field_name) = next_field {
                        self.active_field = Some(field_name.clone());
                        self.cursor_position = 0;
                    }
                    
                    ComponentResult {
                        event: ComponentEvent::FieldChanged(self.active_field.clone().unwrap_or_default()),
                        data: None,
                        completed: false,
                    }
                }
                crossterm::event::KeyCode::Enter => {
                    let next_field = self.get_next_field(self.active_field.as_deref());
                    if let Some(field_name) = next_field {
                        self.active_field = Some(field_name.clone());
                        self.cursor_position = 0;
                    }
                    
                    ComponentResult {
                        event: ComponentEvent::FieldChanged(self.active_field.clone().unwrap_or_default()),
                        data: None,
                        completed: false,
                    }
                }
                crossterm::event::KeyCode::Esc => {
                    ComponentResult {
                        event: ComponentEvent::Cancel,
                        data: None,
                        completed: true,
                    }
                }
                _ => {
                    // Procesar entrada de texto si hay campo activo
                    if let Some(field_name) = &self.active_field {
                        match key_event.code {
                            crossterm::event::KeyCode::Backspace => {
                                if let Some(value) = self.field_values.get_mut(field_name) {
                                    if !value.is_empty() {
                                        value.pop();
                                        self.cursor_position = self.cursor_position.saturating_sub(1);
                                    }
                                }
                            }
                            crossterm::event::KeyCode::Char(c) => {
                                let value = self.field_values.entry(field_name.clone()).or_insert_with(String::new);
                                value.push(c);
                                self.cursor_position += 1;
                            }
                            _ => {}
                        }
                        
                        ComponentResult {
                            event: ComponentEvent::ValueChanged(field_name.clone()),
                            data: None,
                            completed: false,
                        }
                    } else {
                        ComponentResult {
                            event: ComponentEvent::Custom("no_active_field".to_string()),
                            data: None,
                            completed: false,
                        }
                    }
                }
            },
            _ => ComponentResult {
                event: ComponentEvent::Custom("unknown_event".to_string()),
                data: None,
                completed: false,
            }
        }
    }
    
    fn is_focused(&self) -> bool {
        self.focused
    }
    
    fn set_focused(&mut self, focused: bool) {
        self.focused = focused;
        
        // Si se activa el focus y no hay campo activo, seleccionar el primer campo
        if focused && self.active_field.is_none() {
            if let Some(first_field) = self.form.fields.keys().next() {
                self.active_field = Some(first_field.clone());
                self.cursor_position = 0;
            }
        }
    }
}