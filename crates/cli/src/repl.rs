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

    /// Manejar comando IMPORT
    /// Sintaxis: IMPORT 'file.csv' AS table OPTIONS (delimiter=',', header=true)
    fn handle_import(&mut self, file: &str, table: &str, options: &HashMap<String, String>) -> Result<()> {
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
            return Err(NoctraError::Internal(
                format!("Formato de archivo no soportado: {} (solo .csv y .json)", file)
            ));
        }

        // Check file size (max 100MB)
        let path = Path::new(file);
        if path.exists() {
            let metadata = std::fs::metadata(path)?;
            const MAX_FILE_SIZE: u64 = 100 * 1024 * 1024;
            if metadata.len() > MAX_FILE_SIZE {
                return Err(NoctraError::Internal(format!(
                    "Archivo demasiado grande: {} bytes (máx: {} bytes)",
                    metadata.len(),
                    MAX_FILE_SIZE
                )));
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
                return Err(NoctraError::Internal("Archivo CSV vacío".into()));
            };

            let columns: Vec<String> = header_line
                .split(delimiter)
                .map(|s| s.trim().trim_matches('"').to_string())
                .collect();

            if columns.is_empty() {
                return Err(NoctraError::Internal("No se encontraron columnas en CSV".into()));
            }

            // Crear tabla en SQLite
            let column_defs: Vec<String> = columns.iter()
                .map(|col| format!("{} TEXT", col))
                .collect();
            let create_sql = format!("CREATE TABLE IF NOT EXISTS {} ({})", table, column_defs.join(", "));

            self.executor.execute_sql(&self.session, &create_sql)
                .map_err(|e| NoctraError::Internal(format!("Error creando tabla: {}", e)))?;

            println!("✅ Tabla '{}' creada con {} columnas", table, columns.len());

            // Insertar datos
            let mut rows_imported = 0;

            // Si no tiene header, procesar la primera línea como datos
            if !has_header {
                let values: Vec<String> = header_line
                    .split(delimiter)
                    .map(|s| s.trim().trim_matches('"').to_string())
                    .collect();

                // Construir INSERT con valores literales
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
                    eprintln!("⚠️  Advertencia: línea con número incorrecto de columnas, saltando");
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

            println!("✅ Importadas {} filas desde '{}' a tabla '{}'", rows_imported, file, table);
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
                _ => return Err(NoctraError::Internal(
                    "JSON debe ser un array de objetos".into()
                )),
            };

            if array.is_empty() {
                return Err(NoctraError::Internal("Array JSON vacío".into()));
            }

            // Extraer columnas del primer objeto
            let first_obj = match &array[0] {
                JsonValue::Object(obj) => obj,
                _ => return Err(NoctraError::Internal(
                    "Elementos del array deben ser objetos".into()
                )),
            };

            let columns: Vec<String> = first_obj.keys().cloned().collect();

            if columns.is_empty() {
                return Err(NoctraError::Internal("No se encontraron columnas en JSON".into()));
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

            println!("✅ Tabla '{}' creada con {} columnas", table, columns.len());

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

            println!("✅ Importadas {} filas desde '{}' a tabla '{}'", rows_imported, file, table);
        }

        Ok(())
    }

    /// Manejar comando EXPORT
    /// Sintaxis: EXPORT table TO 'file.csv' FORMAT CSV OPTIONS (delimiter=',', header=true)
    fn handle_export(&mut self, query: &str, file: &str, format: &noctra_parser::ExportFormat, options: &HashMap<String, String>) -> Result<()> {
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

                println!("✅ Exportadas {} filas a '{}'", result.rows.len(), file);
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

                println!("✅ Exportadas {} filas a '{}'", result.rows.len(), file);
            }
            noctra_parser::ExportFormat::Xlsx => {
                println!("⚠️  Exportación a XLSX no implementada en M4 (planeado para M5)");
            }
        }

        Ok(())
    }

    /// Manejar comando MAP
    /// Sintaxis: MAP expression1 AS alias1, expression2 AS alias2, ...
    fn handle_map(&mut self, _expressions: &[noctra_parser::MapExpression]) -> Result<()> {
        println!("⚠️  MAP: Transformaciones declarativas");
        println!("No implementado completamente en M4.");
        println!("Use SELECT para transformaciones simples.");
        println!();
        println!("Ejemplo:");
        println!("  SELECT UPPER(nombre) AS nombre, precio * 1.1 AS precio_nuevo");
        println!("  FROM productos;");
        Ok(())
    }

    /// Manejar comando FILTER
    /// Sintaxis: FILTER condition
    fn handle_filter(&mut self, _condition: &str) -> Result<()> {
        println!("⚠️  FILTER: Filtrado declarativo");
        println!("No implementado completamente en M4.");
        println!("Use WHERE en SELECT.");
        println!();
        println!("Ejemplo:");
        println!("  SELECT * FROM productos");
        println!("  WHERE precio > 100;");
        Ok(())
    }

    /// Validar ruta de archivo (sandboxing)
    fn validate_file_path(file: &str) -> Result<()> {
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
                return Err(NoctraError::Internal(format!(
                    "Acceso denegado: No se puede acceder a directorio del sistema: {}",
                    path_str
                )));
            }
        }

        // Prevenir path traversal
        if path_str.contains("..") {
            return Err(NoctraError::Internal(
                "Acceso denegado: Path traversal no permitido".to_string(),
            ));
        }

        // Validar que es un archivo regular
        if path.exists() {
            let metadata = std::fs::metadata(path)?;
            if !metadata.is_file() {
                return Err(NoctraError::Internal(
                    "Acceso denegado: La ruta debe ser un archivo regular".to_string(),
                ));
            }
        }

        Ok(())
    }

    /// Validar nombre de tabla (SQL injection prevention)
    fn validate_table_name(name: &str) -> Result<()> {
        // Solo permitir alfanuméricos, guión bajo y guión
        if name
            .chars()
            .all(|c| c.is_alphanumeric() || c == '_' || c == '-')
        {
            Ok(())
        } else {
            Err(NoctraError::Internal(format!(
                "Nombre de tabla inválido: '{}' (solo alfanuméricos, _, - permitidos)",
                name
            )))
        }
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
