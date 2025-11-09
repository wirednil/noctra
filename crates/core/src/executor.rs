//! Executor principal y backends para Noctra

use crate::datasource::SourceRegistry;
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
    /// Backend subyacente (para backward compatibility)
    backend: Arc<dyn Backend>,

    /// Registry de fuentes de datos para NQL multi-source
    source_registry: SourceRegistry,

    /// Configuración del executor
    config: ExecutorConfig,
}

impl Executor {
    /// Crear executor con backend específico
    pub fn new(backend: Arc<dyn Backend>) -> Self {
        Self {
            backend,
            source_registry: SourceRegistry::new(),
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

        // Detectar si es un statement (INSERT/UPDATE/DELETE/CREATE/DROP/ALTER) o query (SELECT)
        let trimmed = sql.trim().to_uppercase();
        let is_statement = trimmed.starts_with("INSERT")
            || trimmed.starts_with("UPDATE")
            || trimmed.starts_with("DELETE")
            || trimmed.starts_with("CREATE")
            || trimmed.starts_with("DROP")
            || trimmed.starts_with("ALTER");

        if is_statement {
            self.backend.execute_statement(&sql, &rql_query.parameters)
        } else {
            self.backend.execute_query(&sql, &rql_query.parameters)
        }
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

    /// Get access to the source registry (NQL multi-source support)
    pub fn source_registry(&self) -> &SourceRegistry {
        &self.source_registry
    }

    /// Get mutable access to the source registry
    pub fn source_registry_mut(&mut self) -> &mut SourceRegistry {
        &mut self.source_registry
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Value;

    #[test]
    fn test_sqlite_backend_creation() {
        // Test in-memory database
        let backend = SqliteBackend::with_file(":memory:");
        assert!(backend.is_ok());

        // Test executor creation
        let backend = backend.unwrap();
        let executor = Executor::new(Arc::new(backend));

        // Verify executor works by running a simple query
        let session = Session::new();
        let query = RqlQuery::new("SELECT 1", HashMap::new());
        let result = executor.execute_rql(&session, query);
        assert!(result.is_ok());
    }

    #[test]
    fn test_executor_select_query() {
        let backend = SqliteBackend::with_file(":memory:").unwrap();
        let executor = Executor::new(Arc::new(backend));
        let session = Session::new();

        let query = RqlQuery::new("SELECT 1 AS num, 'test' AS text", HashMap::new());
        let result = executor.execute_rql(&session, query);

        assert!(result.is_ok());
        let result_set = result.unwrap();
        assert_eq!(result_set.columns.len(), 2);
        assert_eq!(result_set.rows.len(), 1);
        assert_eq!(result_set.columns[0].name, "num");
        assert_eq!(result_set.columns[1].name, "text");
    }

    #[test]
    fn test_executor_insert_statement() {
        let backend = SqliteBackend::with_file(":memory:").unwrap();
        let executor = Executor::new(Arc::new(backend));
        let session = Session::new();

        // Create table
        let create_query = RqlQuery::new(
            "CREATE TABLE test (id INTEGER PRIMARY KEY, name TEXT)",
            HashMap::new(),
        );
        executor.execute_rql(&session, create_query).unwrap();

        // Insert data
        let insert_query = RqlQuery::new(
            "INSERT INTO test (id, name) VALUES (1, 'Alice')",
            HashMap::new(),
        );
        let result = executor.execute_rql(&session, insert_query);

        assert!(result.is_ok());
        let result_set = result.unwrap();
        assert_eq!(result_set.rows_affected, Some(1));
        assert!(result_set.last_insert_rowid.is_some());
    }

    #[test]
    fn test_executor_update_statement() {
        let backend = SqliteBackend::with_file(":memory:").unwrap();
        let executor = Executor::new(Arc::new(backend));
        let session = Session::new();

        // Setup
        executor
            .execute_rql(
                &session,
                RqlQuery::new("CREATE TABLE test (id INTEGER, value TEXT)", HashMap::new()),
            )
            .unwrap();
        executor
            .execute_rql(
                &session,
                RqlQuery::new("INSERT INTO test VALUES (1, 'old')", HashMap::new()),
            )
            .unwrap();

        // Update
        let update_query =
            RqlQuery::new("UPDATE test SET value = 'new' WHERE id = 1", HashMap::new());
        let result = executor.execute_rql(&session, update_query);

        assert!(result.is_ok());
        let result_set = result.unwrap();
        assert_eq!(result_set.rows_affected, Some(1));
    }

    #[test]
    fn test_executor_delete_statement() {
        let backend = SqliteBackend::with_file(":memory:").unwrap();
        let executor = Executor::new(Arc::new(backend));
        let session = Session::new();

        // Setup
        executor
            .execute_rql(
                &session,
                RqlQuery::new("CREATE TABLE test (id INTEGER)", HashMap::new()),
            )
            .unwrap();
        executor
            .execute_rql(
                &session,
                RqlQuery::new("INSERT INTO test VALUES (1), (2), (3)", HashMap::new()),
            )
            .unwrap();

        // Delete
        let delete_query = RqlQuery::new("DELETE FROM test WHERE id > 1", HashMap::new());
        let result = executor.execute_rql(&session, delete_query);

        assert!(result.is_ok());
        let result_set = result.unwrap();
        assert_eq!(result_set.rows_affected, Some(2));
    }

    #[test]
    fn test_executor_create_table() {
        let backend = SqliteBackend::with_file(":memory:").unwrap();
        let executor = Executor::new(Arc::new(backend));
        let session = Session::new();

        let create_query = RqlQuery::new(
            "CREATE TABLE users (id INTEGER PRIMARY KEY, email TEXT NOT NULL)",
            HashMap::new(),
        );
        let result = executor.execute_rql(&session, create_query);

        assert!(result.is_ok());
    }

    #[test]
    fn test_parameter_mapping() {
        let mut params = HashMap::new();
        params.insert("key1".to_string(), Value::Integer(42));
        params.insert("key2".to_string(), Value::Text("hello".to_string()));
        params.insert("key3".to_string(), Value::Boolean(true));
        params.insert("key4".to_string(), Value::Float(2.5));
        params.insert("key5".to_string(), Value::Null);

        let result = map_parameters_to_sqlite(&params);
        assert!(result.is_ok());

        let mapped = result.unwrap();
        assert_eq!(mapped.len(), 5);
    }

    #[test]
    fn test_backend_info() {
        let backend = SqliteBackend::with_file(":memory:").unwrap();
        let executor = Executor::new(Arc::new(backend));

        let info = executor.backend_info();
        assert_eq!(info.name, "SQLite");
        assert!(!info.version.is_empty());
        assert!(!info.features.is_empty());
    }

    #[test]
    fn test_rql_query_builder() {
        let query = RqlQuery::sql("SELECT * FROM users");
        assert_eq!(query.sql, "SELECT * FROM users");
        assert!(query.parameters.is_empty());

        let mut params = HashMap::new();
        params.insert("id".to_string(), Value::Integer(1));
        let query = RqlQuery::new("SELECT * FROM users WHERE id = :id", params);
        assert_eq!(query.parameters.len(), 1);
    }

    #[test]
    fn test_executor_invalid_sql() {
        let backend = SqliteBackend::with_file(":memory:").unwrap();
        let executor = Executor::new(Arc::new(backend));
        let session = Session::new();

        let invalid_query = RqlQuery::new("INVALID SQL SYNTAX HERE", HashMap::new());
        let result = executor.execute_rql(&session, invalid_query);

        assert!(result.is_err());
    }

    #[test]
    fn test_executor_source_registry_integration() {
        let backend = SqliteBackend::with_file(":memory:").unwrap();
        let mut executor = Executor::new(Arc::new(backend));

        // Verify source registry is initialized
        let registry = executor.source_registry();
        assert_eq!(registry.list_sources().len(), 0);

        // Verify we can access mutable registry
        let _registry_mut = executor.source_registry_mut();

        // This test verifies the basic integration is working
        // Actual multi-source functionality will be tested in NQL execution tests
    }
}
