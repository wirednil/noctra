//! Hybrid Query Engine Demo
//!
//! Demonstrates M6 Phase 2 features:
//! - QueryEngine with hybrid routing
//! - RQL commands (USE, ATTACH)
//! - Cross-source JOINs (CSV + SQLite)
//! - Configuration API
//! - Multi-format support
//!
//! Run with:
//! ```bash
//! DUCKDB_LIB_DIR=/opt/duckdb LD_LIBRARY_PATH=/opt/duckdb cargo run --example hybrid_demo
//! ```

use noctra_core::types::Parameters;
use noctra_core::datasource::DataSource;
use noctra_duckdb::{DuckDBConfig, DuckDBSource, QueryEngine, RqlParser, Statement};
use rusqlite::Connection as SqliteConnection;
use std::fs;
use tempfile::TempDir;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    println!("🚀 Noctra Hybrid Query Engine Demo");
    println!("{}", "=".repeat(60));
    println!();

    // Setup demo environment
    let temp_dir = TempDir::new()?;
    let (csv_path, sqlite_path, json_path) = setup_demo_data(&temp_dir)?;

    // Demo 1: Configuration API
    demo_configuration()?;

    // Demo 2: QueryEngine with file registration
    demo_file_queries(&csv_path, &json_path)?;

    // Demo 3: RQL Parser
    demo_rql_parser(&csv_path, &sqlite_path)?;

    // Demo 4: Cross-Source JOINs
    demo_cross_source_joins(&csv_path, &sqlite_path)?;

    // Demo 5: Complex Analytics
    demo_complex_analytics(&csv_path, &sqlite_path)?;

    println!();
    println!("{}", "=".repeat(60));
    println!("✅ Demo completed successfully!");

    Ok(())
}

/// Demo 1: Configuration API with different presets
fn demo_configuration() -> Result<(), Box<dyn std::error::Error>> {
    println!("📋 Demo 1: Configuration API");
    println!("{}", "-".repeat(60));

    // Local I/O optimized config
    let local_config = DuckDBConfig::local();
    println!("Local config (optimized for local files):");
    println!("  - Threads: {:?}", local_config.threads);
    println!("  - Memory: {:?}", local_config.memory_limit);

    // Remote I/O optimized config (S3, HTTP)
    let remote_config = DuckDBConfig::remote();
    println!("\nRemote config (optimized for S3/HTTP):");
    println!("  - Threads: {:?} (3x cores for network latency)", remote_config.threads);

    // Minimal config for embedded scenarios
    let minimal_config = DuckDBConfig::minimal();
    println!("\nMinimal config (embedded/testing):");
    println!("  - Memory limit: {:?}", minimal_config.memory_limit);
    println!("  - Threads: {:?}", minimal_config.threads);

    // Create source with custom config
    let _source = DuckDBSource::new_in_memory_with_config(local_config)?;
    println!("\n✅ Created DuckDB source with local config");

    println!();
    Ok(())
}

/// Demo 2: QueryEngine with file registration
fn demo_file_queries(csv_path: &str, json_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("📊 Demo 2: File Registration and Queries");
    println!("{}", "-".repeat(60));

    let source = DuckDBSource::new_in_memory()?;
    let mut engine = QueryEngine::new(source);

    // Register CSV file
    println!("Registering CSV file: {}", csv_path);
    engine.register_file(csv_path, "sales")?;
    println!("✅ Registered as 'sales' table");

    // Register JSON file
    println!("\nRegistering JSON file: {}", json_path);
    engine.register_file(json_path, "events")?;
    println!("✅ Registered as 'events' table");

    // Query CSV
    println!("\n📈 Query 1: Total sales by category");
    let sql = "SELECT category, COUNT(*) as count, SUM(price) as total FROM sales GROUP BY category";
    let result = engine.query(sql, &Parameters::new())?;

    println!("Results:");
    for row in &result.rows {
        println!("  {:?}", row);
    }

    // Query JSON
    println!("\n📈 Query 2: Active events");
    let sql = "SELECT * FROM events WHERE status = 'active'";
    let result = engine.query(sql, &Parameters::new())?;
    println!("Found {} active events", result.rows.len());

    // Cross-format JOIN
    println!("\n📈 Query 3: JOIN CSV and JSON");
    let sql = "SELECT s.name, s.price, e.event_type FROM sales s JOIN events e ON s.product_id = e.product_id";
    let result = engine.query(sql, &Parameters::new())?;
    println!("Found {} joined records", result.rows.len());

    println!();
    Ok(())
}

