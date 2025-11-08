//! Noctra TUI - Interfaz principal del sistema
//!
//! Implementación del TUI completo con Ratatui según especificaciones de Noctra.
//! Incluye layout fijo, modos de trabajo y gestión de comandos SQL/RQL.

use crossterm::{
    event::{self, Event, KeyCode, KeyEvent},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table},
    Frame, Terminal,
};
use std::io::{stdout, Stdout};
use std::sync::Arc;
use std::time::Duration;
use tui_textarea::{Input, TextArea};

// Backend integration
use noctra_core::{Executor, ResultSet, Session};

use crate::nwm::UiMode;

/// Estado del TUI de Noctra
pub struct NoctraTui<'a> {
    /// Terminal de Ratatui
    terminal: Terminal<CrosstermBackend<Stdout>>,

    /// Backend executor para ejecutar SQL
    executor: Arc<Executor>,

    /// Sesión de usuario con variables y estado
    session: Session,

    /// Modo actual de la interfaz
    mode: UiMode,

    /// Editor de comandos (para modo Command)
    command_editor: TextArea<'a>,

    /// Historial de comandos ejecutados
    command_history: Vec<String>,

    /// Número de comando actual
    command_number: usize,

    /// Índice en el historial
    history_index: Option<usize>,

    /// Resultados SQL (para modo Result)
    current_results: Option<QueryResults>,

    /// Mensaje de diálogo (para modo Dialog)
    dialog_message: Option<String>,

    /// Opciones de diálogo
    dialog_options: Vec<String>,

    /// Opción seleccionada en diálogo
    dialog_selected: usize,

    /// Flag para salir del TUI
    should_quit: bool,
}

/// Resultados de una query SQL
#[derive(Debug, Clone)]
pub struct QueryResults {
    /// Columnas
    pub columns: Vec<String>,

    /// Filas de datos
    pub rows: Vec<Vec<String>>,

    /// Mensaje de estado
    pub status: String,
}

