//! Interactive Form Executor con Ratatui
//!
//! Ejecutor de formularios con TUI interactivo usando ratatui + crossterm.

use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io::{stdout, Stdout};
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

/// Ejecutor de formularios interactivo con Ratatui
pub struct InteractiveFormExecutor {
    renderer: FormRenderer,
    terminal: Terminal<CrosstermBackend<Stdout>>,
    running: bool,
}

impl InteractiveFormExecutor {
    /// Crear nuevo ejecutor
    pub fn new(form: Form) -> InteractiveResult<Self> {
        // Crear renderer (ratatui se adapta automáticamente al tamaño)
        let renderer = FormRenderer::new(form);

        // Configurar terminal
        enable_raw_mode().map_err(|e| InteractiveError::TerminalError(e.to_string()))?;
        let mut stdout = stdout();
        execute!(stdout, EnterAlternateScreen)
            .map_err(|e| InteractiveError::TerminalError(e.to_string()))?;

        let backend = CrosstermBackend::new(stdout);
        let terminal =
            Terminal::new(backend).map_err(|e| InteractiveError::TerminalError(e.to_string()))?;

        Ok(Self {
            renderer,
            terminal,
            running: true,
        })
    }

    /// Ejecutar formulario de manera interactiva
    pub fn run(&mut self) -> InteractiveResult<Option<std::collections::HashMap<String, String>>> {
        // Loop principal
        let result = self.run_loop();

        // Limpiar terminal
        self.cleanup_terminal()?;

        result
    }

    /// Limpiar terminal
    fn cleanup_terminal(&mut self) -> InteractiveResult<()> {
        disable_raw_mode().map_err(|e| InteractiveError::TerminalError(e.to_string()))?;
        execute!(self.terminal.backend_mut(), LeaveAlternateScreen)
            .map_err(|e| InteractiveError::TerminalError(e.to_string()))?;
        self.terminal
            .show_cursor()
            .map_err(|e| InteractiveError::TerminalError(e.to_string()))?;
        Ok(())
    }

    /// Loop principal de eventos
    fn run_loop(&mut self) -> InteractiveResult<Option<std::collections::HashMap<String, String>>> {
        while self.running {
            // Renderizar usando ratatui
            self.terminal
                .draw(|frame| {
                    self.renderer.render(frame, frame.size());
                })
                .map_err(|e| InteractiveError::TerminalError(e.to_string()))?;

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
                    Event::Resize(_width, _height) => {
                        // Ratatui maneja el resize automáticamente
                    }
                    _ => {}
                }
            }
        }

        // Retornar valores si se submitió
        if self.running {
            Ok(None) // Cancelado
        } else {
            // Validar antes de retornar
            match self.renderer.validate_all() {
                Ok(_) => Ok(Some(self.renderer.get_values())),
                Err(_) => {
                    // Hay errores de validación, no retornar valores
                    Ok(None)
                }
            }
        }
    }

    /// Manejar evento de teclado
    fn handle_key_event(&mut self, event: KeyEvent) -> bool {
        match event.code {
            // ESC - Cancelar
            KeyCode::Esc => {
                self.running = false;
                false
            }

            // Tab - Siguiente campo
            KeyCode::Tab => {
                if event.modifiers.contains(KeyModifiers::SHIFT) {
                    self.renderer.focus_previous();
                } else {
                    self.renderer.focus_next();
                }
                true
            }

            // Enter - Submit formulario
            KeyCode::Enter => {
                // Intentar validar
                match self.renderer.validate_all() {
                    Ok(_) => {
                        // Válido, salir
                        self.running = false;
                        false
                    }
                    Err(_) => {
                        // Errores de validación, continuar editando
                        true
                    }
                }
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
                true
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
                true
            }

            _ => true,
        }
    }
}

impl Drop for InteractiveFormExecutor {
    fn drop(&mut self) {
        // Asegurar que el terminal se limpie incluso si hay pánico
        let _ = disable_raw_mode();
        let _ = execute!(self.terminal.backend_mut(), LeaveAlternateScreen);
        let _ = self.terminal.show_cursor();
    }
}
