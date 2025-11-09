//! Widgets básicos para TUI
//!
//! Widgets reutilizables como botones, labels, inputs y otros elementos
//! básicos de interfaz de usuario.

/// Estilo de texto
#[derive(Debug, Clone, Default)]
pub struct TextStyle {
    /// Color del texto
    pub foreground_color: Option<String>,

    /// Color de fondo
    pub background_color: Option<String>,

    /// Negrita
    pub bold: bool,

    /// Cursiva
    pub italic: bool,

    /// Subrayado
    pub underline: bool,
}

/// Widget base
pub trait Widget {
    /// Renderizar widget como string
    fn render(&self) -> String;

    /// Obtener tamaño del widget
    fn get_size(&self) -> (usize, usize);

    /// Verificar si está enfocado
    fn is_focused(&self) -> bool;
}

/// Label/texto estático
pub struct Label {
    /// Texto del label
    text: String,

    /// Estilo del texto
    style: TextStyle,

    /// Ancho (para alineación)
    width: Option<usize>,

    /// Alineación horizontal
    alignment: TextAlignment,
}

#[derive(Debug, Clone)]
pub enum TextAlignment {
    Left,
    Center,
    Right,
}

impl Label {
    /// Crear nuevo label
    pub fn new<T: Into<String>>(text: T) -> Self {
        Self {
            text: text.into(),
            style: TextStyle::default(),
            width: None,
            alignment: TextAlignment::Left,
        }
    }

    /// Crear label con estilo
    pub fn styled<T: Into<String>>(text: T, style: TextStyle) -> Self {
        Self {
            text: text.into(),
            style,
            width: None,
            alignment: TextAlignment::Left,
        }
    }

    /// Establecer ancho
    pub fn with_width(mut self, width: usize) -> Self {
        self.width = Some(width);
        self
    }

    /// Establecer alineación
    pub fn with_alignment(mut self, alignment: TextAlignment) -> Self {
        self.alignment = alignment;
        self
    }
}

impl Widget for Label {
    fn render(&self) -> String {
        let mut text = self.text.clone();

        // Aplicar ancho si está especificado
        if let Some(width) = self.width {
            if text.len() > width {
                text = text[..width].to_string();
            } else if text.len() < width {
                match self.alignment {
                    TextAlignment::Left => {
                        text.push_str(&" ".repeat(width - text.len()));
                    }
                    TextAlignment::Center => {
                        let left_spaces = (width - text.len()) / 2;
                        let right_spaces = width - text.len() - left_spaces;
                        text = format!(
                            "{}{}{}",
                            " ".repeat(left_spaces),
                            text,
                            " ".repeat(right_spaces)
                        );
                    }
                    TextAlignment::Right => {
                        text = format!("{}{}", " ".repeat(width - text.len()), text);
                    }
                }
            }
        }

        // Aplicar estilo simple (solo para demo)
        if self.style.bold {
            text = format!("**{}**", text);
        }

        text
    }

    fn get_size(&self) -> (usize, usize) {
        let width = self.width.unwrap_or(self.text.len());
        let height = 1;
        (height, width)
    }

    fn is_focused(&self) -> bool {
        false // Los labels nunca tienen focus
    }
}

/// Botón interactivo
pub struct Button {
    /// Texto del botón
    text: String,

    /// Componente activo
    focused: bool,

    /// Callback al hacer click
    on_click: Option<Box<dyn Fn()>>,

    /// Estilo del botón
    style: ButtonStyle,
}

#[derive(Debug, Clone)]
pub struct ButtonStyle {
    /// Texto cuando está inactivo
    pub normal_text: String,

    /// Texto cuando está enfocado
    pub focused_text: String,

    /// Caracteres de borde
    pub border_char: char,

    /// Padding interno
    pub padding: usize,
}

impl Default for ButtonStyle {
    fn default() -> Self {
        Self {
            normal_text: "[ {} ]".to_string(),
            focused_text: "[⚡ {} ⚡]".to_string(),
            border_char: '═',
            padding: 2,
        }
    }
}

impl Button {
    /// Crear nuevo botón
    pub fn new<T: Into<String>>(text: T) -> Self {
        Self {
            text: text.into(),
            focused: false,
            on_click: None,
            style: ButtonStyle::default(),
        }
    }