/// Demo 3: RQL Parser demonstration
fn demo_rql_parser(csv_path: &str, sqlite_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 Demo 3: RQL Parser (Extended SQL Commands)");
    println!("{}", "-".repeat(60));

    // Parse USE command
    let use_cmd = format!("USE '{}' AS products", csv_path);
    println!("Parsing: {}", use_cmd);
    let stmt = RqlParser::parse(&use_cmd)?;
    match stmt {
        Statement::Use { source, alias, source_type } => {
            println!("✅ Parsed USE command:");
            println!("   Source: {}", source);
            println!("   Alias: {}", alias);
            println!("   Type: {:?}", source_type);
        }
        _ => println!("❌ Unexpected statement type"),
    }

    // Parse ATTACH command
    println!();
    let attach_cmd = format!("ATTACH '{}' AS warehouse (TYPE sqlite)", sqlite_path);
    println!("Parsing: {}", attach_cmd);
    let stmt = RqlParser::parse(&attach_cmd)?;
    match stmt {
        Statement::Attach { path, alias, db_type } => {
            println!("✅ Parsed ATTACH command:");
            println!("   Path: {}", path);
            println!("   Alias: {}", alias);
            println!("   DB Type: {}", db_type);
        }
        _ => println!("❌ Unexpected statement type"),
    }

    // Parse DETACH command
    println!();
    let detach_cmd = "DETACH warehouse";
    println!("Parsing: {}", detach_cmd);
    let stmt = RqlParser::parse(detach_cmd)?;
    match stmt {
        Statement::Detach { alias } => {
            println!("✅ Parsed DETACH command:");
            println!("   Alias: {}", alias);
        }
        _ => println!("❌ Unexpected statement type"),
    }

    // Parse SHOW SOURCES command
    println!();
    let show_cmd = "SHOW SOURCES";
    println!("Parsing: {}", show_cmd);
    let stmt = RqlParser::parse(show_cmd)?;
    match stmt {
        Statement::ShowSources => {
            println!("✅ Parsed SHOW SOURCES command");
        }
        _ => println!("❌ Unexpected statement type"),
    }

    // Standard SQL passthrough
    println!();
    let sql = "SELECT * FROM products WHERE price > 100";
    println!("Parsing standard SQL: {}", sql);
    let stmt = RqlParser::parse(sql)?;
    match stmt {
        Statement::Query(query) => {
            println!("✅ Parsed as standard SQL query");
            println!("   Query: {}", query);
        }
        _ => println!("❌ Unexpected statement type"),
    }

    println!();
    Ok(())
}

/// Demo 4: Cross-Source JOINs
fn demo_cross_source_joins(csv_path: &str, sqlite_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("🔗 Demo 4: Cross-Source JOINs (CSV + SQLite)");
    println!("{}", "-".repeat(60));

    let source = DuckDBSource::new_in_memory()?;
    let mut engine = QueryEngine::new(source);

    // Register CSV file
    println!("Registering CSV file as 'products'");
    engine.register_file(csv_path, "products")?;

    // Try to attach SQLite database
    println!("Attempting to attach SQLite database as 'warehouse'");
    match engine.attach_sqlite(sqlite_path, "warehouse") {
        Ok(_) => {
            println!("✅ SQLite database attached successfully");

            // Execute cross-source JOIN
            println!("\n📈 Query: JOIN products (CSV) with inventory (SQLite)");
            let sql = "
                SELECT
                    p.name,
                    p.category,
                    p.price,
                    i.stock_quantity,
                    (p.price * i.stock_quantity) as inventory_value
                FROM products p
                LEFT JOIN warehouse.inventory i ON p.product_id = i.product_id
                ORDER BY inventory_value DESC NULLS LAST
            ";

            let result = engine.query(sql, &Parameters::new())?;
            println!("\nResults ({} rows):", result.rows.len());

            // Print results
            for (i, row) in result.rows.iter().enumerate().take(5) {
                println!("  Row {}: {:?}", i + 1, row);
            }

            if result.rows.len() > 5 {
                println!("  ... ({} more rows)", result.rows.len() - 5);
            }
        }
        Err(e) => {
            println!("⚠️  SQLite extension not available: {}", e);
            println!("   This is normal if DuckDB can't download the SQLite scanner extension");
            println!("   The cross-source JOIN functionality is implemented and tested");
        }
    }

    println!();
    Ok(())
}

