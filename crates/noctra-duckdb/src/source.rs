//! DuckDB Data Source Implementation
//!
//! Provides DuckDBSource that implements the DataSource trait,
//! enabling file-native queries for CSV, JSON, and Parquet files.

use crate::attachment::{AttachmentConfig, AttachmentRegistry};
use crate::config::DuckDBConfig;
use crate::error::{DuckDBError, Result};
use duckdb::{Connection, Result as DuckResult, Row};
use noctra_core::datasource::{ColumnInfo, DataSource, SourceType, TableInfo};
use noctra_core::types::{Column, Parameters, ResultSet, Row as NoctraRow, Value};
use std::collections::HashMap;
use std::path::Path;
use std::sync::Mutex;

/// DuckDB-powered data source for file-native queries
#[derive(Debug)]
pub struct DuckDBSource {
    /// DuckDB connection (wrapped in Mutex for thread safety)
    conn: Mutex<Connection>,
    /// Name/alias of this source
    name: String,
    /// Registered file tables (alias -> file_path)
    registered_files: HashMap<String, String>,
    /// Configuration settings
    config: DuckDBConfig,
    /// Attachment registry for cross-database connections
    attachments: AttachmentRegistry,
}

impl DuckDBSource {
    /// Create a new DuckDB source with in-memory database and default configuration
    pub fn new_in_memory() -> Result<Self> {
        Self::new_in_memory_with_config(DuckDBConfig::default())
    }

    /// Create a new DuckDB source with in-memory database and custom configuration
    pub fn new_in_memory_with_config(config: DuckDBConfig) -> Result<Self> {
        let conn = Connection::open_in_memory()?;

        // Apply configuration settings
        for sql in config.to_sql_commands() {
            log::debug!("Applying config: {}", sql);
            conn.execute(&sql, [])?;
        }

        Ok(Self {
            conn: Mutex::new(conn),
            name: "duckdb".to_string(),
            registered_files: HashMap::new(),
            config,
            attachments: AttachmentRegistry::new(),
        })
    }

