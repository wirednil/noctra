//! REPL (Read-Eval-Print Loop) para Noctra

use crate::cli::ReplArgs;
use crate::config::CliConfig;
use crate::output::format_result_set;
use noctra_core::{Executor, NoctraError, RqlQuery, Session, SqliteBackend};
use noctra_core::{CsvDataSource, CsvOptions};
use noctra_parser::{RqlProcessor, RqlStatement};
use std::collections::HashMap;
use std::io::{self, Write};
use std::sync::Arc;

type Result<T> = std::result::Result<T, NoctraError>;

/// Handler del REPL
#[derive(Debug)]
pub struct ReplHandler {
    /// Configuración
    _config: CliConfig,

    /// Estado del REPL
    state: ReplState,

    /// Historial de comandos
    history: Vec<String>,

    /// Contador de líneas
    line_count: usize,
}

/// Estado del REPL
#[derive(Debug, Clone)]
pub enum ReplState {
    /// Listo para comando
    Ready,

    /// Esperando más líneas (query multi-línea)
    MultiLine,

    /// Esperando parámetro
    WaitingParameter(String),

    /// Error
    Error,
}

/// REPL principal
#[derive(Debug)]
pub struct Repl {
    /// Configuración
    config: CliConfig,

    /// Handler
    handler: ReplHandler,

    /// Executor de queries
    executor: Executor,

    /// Sesión actual
    session: Session,
}

impl Repl {
    /// Crear nuevo REPL
    pub fn new(config: CliConfig, args: ReplArgs) -> Result<Self> {
        let handler = ReplHandler::new(config.clone(), args)?;

        // Crear backend SQLite
        let backend = SqliteBackend::with_file(&config.database.connection_string)?;
        let executor = Executor::new(Arc::new(backend));

        // Crear sesión
        let session = Session::new();

        Ok(Self {
            config,
            handler,
            executor,
            session,
        })
    }

    /// Ejecutar REPL
    pub async fn run(&mut self) -> Result<()> {
        println!("🎯 Noctra REPL iniciado - Escribe 'help' para ayuda");

        loop {
            // Mostrar prompt
            let prompt = self.get_prompt();

            // Leer input
            let input = read_input(&prompt)?;

            // Procesar input
            if self.process_input(&input)? {
                break; // Salir del REPL
            }
        }

        println!("👋 ¡Hasta luego!");
        Ok(())
    }

    /// Obtener prompt actual
    fn get_prompt(&self) -> String {
        match &self.handler.state {
            ReplState::Ready => self.config.repl.prompt.clone(),
            ReplState::MultiLine => self.config.repl.multiline_prompt.clone(),
            ReplState::WaitingParameter(param) => format!(":param {} => ", param),
            ReplState::Error => "ERROR> ".to_string(),
        }
    }

    /// Procesar input del usuario
    fn process_input(&mut self, input: &str) -> Result<bool> {
        let trimmed = input.trim();

        // Comandos especiales
        if trimmed.is_empty() {
            return Ok(false);
        }

        if trimmed == "quit" || trimmed == "exit" || trimmed == "q" {
            return Ok(true); // Salir
        }

        if trimmed == "help" || trimmed == "h" || trimmed == "?" {
            self.show_help();
            return Ok(false);
        }

        if trimmed == "clear" || trimmed == "cls" {
            self.clear_screen();
            return Ok(false);
        }

        if trimmed.starts_with(':') {
            return self.handle_special_command(trimmed);
        }

        // Agregar a historial
        self.handler.history.push(input.to_string());

        // Procesar como SQL/RQL
        self.execute_query(input)
    }

    /// Manejar comandos especiales
    fn handle_special_command(&mut self, cmd: &str) -> Result<bool> {
        match cmd {
            ":help" => {
                self.show_help();
                Ok(false)
            }
            ":clear" | ":cls" => {
                self.clear_screen();
                Ok(false)
            }
            ":version" | ":ver" => {
                println!("Noctra v0.1.0");
                Ok(false)
            }
            ":config" => {
                self.show_config();
                Ok(false)
            }
            ":status" | ":stats" => {
                self.show_status();
                Ok(false)
            }
            cmd => {
                if cmd.starts_with(":set ") {
                    self.handle_set_command(cmd);
                    Ok(false)
                } else {
                    println!("Comando desconocido: {}", cmd);
                    Ok(false)
                }
            }
        }
    }

