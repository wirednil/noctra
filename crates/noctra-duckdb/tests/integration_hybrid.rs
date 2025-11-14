//! Integration Tests for Hybrid Query Engine
//!
//! Tests the complete multi-source integration layer:
//! - Cross-source JOINs (CSV + SQLite)
//! - RQL command execution (USE, ATTACH, DETACH, SHOW SOURCES)
//! - QueryEngine routing
//! - File registration and querying
//! - AttachmentRegistry persistence

use noctra_core::types::Parameters;
use noctra_core::datasource::DataSource;
use noctra_duckdb::{DuckDBSource, QueryEngine, RqlParser, Statement, SourceType};
use rusqlite::Connection as SqliteConnection;
use std::fs;
use tempfile::TempDir;

/// Check if SQLite extension is available for DuckDB
/// We test this by attempting to attach a dummy database
fn sqlite_extension_available() -> bool {
    let temp_dir = TempDir::new().unwrap();
    let test_db = temp_dir.path().join("test.db");

    // Create minimal SQLite database
    let conn = SqliteConnection::open(&test_db).unwrap();
    conn.execute("CREATE TABLE test (id INTEGER)", []).unwrap();
    drop(conn);

    // Try to attach it with DuckDB
    let mut source = DuckDBSource::new_in_memory().unwrap();
    source.attach_sqlite(test_db.to_str().unwrap(), "test_attach").is_ok()
}

/// Setup test environment with sample CSV and SQLite database
struct TestEnvironment {
    _temp_dir: TempDir,
    csv_path: String,
    sqlite_path: String,
}

impl TestEnvironment {
    fn new() -> Self {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");

        // Create sample CSV: products.csv
        let csv_path = temp_dir.path().join("products.csv");
        let csv_data = "product_id,name,category,price\n\
                        1,Laptop,Electronics,999.99\n\
                        2,Mouse,Electronics,29.99\n\
                        3,Desk,Furniture,299.99\n\
                        4,Chair,Furniture,199.99\n";
        fs::write(&csv_path, csv_data).expect("Failed to write CSV");

        // Create sample SQLite database: orders.db
        let sqlite_path = temp_dir.path().join("orders.db");
        let conn = SqliteConnection::open(&sqlite_path).expect("Failed to create SQLite DB");

        conn.execute(
            "CREATE TABLE orders (
                order_id INTEGER PRIMARY KEY,
                product_id INTEGER,
                quantity INTEGER,
                order_date TEXT
            )",
            [],
        ).expect("Failed to create orders table");

        conn.execute(
            "INSERT INTO orders (order_id, product_id, quantity, order_date) VALUES
                (101, 1, 2, '2024-01-15'),
                (102, 2, 5, '2024-01-16'),
                (103, 3, 1, '2024-01-17'),
                (104, 1, 1, '2024-01-18')",
            [],
        ).expect("Failed to insert orders");

        drop(conn);

        Self {
            csv_path: csv_path.to_string_lossy().to_string(),
            sqlite_path: sqlite_path.to_string_lossy().to_string(),
            _temp_dir: temp_dir,
        }
    }
}

#[test]
fn test_rql_use_command() {
    env_logger::try_init().ok();

    let env = TestEnvironment::new();
    let mut engine = QueryEngine::new(DuckDBSource::new_in_memory().unwrap());

    // Parse USE command
    let stmt = RqlParser::parse(&format!("USE '{}' AS products", env.csv_path)).unwrap();

    match stmt {
        Statement::Use { source, alias, source_type } => {
            assert_eq!(alias, "products");
            assert_eq!(source_type, SourceType::CSV);

            // Execute USE by registering file
            engine.register_file(&source, &alias).unwrap();
        }
        _ => panic!("Expected USE statement"),
    }

    // Query the registered file
    let result = engine.query(
        "SELECT * FROM products WHERE category = 'Electronics'",
        &Parameters::new(),
    ).unwrap();

    assert_eq!(result.rows.len(), 2);
    assert_eq!(result.columns.len(), 4);
}

#[test]
fn test_rql_attach_command() {
    env_logger::try_init().ok();

    if !sqlite_extension_available() {
        eprintln!("Skipping test_rql_attach_command - SQLite extension not available");
        return;
    }

    let env = TestEnvironment::new();
    let mut engine = QueryEngine::new(DuckDBSource::new_in_memory().unwrap());

    // Parse ATTACH command
    let stmt = RqlParser::parse(&format!(
        "ATTACH '{}' AS orders_db (TYPE sqlite)",
        env.sqlite_path
    )).unwrap();

    match stmt {
        Statement::Attach { path, alias, db_type } => {
            assert_eq!(alias, "orders_db");
            assert_eq!(db_type, "sqlite");

            // Execute ATTACH
            engine.attach_sqlite(&path, &alias).unwrap();
        }
        _ => panic!("Expected ATTACH statement"),
    }

    // Query the attached database
    let result = engine.query(
        "SELECT COUNT(*) as total_orders FROM orders_db.orders",
        &Parameters::new(),
    ).unwrap();

    assert_eq!(result.rows.len(), 1);
}