impl<'a> NoctraTui<'a> {
    /// Crear nueva instancia del TUI con base de datos en memoria
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let executor = Executor::new_sqlite_memory()?;
        Self::with_executor(Arc::new(executor))
    }

    /// Crear TUI con base de datos desde archivo
    pub fn with_database<P: AsRef<str>>(db_path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let executor = Executor::new_sqlite_file(db_path.as_ref())?;
        Self::with_executor(Arc::new(executor))
    }

    /// Crear TUI con executor personalizado
    fn with_executor(executor: Arc<Executor>) -> Result<Self, Box<dyn std::error::Error>> {
        // Configurar terminal
        enable_raw_mode()?;
        let mut stdout = stdout();
        execute!(stdout, EnterAlternateScreen)?;

        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend)?;

        // Crear editor de comandos
        let mut command_editor = TextArea::default();
        command_editor.set_block(
            Block::default()
                .borders(Borders::NONE)
                .style(Style::default()),
        );
        command_editor.set_cursor_line_style(Style::default());
        command_editor.set_cursor_style(Style::default().add_modifier(Modifier::REVERSED));

        // Crear sesión
        let session = Session::new();

        Ok(Self {
            terminal,
            executor,
            session,
            mode: UiMode::Command,
            command_editor,
            command_history: Vec::new(),
            command_number: 1,
            history_index: None,
            current_results: None,
            dialog_message: None,
            dialog_options: Vec::new(),
            dialog_selected: 0,
            should_quit: false,
        })
    }

    /// Ejecutar el TUI principal
    pub fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        while !self.should_quit {
            // Renderizar
            let mode = self.mode;
            let command_number = self.command_number;
            let current_results = self.current_results.clone();
            let dialog_message = self.dialog_message.clone();
            let dialog_options = self.dialog_options.clone();
            let dialog_selected = self.dialog_selected;

            self.terminal.draw(|frame| {
                Self::render_frame(
                    frame,
                    mode,
                    command_number,
                    &mut self.command_editor,
                    current_results.as_ref(),
                    dialog_message.as_deref(),
                    &dialog_options,
                    dialog_selected,
                );
            })?;

            // Procesar eventos
            if event::poll(Duration::from_millis(100))? {
                if let Event::Key(key) = event::read()? {
                    self.handle_key_event(key)?;
                }
            }
        }

        // Limpiar terminal
        self.cleanup()?;
        Ok(())
    }

    /// Renderizar la interfaz completa (método estático)
    #[allow(clippy::too_many_arguments)]
    fn render_frame(
        frame: &mut Frame,
        mode: UiMode,
        command_number: usize,
        command_editor: &mut TextArea,
        current_results: Option<&QueryResults>,
        dialog_message: Option<&str>,
        dialog_options: &[String],
        dialog_selected: usize,
    ) {
        let size = frame.area();

        // Layout principal: Header + Workspace + Separator + Shortcuts
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Header
                Constraint::Min(10),   // Workspace (área dinámica)
                Constraint::Length(1), // Separator
                Constraint::Length(7), // Shortcuts bar
            ])
            .split(size);

        // Renderizar componentes
        Self::render_header(frame, chunks[0], mode, command_number);
        Self::render_workspace(
            frame,
            chunks[1],
            mode,
            command_editor,
            current_results,
            dialog_message,
            dialog_options,
            dialog_selected,
        );
        Self::render_separator(frame, chunks[2]);
        Self::render_shortcuts(frame, chunks[3]);
    }

    /// Renderizar barra de header
    fn render_header(frame: &mut Frame, area: Rect, mode: UiMode, command_number: usize) {
        let mode_text = match mode {
            UiMode::Command => "INSERTAR",
            UiMode::Result => "RESULTADO",
            UiMode::Form => "FORMULARIO",
            UiMode::Dialog => "DIÁLOGO",
        };

        let header_text = format!("──( {} ) SQL Noctra 0.1.0", mode_text);

        let cmd_text = format!("Cmd: {}───", command_number);

        // Calcular padding para alinear a la derecha
        let padding_len = area
            .width
            .saturating_sub(header_text.len() as u16 + cmd_text.len() as u16);
        let padding = "─".repeat(padding_len as usize);

        let full_header = format!("{}{}{}", header_text, padding, cmd_text);

        let header = Paragraph::new(full_header)
            .style(
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )
            .alignment(Alignment::Left);

        frame.render_widget(header, area);
    }

    /// Renderizar área de trabajo (cambia según el modo)
    #[allow(clippy::too_many_arguments)]
    fn render_workspace(
        frame: &mut Frame,
        area: Rect,
        mode: UiMode,
        command_editor: &mut TextArea,
        current_results: Option<&QueryResults>,
        dialog_message: Option<&str>,
        dialog_options: &[String],
        dialog_selected: usize,
    ) {
        match mode {
            UiMode::Command => Self::render_command_mode(frame, area, command_editor),
            UiMode::Result => Self::render_result_mode(frame, area, current_results),
            UiMode::Dialog => Self::render_dialog_mode(
                frame,
                area,
                dialog_message,
                dialog_options,
                dialog_selected,
            ),
            UiMode::Form => Self::render_form_mode(frame, area),
        }
    }

    /// Renderizar modo Command (editor de SQL)
    fn render_command_mode(frame: &mut Frame, area: Rect, command_editor: &TextArea) {
        frame.render_widget(command_editor, area);
    }

    /// Renderizar modo Result (tabla de resultados)
    fn render_result_mode(frame: &mut Frame, area: Rect, current_results: Option<&QueryResults>) {
        if let Some(results) = current_results {
            // Crear tabla con bordes ASCII
            let header_cells = results.columns.iter().map(|col| {
                Cell::from(col.as_str()).style(Style::default().add_modifier(Modifier::BOLD))
            });

            let header = Row::new(header_cells)
                .style(Style::default().fg(Color::Yellow))
                .height(1);

            let rows = results.rows.iter().map(|row| {
                let cells = row.iter().map(|cell| Cell::from(cell.as_str()));
                Row::new(cells).height(1)
            });

            // Calcular ancho de columnas automáticamente
            let col_widths: Vec<Constraint> = results
                .columns
                .iter()
                .map(|_| Constraint::Percentage((100 / results.columns.len().max(1)) as u16))
                .collect();

            let table = Table::new(rows, col_widths)
                .header(header)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(Color::Green)),
                )
                .style(Style::default().fg(Color::White));

            frame.render_widget(table, area);

            // Mostrar mensaje de estado debajo
            let status_area = Rect {
                y: area.y + area.height.saturating_sub(2),
                height: 1,
                ..area
            };

            let status =
                Paragraph::new(results.status.as_str()).style(Style::default().fg(Color::Gray));

            frame.render_widget(status, status_area);
        } else {
            let empty = Paragraph::new("No hay resultados para mostrar")
                .style(Style::default().fg(Color::Gray))
                .alignment(Alignment::Center);

            frame.render_widget(empty, area);
        }
    }

    /// Renderizar modo Dialog (confirmaciones)
    fn render_dialog_mode(
        frame: &mut Frame,
        area: Rect,
        dialog_message: Option<&str>,
        dialog_options: &[String],
        dialog_selected: usize,
    ) {
        if let Some(message) = dialog_message {
            // Calcular tamaño de la ventana modal
            let dialog_width = 60.min(area.width);
            let dialog_height = 8.min(area.height);

            let dialog_area = Rect {
                x: (area.width.saturating_sub(dialog_width)) / 2,
                y: (area.height.saturating_sub(dialog_height)) / 2,
                width: dialog_width,
                height: dialog_height,
            };

            // Fondo del diálogo
            let dialog_bg = Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Yellow))
                .style(Style::default().bg(Color::Black));

            frame.render_widget(dialog_bg, dialog_area);

            // Contenido interno
            let inner = dialog_area.inner(ratatui::layout::Margin::new(2, 1));

            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(2), // Mensaje
                    Constraint::Length(1), // Espacio
                    Constraint::Length(3), // Botones
                ])
                .split(inner);

            // Mensaje
            let msg = Paragraph::new(message)
                .alignment(Alignment::Center)
                .style(Style::default().fg(Color::White));

            frame.render_widget(msg, chunks[0]);

            // Botones
            let button_layout = Layout::default()
                .direction(Direction::Horizontal)
                .constraints(
                    dialog_options
                        .iter()
                        .map(|_| Constraint::Percentage((100 / dialog_options.len().max(1)) as u16))
                        .collect::<Vec<_>>(),
                )
                .split(chunks[2]);

            for (i, option) in dialog_options.iter().enumerate() {
                let style = if i == dialog_selected {
                    Style::default()
                        .fg(Color::Black)
                        .bg(Color::Yellow)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::Yellow)
                };

                let button = Paragraph::new(format!("  {}  ", option))
                    .alignment(Alignment::Center)
                    .style(style)
                    .block(Block::default().borders(Borders::ALL));

                frame.render_widget(button, button_layout[i]);
            }
        }
    }

    /// Renderizar modo Form (pendiente de implementación)
    fn render_form_mode(frame: &mut Frame, area: Rect) {
        let placeholder = Paragraph::new("Modo formulario - En desarrollo")
            .style(Style::default().fg(Color::Gray))
            .alignment(Alignment::Center);

        frame.render_widget(placeholder, area);
    }

    /// Renderizar línea separadora
    fn render_separator(frame: &mut Frame, area: Rect) {
        let separator = Paragraph::new("─".repeat(area.width as usize))
            .style(Style::default().fg(Color::DarkGray));

        frame.render_widget(separator, area);
    }

    /// Renderizar barra de shortcuts
    fn render_shortcuts(frame: &mut Frame, area: Rect) {
        let shortcuts = vec![
            ("F5", "Procesar comando"),
            ("End", "Terminar sesión de Noctra"),
            ("F1", "Ayuda comandos editor"),
            ("F8", "Interrumpir procesamiento"),
            ("Prox. pantal", "Comando siguiente"),
            ("Pantall. pre", "Comando anterior"),
            ("Insert", "Insertar espacio"),
            ("Delete", "Borrar un carácter"),
            ("Alt+r", "Leer desde archivo"),
            ("Alt+w", "Grabar en archivo"),
        ];

        let lines: Vec<Line> = shortcuts
            .chunks(2)
            .map(|chunk| {
                let mut spans = Vec::new();
                for (key, desc) in chunk {
                    spans.push(Span::styled(
                        format!("{:<15}", key),
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD),
                    ));
                    spans.push(Span::raw(":"));
                    spans.push(Span::styled(
                        format!("{:<35}", desc),
                        Style::default().fg(Color::White),
                    ));
                }
                Line::from(spans)
            })
            .collect();

        let shortcuts_widget = Paragraph::new(lines).style(Style::default().fg(Color::White));

        frame.render_widget(shortcuts_widget, area);
    }

    /// Manejar eventos de teclado
    fn handle_key_event(&mut self, key: KeyEvent) -> Result<(), Box<dyn std::error::Error>> {
        match self.mode {
            UiMode::Command => self.handle_command_keys(key)?,
            UiMode::Result => self.handle_result_keys(key)?,
            UiMode::Dialog => self.handle_dialog_keys(key)?,
            UiMode::Form => self.handle_form_keys(key)?,
        }
        Ok(())
    }

    /// Manejar teclas en modo Command
    fn handle_command_keys(&mut self, key: KeyEvent) -> Result<(), Box<dyn std::error::Error>> {
        match key.code {
            KeyCode::F(5) => {
                // Ejecutar comando
                self.execute_command()?;
            }
            KeyCode::End => {
                // Mostrar diálogo de salida
                self.show_exit_dialog();
            }
            KeyCode::PageDown => {
                // Comando siguiente en historial
                self.next_command();
            }
            KeyCode::PageUp => {
                // Comando anterior en historial
                self.previous_command();
            }
            _ => {
                // Pasar la tecla al editor
                self.command_editor.input(Input::from(key));
            }
        }
        Ok(())
    }

    /// Manejar teclas en modo Result
    fn handle_result_keys(&mut self, key: KeyEvent) -> Result<(), Box<dyn std::error::Error>> {
        match key.code {
            KeyCode::Esc | KeyCode::Char('q') => {
                // Volver a modo Command
                self.mode = UiMode::Command;
            }
            KeyCode::End => {
                self.show_exit_dialog();
            }
            _ => {}
        }
        Ok(())
    }

    /// Manejar teclas en modo Dialog
    fn handle_dialog_keys(&mut self, key: KeyEvent) -> Result<(), Box<dyn std::error::Error>> {
        match key.code {
            KeyCode::Left => {
                if self.dialog_selected > 0 {
                    self.dialog_selected -= 1;
                }
            }
            KeyCode::Right => {
                if self.dialog_selected < self.dialog_options.len().saturating_sub(1) {
                    self.dialog_selected += 1;
                }
            }
            KeyCode::Enter => {
                // Ejecutar acción según la opción seleccionada
                if self.dialog_options[self.dialog_selected] == "SI" {
                    self.should_quit = true;
                } else {
                    // Cancelar - volver a Command
                    self.mode = UiMode::Command;
                    self.dialog_message = None;
                }
            }
            KeyCode::Esc => {
                // Cancelar
                self.mode = UiMode::Command;
                self.dialog_message = None;
            }
            _ => {}
        }
        Ok(())
    }

    /// Manejar teclas en modo Form
    fn handle_form_keys(&mut self, key: KeyEvent) -> Result<(), Box<dyn std::error::Error>> {
        // TODO: Implementar cuando tengamos formularios integrados
        if key.code == KeyCode::Esc {
            self.mode = UiMode::Command;
        }
        Ok(())
    }

    /// Convertir ResultSet de noctra-core a QueryResults del TUI
    fn convert_result_set(&self, result_set: ResultSet, command: &str) -> QueryResults {
        // Extraer nombres de columnas
        let columns: Vec<String> = result_set
            .columns
            .iter()
            .map(|col| col.name.clone())
            .collect();

        // Convertir valores a strings usando Display trait
        let rows: Vec<Vec<String>> = result_set
            .rows
            .iter()
            .map(|row| row.values.iter().map(|value| value.to_string()).collect())
            .collect();

        // Construir mensaje de estado
        let status = if let Some(affected) = result_set.rows_affected {
            // Para INSERT/UPDATE/DELETE
            if let Some(rowid) = result_set.last_insert_rowid {
                format!(
                    "{} fila(s) afectada(s) - Último ID insertado: {} - Comando: {}",
                    affected,
                    rowid,
                    command.trim()
                )
            } else {
                format!(
                    "{} fila(s) afectada(s) - Comando: {}",
                    affected,
                    command.trim()
                )
            }
        } else {
            // Para SELECT
            let row_count = result_set.row_count();
            if row_count == 0 {
                format!("Sin resultados - Comando: {}", command.trim())
            } else {
                format!(
                    "{} fila(s) retornada(s) - Comando: {}",
                    row_count,
                    command.trim()
                )
            }
        };

        QueryResults {
            columns,
            rows,
            status,
        }
    }

    /// Ejecutar comando SQL actual
    fn execute_command(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let command_text = self.command_editor.lines().join("\n");

        if command_text.trim().is_empty() {
            return Ok(());
        }

        // Agregar al historial
        self.command_history.push(command_text.clone());
        self.command_number += 1;

        // ✨ EJECUTAR SQL REAL
        match self.executor.execute_sql(&self.session, &command_text) {
            Ok(result_set) => {
                // Convertir ResultSet a QueryResults
                self.current_results = Some(self.convert_result_set(result_set, &command_text));

                // Cambiar a modo Result
                self.mode = UiMode::Result;
            }
            Err(e) => {
                // Mostrar error en Dialog Mode
                self.show_error_dialog(&format!("❌ Error SQL: {}", e));
            }
        }

        // Limpiar editor para próximo comando
        self.clear_command_editor();

        Ok(())
    }

    /// Limpiar el editor de comandos
    fn clear_command_editor(&mut self) {
        self.command_editor = TextArea::default();
        self.command_editor
            .set_block(Block::default().borders(Borders::NONE));
        self.command_editor.set_cursor_line_style(Style::default());
        self.command_editor
            .set_cursor_style(Style::default().add_modifier(Modifier::REVERSED));
    }

    /// Mostrar diálogo de error
    fn show_error_dialog(&mut self, message: &str) {
        self.dialog_message = Some(message.to_string());
        self.dialog_options = vec!["OK".to_string()];
        self.dialog_selected = 0;
        self.mode = UiMode::Dialog;
    }

    /// Mostrar diálogo de confirmación de salida
    fn show_exit_dialog(&mut self) {
        self.dialog_message = Some("¿Desea terminar la sesión de Noctra?".to_string());
        self.dialog_options = vec!["SI".to_string(), "NO".to_string(), "CANCELAR".to_string()];
        self.dialog_selected = 1; // Default: NO
        self.mode = UiMode::Dialog;
    }

    /// Navegar al siguiente comando en historial
    fn next_command(&mut self) {
        if let Some(idx) = self.history_index {
            if idx < self.command_history.len().saturating_sub(1) {
                self.history_index = Some(idx + 1);
                self.load_command_from_history();
            }
        }
    }

    /// Navegar al comando anterior en historial
    fn previous_command(&mut self) {
        if let Some(idx) = self.history_index {
            if idx > 0 {
                self.history_index = Some(idx - 1);
                self.load_command_from_history();
            }
        } else if !self.command_history.is_empty() {
            self.history_index = Some(self.command_history.len() - 1);
            self.load_command_from_history();
        }
    }

    /// Cargar comando del historial al editor
    fn load_command_from_history(&mut self) {
        if let Some(idx) = self.history_index {
            if let Some(cmd) = self.command_history.get(idx) {
                self.command_editor = TextArea::from(cmd.lines());
                self.command_editor
                    .set_block(Block::default().borders(Borders::NONE));
            }
        }
    }

    /// Limpiar y restaurar terminal
    fn cleanup(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        disable_raw_mode()?;
        execute!(self.terminal.backend_mut(), LeaveAlternateScreen)?;
        self.terminal.show_cursor()?;
        Ok(())
    }
}

impl<'a> Drop for NoctraTui<'a> {
    fn drop(&mut self) {
        let _ = self.cleanup();
    }
}