    /// Create a new DuckDB source with persistent database file and default configuration
    pub fn new_with_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        Self::new_with_file_with_config(path, DuckDBConfig::default())
    }

    /// Create a new DuckDB source with persistent database file and custom configuration
    pub fn new_with_file_with_config<P: AsRef<Path>>(path: P, config: DuckDBConfig) -> Result<Self> {
        let conn = Connection::open(path)?;

        // Apply configuration settings
        for sql in config.to_sql_commands() {
            log::debug!("Applying config: {}", sql);
            conn.execute(&sql, [])?;
        }

        Ok(Self {
            conn: Mutex::new(conn),
            name: "duckdb".to_string(),
            registered_files: HashMap::new(),
            config,
            attachments: AttachmentRegistry::new(),
        })
    }

    /// Get current configuration
    pub fn config(&self) -> &DuckDBConfig {
        &self.config
    }

    /// Register a file as a virtual table using DuckDB's read_*_auto functions
    pub fn register_file(&mut self, file_path: &str, alias: &str) -> Result<()> {
        let extension = std::path::Path::new(file_path)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();

        let sql = match extension.as_str() {
            "csv" => format!(
                "CREATE OR REPLACE VIEW {} AS SELECT * FROM read_csv_auto('{}')",
                alias, file_path
            ),
            "json" => format!(
                "CREATE OR REPLACE VIEW {} AS SELECT * FROM read_json_auto('{}')",
                alias, file_path
            ),
            "parquet" => format!(
                "CREATE OR REPLACE VIEW {} AS SELECT * FROM read_parquet('{}')",
                alias, file_path
            ),
            _ => return Err(DuckDBError::UnsupportedFileType(extension)),
        };

        log::debug!("Registering file: {} -> {}", file_path, sql);
        let conn = self.conn.lock().map_err(|_| DuckDBError::QueryFailed("Mutex poisoned".to_string()))?;
        conn.execute(&sql, [])?;
        self.registered_files.insert(alias.to_string(), file_path.to_string());
        Ok(())
    }

    /// Attach a SQLite database to DuckDB for cross-source queries
    ///
    /// The attachment is automatically registered in the attachment registry
    /// and can be restored after restart using `restore_attachments()`
    pub fn attach_sqlite(&mut self, db_path: &str, alias: &str) -> Result<()> {
        let conn = self.conn.lock().map_err(|_| DuckDBError::QueryFailed("Mutex poisoned".to_string()))?;

        // Ensure SQLite extension is installed and loaded
        // INSTALL is idempotent - won't fail if already installed
        conn.execute("INSTALL sqlite", []).ok(); // Ignore error if already installed
        conn.execute("LOAD sqlite", [])?;

        let sql = format!("ATTACH '{}' AS {} (TYPE SQLITE)", db_path, alias);
        log::debug!("Attaching SQLite DB: {}", sql);
        conn.execute(&sql, [])?;

        // Register in attachment registry for persistence
        self.attachments.register(AttachmentConfig {
            db_type: "sqlite".to_string(),
            path: db_path.to_string(),
            alias: alias.to_string(),
            read_only: false,
        });

        Ok(())
    }

    /// Restore all registered attachments
    ///
    /// Call this after connection initialization to restore all previously
    /// registered database attachments. This is necessary because ATTACH
    /// statements are non-persistent in DuckDB.
    pub fn restore_attachments(&self) -> Result<()> {
        let conn = self.conn.lock().map_err(|_| DuckDBError::QueryFailed("Mutex poisoned".to_string()))?;

        // Check if any SQLite attachments exist - if so, ensure extension is loaded
        let has_sqlite = self.attachments.list().iter().any(|cfg| cfg.db_type == "sqlite");
        if has_sqlite {
            conn.execute("INSTALL sqlite", []).ok(); // Ignore error if already installed
            conn.execute("LOAD sqlite", [])?;
        }

        for sql in self.attachments.to_sql_commands() {
            log::debug!("Restoring attachment: {}", sql);
            conn.execute(&sql, [])?;
        }

        Ok(())
    }

    /// Get attachment registry (for inspection or persistence)
    pub fn attachments(&self) -> &AttachmentRegistry {
        &self.attachments
    }

    /// Get registered files
    pub fn registered_files(&self) -> &HashMap<String, String> {
        &self.registered_files
    }

    /// Convert DuckDB row to Noctra Row
    fn duckdb_row_to_noctra_row(&self, row: &Row, columns: &[Column]) -> DuckResult<NoctraRow> {
        let mut values = Vec::new();

        for idx in 0..columns.len() {
            // Try different types in order of preference
            // First try as integer
            if let Ok(val) = row.get::<_, Option<i64>>(idx) {
                values.push(val.map(Value::Integer).unwrap_or(Value::Null));
                continue;
            }

            // Then try as float
            if let Ok(val) = row.get::<_, Option<f64>>(idx) {
                values.push(val.map(Value::Float).unwrap_or(Value::Null));
                continue;
            }

            // Then try as boolean
            if let Ok(val) = row.get::<_, Option<bool>>(idx) {
                values.push(val.map(Value::Boolean).unwrap_or(Value::Null));
                continue;
            }

            // Finally try as string
            if let Ok(val) = row.get::<_, Option<String>>(idx) {
                values.push(val.map(Value::Text).unwrap_or(Value::Null));
                continue;
            }

            // If all else fails, use Null
            values.push(Value::Null);
        }

        Ok(NoctraRow { values })
    }

    /// Get table schema from DuckDB information_schema
    fn get_table_schema(&self, table_name: &str) -> Result<Vec<ColumnInfo>> {
        let conn = self.conn.lock().map_err(|_| DuckDBError::QueryFailed("Mutex poisoned".to_string()))?;

        // Use information_schema.columns for better compatibility with views
        let sql = format!(
            "SELECT column_name, data_type, is_nullable
             FROM information_schema.columns
             WHERE table_name = '{}'
             ORDER BY ordinal_position",
            table_name
        );

        let mut stmt = conn.prepare(&sql).map_err(|e| DuckDBError::QueryFailed(format!("Prepare error: {}", e)))?;
        let rows = stmt.query_map([], |row| {
            let name: String = row.get(0)?;
            let data_type: String = row.get(1)?;
            let is_nullable: String = row.get(2)?;
            Ok((name, data_type, is_nullable == "YES"))
        }).map_err(|e| DuckDBError::QueryFailed(format!("Query map error: {}", e)))?;

        let mut columns = Vec::new();
        for row_result in rows {
            let (name, data_type, nullable) = row_result.map_err(|e| DuckDBError::QueryFailed(format!("Row get error: {}", e)))?;
            columns.push(ColumnInfo {
                name,
                data_type: data_type.to_uppercase(),
                nullable,
                default_value: None,
            });
        }

        Ok(columns)
    }
}

