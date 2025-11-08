//! Noctra Window Manager (NWM)
//!
//! Sistema de gestión de ventanas jerárquico para terminal.
//! Proporciona modos de interfaz (Command, Result, Form, Dialog)
//! y un stack de ventanas para navegación.

use std::collections::VecDeque;
use thiserror::Error;

use crate::widgets::Widget;
use noctra_core::ResultSet;
use noctra_formlib::{Form, GraphNavigator};

/// Error del NWM
#[derive(Error, Debug)]
pub enum NwmError {
    /// Ventana no encontrada
    #[error("Ventana '{0}' no encontrada")]
    WindowNotFound(String),

    /// Stack de ventanas vacío
    #[error("Stack de ventanas vacío")]
    EmptyWindowStack,

    /// Modo inválido
    #[error("Modo inválido: {0}")]
    InvalidMode(String),

    /// Error de renderizado
    #[error("Error de renderizado: {0}")]
    RenderError(String),
}

/// Resultado de operaciones NWM
pub type NwmResult<T> = Result<T, NwmError>;

/// Modo de interfaz de usuario
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UiMode {
    /// Modo comando (REPL)
    Command,

    /// Modo resultado (visualización de tablas)
    Result,

    /// Modo formulario (entrada de datos)
    Form,

    /// Modo diálogo (mensajes, confirmaciones)
    Dialog,
}

impl UiMode {
    /// Obtener descripción del modo
    pub fn description(&self) -> &'static str {
        match self {
            UiMode::Command => "Command Mode - Interactive REPL",
            UiMode::Result => "Result Mode - Data Display",
            UiMode::Form => "Form Mode - Data Entry",
            UiMode::Dialog => "Dialog Mode - Messages",
        }
    }

    /// Obtener icono del modo
    pub fn icon(&self) -> &'static str {
        match self {
            UiMode::Command => ">_",
            UiMode::Result => "📊",
            UiMode::Form => "📝",
            UiMode::Dialog => "💬",
        }
    }
}

/// Una ventana en el NWM
pub struct NwmWindow {
    /// ID único de la ventana
    pub id: String,

    /// Título de la ventana
    pub title: String,

    /// Modo de la ventana
    pub mode: UiMode,

    /// Contenido de la ventana
    pub content: WindowContent,

    /// Metadata adicional
    pub metadata: std::collections::HashMap<String, String>,
}

impl std::fmt::Debug for NwmWindow {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NwmWindow")
            .field("id", &self.id)
            .field("title", &self.title)
            .field("mode", &self.mode)
            .field("metadata", &self.metadata)
            .finish()
    }
}

/// Contenido de una ventana
pub enum WindowContent {
    /// Contenido de texto plano
    Text(String),

    /// ResultSet de base de datos
    ResultSet(ResultSet),

    /// Formulario FDL2
    Form(Form),

    /// Widget personalizado
    Widget(Box<dyn Widget>),

    /// Contenido vacío
    Empty,
}

impl std::fmt::Debug for WindowContent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WindowContent::Text(text) => f.debug_tuple("Text").field(text).finish(),
            WindowContent::ResultSet(_) => f.debug_tuple("ResultSet").finish(),
            WindowContent::Form(form) => f.debug_tuple("Form").field(&form.title).finish(),
            WindowContent::Widget(_) => f.debug_tuple("Widget").finish(),
            WindowContent::Empty => f.debug_tuple("Empty").finish(),
        }
    }
}

impl NwmWindow {
    /// Crear nueva ventana
    pub fn new(id: String, title: String, mode: UiMode, content: WindowContent) -> Self {
        Self {
            id,
            title,
            mode,
            content,
            metadata: std::collections::HashMap::new(),
        }
    }

    /// Crear ventana de comando
    pub fn command(id: String, title: String) -> Self {
        Self::new(id, title, UiMode::Command, WindowContent::Empty)
    }

    /// Crear ventana de resultado
    pub fn result(id: String, title: String, result_set: ResultSet) -> Self {
        Self::new(
            id,
            title,
            UiMode::Result,
            WindowContent::ResultSet(result_set),
        )
    }

    /// Crear ventana de formulario
    pub fn form(id: String, title: String, form: Form) -> Self {
        Self::new(id, title, UiMode::Form, WindowContent::Form(form))
    }

    /// Crear ventana de diálogo
    pub fn dialog(id: String, title: String, message: String) -> Self {
        Self::new(id, title, UiMode::Dialog, WindowContent::Text(message))
    }

    /// Agregar metadata
    pub fn with_metadata(mut self, key: String, value: String) -> Self {
        self.metadata.insert(key, value);
        self
    }
}

/// Noctra Window Manager
pub struct NoctraWindowManager {
    /// Stack de ventanas (LIFO)
    window_stack: VecDeque<NwmWindow>,

    /// Configuración del NWM
    config: NwmConfig,

    /// Navegador de grafo (opcional)
    navigator: Option<GraphNavigator>,

    /// Historial de IDs de ventanas cerradas (para referencia)
    closed_history: Vec<String>,