#[test]
fn test_cross_source_join() {
    env_logger::try_init().ok();

    if !sqlite_extension_available() {
        eprintln!("Skipping test_cross_source_join - SQLite extension not available");
        return;
    }

    let env = TestEnvironment::new();
    let mut engine = QueryEngine::new(DuckDBSource::new_in_memory().unwrap());

    // Register CSV file
    engine.register_file(&env.csv_path, "products").unwrap();

    // Attach SQLite database
    engine.attach_sqlite(&env.sqlite_path, "orders_db").unwrap();

    // Execute cross-source JOIN query
    let sql = "
        SELECT
            p.name,
            p.price,
            o.order_id,
            o.quantity,
            o.order_date,
            (p.price * o.quantity) as total_amount
        FROM products p
        JOIN orders_db.orders o ON p.product_id = o.product_id
        ORDER BY o.order_date
    ";

    let result = engine.query(sql, &Parameters::new()).unwrap();

    // Should return 4 orders with joined product information
    assert_eq!(result.rows.len(), 4);

    // Verify columns exist
    let column_names: Vec<&str> = result.columns.iter().map(|c| c.name.as_str()).collect();
    assert!(column_names.contains(&"name"));
    assert!(column_names.contains(&"price"));
    assert!(column_names.contains(&"total_amount"));
}

#[test]
fn test_aggregation_across_sources() {
    env_logger::try_init().ok();

    if !sqlite_extension_available() {
        eprintln!("Skipping test_aggregation_across_sources - SQLite extension not available");
        return;
    }

    let env = TestEnvironment::new();
    let mut engine = QueryEngine::new(DuckDBSource::new_in_memory().unwrap());

    engine.register_file(&env.csv_path, "products").unwrap();
    engine.attach_sqlite(&env.sqlite_path, "orders_db").unwrap();

    // Aggregate query across sources
    let sql = "
        SELECT
            p.category,
            SUM(o.quantity) as total_quantity,
            COUNT(DISTINCT o.order_id) as order_count
        FROM products p
        JOIN orders_db.orders o ON p.product_id = o.product_id
        GROUP BY p.category
        ORDER BY p.category
    ";

    let result = engine.query(sql, &Parameters::new()).unwrap();

    // Should have 2 categories: Electronics and Furniture
    assert_eq!(result.rows.len(), 2);
}

#[test]
fn test_attachment_registry_persistence() {
    env_logger::try_init().ok();

    if !sqlite_extension_available() {
        eprintln!("Skipping test_attachment_registry_persistence - SQLite extension not available");
        return;
    }

    let env = TestEnvironment::new();

    // Create source and attach database
    let mut source = DuckDBSource::new_in_memory().unwrap();
    source.attach_sqlite(&env.sqlite_path, "orders_db").unwrap();

    // Verify attachment is registered
    let attachments = source.attachments();
    assert_eq!(attachments.len(), 1);
    assert!(attachments.contains("orders_db"));

    // Test SQL generation for restoration
    let sql_commands = source.attachments().to_sql_commands();
    assert_eq!(sql_commands.len(), 1);
    assert!(sql_commands[0].contains("ATTACH"));
    assert!(sql_commands[0].contains("orders_db"));
    assert!(sql_commands[0].contains("TYPE sqlite"));
}