    /// Crear botón con callback
    pub fn with_callback<T: Into<String>, F: Fn() + 'static>(text: T, callback: F) -> Self {
        Self {
            text: text.into(),
            focused: false,
            on_click: Some(Box::new(callback)),
            style: ButtonStyle::default(),
        }
    }

    /// Establecer callback
    pub fn on_click<F: Fn() + 'static>(mut self, callback: F) -> Self {
        self.on_click = Some(Box::new(callback));
        self
    }

    /// Hacer click en el botón
    pub fn click(&self) {
        if let Some(callback) = &self.on_click {
            callback();
        }
    }
}

impl Widget for Button {
    fn render(&self) -> String {
        let template = if self.focused {
            &self.style.focused_text
        } else {
            &self.style.normal_text
        };
        template.replace("{}", &self.text)
    }

    fn get_size(&self) -> (usize, usize) {
        let width = self.text.len() + self.style.padding * 2 + 4; // +4 para los brackets
        let height = 1;
        (height, width)
    }

    fn is_focused(&self) -> bool {
        self.focused
    }
}

/// Campo de entrada de texto
pub struct TextInput {
    /// Valor actual del input
    value: String,

    /// Placeholder cuando está vacío
    placeholder: String,

    /// Posición del cursor
    cursor_position: usize,

    /// Campo enfocado
    focused: bool,

    /// Máxima longitud
    max_length: Option<usize>,

    /// Flag de solo lectura
    read_only: bool,

    /// Caracteres enmascarados (para passwords)
    masked_char: Option<char>,
}

impl TextInput {
    /// Crear nuevo campo de texto
    pub fn new() -> Self {
        Self {
            value: String::new(),
            placeholder: "".to_string(),
            cursor_position: 0,
            focused: false,
            max_length: None,
            read_only: false,
            masked_char: None,
        }
    }

    /// Crear con placeholder
    pub fn with_placeholder<T: Into<String>>(placeholder: T) -> Self {
        Self {
            placeholder: placeholder.into(),
            ..Self::new()
        }
    }

    /// Establecer valor
    pub fn set_value(&mut self, value: &str) {
        if !self.read_only {
            self.value = value.to_string();
            self.cursor_position = self.cursor_position.min(self.value.len());
        }
    }

    /// Obtener valor
    pub fn get_value(&self) -> &str {
        &self.value
    }

    /// Agregar carácter
    pub fn add_char(&mut self, c: char) {
        if !self.read_only {
            if let Some(max_len) = self.max_length {
                if self.value.len() >= max_len {
                    return;
                }
            }

            self.value.insert(self.cursor_position, c);
            self.cursor_position += 1;
        }
    }

    /// Eliminar carácter
    pub fn delete_char(&mut self) {
        if !self.read_only && self.cursor_position > 0 {
            self.cursor_position -= 1;
            self.value.remove(self.cursor_position);
        }
    }

    /// Eliminar carácter hacia adelante
    pub fn delete_forward(&mut self) {
        if !self.read_only && self.cursor_position < self.value.len() {
            self.value.remove(self.cursor_position);
        }
    }

    /// Mover cursor
    pub fn move_cursor(&mut self, delta: isize) {
        let new_pos = self.cursor_position.saturating_add_signed(delta);
        self.cursor_position = new_pos.min(self.value.len());
    }

    /// Limpiar campo
    pub fn clear(&mut self) {
        if !self.read_only {
            self.value.clear();
            self.cursor_position = 0;
        }
    }
}

impl Default for TextInput {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for TextInput {
    fn render(&self) -> String {
        let display_value = if self.value.is_empty() {
            &self.placeholder
        } else if let Some(mask_char) = self.masked_char {
            &mask_char.to_string().repeat(self.value.len())
        } else {
            &self.value
        };

        if self.focused {
            format!("[{}]", display_value)
        } else {
            display_value.to_string()
        }
    }

    fn get_size(&self) -> (usize, usize) {
        let width = self.value.len().max(self.placeholder.len()) + 4; // +4 para brackets
        let height = 1;
        (height, width)
    }

    fn is_focused(&self) -> bool {
        self.focused
    }
}

/// Panel contenedor
pub struct Panel {
    /// Widgets contenidos
    widgets: Vec<Box<dyn Widget>>,

    /// Título del panel
    title: Option<String>,

    /// Dimensiones del panel
    width: usize,
    height: usize,

    /// Padding interno
    padding: usize,

