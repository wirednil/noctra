//! Interactive Form Executor
//!
//! Ejecutor de formularios con TUI interactivo usando crossterm.

use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::io::{stdout, Write};
use std::time::Duration;

use noctra_formlib::Form;
use noctra_tui::FormRenderer;

/// Error del ejecutor interactivo
#[derive(Debug)]
pub enum InteractiveError {
    /// Error de terminal
    TerminalError(String),
    /// Error de formulario
    FormError(String),
}

impl std::fmt::Display for InteractiveError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InteractiveError::TerminalError(msg) => write!(f, "Terminal error: {}", msg),
            InteractiveError::FormError(msg) => write!(f, "Form error: {}", msg),
        }
    }
}

impl std::error::Error for InteractiveError {}

/// Resultado de ejecución interactiva
pub type InteractiveResult<T> = Result<T, InteractiveError>;

/// Ejecutor de formularios interactivo
pub struct InteractiveFormExecutor {
    renderer: FormRenderer,
    running: bool,
}

impl InteractiveFormExecutor {
    /// Crear nuevo ejecutor
    pub fn new(form: Form) -> Self {
        Self {
            renderer: FormRenderer::new(form),
            running: false,
        }
    }

    /// Ejecutar formulario de manera interactiva
    pub fn run(&mut self) -> InteractiveResult<Option<std::collections::HashMap<String, String>>> {
        // Inicializar terminal
        self.init_terminal()?;

        // Loop principal
        let result = self.run_loop();

        // Restaurar terminal
        self.cleanup_terminal()?;

        result
    }

    /// Inicializar terminal en modo raw
    fn init_terminal(&mut self) -> InteractiveResult<()> {
        enable_raw_mode().map_err(|e| InteractiveError::TerminalError(e.to_string()))?;
        execute!(stdout(), EnterAlternateScreen)
            .map_err(|e| InteractiveError::TerminalError(e.to_string()))?;
        self.running = true;
        Ok(())
    }

    /// Limpiar terminal
    fn cleanup_terminal(&mut self) -> InteractiveResult<()> {
        execute!(stdout(), LeaveAlternateScreen)
            .map_err(|e| InteractiveError::TerminalError(e.to_string()))?;
        disable_raw_mode().map_err(|e| InteractiveError::TerminalError(e.to_string()))?;
        Ok(())
    }

    /// Loop principal de eventos
    fn run_loop(&mut self) -> InteractiveResult<Option<std::collections::HashMap<String, String>>> {
        while self.running {
            // Renderizar
            self.render()?;

            // Procesar eventos
            if event::poll(Duration::from_millis(100))
                .map_err(|e| InteractiveError::TerminalError(e.to_string()))?
            {
                match event::read().map_err(|e| InteractiveError::TerminalError(e.to_string()))? {
                    Event::Key(key_event) => {
                        if !self.handle_key_event(key_event) {
                            break;
                        }
                    }
                    Event::Resize(_, _) => {
                        // Manejar resize si es necesario
                    }
                    _ => {}
                }
            }
        }

        // Retornar valores si se submitió
        if self.running {
            Ok(None) // Cancelado
        } else {
            Ok(Some(self.renderer.get_values()))
        }
    }

    /// Renderizar formulario
    fn render(&self) -> InteractiveResult<()> {
        let output = self.renderer.render();

        execute!(
            stdout(),
            crossterm::terminal::Clear(crossterm::terminal::ClearType::All),
            crossterm::cursor::MoveTo(0, 0)
        )
        .map_err(|e| InteractiveError::TerminalError(e.to_string()))?;

        print!("{}", output);
        stdout()
            .flush()
            .map_err(|e| InteractiveError::TerminalError(e.to_string()))?;

        Ok(())
    }

    /// Manejar evento de teclado
    fn handle_key_event(&mut self, event: KeyEvent) -> bool {
        match event.code {
            // ESC - Cancelar
            KeyCode::Esc => {
                self.running = false;
                return false;
            }

            // Tab - Siguiente campo
            KeyCode::Tab => {
                if event.modifiers.contains(KeyModifiers::SHIFT) {
                    self.renderer.focus_previous();
                } else {
                    self.renderer.focus_next();
                }
            }

            // Enter - Submit (por ahora)
            KeyCode::Enter => {
                // Validar formulario
                if let Ok(()) = self.renderer.validate_form() {
                    self.running = false;
                    return false;
                }
                // Si hay errores, continuar editando
            }

            // Backspace - Eliminar carácter
            KeyCode::Backspace => {
                if let Some(field_name) = self.renderer.get_focused_field().map(|s| s.to_string()) {
                    let current_value = self
                        .renderer
                        .get_field_value(&field_name)
                        .unwrap_or("")
                        .to_string();
                    if !current_value.is_empty() {
                        let new_value = current_value[..current_value.len() - 1].to_string();
                        let _ = self.renderer.set_field_value(&field_name, new_value);
                    }
                }
            }

            // Caracteres normales
            KeyCode::Char(c) => {
                if let Some(field_name) = self.renderer.get_focused_field().map(|s| s.to_string()) {
                    let current_value = self
                        .renderer
                        .get_field_value(&field_name)
                        .unwrap_or("")
                        .to_string();
                    let new_value = format!("{}{}", current_value, c);
                    let _ = self.renderer.set_field_value(&field_name, new_value);
                }
            }

            _ => {}
        }

        true
    }
}

impl Drop for InteractiveFormExecutor {
    fn drop(&mut self) {
        // Asegurar que el terminal se restaura
        let _ = self.cleanup_terminal();
    }
}
