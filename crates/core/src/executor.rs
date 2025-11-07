//! Executor principal y backends para Noctra

use crate::error::{NoctraError, Result};
use crate::session::Session;
use crate::types::{Parameters, ResultSet, Value};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

/// Trait para backends de base de datos (dyn-compatible)
pub trait Backend: Send + Sync + std::fmt::Debug {
    /// Ejecutar query SQL
    fn execute_query(&self, _sql: &str, _parameters: &Parameters) -> Result<ResultSet>;

    /// Ejecutar statement SQL (INSERT/UPDATE/DELETE)
    fn execute_statement(&self, _sql: &str, _parameters: &Parameters) -> Result<ResultSet>;

    /// Verificar conexión
    fn ping(&self) -> Result<()>;

    /// Obtener información del backend
    fn backend_info(&self) -> BackendInfo;
}

/// Información del backend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackendInfo {
    pub name: String,
    pub version: String,
    pub url: String,
    pub features: Vec<String>,
}

/// Backend SQLite
#[cfg(feature = "sqlite")]
#[derive(Debug)]
pub struct SqliteBackend {
    /// Conexión a la base de datos
    conn: Arc<std::sync::Mutex<rusqlite::Connection>>,

    /// URL de conexión
    url: String,

    /// Configuración del backend
    #[allow(dead_code)]
    config: SqliteConfig,
}

/// Configuración para SQLite
#[derive(Debug, Clone)]
pub struct SqliteConfig {
    pub url: String,
    pub timeout: u64,
    pub enable_wal_mode: bool,
    pub cache_size: i32,
}

impl SqliteConfig {
    /// Crear configuración por defecto para archivo
    pub fn for_file<T: Into<String>>(filename: T) -> Self {
        Self {
            url: format!("sqlite://{}", filename.into()),
            timeout: 30000, // 30 segundos
            enable_wal_mode: true,
            cache_size: -2000, // 2MB
        }
    }

    /// Crear configuración por defecto para base de datos en memoria
    pub fn for_memory() -> Self {
        Self {
            url: "sqlite://:memory:".to_string(),
            timeout: 30000,
            enable_wal_mode: false, // WAL no funciona en memoria
            cache_size: -2000,
        }
    }
}

#[cfg(feature = "sqlite")]
impl SqliteBackend {
    /// Crear nuevo backend SQLite
    pub fn new(config: SqliteConfig) -> Self {
        Self {
            conn: Arc::new(std::sync::Mutex::new(
                rusqlite::Connection::open_in_memory()
                    .unwrap_or_else(|_| panic!("Failed to create in-memory SQLite database")),
            )),
            url: config.url.clone(),
            config,
        }
    }

    /// Crear backend para archivo específico
    pub fn with_file<T: Into<String>>(filename: T) -> Result<Self> {
        let config = SqliteConfig::for_file(filename);
        let conn = rusqlite::Connection::open(config.url.trim_start_matches("sqlite://"))?;

        Ok(Self {
            conn: Arc::new(std::sync::Mutex::new(conn)),
            url: config.url.clone(),
            config,
        })
    }
}

#[cfg(feature = "sqlite")]
impl Backend for SqliteBackend {
    fn execute_query(&self, sql: &str, parameters: &Parameters) -> Result<ResultSet> {
        let conn = self
            .conn
            .lock()
            .map_err(|_| NoctraError::database("Cannot access SQLite connection".to_string()))?;

        let mut stmt = conn.prepare(sql).map_err(|e| {
            NoctraError::sql_execution(format!("Failed to prepare statement: {}", e))
        })?;

        let columns: Vec<String> = stmt.column_names().into_iter().map(String::from).collect();

        let mut result_set = ResultSet::new(
            columns
                .iter()
                .enumerate()
                .map(|(i, name)| {
                    crate::types::Column {
                        name: name.clone(),
                        data_type: "TEXT".to_string(), // Default type
                        ordinal: i,
                    }
                })
                .collect(),
        );

        let sqlite_params = map_parameters_to_sqlite(parameters)?;
        let params: Vec<&dyn rusqlite::ToSql> = sqlite_params
            .iter()
            .map(|v| v as &dyn rusqlite::ToSql)
            .collect();

        let mut rows = if parameters.is_empty() {
            stmt.query(()).map_err(|e| {
                NoctraError::sql_execution(format!("Failed to execute query: {}", e))
            })?
        } else {
            stmt.query(&*params).map_err(|e| {
                NoctraError::sql_execution(format!("Failed to execute query: {}", e))
            })?
        };

        while let Ok(Some(row)) = rows.next() {
            let mut values = Vec::new();
            for i in 0..columns.len() {
                let value_ref = row.get_ref(i).unwrap_or(rusqlite::types::ValueRef::Null);
                let value = map_sqlite_value_to_noctra(value_ref).map_err(|e| {
                    NoctraError::sql_execution(format!("Failed to map value: {}", e))
                })?;
                values.push(value);
            }
            result_set.add_row(crate::types::Row { values });
        }

        Ok(result_set)
    }