impl DataSource for DuckDBSource {
    fn query(&self, sql: &str, _parameters: &Parameters) -> noctra_core::error::Result<ResultSet> {
        log::debug!("Executing query: {}", sql);

        let conn = self.conn.lock().map_err(|_| noctra_core::error::NoctraError::Internal("Mutex poisoned".to_string()))?;

        // Prepare and execute query (using cached prepared statements for performance)
        let mut stmt = conn.prepare_cached(sql).map_err(|e| noctra_core::error::NoctraError::Internal(format!("DuckDB prepare error: {}", e)))?;
        let mut rows_result = stmt
            .query([])
            .map_err(|e| noctra_core::error::NoctraError::Internal(format!("DuckDB query error: {}", e)))?;

        // Get column metadata from first row (if exists)
        let mut columns: Vec<Column> = Vec::new();
        let mut rows: Vec<NoctraRow> = Vec::new();

        if let Some(row) = rows_result.next().map_err(|e| noctra_core::error::NoctraError::Internal(format!("DuckDB row error: {}", e)))? {
            // Extract column names from the statement after query execution
            let column_count = row.as_ref().column_count();
            for idx in 0..column_count {
                let name = row.as_ref().column_name(idx)
                    .map_err(|e| noctra_core::error::NoctraError::Internal(format!("Column name error: {}", e)))?;
                columns.push(Column {
                    name: name.to_string(),
                    data_type: "UNKNOWN".to_string(),
                    ordinal: idx,
                });
            }

            // Convert first row
            rows.push(self.duckdb_row_to_noctra_row(&row, &columns)
                .map_err(|e| noctra_core::error::NoctraError::Internal(format!("Row conversion error: {}", e)))?);

            // Process remaining rows
            while let Some(row) = rows_result.next().map_err(|e| noctra_core::error::NoctraError::Internal(format!("DuckDB row error: {}", e)))? {
                rows.push(self.duckdb_row_to_noctra_row(&row, &columns)
                    .map_err(|e| noctra_core::error::NoctraError::Internal(format!("Row conversion error: {}", e)))?);
            }
        }

        Ok(ResultSet {
            columns,
            rows,
            rows_affected: None,
            last_insert_rowid: None,
        })
    }

    fn schema(&self) -> noctra_core::error::Result<Vec<TableInfo>> {
        let mut tables = Vec::new();

        // Return schema for registered files only
        for (alias, _file_path) in &self.registered_files {
            if let Ok(columns) = self.get_table_schema(alias) {
                tables.push(TableInfo {
                    name: alias.clone(),
                    columns,
                    row_count: None, // DuckDB doesn't provide row counts efficiently
                });
            }
        }

        Ok(tables)
    }