    /// Ejecutar query SQL/RQL
    fn execute_query(&mut self, query: &str) -> Result<bool> {
        // Parsear query con RqlProcessor en thread separado
        // para evitar conflictos con runtime de Tokio existente
        let query_str = query.to_string();
        let result = std::thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            let processor = RqlProcessor::new();
            rt.block_on(async {
                processor.process(&query_str).await
            })
        }).join();

        let ast = match result {
            Ok(r) => r,
            Err(_) => return Err(NoctraError::Internal("Thread panic during parsing".to_string())),
        }.map_err(|e| NoctraError::Internal(format!("Parse error: {}", e)))?;

        // Procesar cada statement
        for statement in &ast.statements {
            match statement {
                RqlStatement::Sql { sql, .. } => {
                    // Ejecutar SQL normal
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
                    println!("⚠️  Comando no implementado aún en REPL: {:?}", statement.statement_type());
                }
            }
        }

        Ok(false)
    }

    /// Ejecutar statement SQL directo
    fn execute_sql_statement(&mut self, sql: &str) -> Result<()> {
        let params = HashMap::new();
        let rql_query = RqlQuery::new(sql, params);

        match self.executor.execute_rql(&self.session, rql_query) {
            Ok(result_set) => {
                // Mostrar resultados
                if result_set.rows.is_empty() {
                    if let Some(affected) = result_set.rows_affected {
                        if affected > 0 {
                            println!("✅ {} filas afectadas", affected);
                        } else {
                            println!("✅ Query ejecutado (0 filas)");
                        }
                    } else {
                        println!("✅ Query ejecutado");
                    }
                } else {
                    let table = format_result_set(&result_set);
                    println!("{}", table);
                    println!();
                    println!("({} filas)", result_set.rows.len());
                }
                Ok(())
            }
            Err(e) => {
                println!("❌ Error de ejecución: {}", e);
                Err(e)
            }
        }
    }

    /// Manejar comando USE SOURCE
    fn handle_use_source(&mut self, path: &str, alias: Option<&str>, _options: &HashMap<String, String>) -> Result<()> {
        // Detectar tipo de fuente por extensión
        if path.ends_with(".csv") {
            // Crear fuente CSV
            let source_name = alias.unwrap_or(path);
            eprintln!("[DEBUG] Loading CSV source: {} as {}", path, source_name);

            let csv_source = CsvDataSource::new(
                path,
                source_name.to_string(),
                CsvOptions::default()
            ).map_err(|e| NoctraError::Internal(format!("Error loading CSV: {}", e)))?;

            eprintln!("[DEBUG] CSV source created successfully");

            // Registrar fuente
            self.executor.source_registry_mut()
                .register(source_name.to_string(), Box::new(csv_source))
                .map_err(|e| NoctraError::Internal(format!("Error registering source: {}", e)))?;

            eprintln!("[DEBUG] CSV source registered");
            eprintln!("[DEBUG] Active source after registration: {:?}",
                self.executor.source_registry().active().map(|s| s.name()));

            println!("✅ Fuente CSV '{}' cargada como '{}'", path, source_name);
        } else {
            println!("❌ Tipo de fuente no soportado: {}", path);
            println!("   (Actualmente solo se soportan archivos .csv)");
        }

        Ok(())
    }

    /// Manejar comando SHOW SOURCES
    fn handle_show_sources(&self) -> Result<()> {
        let sources = self.executor.source_registry().list_sources();

        if sources.is_empty() {
            println!("ℹ️  No hay fuentes registradas");
        } else {
            println!("📊 Fuentes disponibles:");
            for (alias, source_type) in sources {
                println!("  • {} ({}) - {}", alias, source_type.type_name(), source_type.display_path());
            }
        }

        Ok(())
    }

    /// Manejar comando SHOW TABLES
    fn handle_show_tables(&self, source: Option<&str>) -> Result<()> {
        if let Some(source_name) = source {
            // Mostrar tablas de una fuente específica
            if let Some(data_source) = self.executor.source_registry().get(source_name) {
                match data_source.schema() {
                    Ok(tables) => {
                        if tables.is_empty() {
                            println!("ℹ️  No hay tablas en '{}'", source_name);
                        } else {
                            println!("📋 Tablas en '{}':", source_name);
                            for table in tables {
                                println!("  • {} ({} columnas)", table.name, table.columns.len());
                            }
                        }
                    }
                    Err(e) => println!("❌ Error obteniendo schema: {}", e),
                }
            } else {
                println!("❌ Fuente '{}' no encontrada", source_name);
            }
        } else {
            // Mostrar todas las tablas de todas las fuentes
            let sources = self.executor.source_registry().list_sources();
            if sources.is_empty() {
                println!("ℹ️  No hay fuentes registradas");
            } else {
                for (alias, _) in sources {
                    if let Some(data_source) = self.executor.source_registry().get(&alias) {
                        if let Ok(tables) = data_source.schema() {
                            if !tables.is_empty() {
                                println!("📋 Tablas en '{}':", alias);
                                for table in tables {
                                    println!("  • {} ({} columnas)", table.name, table.columns.len());
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Manejar comando SHOW VARS
    fn handle_show_vars(&self) -> Result<()> {
        let vars = self.session.list_variables();

        if vars.is_empty() {
            println!("ℹ️  No hay variables de sesión definidas");
        } else {
            println!("🔧 Variables de sesión:");
            for (name, value) in vars {
                println!("  {} = {}", name, value);
            }
        }

        Ok(())
    }

    /// Manejar comando DESCRIBE
    fn handle_describe(&self, source: Option<&str>, table: &str) -> Result<()> {
        if let Some(source_name) = source {
            // Describir tabla de una fuente específica
            if let Some(data_source) = self.executor.source_registry().get(source_name) {
                match data_source.schema() {
                    Ok(tables) => {
                        if let Some(table_info) = tables.iter().find(|t| t.name == table) {
                            println!("📊 Estructura de {}.{}:", source_name, table);
                            println!("  Columnas:");
                            for col in &table_info.columns {
                                println!("    • {} ({})", col.name, col.data_type);
                            }
                            if let Some(row_count) = table_info.row_count {
                                println!("  Filas: {}", row_count);
                            }
                        } else {
                            println!("❌ Tabla '{}' no encontrada en '{}'", table, source_name);
                        }
                    }
                    Err(e) => println!("❌ Error obteniendo schema: {}", e),
                }
            } else {
                println!("❌ Fuente '{}' no encontrada", source_name);
            }
        } else {
            println!("❌ DESCRIBE requiere especificar la fuente: DESCRIBE source.table");
        }

        Ok(())
    }

    /// Manejar comando LET
    fn handle_let(&mut self, variable: &str, expression: &str) -> Result<()> {
        // Evaluar la expresión (por ahora, simplemente tomar el valor literal)
        let value = expression.trim_matches('\'').trim_matches('"');
        self.session.set_variable(variable.to_string(), value.to_string());
        println!("✅ Variable '{}' = '{}'", variable, value);
        Ok(())
    }

    /// Manejar comando UNSET
    fn handle_unset(&mut self, variables: &[String]) -> Result<()> {
        for var in variables {
            self.session.remove_variable(var);
            println!("✅ Variable '{}' eliminada", var);
        }
        Ok(())
    }

    /// Mostrar ayuda
    fn show_help(&self) {
        println!("🐍 Noctra - Comandos disponibles:");
        println!("  help, h, ?       - Mostrar esta ayuda");
        println!("  clear, cls       - Limpiar pantalla");
        println!("  quit, exit, q    - Salir del REPL");
        println!("  :version, :ver   - Mostrar versión");
        println!("  :config          - Mostrar configuración");
        println!("  :status, :stats  - Mostrar estado");
        println!("  :set KEY=VALUE   - Configurar variable");
        println!();
        println!("📋 Comandos SQL/RQL:");
        println!("  SELECT * FROM employees WHERE dept = 'IT';");
        println!("  LET dept = 'SALES';");
        println!("  SHOW VARS;");
        println!();
        println!("🌐 Comandos NQL (Multi-fuente):");
        println!("  USE 'data.csv' AS csv;              - Cargar archivo CSV");
        println!("  SHOW SOURCES;                       - Listar fuentes activas");
        println!("  SHOW TABLES;                        - Listar tablas de todas las fuentes");
        println!("  SHOW TABLES FROM csv;               - Listar tablas de fuente específica");
        println!("  DESCRIBE csv.clientes;              - Describir estructura de tabla");
        println!("  UNSET variable;                     - Eliminar variable de sesión");
        println!();
    }

    /// Limpiar pantalla
    fn clear_screen(&self) {
        print!("\x1B[2J\x1B[H");
        io::stdout().flush().unwrap();
    }

    /// Mostrar configuración
    fn show_config(&self) {
        println!("⚙️  Configuración actual:");
        println!("  Database: {:?}", self.config.database.backend_type);
        println!("  Connection: {}", self.config.database.connection_string);
        println!("  Theme: {:?}", self.config.global.theme);
        println!("  Color Mode: {:?}", self.config.global.color_mode);
    }

    /// Mostrar estado
    fn show_status(&self) {
        println!("📊 Estado del REPL:");
        println!("  Líneas procesadas: {}", self.handler.line_count);
        println!("  Comandos en historial: {}", self.handler.history.len());
        println!("  Estado: {:?}", self.handler.state);
    }

    /// Manejar comando SET
    fn handle_set_command(&mut self, cmd: &str) {
        let parts: Vec<&str> = cmd.splitn(2, ' ').collect();
        if parts.len() == 2 {
            let key_value = parts[1];
            if let Some((key, value)) = key_value.split_once('=') {
                println!(
                    "📝 Variable '{}' configurada a '{}'",
                    key.trim(),
                    value.trim()
                );
            } else {
                println!("❌ Formato inválido. Usa: :set KEY=VALUE");
            }
        } else {
            println!("❌ Formato inválido. Usa: :set KEY=VALUE");
        }
    }
}

impl ReplHandler {
    /// Crear nuevo handler
    fn new(config: CliConfig, _args: ReplArgs) -> Result<Self> {
        Ok(Self {
            _config: config,
            state: ReplState::Ready,
            history: Vec::new(),
            line_count: 0,
        })
    }
}

/// Leer input con prompt
fn read_input(prompt: &str) -> Result<String> {
    print!("{}", prompt);
    io::stdout()
        .flush()
        .map_err(|e| NoctraError::Io(e.to_string()))?;

    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .map_err(|e| NoctraError::Io(e.to_string()))?;

    Ok(input.trim().to_string())
}

/// Resultado de comando
pub type CommandResult = Result<bool>;