#[test]
fn test_multiple_file_formats() {
    env_logger::try_init().ok();

    let temp_dir = TempDir::new().unwrap();

    // Create CSV file
    let csv_path = temp_dir.path().join("data.csv");
    fs::write(&csv_path, "id,value\n1,100\n2,200\n").unwrap();

    // Create JSON file
    let json_path = temp_dir.path().join("data.json");
    fs::write(&json_path, r#"[{"id":1,"status":"active"},{"id":2,"status":"inactive"}]"#).unwrap();

    let mut engine = QueryEngine::new(DuckDBSource::new_in_memory().unwrap());

    // Register both files
    engine.register_file(csv_path.to_str().unwrap(), "csv_data").unwrap();
    engine.register_file(json_path.to_str().unwrap(), "json_data").unwrap();

    // Query CSV
    let result = engine.query("SELECT SUM(value) as total FROM csv_data", &Parameters::new()).unwrap();
    assert_eq!(result.rows.len(), 1);

    // Query JSON
    let result = engine.query("SELECT * FROM json_data WHERE status = 'active'", &Parameters::new()).unwrap();
    assert_eq!(result.rows.len(), 1);

    // JOIN across formats
    let result = engine.query(
        "SELECT c.id, c.value, j.status FROM csv_data c JOIN json_data j ON c.id = j.id",
        &Parameters::new(),
    ).unwrap();
    assert_eq!(result.rows.len(), 2);
}

#[test]
fn test_rql_parser_statement_types() {
    env_logger::try_init().ok();

    // Test USE parsing
    let stmt = RqlParser::parse("USE 'data.csv' AS mydata").unwrap();
    assert!(matches!(stmt, Statement::Use { .. }));

    // Test ATTACH parsing
    let stmt = RqlParser::parse("ATTACH 'db.sqlite' AS mydb (TYPE sqlite)").unwrap();
    assert!(matches!(stmt, Statement::Attach { .. }));

    // Test DETACH parsing
    let stmt = RqlParser::parse("DETACH mydata").unwrap();
    assert!(matches!(stmt, Statement::Detach { alias } if alias == "mydata"));

    // Test SHOW SOURCES parsing
    let stmt = RqlParser::parse("SHOW SOURCES").unwrap();
    assert!(matches!(stmt, Statement::ShowSources));

    // Test standard SQL passthrough
    let stmt = RqlParser::parse("SELECT * FROM table").unwrap();
    assert!(matches!(stmt, Statement::Query(_)));
}

#[test]
fn test_case_insensitive_commands() {
    env_logger::try_init().ok();

    // USE command - various cases
    let stmt = RqlParser::parse("use 'file.csv' as data").unwrap();
    assert!(matches!(stmt, Statement::Use { .. }));

    let stmt = RqlParser::parse("UsE 'file.csv' As data").unwrap();
    assert!(matches!(stmt, Statement::Use { .. }));

    // ATTACH command
    let stmt = RqlParser::parse("attach 'db.sqlite' as mydb").unwrap();
    assert!(matches!(stmt, Statement::Attach { .. }));

    // SHOW SOURCES
    let stmt = RqlParser::parse("show sources").unwrap();
    assert!(matches!(stmt, Statement::ShowSources));
}

#[test]
fn test_complex_cross_source_query() {
    env_logger::try_init().ok();

    if !sqlite_extension_available() {
        eprintln!("Skipping test_complex_cross_source_query - SQLite extension not available");
        return;
    }

    let env = TestEnvironment::new();
    let mut engine = QueryEngine::new(DuckDBSource::new_in_memory().unwrap());

    // Register multiple sources
    engine.register_file(&env.csv_path, "products").unwrap();
    engine.attach_sqlite(&env.sqlite_path, "orders_db").unwrap();

    // Complex query with subquery and CTE
    let sql = "
        WITH order_summary AS (
            SELECT
                product_id,
                SUM(quantity) as total_qty,
                COUNT(*) as order_count
            FROM orders_db.orders
            GROUP BY product_id
        )
        SELECT
            p.name,
            p.category,
            p.price,
            os.total_qty,
            os.order_count,
            (p.price * os.total_qty) as revenue
        FROM products p
        LEFT JOIN order_summary os ON p.product_id = os.product_id
        WHERE p.price > 50
        ORDER BY revenue DESC NULLS LAST
    ";

    let result = engine.query(sql, &Parameters::new()).unwrap();

    // Should return products over $50 with their order summaries
    assert!(result.rows.len() > 0);

    // Verify all columns are present
    let column_names: Vec<&str> = result.columns.iter().map(|c| c.name.as_str()).collect();
    assert!(column_names.contains(&"name"));
    assert!(column_names.contains(&"revenue"));
}

#[test]
fn test_error_handling_invalid_attachment() {
    env_logger::try_init().ok();

    let mut engine = QueryEngine::new(DuckDBSource::new_in_memory().unwrap());

    // Try to attach non-existent database
    let result = engine.attach_sqlite("/nonexistent/path/db.sqlite", "invalid_db");

    // Should return error
    assert!(result.is_err());
}

#[test]
fn test_query_registered_sources() {
    env_logger::try_init().ok();

    let env = TestEnvironment::new();
    let mut engine = QueryEngine::new(DuckDBSource::new_in_memory().unwrap());

    // Register file
    engine.register_file(&env.csv_path, "products").unwrap();

    // Verify we can query it
    let result = engine.query("SELECT COUNT(*) as cnt FROM products", &Parameters::new()).unwrap();
    assert_eq!(result.rows.len(), 1);

    // Register another file with same alias should work (overwrites)
    engine.register_file(&env.csv_path, "products").unwrap();

    // Query still works
    let result = engine.query("SELECT * FROM products", &Parameters::new()).unwrap();
    assert_eq!(result.rows.len(), 4);
}