/// Demo 5: Complex Analytics with CTEs
fn demo_complex_analytics(csv_path: &str, sqlite_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("📊 Demo 5: Complex Analytics (CTEs + Aggregations)");
    println!("{}", "-".repeat(60));

    let source = DuckDBSource::new_in_memory()?;
    let mut engine = QueryEngine::new(source);

    engine.register_file(csv_path, "products")?;

    match engine.attach_sqlite(sqlite_path, "warehouse") {
        Ok(_) => {
            println!("Running complex CTE-based analytics query...\n");

            let sql = "
                WITH inventory_summary AS (
                    SELECT
                        product_id,
                        SUM(stock_quantity) as total_stock,
                        AVG(stock_quantity) as avg_stock
                    FROM warehouse.inventory
                    GROUP BY product_id
                ),
                category_stats AS (
                    SELECT
                        p.category,
                        COUNT(*) as product_count,
                        AVG(p.price) as avg_price
                    FROM products p
                    GROUP BY p.category
                )
                SELECT
                    cs.category,
                    cs.product_count,
                    ROUND(cs.avg_price, 2) as avg_price,
                    COUNT(DISTINCT p.product_id) as products_in_stock
                FROM category_stats cs
                JOIN products p ON cs.category = p.category
                LEFT JOIN inventory_summary i ON p.product_id = i.product_id
                WHERE i.total_stock > 0
                GROUP BY cs.category, cs.product_count, cs.avg_price
                ORDER BY cs.product_count DESC
            ";

            let result = engine.query(sql, &Parameters::new())?;

            println!("📈 Category Analytics:");
            println!("   Total categories analyzed: {}", result.rows.len());

            for row in &result.rows {
                println!("   {:?}", row);
            }
        }
        Err(_) => {
            println!("⚠️  SQLite extension not available");
            println!("   Demonstrating CSV-only analytics instead\n");

            let sql = "
                SELECT
                    category,
                    COUNT(*) as product_count,
                    ROUND(AVG(price), 2) as avg_price,
                    ROUND(MIN(price), 2) as min_price,
                    ROUND(MAX(price), 2) as max_price
                FROM products
                GROUP BY category
                ORDER BY avg_price DESC
            ";

            let result = engine.query(sql, &Parameters::new())?;

            println!("📈 Category Analytics (CSV only):");
            for row in &result.rows {
                println!("   {:?}", row);
            }
        }
    }

    println!();
    Ok(())
}

/// Setup demo data files
fn setup_demo_data(temp_dir: &TempDir) -> Result<(String, String, String), Box<dyn std::error::Error>> {
    println!("🔧 Setting up demo data...");

    // Create CSV file: products.csv
    let csv_path = temp_dir.path().join("products.csv");
    let csv_data = "product_id,name,category,price
1,Gaming Laptop,Electronics,1299.99
2,Wireless Mouse,Electronics,29.99
3,Mechanical Keyboard,Electronics,149.99
4,Standing Desk,Furniture,499.99
5,Ergonomic Chair,Furniture,399.99
6,Monitor 27inch,Electronics,349.99
7,Desk Lamp,Furniture,79.99
8,USB-C Hub,Electronics,59.99";
    fs::write(&csv_path, csv_data)?;
    println!("  ✅ Created products.csv ({} products)", 8);

    // Create JSON file: events.json
    let json_path = temp_dir.path().join("events.json");
    let json_data = r#"[
        {"event_id":1,"product_id":1,"event_type":"sale","status":"active","timestamp":"2024-11-01"},
        {"event_id":2,"product_id":3,"event_type":"restock","status":"active","timestamp":"2024-11-02"},
        {"event_id":3,"product_id":5,"event_type":"sale","status":"completed","timestamp":"2024-11-03"},
        {"event_id":4,"product_id":2,"event_type":"promotion","status":"active","timestamp":"2024-11-04"}
    ]"#;
    fs::write(&json_path, json_data)?;
    println!("  ✅ Created events.json (4 events)");

    // Create SQLite database: warehouse.db
    let sqlite_path = temp_dir.path().join("warehouse.db");
    let conn = SqliteConnection::open(&sqlite_path)?;

    conn.execute(
        "CREATE TABLE inventory (
            inventory_id INTEGER PRIMARY KEY,
            product_id INTEGER,
            stock_quantity INTEGER,
            warehouse_location TEXT
        )",
        [],
    )?;

    conn.execute(
        "INSERT INTO inventory (inventory_id, product_id, stock_quantity, warehouse_location) VALUES
            (1, 1, 15, 'Warehouse A'),
            (2, 2, 150, 'Warehouse B'),
            (3, 3, 45, 'Warehouse A'),
            (4, 4, 8, 'Warehouse C'),
            (5, 5, 12, 'Warehouse C'),
            (6, 6, 25, 'Warehouse B'),
            (7, 7, 60, 'Warehouse A')",
        [],
    )?;

    drop(conn);
    println!("  ✅ Created warehouse.db (7 inventory records)");
    println!();

    Ok((
        csv_path.to_string_lossy().to_string(),
        sqlite_path.to_string_lossy().to_string(),
        json_path.to_string_lossy().to_string(),
    ))
}
