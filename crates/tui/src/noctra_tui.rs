//! Noctra TUI - Interfaz principal del sistema
//!
//! Implementación del TUI completo con Ratatui según especificaciones de Noctra.
//! Incluye layout fijo, modos de trabajo y gestión de comandos SQL/RQL.

use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, EnableBracketedPaste, DisableBracketedPaste},
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
use std::collections::HashMap;
use std::io::{stdout, Stdout};
use std::time::Duration;
use tui_textarea::{Input, TextArea};

// Backend integration
use noctra_core::{Executor, ResultSet, Session, RqlQuery, NoctraError};
use noctra_parser::{RqlProcessor, RqlStatement};

use crate::nwm::UiMode;

/// Estado del TUI de Noctra
pub struct NoctraTui<'a> {
    /// Terminal de Ratatui
    terminal: Terminal<CrosstermBackend<Stdout>>,

    /// Backend executor para ejecutar SQL
    executor: Executor,

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

    /// Offset horizontal para scroll en tablas de resultados
    result_scroll_offset_x: usize,

    /// Offset vertical para scroll en tablas de resultados
    result_scroll_offset_y: usize,

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
        Self::with_executor(executor)
    }

    /// Crear TUI con base de datos desde archivo
    pub fn with_database<P: AsRef<str>>(db_path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let executor = Executor::new_sqlite_file(db_path.as_ref())?;
        Self::with_executor(executor)
    }

    /// Crear TUI con executor personalizado
    fn with_executor(executor: Executor) -> Result<Self, Box<dyn std::error::Error>> {
        // Configurar terminal
        enable_raw_mode()?;
        let mut stdout = stdout();
        execute!(stdout, EnterAlternateScreen, EnableBracketedPaste)?;

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
            result_scroll_offset_x: 0,
            result_scroll_offset_y: 0,
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

            // Obtener fuente activa y tabla actual
            let active_source = self.executor.source_registry()
                .active()
                .map(|source| {
                    let source_name = source.name().to_string();

                    // Si es DuckDB, intentar obtener la lista de tablas registradas
                    if source_name == "duckdb" {
                        // Try to downcast to DuckDBSource to get registered files
                        if let Ok(schema) = source.schema() {
                            let table_names: Vec<String> = schema
                                .iter()
                                .map(|table| table.name.clone())
                                .collect();
                            if !table_names.is_empty() {
                                return table_names.join(", ");
                            }
                        }
                    }

                    // Intentar extraer nombre de tabla del último resultado
                    if let Some(results) = &current_results {
                        // Extraer tabla del comando SQL (ej: "SELECT * FROM clientes")
                        if let Some(table) = Self::extract_table_name(&results.status) {
                            return format!("{}:{}", source_name, table);
                        }
                    }

                    source_name
                });

            self.terminal.draw(|frame| {
                Self::render_frame(
                    frame,
                    mode,
                    command_number,
                    &mut self.command_editor,
                    current_results.as_ref(),
                    self.result_scroll_offset_x,
                    self.result_scroll_offset_y,
                    dialog_message.as_deref(),
                    &dialog_options,
                    dialog_selected,
                    active_source.as_deref(),
                );
            })?;

            // Procesar eventos
            if event::poll(Duration::from_millis(100))? {
                match event::read()? {
                    Event::Key(key) => {
                        self.handle_key_event(key)?;
                    }
                    Event::Paste(data) => {
                        // Handle pasted text - insert it with proper line breaks
                        if matches!(self.mode, UiMode::Command) {
                            let lines: Vec<&str> = data.lines().collect();
                            for (idx, line) in lines.iter().enumerate() {
                                self.command_editor.insert_str(line);
                                // Add newline after each line except the last one
                                if idx < lines.len() - 1 {
                                    self.command_editor.insert_newline();
                                }
                            }
                        }
                    }
                    _ => {}
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
        scroll_offset_x: usize,
        scroll_offset_y: usize,
        dialog_message: Option<&str>,
        dialog_options: &[String],
        dialog_selected: usize,
        active_source: Option<&str>,
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
        Self::render_header(frame, chunks[0], mode, command_number, active_source);
        Self::render_workspace(
            frame,
            chunks[1],
            mode,
            command_editor,
            current_results,
            scroll_offset_x,
            scroll_offset_y,
            dialog_message,
            dialog_options,
            dialog_selected,
        );
        Self::render_separator(frame, chunks[2]);
        Self::render_shortcuts(frame, chunks[3]);
    }

    /// Renderizar barra de header
    fn render_header(frame: &mut Frame, area: Rect, mode: UiMode, command_number: usize, active_source: Option<&str>) {
        let mode_text = match mode {
            UiMode::Command => "INSERTAR",
            UiMode::Result => "RESULTADO",
            UiMode::Form => "FORMULARIO",
            UiMode::Dialog => "DIÁLOGO",
        };

        let header_text = format!("──( {} ) SQL Noctra 0.1.0", mode_text);

        // Agregar indicador de fuente activa si existe
        let source_text = if let Some(source_name) = active_source {
            format!(" ── Fuente: {} ──", source_name)
        } else {
            String::new()
        };

        let cmd_text = format!("Cmd: {}───", command_number);

        // Calcular padding para alinear a la derecha
        let padding_len = area
            .width
            .saturating_sub(header_text.len() as u16 + source_text.len() as u16 + cmd_text.len() as u16);
        let padding = "─".repeat(padding_len as usize);

        let full_header = format!("{}{}{}{}", header_text, source_text, padding, cmd_text);

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
        scroll_offset_x: usize,
        scroll_offset_y: usize,
        dialog_message: Option<&str>,
        dialog_options: &[String],
        dialog_selected: usize,
    ) {
        match mode {
            UiMode::Command => Self::render_command_mode(frame, area, command_editor),
            UiMode::Result => Self::render_result_mode(frame, area, current_results, scroll_offset_x, scroll_offset_y),
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

    /// Renderizar modo Result (tabla de resultados) con scroll horizontal y vertical
    fn render_result_mode(
        frame: &mut Frame,
        area: Rect,
        current_results: Option<&QueryResults>,
        scroll_offset_x: usize,
        scroll_offset_y: usize,
    ) {
        if let Some(results) = current_results {
            // Calcular ancho de columnas basado en contenido
            let mut col_widths: Vec<usize> = results.columns.iter().map(|col| col.len()).collect();

            // Actualizar anchos considerando valores de todas las filas
            for row in &results.rows {
                for (i, cell) in row.iter().enumerate() {
                    if i < col_widths.len() {
                        col_widths[i] = col_widths[i].max(cell.len());
                    }
                }
            }

            // Espacio disponible para la tabla (reservar espacio para indicadores de scroll)
            let available_width = area.width.saturating_sub(4); // bordes + indicadores
            let available_height = area.height.saturating_sub(5); // header + bordes + status + indicadores

            // Determinar qué columnas son visibles basado en el scroll horizontal
            let mut visible_columns = Vec::new();
            let mut visible_widths = Vec::new();
            let mut current_width = 0u16;
            let start_col = scroll_offset_x.min(results.columns.len().saturating_sub(1));

            for i in start_col..results.columns.len() {
                // +2 para padding, +2 para " │" separador (espacio + barra)
                let col_width = (col_widths[i] + 4).max(6) as u16;
                if current_width + col_width <= available_width {
                    visible_columns.push(i);
                    visible_widths.push(col_width);
                    current_width += col_width;
                } else {
                    break;
                }
            }

            // Si no hay columnas visibles, mostrar al menos una
            if visible_columns.is_empty() && !results.columns.is_empty() {
                visible_columns.push(start_col);
                visible_widths.push((col_widths[start_col] + 4).max(6) as u16);
            }

            // Determinar qué filas son visibles basado en el scroll vertical
            let start_row = scroll_offset_y.min(results.rows.len().saturating_sub(1));
            let end_row = (start_row + available_height as usize).min(results.rows.len());
            let visible_rows = &results.rows[start_row..end_row];

            // Calcular tamaño de la tabla visible (separadores ya incluidos en col_widths)
            let table_width = visible_widths.iter().sum::<u16>() + 2; // +2 para bordes

            let table_height = (visible_rows.len() + 3).min(area.height as usize) as u16; // +3 para header y bordes

            // Centrar la tabla
            let table_area = Rect {
                x: area.x + (area.width.saturating_sub(table_width)) / 2,
                y: area.y + (area.height.saturating_sub(table_height + 2)) / 2, // +2 para status e indicadores
                width: table_width,
                height: table_height,
            };

            // Crear header con columnas visibles y separadores verticales
            let header_cells: Vec<Cell> = visible_columns.iter().enumerate().map(|(idx, &i)| {
                let col_name = &results.columns[i];
                // Agregar separador vertical después de cada columna excepto la última
                let text = if idx < visible_columns.len() - 1 {
                    format!("{} │", col_name)
                } else {
                    col_name.to_string()
                };
                Cell::from(text)
                    .style(Style::default().add_modifier(Modifier::BOLD))
            }).collect();

            let header = Row::new(header_cells)
                .style(Style::default().fg(Color::Yellow))
                .height(1);

            // Crear filas visibles con solo las columnas visibles y separadores verticales
            let rows = visible_rows.iter().map(|row| {
                let cells: Vec<Cell> = visible_columns.iter().enumerate().map(|(idx, &i)| {
                    let cell_text = row.get(i).map(|s| s.as_str()).unwrap_or("");
                    // Agregar separador vertical después de cada columna excepto la última
                    let text = if idx < visible_columns.len() - 1 {
                        format!("{} │", cell_text)
                    } else {
                        cell_text.to_string()
                    };
                    Cell::from(text)
                }).collect();
                Row::new(cells).height(1)
            });

            // Constraints para columnas visibles
            let col_constraints: Vec<Constraint> = visible_widths
                .iter()
                .map(|&width| Constraint::Length(width))
                .collect();

            let table = Table::new(rows, col_constraints)
                .header(header)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(Color::Green)),
                )
                .style(Style::default().fg(Color::White));

            frame.render_widget(table, table_area);

            // Mostrar indicadores de scroll y estado
            let status_area = Rect {
                x: table_area.x,
                y: table_area.y + table_area.height,
                width: table_area.width,
                height: 1,
            };

            // Indicadores de scroll
            let total_cols = results.columns.len();
            let total_rows = results.rows.len();
            let has_more_left = scroll_offset_x > 0;
            let has_more_right = scroll_offset_x + visible_columns.len() < total_cols;
            let has_more_up = scroll_offset_y > 0;
            let has_more_down = end_row < total_rows;

            let scroll_indicator = format!(
                "{}Col {}-{}/{} | Fil {}-{}/{}{} | {}",
                if has_more_left { "← " } else { "" },
                scroll_offset_x + 1,
                scroll_offset_x + visible_columns.len(),
                total_cols,
                start_row + 1,
                end_row,
                total_rows,
                if has_more_right { " →" } else { "" },
                results.status
            );

            let status = Paragraph::new(scroll_indicator)
                .style(Style::default().fg(Color::Gray));

            frame.render_widget(status, status_area);

            // Indicadores visuales adicionales
            if has_more_up || has_more_down {
                let arrow_area = Rect {
                    x: table_area.x + table_area.width,
                    y: table_area.y,
                    width: 1,
                    height: table_area.height,
                };

                let arrows = if has_more_up && has_more_down {
                    "↕"
                } else if has_more_up {
                    "↑"
                } else {
                    "↓"
                };

                let arrow_widget = Paragraph::new(arrows)
                    .style(Style::default().fg(Color::Cyan));

                frame.render_widget(arrow_widget, arrow_area);
            }
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
                // Volver a modo Command y resetear scroll
                self.mode = UiMode::Command;
                self.result_scroll_offset_x = 0;
                self.result_scroll_offset_y = 0;
            }
            KeyCode::End => {
                self.show_exit_dialog();
            }
            KeyCode::Left => {
                // Scroll izquierda
                self.result_scroll_offset_x = self.result_scroll_offset_x.saturating_sub(1);
            }
            KeyCode::Right => {
                // Scroll derecha
                if let Some(results) = &self.current_results {
                    if self.result_scroll_offset_x < results.columns.len().saturating_sub(1) {
                        self.result_scroll_offset_x += 1;
                    }
                }
            }
            KeyCode::Up => {
                // Scroll arriba
                self.result_scroll_offset_y = self.result_scroll_offset_y.saturating_sub(1);
            }
            KeyCode::Down => {
                // Scroll abajo
                if let Some(results) = &self.current_results {
                    if self.result_scroll_offset_y < results.rows.len().saturating_sub(1) {
                        self.result_scroll_offset_y += 1;
                    }
                }
            }
            KeyCode::PageDown => {
                // Scroll página abajo (10 filas)
                if let Some(results) = &self.current_results {
                    self.result_scroll_offset_y = (self.result_scroll_offset_y + 10)
                        .min(results.rows.len().saturating_sub(1));
                }
            }
            KeyCode::PageUp => {
                // Scroll página arriba (10 filas)
                self.result_scroll_offset_y = self.result_scroll_offset_y.saturating_sub(10);
            }
            KeyCode::Home => {
                // Ir al inicio (primera columna)
                self.result_scroll_offset_x = 0;
            }
            KeyCode::Char('h') => {
                // Vim-style: scroll izquierda
                self.result_scroll_offset_x = self.result_scroll_offset_x.saturating_sub(1);
            }
            KeyCode::Char('l') => {
                // Vim-style: scroll derecha
                if let Some(results) = &self.current_results {
                    if self.result_scroll_offset_x < results.columns.len().saturating_sub(1) {
                        self.result_scroll_offset_x += 1;
                    }
                }
            }
            KeyCode::Char('k') => {
                // Vim-style: scroll arriba
                self.result_scroll_offset_y = self.result_scroll_offset_y.saturating_sub(1);
            }
            KeyCode::Char('j') => {
                // Vim-style: scroll abajo
                if let Some(results) = &self.current_results {
                    if self.result_scroll_offset_y < results.rows.len().saturating_sub(1) {
                        self.result_scroll_offset_y += 1;
                    }
                }
            }
            KeyCode::Char('g') => {
                // Ir al inicio de la tabla (arriba)
                self.result_scroll_offset_y = 0;
            }
            KeyCode::Char('G') => {
                // Ir al final de la tabla (abajo)
                if let Some(results) = &self.current_results {
                    self.result_scroll_offset_y = results.rows.len().saturating_sub(1);
                }
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

        // Parsear con RqlProcessor
        // Ejecutar en un thread separado para evitar conflictos con runtime de Tokio
        let cmd = command_text.clone();
        let result = std::thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            let processor = RqlProcessor::new();
            rt.block_on(async {
                processor.process(&cmd).await
            })
        }).join();

        let ast = match result {
            Ok(r) => r,
            Err(_) => return Err("Thread panic during parsing".into()),
        };

        match ast {
            Ok(ast) => {
                // Procesar cada statement
                for statement in &ast.statements {
                    match statement {
                        RqlStatement::Sql { sql, .. } => {
                            // Ejecutar SQL normal con execute_rql (usa fuente activa si existe)
                            self.execute_sql_statement(sql)?;
                        }
                        RqlStatement::UseSource { path, alias, options } => {
                            self.handle_use_source(path, alias.as_deref(), options)?;
                        }
                        RqlStatement::ShowSources => {
                            self.handle_show_sources()?;
                        }
                        RqlStatement::ShowTables { source } => {
                            self.handle_show_tables(source.as_deref())?;
                        }
                        RqlStatement::ShowVars => {
                            self.handle_show_vars()?;
                        }
                        RqlStatement::Describe { source, table } => {
                            self.handle_describe(source.as_deref(), table)?;
                        }
                        RqlStatement::Let { variable, expression } => {
                            self.handle_let(variable, expression)?;
                        }
                        RqlStatement::Unset { variables } => {
                            self.handle_unset(variables)?;
                        }
                        RqlStatement::Import { file, table, options } => {
                            self.handle_import(file, table, options)?;
                        }
                        RqlStatement::Export { query, file, format, options } => {
                            self.handle_export(query, file, format, options)?;
                        }
                        RqlStatement::Map { expressions } => {
                            self.handle_map(expressions)?;
                        }
                        RqlStatement::Filter { condition } => {
                            self.handle_filter(condition)?;
                        }
                        _ => {
                            self.show_error_dialog(&format!("⚠️ Comando no implementado: {:?}", statement.statement_type()));
                        }
                    }
                }
            }
            Err(e) => {
                self.show_error_dialog(&format!("❌ Error de parseo: {}", e));
            }
        }

        // Limpiar editor para próximo comando
        self.clear_command_editor();

        Ok(())
    }

    /// Ejecutar statement SQL directo
    fn execute_sql_statement(&mut self, sql: &str) -> Result<(), Box<dyn std::error::Error>> {
        let params = HashMap::new();
        let rql_query = RqlQuery::new(sql, params);

        match self.executor.execute_rql(&self.session, rql_query) {
            Ok(result_set) => {
                // Convertir ResultSet a QueryResults
                self.current_results = Some(self.convert_result_set(result_set, sql));

                // Resetear scroll para nuevos resultados
                self.result_scroll_offset_x = 0;
                self.result_scroll_offset_y = 0;

                // Cambiar a modo Result
                self.mode = UiMode::Result;
                Ok(())
            }
            Err(e) => {
                // Mostrar error en Dialog Mode
                self.show_error_dialog(&format!("❌ Error de ejecución SQL: {}", e));
                Err(Box::new(e))
            }
        }
    }

    /// Manejar comando USE SOURCE
    fn handle_use_source(&mut self, path: &str, alias: Option<&str>, _options: &HashMap<String, String>) -> Result<(), Box<dyn std::error::Error>> {
        // Detectar tipo de fuente por extensión
        if path.ends_with(".csv") || path.ends_with(".json") || path.ends_with(".parquet") {
            let table_name = alias.unwrap_or(path);

            #[cfg(debug_assertions)]
            eprintln!("[DEBUG TUI] Loading file into DuckDB: {} as {}", path, table_name);

            // Buscar si ya existe una fuente DuckDB registrada
            let registry = self.executor.source_registry_mut();
            let duckdb_source_exists = registry.list_sources()
                .iter()
                .any(|(name, _)| name == "duckdb");

            if duckdb_source_exists {
                // Agregar archivo a la instancia DuckDB existente
                #[cfg(debug_assertions)]
                eprintln!("[DEBUG TUI] Using existing DuckDB source");

                if let Some(source) = registry.get_mut("duckdb") {
                    // Downcast a DuckDBSource
                    let duckdb_source = source
                        .as_any_mut()
                        .downcast_mut::<noctra_duckdb::DuckDBSource>()
                        .ok_or_else(|| NoctraError::Internal("Source 'duckdb' is not a DuckDBSource".to_string()))?;

                    duckdb_source.register_file(path, table_name)
                        .map_err(|e| NoctraError::Internal(format!("Error registering file: {}", e)))?;

                    #[cfg(debug_assertions)]
                    eprintln!("[DEBUG TUI] File registered in existing DuckDB instance");
                } else {
                    return Err(Box::new(NoctraError::Internal("DuckDB source not found".to_string())));
                }
            } else {
                // Crear nueva instancia DuckDB y registrarla
                #[cfg(debug_assertions)]
                eprintln!("[DEBUG TUI] Creating new DuckDB source");

                let mut duckdb_source = noctra_duckdb::DuckDBSource::new_in_memory()
                    .map_err(|e| NoctraError::Internal(format!("Error creating DuckDB source: {}", e)))?;

                duckdb_source.register_file(path, table_name)
                    .map_err(|e| NoctraError::Internal(format!("Error registering file: {}", e)))?;

                #[cfg(debug_assertions)]
                eprintln!("[DEBUG TUI] DuckDB source created successfully");

                // Registrar fuente con nombre fijo "duckdb"
                registry.register("duckdb".to_string(), Box::new(duckdb_source))
                    .map_err(|e| NoctraError::Internal(format!("Error registering source: {}", e)))?;

                #[cfg(debug_assertions)]
                eprintln!("[DEBUG TUI] DuckDB source registered as 'duckdb'");
            }

            #[cfg(debug_assertions)]
            eprintln!("[DEBUG TUI] Active source: {:?}",
                self.executor.source_registry().active().map(|s| s.name()));

            // Success - no dialog in release mode
        } else {
            self.show_error_dialog(&format!("❌ Tipo de fuente no soportado: {}\n(Soportados: .csv, .json, .parquet)", path));
        }

        Ok(())
    }

    /// Mostrar diálogo informativo
    fn show_info_dialog(&mut self, message: &str) {
        self.dialog_message = Some(message.to_string());
        self.dialog_options = vec!["OK".to_string()];
        self.dialog_selected = 0;
        self.mode = UiMode::Dialog;
    }

    /// Manejar comando SHOW SOURCES
    fn handle_show_sources(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        use noctra_core::types::{Column, Row, Value};

        let sources = self.executor.source_registry().list_sources();

        // Crear columnas
        let columns = vec![
            Column { name: "Alias".to_string(), data_type: "TEXT".to_string(), ordinal: 0 },
            Column { name: "Tipo".to_string(), data_type: "TEXT".to_string(), ordinal: 1 },
            Column { name: "Path".to_string(), data_type: "TEXT".to_string(), ordinal: 2 },
        ];

        // Crear filas
        let rows: Vec<Row> = sources.iter().map(|(alias, source_type)| {
            Row {
                values: vec![
                    Value::Text(alias.clone()),
                    Value::Text(source_type.type_name().to_string()),
                    Value::Text(source_type.display_path().to_string()),
                ]
            }
        }).collect();

        let result_set = ResultSet {
            columns,
            rows,
            rows_affected: None,
            last_insert_rowid: None,
        };

        // Mostrar como resultado de tabla
        self.current_results = Some(self.convert_result_set(result_set, "SHOW SOURCES"));
        self.result_scroll_offset_x = 0;
        self.result_scroll_offset_y = 0;
        self.mode = UiMode::Result;

        Ok(())
    }

    /// Manejar comando SHOW TABLES
    fn handle_show_tables(&mut self, source: Option<&str>) -> Result<(), Box<dyn std::error::Error>> {
        use noctra_core::types::{Column, Row, Value};

        let mut table_list = Vec::new();

        if let Some(source_name) = source {
            // Mostrar tablas de una fuente específica
            if let Some(data_source) = self.executor.source_registry().get(source_name) {
                match data_source.schema() {
                    Ok(tables) => {
                        for table in tables {
                            table_list.push(table.name);
                        }
                    }
                    Err(e) => {
                        return Err(Box::new(NoctraError::Internal(format!("Error obteniendo schema: {}", e))));
                    }
                }
            } else {
                return Err(Box::new(NoctraError::Internal(format!("Fuente '{}' no encontrada", source_name))));
            }
        } else {
            // Mostrar todas las tablas de todas las fuentes
            let sources = self.executor.source_registry().list_sources();
            for (alias, _) in sources {
                if let Some(data_source) = self.executor.source_registry().get(&alias) {
                    if let Ok(tables) = data_source.schema() {
                        for table in tables {
                            table_list.push(table.name);
                        }
                    }
                }
            }
        }

        // Crear columnas
        let columns = vec![
            Column { name: "table".to_string(), data_type: "TEXT".to_string(), ordinal: 0 },
        ];

        // Crear filas
        let rows: Vec<Row> = table_list.iter().map(|table_name| {
            Row {
                values: vec![Value::Text(table_name.clone())]
            }
        }).collect();

        let result_set = ResultSet {
            columns,
            rows,
            rows_affected: None,
            last_insert_rowid: None,
        };

        // Mostrar como resultado de tabla
        self.current_results = Some(self.convert_result_set(result_set, "SHOW TABLES"));
        self.result_scroll_offset_x = 0;
        self.result_scroll_offset_y = 0;
        self.mode = UiMode::Result;

        Ok(())
    }

    /// Manejar comando SHOW VARS
    fn handle_show_vars(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        use noctra_core::types::{Column, Row, Value};

        let vars = self.session.list_variables();

        // Crear columnas
        let columns = vec![
            Column { name: "Variable".to_string(), data_type: "TEXT".to_string(), ordinal: 0 },
            Column { name: "Valor".to_string(), data_type: "TEXT".to_string(), ordinal: 1 },
        ];

        // Crear filas
        let rows: Vec<Row> = vars.iter().map(|(name, value)| {
            Row {
                values: vec![
                    Value::Text(name.clone()),
                    Value::Text(value.to_string()),
                ]
            }
        }).collect();

        let result_set = ResultSet {
            columns,
            rows,
            rows_affected: None,
            last_insert_rowid: None,
        };

        // Mostrar como resultado de tabla
        self.current_results = Some(self.convert_result_set(result_set, "SHOW VARS"));
        self.result_scroll_offset_x = 0;
        self.result_scroll_offset_y = 0;
        self.mode = UiMode::Result;

        Ok(())
    }

    /// Manejar comando DESCRIBE
    fn handle_describe(&mut self, source: Option<&str>, table: &str) -> Result<(), Box<dyn std::error::Error>> {
        use noctra_core::types::{Column, Row, Value};

        if let Some(source_name) = source {
            // Describir tabla de una fuente específica
            if let Some(data_source) = self.executor.source_registry().get(source_name) {
                match data_source.schema() {
                    Ok(tables) => {
                        if let Some(table_info) = tables.iter().find(|t| t.name == table) {
                            // Crear columnas
                            let columns = vec![
                                Column { name: "Campos".to_string(), data_type: "TEXT".to_string(), ordinal: 0 },
                                Column { name: "Tipo".to_string(), data_type: "TEXT".to_string(), ordinal: 1 },
                            ];

                            // Crear filas
                            let rows: Vec<Row> = table_info.columns.iter().map(|col| {
                                Row {
                                    values: vec![
                                        Value::Text(col.name.clone()),
                                        Value::Text(col.data_type.clone()),
                                    ]
                                }
                            }).collect();

                            let result_set = ResultSet {
                                columns,
                                rows,
                                rows_affected: None,
                                last_insert_rowid: None,
                            };

                            // Mostrar como resultado de tabla
                            self.current_results = Some(self.convert_result_set(result_set, &format!("DESCRIBE {}.{}", source_name, table)));
                            self.result_scroll_offset_x = 0;
                            self.result_scroll_offset_y = 0;
                            self.mode = UiMode::Result;

                            return Ok(());
                        } else {
                            return Err(Box::new(NoctraError::Internal(format!("Tabla '{}' no encontrada en '{}'", table, source_name))));
                        }
                    }
                    Err(e) => {
                        return Err(Box::new(NoctraError::Internal(format!("Error obteniendo schema: {}", e))));
                    }
                }
            } else {
                return Err(Box::new(NoctraError::Internal(format!("Fuente '{}' no encontrada", source_name))));
            }
        } else {
            return Err(Box::new(NoctraError::Internal("DESCRIBE requiere especificar la fuente: DESCRIBE source.table".to_string())));
        }
    }

    /// Manejar comando LET
    fn handle_let(&mut self, variable: &str, expression: &str) -> Result<(), Box<dyn std::error::Error>> {
        // Evaluar la expresión (por ahora, simplemente tomar el valor literal)
        let value = expression.trim_matches('\'').trim_matches('"');
        self.session.set_variable(variable.to_string(), value.to_string());

        self.show_info_dialog(&format!("✅ Variable '{}' = '{}'", variable, value));
        Ok(())
    }

    /// Manejar comando UNSET
    fn handle_unset(&mut self, variables: &[String]) -> Result<(), Box<dyn std::error::Error>> {
        let mut message = String::from("✅ Variables eliminadas:\n\n");
        for var in variables {
            self.session.remove_variable(var);
            message.push_str(&format!("  • {}\n", var));
        }

        self.show_info_dialog(&message);
        Ok(())
    }

    /// Manejar comando IMPORT
    /// Sintaxis: IMPORT 'file.csv' AS table OPTIONS (delimiter=',', header=true)
    fn handle_import(&mut self, file: &str, table: &str, options: &HashMap<String, String>) -> Result<(), Box<dyn std::error::Error>> {
        use std::fs::File;
        use std::io::{BufRead, BufReader};
        use std::path::Path;

        // Validar ruta de archivo (sandboxing)
        Self::validate_file_path(file)?;

        // Validar nombre de tabla (SQL injection prevention)
        Self::validate_table_name(table)?;

        // Detectar formato por extensión
        let is_csv = file.ends_with(".csv");
        let is_json = file.ends_with(".json");

        if !is_csv && !is_json {
            return Err(Box::new(NoctraError::Internal(
                format!("Formato de archivo no soportado: {} (solo .csv y .json)", file)
            )));
        }

        // Check file size (max 100MB)
        let path = Path::new(file);
        if path.exists() {
            let metadata = std::fs::metadata(path)?;
            const MAX_FILE_SIZE: u64 = 100 * 1024 * 1024;
            if metadata.len() > MAX_FILE_SIZE {
                return Err(Box::new(NoctraError::Internal(format!(
                    "Archivo demasiado grande: {} bytes (máx: {} bytes)",
                    metadata.len(),
                    MAX_FILE_SIZE
                ))));
            }
        }

        // Leer archivo
        let file_handle = File::open(file)
            .map_err(|e| NoctraError::Internal(format!("Error abriendo archivo: {}", e)))?;
        let reader = BufReader::new(file_handle);

        if is_csv {
            // Importar CSV
            let delimiter = options.get("delimiter")
                .and_then(|d| d.chars().next())
                .unwrap_or(',');
            let has_header = options.get("header")
                .map(|h| h == "true")
                .unwrap_or(true);

            let mut lines = reader.lines();

            // Leer header
            let header_line = if let Some(Ok(line)) = lines.next() {
                line
            } else {
                return Err(Box::new(NoctraError::Internal("Archivo CSV vacío".into())));
            };

            let columns: Vec<String> = header_line
                .split(delimiter)
                .map(|s| s.trim().trim_matches('"').to_string())
                .collect();

            if columns.is_empty() {
                return Err(Box::new(NoctraError::Internal("No se encontraron columnas en CSV".into())));
            }

            // Crear tabla en SQLite
            let column_defs: Vec<String> = columns.iter()
                .map(|col| format!("{} TEXT", col))
                .collect();
            let create_sql = format!("CREATE TABLE IF NOT EXISTS {} ({})", table, column_defs.join(", "));

            self.executor.execute_sql(&self.session, &create_sql)
                .map_err(|e| NoctraError::Internal(format!("Error creando tabla: {}", e)))?;

            // Insertar datos
            let mut rows_imported = 0;

            // Si no tiene header, procesar la primera línea como datos
            if !has_header {
                let values: Vec<String> = header_line
                    .split(delimiter)
                    .map(|s| s.trim().trim_matches('"').to_string())
                    .collect();

                // Construir INSERT con valores literales (por simplicidad)
                let values_str = values.iter()
                    .map(|v| format!("'{}'", v.replace('\'', "''")))
                    .collect::<Vec<_>>()
                    .join(", ");
                let insert = format!("INSERT INTO {} VALUES ({})", table, values_str);
                self.executor.execute_sql(&self.session, &insert)?;
                rows_imported += 1;
            }

            // Procesar resto de líneas
            for line_result in lines {
                let line = line_result
                    .map_err(|e| NoctraError::Internal(format!("Error leyendo línea: {}", e)))?;

                let values: Vec<String> = line
                    .split(delimiter)
                    .map(|s| s.trim().trim_matches('"').to_string())
                    .collect();

                if values.len() != columns.len() {
                    eprintln!("⚠️ Advertencia: línea con número incorrecto de columnas, saltando");
                    continue;
                }

                // Construir INSERT con valores literales
                let values_str = values.iter()
                    .map(|v| format!("'{}'", v.replace('\'', "''")))
                    .collect::<Vec<_>>()
                    .join(", ");
                let insert = format!("INSERT INTO {} VALUES ({})", table, values_str);
                self.executor.execute_sql(&self.session, &insert)?;
                rows_imported += 1;
            }

            self.show_info_dialog(&format!("✅ Importadas {} filas desde '{}' a tabla '{}'", rows_imported, file, table));
        } else if is_json {
            // Importar JSON (array de objetos)
            use serde_json::Value as JsonValue;

            // Leer todo el archivo
            let json_content = std::io::read_to_string(reader)
                .map_err(|e| NoctraError::Internal(format!("Error leyendo JSON: {}", e)))?;

            // Parsear JSON
            let json_data: JsonValue = serde_json::from_str(&json_content)
                .map_err(|e| NoctraError::Internal(format!("Error parseando JSON: {}", e)))?;

            // Verificar que es un array
            let array = match json_data {
                JsonValue::Array(arr) => arr,
                _ => return Err(Box::new(NoctraError::Internal(
                    "JSON debe ser un array de objetos".into()
                ))),
            };

            if array.is_empty() {
                return Err(Box::new(NoctraError::Internal("Array JSON vacío".into())));
            }

            // Extraer columnas del primer objeto
            let first_obj = match &array[0] {
                JsonValue::Object(obj) => obj,
                _ => return Err(Box::new(NoctraError::Internal(
                    "Elementos del array deben ser objetos".into()
                ))),
            };

            let columns: Vec<String> = first_obj.keys().cloned().collect();

            if columns.is_empty() {
                return Err(Box::new(NoctraError::Internal("No se encontraron columnas en JSON".into())));
            }

            // Inferir tipos de datos del primer objeto
            let column_types: Vec<(&str, &str)> = columns.iter().map(|col| {
                let value = &first_obj[col];
                let sql_type = match value {
                    JsonValue::Number(n) => {
                        if n.is_i64() {
                            "INTEGER"
                        } else {
                            "REAL"
                        }
                    }
                    JsonValue::Bool(_) => "INTEGER", // SQLite usa INTEGER para booleanos
                    JsonValue::String(_) => "TEXT",
                    JsonValue::Null => "TEXT", // Default para NULL
                    _ => "TEXT", // Arrays y objects como TEXT (JSON string)
                };
                (col.as_str(), sql_type)
            }).collect();

            // Crear tabla en SQLite
            let column_defs: Vec<String> = column_types.iter()
                .map(|(name, typ)| format!("{} {}", name, typ))
                .collect();
            let create_sql = format!("CREATE TABLE IF NOT EXISTS {} ({})", table, column_defs.join(", "));

            self.executor.execute_sql(&self.session, &create_sql)
                .map_err(|e| NoctraError::Internal(format!("Error creando tabla: {}", e)))?;

            // Insertar datos
            let mut rows_imported = 0;

            for item in &array {
                let obj = match item {
                    JsonValue::Object(o) => o,
                    _ => {
                        eprintln!("⚠️  Advertencia: elemento no es objeto, saltando");
                        continue;
                    }
                };

                // Extraer valores en orden de columnas
                let values: Vec<String> = columns.iter().map(|col| {
                    let value = obj.get(col).unwrap_or(&JsonValue::Null);
                    match value {
                        JsonValue::String(s) => format!("'{}'", s.replace('\'', "''")),
                        JsonValue::Number(n) => n.to_string(),
                        JsonValue::Bool(b) => if *b { "1" } else { "0" }.to_string(),
                        JsonValue::Null => "NULL".to_string(),
                        JsonValue::Array(_) | JsonValue::Object(_) => {
                            // Serializar a JSON string
                            format!("'{}'", serde_json::to_string(value)
                                .unwrap_or_default()
                                .replace('\'', "''"))
                        }
                    }
                }).collect();

                // Construir INSERT con valores
                let insert = format!("INSERT INTO {} VALUES ({})", table, values.join(", "));
                self.executor.execute_sql(&self.session, &insert)?;
                rows_imported += 1;
            }

            self.show_info_dialog(&format!("✅ Importadas {} filas desde '{}' a tabla '{}'", rows_imported, file, table));
        }

        Ok(())
    }

    /// Manejar comando EXPORT
    /// Sintaxis: EXPORT table TO 'file.csv' FORMAT CSV OPTIONS (delimiter=',', header=true)
    fn handle_export(&mut self, query: &str, file: &str, format: &noctra_parser::ExportFormat, options: &HashMap<String, String>) -> Result<(), Box<dyn std::error::Error>> {
        use std::fs::File;
        use std::io::Write;

        // Validar ruta de archivo (sandboxing)
        Self::validate_file_path(file)?;

        // Validar nombre de tabla si no es SELECT
        if !query.to_uppercase().starts_with("SELECT ") {
            Self::validate_table_name(query)?;
        }

        // Ejecutar query para obtener datos
        let result = if query.to_uppercase().starts_with("SELECT ") {
            // Es una query completa
            let params = HashMap::new();
            let rql_query = RqlQuery::new(query, params);
            self.executor.execute_rql(&self.session, rql_query)?
        } else {
            // Es un nombre de tabla, generar SELECT *
            let select_query = format!("SELECT * FROM {}", query);
            let params = HashMap::new();
            let rql_query = RqlQuery::new(&select_query, params);
            self.executor.execute_rql(&self.session, rql_query)?
        };

        match format {
            noctra_parser::ExportFormat::Csv => {
                let delimiter = options.get("delimiter")
                    .and_then(|d| d.chars().next())
                    .unwrap_or(',');
                let has_header = options.get("header")
                    .map(|h| h == "true")
                    .unwrap_or(true);

                let mut file_handle = File::create(file)
                    .map_err(|e| NoctraError::Internal(format!("Error creando archivo: {}", e)))?;

                // Escribir header si está habilitado
                if has_header {
                    let header_names: Vec<String> = result.columns.iter()
                        .map(|col| col.name.clone())
                        .collect();
                    let header_line = header_names.join(&delimiter.to_string());
                    writeln!(file_handle, "{}", header_line)
                        .map_err(|e| NoctraError::Internal(format!("Error escribiendo header: {}", e)))?;
                }

                // Escribir filas
                for row in &result.rows {
                    let row_values: Vec<String> = row.values.iter()
                        .map(|v| {
                            match v {
                                noctra_core::Value::Text(s) => {
                                    // Escapar comillas dobles y envolver en comillas si contiene delimitador
                                    if s.contains(delimiter) || s.contains('"') || s.contains('\n') {
                                        format!("\"{}\"", s.replace('"', "\"\""))
                                    } else {
                                        s.clone()
                                    }
                                }
                                noctra_core::Value::Integer(i) => i.to_string(),
                                noctra_core::Value::Float(f) => f.to_string(),
                                noctra_core::Value::Boolean(b) => b.to_string(),
                                noctra_core::Value::Null => String::new(),
                                _ => format!("{:?}", v),
                            }
                        })
                        .collect();

                    writeln!(file_handle, "{}", row_values.join(&delimiter.to_string()))
                        .map_err(|e| NoctraError::Internal(format!("Error escribiendo fila: {}", e)))?;
                }

                self.show_info_dialog(&format!("✅ Exportadas {} filas a '{}'", result.rows.len(), file));
            }
            noctra_parser::ExportFormat::Json => {
                use serde_json::{json, Value as JsonValue};

                let mut file_handle = File::create(file)
                    .map_err(|e| NoctraError::Internal(format!("Error creando archivo: {}", e)))?;

                // Convertir ResultSet a JSON array
                let rows_json: Vec<JsonValue> = result.rows.iter()
                    .map(|row| {
                        let mut obj = serde_json::Map::new();
                        for (i, col) in result.columns.iter().enumerate() {
                            let value = &row.values[i];
                            let json_val = match value {
                                noctra_core::Value::Text(s) => JsonValue::String(s.clone()),
                                noctra_core::Value::Integer(i) => JsonValue::Number((*i).into()),
                                noctra_core::Value::Float(f) => {
                                    if let Some(num) = serde_json::Number::from_f64(*f) {
                                        JsonValue::Number(num)
                                    } else {
                                        JsonValue::Null
                                    }
                                }
                                noctra_core::Value::Boolean(b) => JsonValue::Bool(*b),
                                noctra_core::Value::Null => JsonValue::Null,
                                _ => JsonValue::String(format!("{:?}", value)),
                            };
                            obj.insert(col.name.clone(), json_val);
                        }
                        JsonValue::Object(obj)
                    })
                    .collect();

                let json_output = json!(rows_json);
                writeln!(file_handle, "{}", serde_json::to_string_pretty(&json_output)
                    .map_err(|e| NoctraError::Internal(format!("Error serializando JSON: {}", e)))?)
                    .map_err(|e| NoctraError::Internal(format!("Error escribiendo JSON: {}", e)))?;

                self.show_info_dialog(&format!("✅ Exportadas {} filas a '{}'", result.rows.len(), file));
            }
            noctra_parser::ExportFormat::Xlsx => {
                return Err(Box::new(NoctraError::Internal(
                    "Exportación a XLSX no implementada en M4 (planeado para M5)".into()
                )));
            }
        }

        Ok(())
    }

    /// Manejar comando MAP
    /// Sintaxis: MAP expression1 AS alias1, expression2 AS alias2, ...
    fn handle_map(&mut self, _expressions: &[noctra_parser::MapExpression]) -> Result<(), Box<dyn std::error::Error>> {
        // MAP no implementado completamente en M4 - requiere pipeline de transformación
        // Por ahora, mostrar mensaje informativo
        self.show_info_dialog("⚠️ MAP: Transformaciones declarativas\n\nNo implementado completamente en M4.\nUse SELECT para transformaciones simples.\n\nEjemplo:\nSELECT UPPER(nombre) AS nombre, precio * 1.1 AS precio_nuevo\nFROM productos;");
        Ok(())
    }

    /// Manejar comando FILTER
    /// Sintaxis: FILTER condition
    fn handle_filter(&mut self, _condition: &str) -> Result<(), Box<dyn std::error::Error>> {
        // FILTER no implementado completamente en M4 - requiere pipeline de transformación
        // Por ahora, mostrar mensaje informativo
        self.show_info_dialog("⚠️ FILTER: Filtrado declarativo\n\nNo implementado completamente en M4.\nUse WHERE en SELECT.\n\nEjemplo:\nSELECT * FROM productos\nWHERE precio > 100;");
        Ok(())
    }

    /// Validar ruta de archivo (sandboxing)
    fn validate_file_path(file: &str) -> Result<(), Box<dyn std::error::Error>> {
        use std::path::Path;

        let path = Path::new(file);
        let path_str = path.to_string_lossy();

        // Directorios bloqueados
        let blocked_dirs = [
            "/etc/",
            "/sys/",
            "/proc/",
            "/dev/",
            "/root/",
            "/boot/",
            "C:\\Windows\\",
            "C:\\Program Files\\",
        ];

        for blocked in &blocked_dirs {
            if path_str.starts_with(blocked) {
                return Err(Box::new(NoctraError::Internal(format!(
                    "Acceso denegado: No se puede acceder a directorio del sistema: {}",
                    path_str
                ))));
            }
        }

        // Prevenir path traversal
        if path_str.contains("..") {
            return Err(Box::new(NoctraError::Internal(
                "Acceso denegado: Path traversal no permitido".to_string(),
            )));
        }

        // Validar que es un archivo regular
        if path.exists() {
            let metadata = std::fs::metadata(path)?;
            if !metadata.is_file() {
                return Err(Box::new(NoctraError::Internal(
                    "Acceso denegado: La ruta debe ser un archivo regular".to_string(),
                )));
            }
        }

        Ok(())
    }

    /// Validar nombre de tabla (SQL injection prevention)
    fn validate_table_name(name: &str) -> Result<(), Box<dyn std::error::Error>> {
        // Solo permitir alfanuméricos, guión bajo y guión
        if name
            .chars()
            .all(|c| c.is_alphanumeric() || c == '_' || c == '-')
        {
            Ok(())
        } else {
            Err(Box::new(NoctraError::Internal(format!(
                "Nombre de tabla inválido: '{}' (solo alfanuméricos, _, - permitidos)",
                name
            ))))
        }
    }

    /// Extraer nombre de tabla de un comando SQL
    fn extract_table_name(sql: &str) -> Option<String> {
        let sql_upper = sql.to_uppercase();

        // Intentar extraer de "SELECT * FROM tabla"
        if let Some(pos) = sql_upper.find(" FROM ") {
            let after_from = &sql[pos + 6..];
            let table_name = after_from
                .split_whitespace()
                .next()?
                .trim_end_matches(';')
                .trim();
            return Some(table_name.to_string());
        }

        // Intentar extraer de "DESCRIBE source.tabla"
        if sql_upper.starts_with("DESCRIBE ") {
            let after_describe = &sql[9..];
            let parts: Vec<&str> = after_describe.split('.').collect();
            if parts.len() == 2 {
                return Some(parts[1].trim_end_matches(';').trim().to_string());
            }
        }

        None
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
        execute!(
            self.terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableBracketedPaste
        )?;
        self.terminal.show_cursor()?;
        Ok(())
    }
}

impl<'a> Drop for NoctraTui<'a> {
    fn drop(&mut self) {
        let _ = self.cleanup();
    }
}