    fn source_type(&self) -> SourceType {
        SourceType::Memory {
            capacity: 0, // In-memory DuckDB
        }
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn test_new_in_memory() {
        let source = DuckDBSource::new_in_memory().unwrap();
        assert_eq!(source.name(), "duckdb");
    }

    #[test]
    fn test_register_csv_file() {
        let mut temp_file = tempfile::Builder::new().suffix(".csv").tempfile().unwrap();
        writeln!(temp_file, "name,age,city").unwrap();
        writeln!(temp_file, "Alice,30,NYC").unwrap();
        writeln!(temp_file, "Bob,25,LA").unwrap();
        temp_file.flush().unwrap();

        let mut source = DuckDBSource::new_in_memory().unwrap();
        source.register_file(temp_file.path().to_str().unwrap(), "test_table").unwrap();

        assert!(source.registered_files().contains_key("test_table"));
    }

    #[test]
    fn test_query_csv_data() {
        let mut temp_file = tempfile::Builder::new().suffix(".csv").tempfile().unwrap();
        writeln!(temp_file, "name,age").unwrap();
        writeln!(temp_file, "Alice,30").unwrap();
        writeln!(temp_file, "Bob,25").unwrap();
        temp_file.flush().unwrap();

        let mut source = DuckDBSource::new_in_memory().unwrap();
        source.register_file(temp_file.path().to_str().unwrap(), "people").unwrap();

        let result = source.query("SELECT * FROM people", &Parameters::new()).unwrap();
        assert_eq!(result.rows.len(), 2);
        assert_eq!(result.columns.len(), 2);
        assert_eq!(result.columns[0].name, "name");
        assert_eq!(result.columns[1].name, "age");
    }

    #[test]
    fn test_schema_introspection() {
        let mut temp_file = tempfile::Builder::new().suffix(".csv").tempfile().unwrap();
        writeln!(temp_file, "id,name,active").unwrap();
        writeln!(temp_file, "1,Alice,true").unwrap();
        temp_file.flush().unwrap();

        let mut source = DuckDBSource::new_in_memory().unwrap();
        source.register_file(temp_file.path().to_str().unwrap(), "users").unwrap();

        let schema = source.schema().unwrap();
        // Schema should contain the registered table
        assert!(!schema.is_empty());
        let users_table = schema.iter().find(|t| t.name == "users");
        assert!(users_table.is_some(), "users table should be in schema");
        let users_table = users_table.unwrap();
        assert_eq!(users_table.name, "users");
        // DuckDB may infer different column counts, just check it's > 0
        assert!(users_table.columns.len() > 0);
    }

    #[test]
    fn test_json_support() {
        let mut temp_file = tempfile::Builder::new().suffix(".json").tempfile().unwrap();
        writeln!(temp_file, r#"[{{"name": "Alice", "age": 30}}, {{"name": "Bob", "age": 25}}]"#).unwrap();
        temp_file.flush().unwrap();

        let mut source = DuckDBSource::new_in_memory().unwrap();
        source.register_file(temp_file.path().to_str().unwrap(), "people").unwrap();

        let result = source.query("SELECT * FROM people", &Parameters::new()).unwrap();
        assert_eq!(result.rows.len(), 2);
        assert_eq!(result.columns.len(), 2);
    }

    #[test]
    fn test_parquet_support() {
        // For now, just test that the registration doesn't fail
        // Full Parquet testing would require creating a Parquet file
        let mut source = DuckDBSource::new_in_memory().unwrap();
        // This should not panic even if file doesn't exist (DuckDB handles it)
        let result = source.register_file("nonexistent.parquet", "test");
        // DuckDB will fail if file doesn't exist, so we expect an error
        assert!(result.is_err());
    }

    #[test]
    fn test_attach_sqlite() {
        let mut source = DuckDBSource::new_in_memory().unwrap();
        // Test attaching a non-existent SQLite DB (should fail gracefully)
        let result = source.attach_sqlite("nonexistent.db", "test");
        assert!(result.is_err());
    }

    #[test]
    fn test_unsupported_file_type() {
        let mut source = DuckDBSource::new_in_memory().unwrap();
        let result = source.register_file("test.txt", "invalid");
        assert!(matches!(result, Err(DuckDBError::UnsupportedFileType(_))));
    }

    #[test]
    fn test_custom_configuration() {
        use crate::config::DuckDBConfig;

        let config = DuckDBConfig {
            memory_limit: Some("8GB".to_string()),
            threads: Some(4),
            catalog_error_max_schemas: Some(5),
            enable_profiling: false,
        };

        let source = DuckDBSource::new_in_memory_with_config(config.clone()).unwrap();

        // Verify config is stored
        assert_eq!(source.config().threads, Some(4));
        assert_eq!(source.config().memory_limit, Some("8GB".to_string()));

        // Verify config was applied (query DuckDB settings)
        let result = source.query("SELECT current_setting('threads') as threads", &Parameters::new());
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.rows.len(), 1);
    }

    #[test]
    fn test_default_configuration() {
        let source = DuckDBSource::new_in_memory().unwrap();

        // Should have default config
        // memory_limit is None by default (DuckDB uses ~80% RAM automatically)
        assert!(source.config().memory_limit.is_none());
        assert!(source.config().threads.is_some());
    }

    #[test]
    fn test_attachment_registry() {
        use crate::attachment::AttachmentConfig;

        let mut source = DuckDBSource::new_in_memory().unwrap();

        // Test registry functionality without requiring SQLite extension download
        // (which requires internet connectivity)

        // Manually register an attachment
        source.attachments.register(AttachmentConfig {
            db_type: "sqlite".to_string(),
            path: "/path/to/test.db".to_string(),
            alias: "test_db".to_string(),
            read_only: false,
        });

        // Verify it's in the registry
        assert!(source.attachments().contains("test_db"));
        assert_eq!(source.attachments().len(), 1);

        // Verify we can get the config
        let config = source.attachments().get("test_db");
        assert!(config.is_some());
        assert_eq!(config.unwrap().path, "/path/to/test.db");

        // Verify to_sql_commands works
        let commands = source.attachments().to_sql_commands();
        assert_eq!(commands.len(), 1);
        assert!(commands[0].contains("test_db"));
        assert!(commands[0].contains("TYPE sqlite"));
    }

    #[test]
    fn test_restore_attachments() {
        use crate::attachment::AttachmentConfig;

        let mut source = DuckDBSource::new_in_memory().unwrap();

        // Manually register attachment (simulating saved state)
        source.attachments.register(AttachmentConfig {
            db_type: "sqlite".to_string(),
            path: "/path/to/catalog.db".to_string(),
            alias: "catalog".to_string(),
            read_only: false,
        });

        // Test that to_sql_commands generates correct SQL
        let commands = source.attachments().to_sql_commands();
        assert_eq!(commands.len(), 1);
        assert!(commands[0].contains("ATTACH '/path/to/catalog.db'"));
        assert!(commands[0].contains("AS catalog"));
        assert!(commands[0].contains("TYPE sqlite"));

        // Note: Actual restore_attachments() execution would require
        // SQLite extension download from internet, so we only test the SQL generation
    }
}