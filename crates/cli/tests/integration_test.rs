//! Tests de integración end-to-end para Noctra CLI

use noctra_cli::{CliConfig, Repl, ReplArgs};
use noctra_core::{Executor, RqlQuery, Session, SqliteBackend};
use std::collections::HashMap;
use std::sync::Arc;

#[tokio::test]
async fn test_simple_select_query() {
    // Crear configuración para SQLite en memoria
    let config = CliConfig::for_memory_sqlite();

    // Crear backend SQLite en memoria
    let backend = SqliteBackend::with_file(":memory:").unwrap();
    let executor = Executor::new(Arc::new(backend));
    let session = Session::new();

    // Ejecutar query simple
    let query = RqlQuery::new("SELECT 1 + 1 AS result", HashMap::new());
    let result = executor.execute_rql(&session, query).unwrap();

    // Verificar resultados
    assert_eq!(result.columns.len(), 1);
    assert_eq!(result.columns[0].name, "result");
    assert_eq!(result.rows.len(), 1);
    assert_eq!(result.rows[0].values.len(), 1);
}

#[tokio::test]
async fn test_create_and_select_table() {
    let config = CliConfig::for_memory_sqlite();
    let backend = SqliteBackend::with_file(":memory:").unwrap();
    let executor = Executor::new(Arc::new(backend));
    let session = Session::new();

    // Crear tabla
    let create_query = RqlQuery::new(
        "CREATE TABLE users (id INTEGER PRIMARY KEY, name TEXT)",
        HashMap::new(),
    );
    executor.execute_rql(&session, create_query).unwrap();

    // Insertar datos
    let insert_query = RqlQuery::new(
        "INSERT INTO users (id, name) VALUES (1, 'Alice'), (2, 'Bob')",
        HashMap::new(),
    );
    let insert_result = executor.execute_rql(&session, insert_query).unwrap();

    // Verificar filas afectadas
    assert_eq!(insert_result.rows_affected, Some(2));

    // Seleccionar datos
    let select_query = RqlQuery::new("SELECT * FROM users ORDER BY id", HashMap::new());
    let result = executor.execute_rql(&session, select_query).unwrap();

    // Verificar resultados
    assert_eq!(result.columns.len(), 2);
    assert_eq!(result.rows.len(), 2);
    assert_eq!(result.rows[0].values.len(), 2);
}

#[tokio::test]
async fn test_repl_creation() {
    // Test que el REPL se puede crear sin errores
    let config = CliConfig::for_memory_sqlite();
    let args = ReplArgs::default();

    let repl_result = Repl::new(config, args);
    assert!(repl_result.is_ok(), "REPL debería crearse exitosamente");
}

#[test]
fn test_query_formatting() {
    // Test que el formateador de output funciona
    use noctra_cli::format_result_set;
    use noctra_core::{Column, ResultSet, Row, Value};

    let result = ResultSet {
        columns: vec![
            Column::new("id", "INTEGER", 0),
            Column::new("name", "TEXT", 1),
        ],
        rows: vec![
            Row {
                values: vec![Value::Integer(1), Value::Text("Alice".to_string())],
            },
            Row {
                values: vec![Value::Integer(2), Value::Text("Bob".to_string())],
            },
        ],
        rows_affected: None,
        last_insert_rowid: None,
    };

    let table = format_result_set(&result);

    // Verificar que contiene los datos
    assert!(table.contains("id"));
    assert!(table.contains("name"));
    assert!(table.contains("Alice"));
    assert!(table.contains("Bob"));
}