    /// Límite de historial
    max_history: usize,
}

/// Configuración del NWM
#[derive(Debug, Clone)]
pub struct NwmConfig {
    /// Mostrar breadcrumbs
    pub show_breadcrumbs: bool,

    /// Mostrar barra de estado
    pub show_status_bar: bool,

    /// Altura de header
    pub header_height: usize,

    /// Altura de footer
    pub footer_height: usize,

    /// Tema visual
    pub theme: String,

    /// Tamaño mínimo de ventana (ancho, alto)
    pub min_window_size: (usize, usize),
}

impl Default for NwmConfig {
    fn default() -> Self {
        Self {
            show_breadcrumbs: true,
            show_status_bar: true,
            header_height: 3,
            footer_height: 2,
            theme: "default".to_string(),
            min_window_size: (80, 24),
        }
    }
}

impl NoctraWindowManager {
    /// Crear nuevo NWM
    pub fn new(config: NwmConfig) -> Self {
        Self {
            window_stack: VecDeque::new(),
            config,
            navigator: None,
            closed_history: Vec::new(),
            max_history: 10,
        }
    }

    /// Crear con configuración por defecto
    pub fn default() -> Self {
        Self::new(NwmConfig::default())
    }

    /// Establecer navegador de grafo
    pub fn with_navigator(mut self, navigator: GraphNavigator) -> Self {
        self.navigator = Some(navigator);
        self
    }

    /// Obtener ventana actual
    pub fn current_window(&self) -> NwmResult<&NwmWindow> {
        self.window_stack
            .back()
            .ok_or(NwmError::EmptyWindowStack)
    }

    /// Obtener ventana actual (mutable)
    pub fn current_window_mut(&mut self) -> NwmResult<&mut NwmWindow> {
        self.window_stack
            .back_mut()
            .ok_or(NwmError::EmptyWindowStack)
    }

    /// Agregar ventana al stack (push)
    pub fn push_window(&mut self, window: NwmWindow) {
        self.window_stack.push_back(window);
    }

    /// Remover ventana del stack (pop)
    pub fn pop_window(&mut self) -> NwmResult<NwmWindow> {
        let window = self
            .window_stack
            .pop_back()
            .ok_or(NwmError::EmptyWindowStack)?;

        // Agregar ID al historial
        if self.closed_history.len() >= self.max_history {
            self.closed_history.remove(0);
        }
        self.closed_history.push(window.id.clone());

        Ok(window)
    }

    /// Cerrar ventana actual y volver a la anterior
    pub fn close_current_window(&mut self) -> NwmResult<()> {
        if self.window_stack.len() > 1 {
            self.pop_window()?;
            Ok(())
        } else {
            Err(NwmError::WindowNotFound(
                "No se puede cerrar la última ventana".to_string(),
            ))
        }
    }

    /// Reemplazar ventana actual
    pub fn replace_window(&mut self, window: NwmWindow) -> NwmResult<()> {
        if !self.window_stack.is_empty() {
            self.pop_window()?;
        }
        self.push_window(window);
        Ok(())
    }

    /// Limpiar stack y agregar ventana raíz
    pub fn reset_to_window(&mut self, window: NwmWindow) {
        self.window_stack.clear();
        self.window_stack.push_back(window);
    }

    /// Obtener número de ventanas en el stack
    pub fn window_count(&self) -> usize {
        self.window_stack.len()
    }

    /// Verificar si hay ventanas
    pub fn is_empty(&self) -> bool {
        self.window_stack.is_empty()
    }

    /// Obtener breadcrumb de navegación
    pub fn get_breadcrumb(&self) -> Vec<String> {
        self.window_stack
            .iter()
            .map(|w| w.title.clone())
            .collect()
    }

    /// Renderizar layout completo
    pub fn render_layout(&self, terminal_size: (usize, usize)) -> NwmResult<String> {
        let (term_height, term_width) = terminal_size;

        if term_width < self.config.min_window_size.0
            || term_height < self.config.min_window_size.1
        {
            return Err(NwmError::RenderError(format!(
                "Terminal muy pequeño: {}x{} (mínimo {}x{})",
                term_width,
                term_height,
                self.config.min_window_size.0,
                self.config.min_window_size.1
            )));
        }

        let mut output = String::new();

        // Header
        output.push_str(&self.render_header(term_width)?);

        // Main area (ventana actual)
        let main_height = term_height
            .saturating_sub(self.config.header_height)
            .saturating_sub(self.config.footer_height);

        output.push_str(&self.render_main_area(term_width, main_height)?);

        // Footer
        output.push_str(&self.render_footer(term_width)?);

        Ok(output)
    }

    /// Renderizar header
    fn render_header(&self, width: usize) -> NwmResult<String> {
        let mut output = String::new();

        // Línea superior
        output.push_str(&"═".repeat(width));
        output.push('\n');

        // Breadcrumb
        if self.config.show_breadcrumbs {
            let breadcrumb = self.get_breadcrumb().join(" > ");
            let truncated = if breadcrumb.len() > width - 4 {
                format!("...{}", &breadcrumb[breadcrumb.len() - (width - 7)..])
            } else {
                breadcrumb
            };

            output.push_str(&format!("  {}  ", truncated));
            output.push_str(&" ".repeat(width.saturating_sub(truncated.len() + 4)));
            output.push('\n');
        }

        // Línea inferior del header
        output.push_str(&"─".repeat(width));
        output.push('\n');

        Ok(output)
    }

