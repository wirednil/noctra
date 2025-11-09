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
use std::collections::HashMap;
use std::io::{stdout, Stdout};
use std::time::Duration;
use tui_textarea::{Input, TextArea};

// Backend integration
use noctra_core::{Executor, ResultSet, Session, RqlQuery, NoctraError};
use noctra_core::{CsvDataSource, CsvOptions};
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

            // Obtener fuente activa y tabla actual
            let active_source = self.executor.source_registry()
                .active()
                .map(|source| {
                    let source_name = source.name().to_string();

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
                    dialog_message.as_deref(),
                    &dialog_options,
                    dialog_selected,
                    active_source.as_deref(),
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
        if path.ends_with(".csv") {
            // Crear fuente CSV
            let source_name = alias.unwrap_or(path);
            eprintln!("[DEBUG TUI] Loading CSV source: {} as {}", path, source_name);

            let csv_source = CsvDataSource::new(
                path,
                source_name.to_string(),
                CsvOptions::default()
            ).map_err(|e| NoctraError::Internal(format!("Error loading CSV: {}", e)))?;

            eprintln!("[DEBUG TUI] CSV source created successfully");

            // Registrar fuente
            self.executor.source_registry_mut()
                .register(source_name.to_string(), Box::new(csv_source))
                .map_err(|e| NoctraError::Internal(format!("Error registering source: {}", e)))?;

            eprintln!("[DEBUG TUI] CSV source registered");
            eprintln!("[DEBUG TUI] Active source: {:?}",
                self.executor.source_registry().active().map(|s| s.name()));

            self.show_info_dialog(&format!("✅ Fuente CSV '{}' cargada como '{}'", path, source_name));
        } else {
            self.show_error_dialog(&format!("❌ Tipo de fuente no soportado: {}\n(Solo .csv por ahora)", path));
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