    fn execute_statement(&self, sql: &str, parameters: &Parameters) -> Result<ResultSet> {
        let conn = self
            .conn
            .lock()
            .map_err(|_| NoctraError::database("Cannot access SQLite connection".to_string()))?;

        let sqlite_params = map_parameters_to_sqlite(parameters)?;
        let params: Vec<&dyn rusqlite::ToSql> = sqlite_params
            .iter()
            .map(|v| v as &dyn rusqlite::ToSql)
            .collect();

        let result = if parameters.is_empty() {
            conn.execute(sql, ())
        } else {
            conn.execute(sql, &*params)
        };

        match result {
            Ok(rows_affected) => {
                let mut result_set = ResultSet::empty();
                result_set.rows_affected = Some(rows_affected as u64);

                // Para INSERT statements, obtener last insert rowid
                if sql.trim().to_uppercase().starts_with("INSERT") {
                    let rowid = conn.last_insert_rowid();
                    result_set.last_insert_rowid = Some(rowid);
                }

                Ok(result_set)
            }
            Err(e) => Err(NoctraError::sql_execution(format!(
                "Failed to execute statement: {}",
                e
            ))),
        }
    }

    fn ping(&self) -> Result<()> {
        let conn = self
            .conn
            .lock()
            .map_err(|_| NoctraError::database("Cannot access SQLite connection".to_string()))?;

        conn.execute("SELECT 1", ())
            .map_err(|e| NoctraError::database(format!("Failed to ping SQLite: {}", e)))?;
        Ok(())
    }

    fn backend_info(&self) -> BackendInfo {
        BackendInfo {
            name: "SQLite".to_string(),
            version: rusqlite::version().to_string(),
            url: self.url.clone(),
            features: vec![
                "sql".to_string(),
                "transactions".to_string(),
                "foreign_keys".to_string(),
                "wal_mode".to_string(),
            ],
        }
    }
}

/// Executor principal de Noctra
#[derive(Debug)]
pub struct Executor {
    /// Backend subyacente
    backend: Arc<dyn Backend>,

    /// Configuración del executor
    config: ExecutorConfig,
}

impl Executor {
    /// Crear executor con backend específico
    pub fn new(backend: Arc<dyn Backend>) -> Self {
        Self {
            backend,
            config: ExecutorConfig::default(),
        }
    }

    /// Crear executor SQLite en memoria
    #[cfg(feature = "sqlite")]
    pub fn new_sqlite_memory() -> Result<Self> {
        let config = SqliteConfig::for_memory();
        let backend = SqliteBackend::new(config);
        Ok(Self::new(Arc::new(backend)))
    }

    /// Crear executor SQLite con archivo
    #[cfg(feature = "sqlite")]
    pub fn new_sqlite_file<T: Into<String>>(filename: T) -> Result<Self> {
        let backend = SqliteBackend::with_file(filename)?;
        Ok(Self::new(Arc::new(backend)))
    }

    /// Conectar al backend
    pub fn connect(&mut self) -> Result<()> {
        Ok(()) // No connection needed for sync backends
    }

    /// Desconectar del backend
    pub fn disconnect(&mut self) -> Result<()> {
        Ok(()) // No disconnection needed for sync backends
    }