    /// Renderizar área principal
    fn render_main_area(&self, width: usize, height: usize) -> NwmResult<String> {
        let window = self.current_window()?;

        let mut output = String::new();

        // Título de la ventana
        let title_line = format!(" {} {} - {} ", window.mode.icon(), window.title, window.mode.description());
        output.push_str(&title_line);
        output.push_str(&" ".repeat(width.saturating_sub(title_line.len())));
        output.push('\n');

        // Contenido (simplificado por ahora)
        match &window.content {
            WindowContent::Text(text) => {
                for line in text.lines().take(height - 2) {
                    let truncated = if line.len() > width {
                        &line[..width]
                    } else {
                        line
                    };
                    output.push_str(truncated);
                    output.push_str(&" ".repeat(width.saturating_sub(truncated.len())));
                    output.push('\n');
                }
            }

            WindowContent::ResultSet(result_set) => {
                // Renderizado simplificado de ResultSet
                output.push_str(&format!(
                    " ResultSet: {} rows, {} columns\n",
                    result_set.rows.len(),
                    result_set.columns.len()
                ));
            }

            WindowContent::Form(form) => {
                // Renderizado simplificado de Form
                output.push_str(&format!(" Form: {}\n", form.title));
                output.push_str(&format!(" Fields: {}\n", form.fields.len()));
            }

            WindowContent::Widget(_) => {
                output.push_str(" [Custom Widget]\n");
            }

            WindowContent::Empty => {
                output.push_str(" [Empty Window]\n");
            }
        }

        // Rellenar espacio restante
        let lines_written = output.lines().count();
        for _ in lines_written..height {
            output.push_str(&" ".repeat(width));
            output.push('\n');
        }

        Ok(output)
    }

    /// Renderizar footer
    fn render_footer(&self, width: usize) -> NwmResult<String> {
        let mut output = String::new();

        // Línea superior del footer
        output.push_str(&"─".repeat(width));
        output.push('\n');

        // Status bar
        if self.config.show_status_bar {
            let window_info = format!(
                " Windows: {} | Mode: {:?} ",
                self.window_count(),
                self.current_window()?.mode
            );

            let help_text = " F1=Help | F10=Exit | ESC=Back ";

            let padding_width = width
                .saturating_sub(window_info.len())
                .saturating_sub(help_text.len());

            output.push_str(&window_info);
            output.push_str(&" ".repeat(padding_width));
            output.push_str(help_text);
            output.push('\n');
        }

        Ok(output)
    }

    /// Obtener referencia al navegador
    pub fn navigator(&self) -> Option<&GraphNavigator> {
        self.navigator.as_ref()
    }

    /// Obtener referencia mutable al navegador
    pub fn navigator_mut(&mut self) -> Option<&mut GraphNavigator> {
        self.navigator.as_mut()
    }

    /// Obtener configuración
    pub fn config(&self) -> &NwmConfig {
        &self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ui_mode() {
        assert_eq!(UiMode::Command.description(), "Command Mode - Interactive REPL");
        assert_eq!(UiMode::Result.icon(), "📊");
    }

    #[test]
    fn test_window_creation() {
        let window = NwmWindow::command("cmd1".to_string(), "Command Window".to_string());
        assert_eq!(window.id, "cmd1");
        assert_eq!(window.mode, UiMode::Command);
    }

    #[test]
    fn test_nwm_stack() {
        let mut nwm = NoctraWindowManager::default();

        assert!(nwm.is_empty());

        let window1 = NwmWindow::command("w1".to_string(), "Window 1".to_string());
        nwm.push_window(window1);

        assert_eq!(nwm.window_count(), 1);

        let window2 = NwmWindow::command("w2".to_string(), "Window 2".to_string());
        nwm.push_window(window2);

        assert_eq!(nwm.window_count(), 2);

        let current = nwm.current_window().unwrap();
        assert_eq!(current.id, "w2");

        nwm.pop_window().unwrap();
        assert_eq!(nwm.window_count(), 1);
    }

    #[test]
    fn test_breadcrumb() {
        let mut nwm = NoctraWindowManager::default();

        nwm.push_window(NwmWindow::command("root".to_string(), "Root".to_string()));
        nwm.push_window(NwmWindow::form("forms".to_string(), "Forms".to_string(), noctra_formlib::Form {
            title: "Test".to_string(),
            schema: None,
            description: None,
            fields: std::collections::HashMap::new(),
            actions: std::collections::HashMap::new(),
            ui_config: None,
            pagination: None,
        }));

        let breadcrumb = nwm.get_breadcrumb();
        assert_eq!(breadcrumb, vec!["Root", "Forms"]);
    }
}