    /// Widget enfocado
    focused_widget: Option<usize>,
}

impl Panel {
    /// Crear nuevo panel
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            widgets: Vec::new(),
            title: None,
            width,
            height,
            padding: 1,
            focused_widget: None,
        }
    }

    /// Crear con título
    pub fn with_title<T: Into<String>>(mut self, title: T) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Agregar widget (builder pattern)
    pub fn add_widget<T: Widget + 'static>(mut self, widget: T) -> Self {
        self.widgets.push(Box::new(widget));
        self
    }

    /// Agregar widget (mutable)
    pub fn add_widget_mut<T: Widget + 'static>(&mut self, widget: T) {
        self.widgets.push(Box::new(widget));
    }

    /// Establecer widget enfocado
    pub fn set_focused_widget(&mut self, index: usize) {
        if index < self.widgets.len() {
            self.focused_widget = Some(index);
        }
    }

    /// Obtener widget enfocado
    pub fn get_focused_widget(&mut self) -> Option<&mut Box<dyn Widget>> {
        if let Some(index) = self.focused_widget {
            self.widgets.get_mut(index)
        } else {
            None
        }
    }

    /// Establecer ancho
    pub fn set_width(&mut self, width: usize) {
        self.width = width;
    }

    /// Establecer alto
    pub fn set_height(&mut self, height: usize) {
        self.height = height;
    }

    /// Obtener ancho
    pub fn width(&self) -> usize {
        self.width
    }

    /// Obtener alto
    pub fn height(&self) -> usize {
        self.height
    }
}

impl Widget for Panel {
    fn render(&self) -> String {
        let mut output = String::new();

        // Título si existe
        if let Some(title) = &self.title {
            let title_line = format!("=== {} ===", title);
            output.push_str(&title_line);
            output.push('\n');
        }

        // Renderizar widgets
        for widget in &self.widgets {
            output.push_str(&widget.render());
            output.push('\n');
        }

        // Padding inferior
        for _ in 0..self.padding {
            output.push('\n');
        }

        output
    }

    fn get_size(&self) -> (usize, usize) {
        (self.height, self.width)
    }

    fn is_focused(&self) -> bool {
        false // Panel no tiene focus individual
    }
}

/// Lista de opciones
pub struct OptionList {
    /// Opciones disponibles
    options: Vec<String>,

    /// Opción seleccionada
    selected_index: usize,

    /// Opción enfocada
    focused: bool,

    /// Mostrar números de opción
    show_numbers: bool,

    /// Scroll offset
    scroll_offset: usize,
}

impl OptionList {
    /// Crear nueva lista
    pub fn new(options: Vec<String>) -> Self {
        Self {
            options,
            selected_index: 0,
            focused: false,
            show_numbers: false,
            scroll_offset: 0,
        }
    }

    /// Crear con números
    pub fn with_numbers(mut self, show_numbers: bool) -> Self {
        self.show_numbers = show_numbers;
        self
    }

    /// Seleccionar opción
    pub fn select(&mut self, index: usize) {
        if index < self.options.len() {
            self.selected_index = index;
        }
    }

    /// Obtener opción seleccionada
    pub fn get_selected(&self) -> Option<&String> {
        self.options.get(self.selected_index)
    }

    /// Mover selección
    pub fn move_selection(&mut self, delta: isize) {
        let new_index = self.selected_index.saturating_add_signed(delta);
        self.selected_index = new_index.min(self.options.len().saturating_sub(1));
    }

    /// Obtener número de opciones
    pub fn len(&self) -> usize {
        self.options.len()
    }

    /// Verificar si la lista está vacía
    pub fn is_empty(&self) -> bool {
        self.options.is_empty()
    }
}

impl Widget for OptionList {
    fn render(&self) -> String {
        let mut output = String::new();

        for (i, option) in self.options.iter().enumerate() {
            if i < self.scroll_offset {
                continue; // Skip items above scroll
            }

            let marker = if i == self.selected_index {
                if self.focused {
                    "⚡ "
                } else {
                    "> "
                }
            } else {
                "  "
            };

            let number = if self.show_numbers {
                format!("{}. ", i + 1)
            } else {
                String::new()
            };

            output.push_str(&format!("{}{}{}\n", marker, number, option));
        }

        output
    }

    fn get_size(&self) -> (usize, usize) {
        let height = self.options.len().saturating_sub(self.scroll_offset);
        let width = self.options.iter().map(|opt| opt.len()).max().unwrap_or(0);
        (height, width)
    }

    fn is_focused(&self) -> bool {
        self.focused
    }
}