    /// Ping al backend
    pub fn ping(&self) -> Result<()> {
        self.backend.ping()
    }

    /// Ejecutar query RQL (parseado)
    pub fn execute_rql(&self, session: &Session, rql_query: RqlQuery) -> Result<ResultSet> {
        let sql = self.process_templates(&rql_query.sql, session)?;
        self.backend.execute_query(&sql, &rql_query.parameters)
    }

    /// Ejecutar query SQL directo
    pub fn execute_sql(&self, session: &Session, sql: &str) -> Result<ResultSet> {
        self.backend.execute_query(sql, session.list_parameters())
    }

    /// Ejecutar statement SQL directo
    pub fn execute_statement(&self, session: &Session, sql: &str) -> Result<ResultSet> {
        self.backend
            .execute_statement(sql, session.list_parameters())
    }

    /// Obtener información del backend
    pub fn backend_info(&self) -> BackendInfo {
        self.backend.backend_info()
    }

    /// Configuración del executor
    pub fn config(&self) -> &ExecutorConfig {
        &self.config
    }

    /// Procesar templates en SQL con variables de sesión
    fn process_templates(&self, sql: &str, session: &Session) -> Result<String> {
        let mut processed_sql = sql.to_string();

        // Reemplazar variables de sesión
        for (name, value) in session.list_variables() {
            let placeholder = format!("#{}", name);
            processed_sql = processed_sql.replace(&placeholder, &value.to_string());
        }

        Ok(processed_sql)
    }
}

/// Configuración del executor
#[derive(Debug, Clone)]
pub struct ExecutorConfig {
    /// Timeout de query en segundos
    pub query_timeout: u64,

    /// Límite de filas
    pub row_limit: Option<usize>,

    /// Modo debug
    pub debug_mode: bool,

    /// Auto-escapar parámetros
    pub auto_escape: bool,
}

impl Default for ExecutorConfig {
    fn default() -> Self {
        Self {
            query_timeout: 30,
            row_limit: Some(1000),
            debug_mode: false,
            auto_escape: true,
        }
    }
}

/// Query RQL ya parseado
#[derive(Debug, Clone)]
pub struct RqlQuery {
    /// SQL procesado
    pub sql: String,

    /// Parámetros
    pub parameters: Parameters,
}

impl RqlQuery {
    /// Crear nuevo query RQL
    pub fn new<T: Into<String>>(sql: T, parameters: Parameters) -> Self {
        Self {
            sql: sql.into(),
            parameters,
        }
    }

    /// Crear query SQL simple
    pub fn sql<T: Into<String>>(sql: T) -> Self {
        Self {
            sql: sql.into(),
            parameters: HashMap::new(),
        }
    }
}

// Funciones auxiliares para mapping de tipos

fn map_parameters_to_sqlite(parameters: &Parameters) -> Result<Vec<rusqlite::types::Value>> {
    let mut sqlite_params = Vec::new();

    for value in parameters.values() {
        let param = match value {
            Value::Null => rusqlite::types::Value::Null,
            Value::Integer(i) => rusqlite::types::Value::Integer(*i),
            Value::Text(s) => rusqlite::types::Value::Text(s.clone()),
            Value::Boolean(b) => rusqlite::types::Value::Integer(if *b { 1 } else { 0 }),
            Value::Float(f) => rusqlite::types::Value::Real(*f),
            _ => rusqlite::types::Value::Null,
        };
        sqlite_params.push(param);
    }

    Ok(sqlite_params)
}

fn map_sqlite_value_to_noctra(value: rusqlite::types::ValueRef<'_>) -> Result<Value> {
    match value {
        rusqlite::types::ValueRef::Null => Ok(Value::Null),
        rusqlite::types::ValueRef::Integer(i) => Ok(Value::Integer(i)),
        rusqlite::types::ValueRef::Text(s) => {
            let text = std::str::from_utf8(s).unwrap_or("");
            Ok(Value::Text(text.to_string()))
        }
        rusqlite::types::ValueRef::Blob(b) => Ok(Value::Text(format!("Blob({} bytes)", b.len()))),
        rusqlite::types::ValueRef::Real(f) => Ok(Value::Float(f)),
    }
}
