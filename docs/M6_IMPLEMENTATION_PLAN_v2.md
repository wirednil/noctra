# Milestone 6 v2 — "FABRIC" Implementation Plan (DuckDB Integration)

**Noctra(🦆) 2.0: Data Fabric Engine**
**Fecha de Inicio:** 11 de noviembre de 2025
**Duración:** 7 semanas (11 nov — 30 dic 2025) ← **EXTENDIDO por hallazgos de investigación**
**Versión Target:** v0.6.0
**Branch:** `milestone/6/*`
**Versión del Plan:** 2.0 (Actualizado con DuckDB Research Findings)

---

## 📋 CAMBIOS RESPECTO A v1

### Hallazgos Críticos de Investigación DuckDB

Esta versión incorpora hallazgos de investigación exhaustiva sobre DuckDB (Nov 2025) que requieren cambios arquitectónicos fundamentales:

| Área | v1 (Original) | **v2 (Research-Driven)** | Justificación |
|------|---------------|---------------------------|---------------|
| **Arrow Integration** | Opcional (Fase 5) | **MANDATORIO (Fase 1)** | Zero-copy, 10-50% mejor performance |
| **Thread Config** | Estático | **Dinámico (Local vs Remote)** | 2-5x speedup en I/O remoto |
| **Prepared Statements** | Implícito | **`prepare_cached()` obligatorio** | Cache reutiliza handles SQL |
| **ATTACH Persistence** | Asumido | **Non-persistent, re-attach required** | Cambio en init lifecycle |
| **Performance Targets** | CSV 10MB <500ms | **CSV 10MB <200ms** | Arrow + predicate pushdown |
| **Timeline** | 6 semanas | **7 semanas** | +1 semana para performance layer |

### Nueva Fase 1.5: Performance Configuration Layer

**Razón**: Los hallazgos revelaron que la configuración dinámica de threads y memoria es crítica para performance en producción, no opcional.

---

## 🎯 VISION STATEMENT

> **"No importes datos. Consúltalos."**
> **"Un archivo. Una tabla. Un lenguaje."**
> **"Noctra no necesita una base de datos. Tú sí."**

---

## 📊 PANORAMA EJECUTIVO

### Objetivo Estratégico

Transformar Noctra en un **Data Fabric Engine** mediante DuckDB, habilitando:

- 🦆 **Queries directos sobre archivos** sin staging (CSV, Parquet, JSON)
- ⚡ **Performance 10x superior** con zero-copy y lectura columnar
- 🔗 **JOINs cross-source** nativos (CSV + Parquet + SQLite)
- 📦 **Parquet first-class** para datasets grandes
- 🎯 **Modo híbrido**: DuckDB para archivos, SQLite para persistencia

### Arquitectura Transformación

```
ANTES (Pre-M6)                    DESPUÉS (M6 v2)
─────────────────                 ───────────────────
IMPORT 'file.csv'                 USE 'file.csv' AS t
  ↓ Staging                         ↓ Zero-copy
  ↓ SQLite                          ↓ DuckDB (Arrow)
  ↓ Manual parsing                  ↓ read_csv_auto()
SELECT * FROM table               SELECT * FROM t

csv_backend.rs (900 líneas)       ✅ ELIMINADO
CSV manual parsing                🦆 DuckDB nativo
JOIN imposible                    ✅ Cross-source JOINs
Max 100MB                         ♾️ Streaming ilimitado
```

---

## 🗓️ TIMELINE — 7 Semanas (Actualizado)

```
Noviembre 2025          Diciembre 2025
11  12  13  14  15  16  17  18  19  20  21  22  23  24  25  26  27  28  29  30
│   │   │   │   │   │   │   │   │   │   │   │   │   │   │   │   │   │   │   │
├───┴───┴───┴───┴───┴───┴───┼───┴───┴───┴───┴───┴───┴───┼───┴───┴───┴───┴───┤
│ FASE 1: FUNDACIÓN        │ FASE 1.5: PERF CONFIG  │ FASE 2: HÍBRIDO     │
├───────────────────────────┼─────────────────────────┼─────────────────────┤
│                           │                         │ FASE 3: RQL 4GL     │
├───────────────────────────┼─────────────────────────┼─────────────────────┤
│                           │                         │ FASE 4: EXPORT      │
├───────────────────────────┼─────────────────────────┼─────────────────────┤
│                           │                         │ FASE 5: TUI/UX      │
├───────────────────────────┼─────────────────────────┼─────────────────────┤
│                           │                         │ FASE 6: RELEASE     │
└───────────────────────────┴─────────────────────────┴─────────────────────┘
         Semana 1                  Semana 2               Semanas 3-7
```

**Cambio de Timeline**: +1 semana para implementar hallazgos de performance críticos.

---

## 📦 FASE 1: FUNDACIÓN — DuckDB Core (7 días)

**Fecha:** 11-17 nov 2025
**Objetivo:** Integración core de DuckDB con **Arrow desde día 1**

### 1.1 Nuevo Crate: `noctra-duckdb`

**Estructura:**
```
crates/noctra-duckdb/
├── Cargo.toml
├── src/
│   ├── lib.rs              # Entry point, re-exports
│   ├── source.rs           # DuckDBSource (DataSource trait impl)
│   ├── engine.rs           # Query execution engine
│   ├── arrow_convert.rs    # ← NUEVO v2: Arrow ↔ Noctra
│   ├── config.rs           # ← NUEVO v2: PerformanceConfig
│   ├── cache.rs            # ← NUEVO v2: Prepared statement cache
│   └── error.rs            # DuckDBError type
└── tests/
    └── integration.rs
```

**Cargo.toml (ACTUALIZADO v2):**
```toml
[package]
name = "noctra-duckdb"
version = "0.6.0"
edition = "2021"

[dependencies]
duckdb = { version = "1.1", features = ["bundled", "parquet", "json", "arrow"] }
arrow = "56.0"          # ← MANDATORIO en v2 (era opcional en v1)
noctra-core = { path = "../noctra-core" }
anyhow = "1.0"
log = "0.4"
thiserror = "1.0"

[dev-dependencies]
tempfile = "3.8"
env_logger = "0.11"
```

### 1.2 DuckDBSource Implementation (Arrow-First)

**Archivo:** `crates/noctra-duckdb/src/source.rs`

```rust
use duckdb::{Connection, params};
use arrow::record_batch::RecordBatch;
use noctra_core::{DataSource, ResultSet, Parameters, Column};

pub struct DuckDBSource {
    conn: Arc<Mutex<Connection>>,
    name: String,
    registered_files: HashMap<String, String>,  // alias → path
}

impl DuckDBSource {
    pub fn new_in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory()?;
        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
            name: "duckdb".to_string(),
            registered_files: HashMap::new(),
        })
    }

    /// Register file as virtual table using DuckDB functions
    pub fn register_file(&mut self, file_path: &str, alias: &str) -> Result<()> {
        let extension = Path::new(file_path)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");

        let sql = match extension {
            "csv" | "tsv" => {
                format!("CREATE VIEW {} AS SELECT * FROM read_csv_auto('{}')",
                    alias, file_path)
            },
            "json" => {
                format!("CREATE VIEW {} AS SELECT * FROM read_json_auto('{}')",
                    alias, file_path)
            },
            "parquet" => {
                format!("CREATE VIEW {} AS SELECT * FROM read_parquet('{}')",
                    alias, file_path)
            },
            _ => return Err(anyhow!("Unsupported file type: {}", extension)),
        };

        log::debug!("Registering file: {} -> {}", file_path, sql);
        let conn = self.conn.lock().map_err(|_| DuckDBError::QueryFailed("Mutex poisoned".to_string()))?;
        conn.execute(&sql, [])?;
        self.registered_files.insert(alias.to_string(), file_path.to_string());
        Ok(())
    }

    /// Attach SQLite database to DuckDB for cross-source queries
    pub fn attach_sqlite(&mut self, db_path: &str, alias: &str) -> Result<()> {
        let conn = self.conn.lock().map_err(|_| DuckDBError::QueryFailed("Mutex poisoned".to_string()))?;
        let sql = format!("ATTACH '{}' AS {} (TYPE SQLITE)", db_path, alias);
        log::debug!("Attaching SQLite DB: {}", sql);
        conn.execute(&sql, [])?;
        Ok(())
    }

    pub fn registered_files(&self) -> &HashMap<String, String> {
        &self.registered_files
    }
}

impl DataSource for DuckDBSource {
    /// Query using Arrow for zero-copy performance
    fn query(&self, sql: &str, _parameters: &Parameters) -> noctra_core::error::Result<ResultSet> {
        let conn = self.conn.lock()
            .map_err(|_| noctra_core::error::NoctraError::Internal("Mutex poisoned".to_string()))?;

        // v2: Use prepare_cached for performance
        let mut stmt = conn.prepare_cached(sql)
            .map_err(|e| noctra_core::error::NoctraError::Internal(format!("DuckDB prepare error: {}", e)))?;

        // v2: ARROW FIRST - Zero-copy data transfer
        let arrow_batches: Vec<RecordBatch> = stmt.query_arrow([])
            .map_err(|e| noctra_core::error::NoctraError::Internal(format!("DuckDB query error: {}", e)))?
            .collect();

        // Convert Arrow → Noctra ResultSet
        crate::arrow_convert::arrow_to_result_set(arrow_batches)
            .map_err(|e| noctra_core::error::NoctraError::Internal(format!("Arrow conversion error: {}", e)))
    }

    fn schema(&self) -> noctra_core::error::Result<Vec<noctra_core::TableInfo>> {
        let conn = self.conn.lock()
            .map_err(|_| noctra_core::error::NoctraError::Internal("Mutex poisoned".to_string()))?;

        let sql = "SELECT table_name FROM information_schema.tables WHERE table_schema = 'main'";
        let mut stmt = conn.prepare(sql)
            .map_err(|e| noctra_core::error::NoctraError::Internal(format!("DuckDB prepare error: {}", e)))?;

        let table_names: Vec<String> = stmt
            .query_map([], |row| {
                row.get(0)
            })
            .map_err(|e| noctra_core::error::NoctraError::Internal(format!("DuckDB query error: {}", e)))?
            .collect::<DuckResult<Vec<_>>>()
            .map_err(|e| noctra_core::error::NoctraError::Internal(format!("DuckDB collect error: {}", e)))?;

        let mut tables = Vec::new();
        for table in table_names {
            let columns = self.get_table_columns(&table)?;
            tables.push(noctra_core::TableInfo {
                name: table,
                columns,
            });
        }

        Ok(tables)
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn source_type(&self) -> &noctra_core::SourceType {
        &noctra_core::SourceType::DuckDB
    }
}
```

### 1.3 Arrow Conversion Layer (NUEVO v2)

**Archivo:** `crates/noctra-duckdb/src/arrow_convert.rs`

```rust
use arrow::record_batch::RecordBatch;
use arrow::datatypes::{DataType, Schema};
use noctra_core::{ResultSet, Column, Row, Value};
use anyhow::Result;

/// Convert Arrow RecordBatches to Noctra ResultSet (zero-copy where possible)
pub fn arrow_to_result_set(batches: Vec<RecordBatch>) -> Result<ResultSet> {
    if batches.is_empty() {
        return Ok(ResultSet {
            columns: vec![],
            rows: vec![],
            rows_affected: None,
            last_insert_rowid: None,
        });
    }

    // Extract schema from first batch
    let schema = batches[0].schema();
    let columns = arrow_schema_to_columns(&schema);

    // Convert all batches to rows
    let mut rows = Vec::new();
    for batch in batches {
        for row_idx in 0..batch.num_rows() {
            let row = arrow_row_to_noctra_row(&batch, row_idx)?;
            rows.push(row);
        }
    }

    Ok(ResultSet {
        columns,
        rows,
        rows_affected: None,
        last_insert_rowid: None,
    })
}

/// Convert Arrow Schema to Noctra Columns
fn arrow_schema_to_columns(schema: &Schema) -> Vec<Column> {
    schema.fields().iter().enumerate().map(|(idx, field)| {
        Column {
            name: field.name().clone(),
            data_type: arrow_type_to_string(field.data_type()),
            ordinal: idx,
        }
    }).collect()
}

/// Map Arrow DataType to SQL type string
fn arrow_type_to_string(dtype: &DataType) -> String {
    match dtype {
        DataType::Int8 | DataType::Int16 | DataType::Int32 | DataType::Int64 => "INTEGER".to_string(),
        DataType::UInt8 | DataType::UInt16 | DataType::UInt32 | DataType::UInt64 => "INTEGER".to_string(),
        DataType::Float32 | DataType::Float64 => "REAL".to_string(),
        DataType::Boolean => "BOOLEAN".to_string(),
        DataType::Utf8 | DataType::LargeUtf8 => "TEXT".to_string(),
        DataType::Date32 | DataType::Date64 => "DATE".to_string(),
        DataType::Timestamp(..) => "DATETIME".to_string(),
        _ => "TEXT".to_string(),  // Fallback
    }
}

/// Convert Arrow row to Noctra Row
fn arrow_row_to_noctra_row(batch: &RecordBatch, row_idx: usize) -> Result<Row> {
    let mut values = Vec::new();

    for col_idx in 0..batch.num_columns() {
        let array = batch.column(col_idx);
        let value = arrow_value_to_noctra(array, row_idx)?;
        values.push(value);
    }

    Ok(Row { values })
}

/// Convert Arrow value to Noctra Value
fn arrow_value_to_noctra(array: &dyn arrow::array::Array, row_idx: usize) -> Result<Value> {
    use arrow::array::*;

    if array.is_null(row_idx) {
        return Ok(Value::Null);
    }

    match array.data_type() {
        DataType::Int32 => {
            let array = array.as_any().downcast_ref::<Int32Array>().unwrap();
            Ok(Value::Integer(array.value(row_idx) as i64))
        },
        DataType::Int64 => {
            let array = array.as_any().downcast_ref::<Int64Array>().unwrap();
            Ok(Value::Integer(array.value(row_idx)))
        },
        DataType::Float64 => {
            let array = array.as_any().downcast_ref::<Float64Array>().unwrap();
            Ok(Value::Float(array.value(row_idx)))
        },
        DataType::Boolean => {
            let array = array.as_any().downcast_ref::<BooleanArray>().unwrap();
            Ok(Value::Boolean(array.value(row_idx)))
        },
        DataType::Utf8 => {
            let array = array.as_any().downcast_ref::<StringArray>().unwrap();
            Ok(Value::Text(array.value(row_idx).to_string()))
        },
        _ => {
            // Fallback: convert to string
            Ok(Value::Text(format!("{:?}", array)))
        }
    }
}
```

### 1.4 Tests Básicos

**Archivo:** `crates/noctra-duckdb/tests/integration.rs`

```rust
use noctra_duckdb::DuckDBSource;
use noctra_core::datasource::DataSource;
use std::io::Write;

#[test]
fn test_register_csv_and_query() {
    let mut temp_file = tempfile::Builder::new().suffix(".csv").tempfile().unwrap();
    writeln!(temp_file, "name,age,city").unwrap();
    writeln!(temp_file, "Alice,30,NYC").unwrap();
    writeln!(temp_file, "Bob,25,LA").unwrap();
    temp_file.flush().unwrap();

    let mut source = DuckDBSource::new_in_memory().unwrap();
    source.register_file(temp_file.path().to_str().unwrap(), "test_table").unwrap();

    let result = source.query("SELECT * FROM test_table", &noctra_core::Parameters::new()).unwrap();
    assert_eq!(result.rows.len(), 2);
    assert_eq!(result.columns.len(), 3);
}

#[test]
fn test_arrow_conversion() {
    let mut source = DuckDBSource::new_in_memory().unwrap();
    source.query("SELECT 1 AS num, 'test' AS str", &noctra_core::Parameters::new()).unwrap();
    // Should not panic - Arrow conversion works
}
```

### Entregables Fase 1

- [x] Crate `noctra-duckdb` creado
- [x] `DuckDBSource` implementado con `DataSource` trait
- [x] `register_file()` para CSV, JSON, Parquet
- [x] `attach_sqlite()` para cross-source
- [x] **Arrow conversion layer completo** (v2)
- [x] **`prepare_cached()` usage** (v2)
- [x] Tests básicos pasando

**Criterio de Éxito v2:**
```rust
let mut source = DuckDBSource::new_in_memory()?;
source.register_file("ventas.csv", "v")?;

// Arrow zero-copy
let result = source.query("SELECT * FROM v LIMIT 5", &Parameters::new())?;
assert_eq!(result.rows.len(), 5);
```

---

## ⚡ FASE 1.5: PERFORMANCE CONFIGURATION LAYER (NUEVO v2)

**Fecha:** 18-19 nov 2025 (2 días)
**Objetivo:** Implementar configuración dinámica de threads y memoria

**Razón de Existencia**: Los hallazgos de investigación revelaron que:
1. **Thread count debe ser dinámico**: Local (CPU cores) vs Remote (3-5x cores)
2. **Memory limits son críticos**: Buffer manager puede desbordar
3. **Prepared statement cache**: Debe configurarse explícitamente

### 1.5.1 Performance Configuration

**Archivo:** `crates/noctra-duckdb/src/config.rs`

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceConfig {
    /// Base thread count (typically CPU cores)
    pub base_threads: usize,

    /// Multiplier for remote I/O (2.0-5.0)
    pub remote_multiplier: f32,

    /// Memory limit for DuckDB buffer manager
    pub memory_limit: String,  // e.g., "16GB"

    /// I/O type detection strategy
    pub io_type: IOType,

    /// Prepared statement cache size
    pub stmt_cache_capacity: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IOType {
    /// Local file system (threads = base_threads)
    Local,

    /// Remote storage (threads = base_threads * multiplier)
    Remote,

    /// Auto-detect based on path (s3://, http://, etc.)
    Adaptive,
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            base_threads: num_cpus::get(),
            remote_multiplier: 3.0,
            memory_limit: "2GB".to_string(),
            io_type: IOType::Adaptive,
            stmt_cache_capacity: 100,
        }
    }
}

impl PerformanceConfig {
    /// Calculate optimal thread count based on I/O type
    pub fn calculate_threads(&self, path: &str) -> usize {
        match self.io_type {
            IOType::Local => self.base_threads,
            IOType::Remote => (self.base_threads as f32 * self.remote_multiplier) as usize,
            IOType::Adaptive => {
                if Self::is_remote_path(path) {
                    (self.base_threads as f32 * self.remote_multiplier) as usize
                } else {
                    self.base_threads
                }
            }
        }
    }

    fn is_remote_path(path: &str) -> bool {
        path.starts_with("s3://") ||
        path.starts_with("http://") ||
        path.starts_with("https://")
    }
}
```

### 1.5.2 Integrar en DuckDBSource

```rust
impl DuckDBSource {
    pub fn configure_performance(&self, config: &PerformanceConfig, path: &str) -> Result<()> {
        let conn = self.conn.lock().map_err(|_| DuckDBError::QueryFailed("Mutex poisoned".to_string()))?;

        // Set thread count
        let threads = config.calculate_threads(path);
        conn.execute(&format!("SET threads = {}", threads), [])?;
        log::debug!("Configured {} threads for path: {}", threads, path);

        // Set memory limit
        conn.execute(&format!("SET memory_limit = '{}'", config.memory_limit), [])?;
        log::debug!("Memory limit: {}", config.memory_limit);

        // Set prepared statement cache capacity
        conn.set_prepared_statement_cache_capacity(config.stmt_cache_capacity);
        log::debug!("Statement cache capacity: {}", config.stmt_cache_capacity);

        Ok(())
    }
}
```

### 1.5.3 Configuración Global

**Archivo:** `~/.config/noctra/config.toml`

```toml
[duckdb.performance]
base_threads = 8
remote_multiplier = 3.0
memory_limit = "16GB"
io_type = "adaptive"
stmt_cache_capacity = 100

[duckdb.tuning]
# Diagnostic settings
catalog_error_max_schemas = 10  # Speed up error messages
enable_profiling = false
```

### Entregables Fase 1.5

- [ ] `PerformanceConfig` struct implementado
- [ ] `IOType::Adaptive` con auto-detección
- [ ] `configure_performance()` method
- [ ] Config TOML support
- [ ] Tests: local vs remote thread calculation
- [ ] Documentación de performance tuning

**Criterio de Éxito:**
```rust
let config = PerformanceConfig::default();

// Local file: 8 threads
assert_eq!(config.calculate_threads("./data.csv"), 8);

// S3 file: 24 threads (8 * 3.0)
assert_eq!(config.calculate_threads("s3://bucket/data.csv"), 24);
```

---

## 🔗 FASE 2: MOTOR HÍBRIDO (7 días)

**Fecha:** 20-26 nov 2025
**Objetivo:** Modo híbrido DuckDB + SQLite

### 2.1 QueryEngine Evolution

**Archivo:** `crates/core/src/engine.rs`

```rust
pub enum QueryEngine {
    Sqlite(Box<dyn DatabaseBackend>),
    DuckDB(DuckDBSource),        // ← NUEVO
    Hybrid {                      // ← NUEVO (default)
        duckdb: DuckDBSource,
        sqlite: SqliteBackend,
        attachment_registry: AttachmentRegistry,  // ← v2: Track attachments
    },
}

impl QueryEngine {
    pub fn new_hybrid() -> Result<Self> {
        Ok(Self::Hybrid {
            duckdb: DuckDBSource::new_in_memory()?,
            sqlite: SqliteBackend::new_in_memory()?,
            attachment_registry: AttachmentRegistry::new(),
        })
    }

    /// v2: Initialize with re-attachment of databases
    pub fn initialize(&mut self) -> Result<()> {
        if let Self::Hybrid { duckdb, attachment_registry, .. } = self {
            // Load extensions
            duckdb.load_extensions()?;

            // Re-attach all registered databases (v2: non-persistent fix)
            for (alias, path) in attachment_registry.entries() {
                duckdb.attach_sqlite(path, alias)?;
            }
        }
        Ok(())
    }

    pub fn execute(&mut self, nql: &NqlStatement, params: &Parameters) -> Result<ResultSet> {
        match self {
            Self::DuckDB(source) => source.query(nql.as_sql(), params),
            Self::Hybrid { duckdb, sqlite, .. } => {
                // Route based on source type
                match nql.source_type()? {
                    SourceType::Csv | SourceType::Json | SourceType::Parquet
                        => duckdb.query(nql.as_sql(), params),
                    SourceType::Sqlite
                        => sqlite.execute(nql.as_sql(), params),
                    SourceType::DuckDB
                        => duckdb.query(nql.as_sql(), params),
                }
            },
            Self::Sqlite(backend) => backend.execute(nql.as_sql(), params),
        }
    }
}
```

### 2.2 Attachment Registry (NUEVO v2)

**Archivo:** `crates/noctra-duckdb/src/attachment.rs`

**Razón**: ATTACH statements no persisten entre sesiones (hallazgo v2)

```rust
use std::collections::HashMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttachmentRegistry {
    entries: HashMap<String, String>,  // alias → path
}

impl AttachmentRegistry {
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
        }
    }

    pub fn register(&mut self, alias: String, path: String) {
        self.entries.insert(alias, path);
    }

    pub fn entries(&self) -> &HashMap<String, String> {
        &self.entries
    }

    /// Save to session file
    pub fn save(&self, path: &Path) -> Result<()> {
        let toml = toml::to_string(&self)?;
        std::fs::write(path, toml)?;
        Ok(())
    }

    /// Load from session file
    pub fn load(path: &Path) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        Ok(toml::from_str(&content)?)
    }
}
```

### 2.3 Materialization Strategy (v2: Performance)

**Hallazgo v2**: Large SQLite tables should be materialized in DuckDB for performance

```rust
impl QueryEngine {
    /// Materialize SQLite table into DuckDB for analytical queries
    pub fn materialize_sqlite_table(&mut self, table: &str, threshold_rows: usize) -> Result<()> {
        if let Self::Hybrid { duckdb, sqlite, .. } = self {
            // Check table size
            let count: i64 = sqlite.query(&format!("SELECT COUNT(*) FROM {}", table), &Parameters::new())?
                .rows[0].values[0].as_integer()?;

            if count as usize > threshold_rows {
                log::info!("Materializing table {} ({} rows) into DuckDB", table, count);

                // Materialize: INSERT INTO duckdb FROM sqlite
                duckdb.query(&format!(
                    "CREATE TABLE {} AS SELECT * FROM sqlite_db.{}",
                    table, table
                ), &Parameters::new())?;
            }
        }
        Ok(())
    }
}
```

### Entregables Fase 2

- [ ] `QueryEngine::Hybrid` implementado
- [ ] `AttachmentRegistry` para tracking
- [ ] `initialize()` con re-attachment
- [ ] `materialize_sqlite_table()` strategy
- [ ] Tests: cross-source JOIN
- [ ] Tests: re-attachment after restart

**Criterio de Éxito v2:**
```rust
let mut engine = QueryEngine::new_hybrid()?;
engine.initialize()?;  // Re-attach databases

// Cross-source JOIN
let result = engine.execute(
    &parse("SELECT c.name, v.total FROM 'ventas.csv' v JOIN clientes.db::users c ON v.id = c.id")?,
    &Parameters::new()
)?;
```

---

## 🎯 FASE 3: RQL 4GL CONSOLIDATION (7 días)

**Fecha:** 27 nov - 3 dic 2025

### Extensions to MAINTAIN

```rust
// Already implemented, keep as-is
LET var = valor                    // ✅ Session variables
#var in SQL                        // ✅ Variable interpolation
SHOW VARS                          // ✅ Variable listing
SHOW SOURCES                       // ✅ Source catalog
DESCRIBE source.table              // ✅ Schema introspection
```

### Extensions to DEPRECATE

```rust
MAP expression                     // ❌ DEPRECATE → Use SQL SELECT
FILTER condition                   // ❌ DEPRECATE → Use SQL WHERE
OUTPUT TO 'file'                   // ❌ DEPRECATE → Use EXPORT
```

**Migration Guide:**
```sql
-- OLD (M5)
MAP nombre = UPPER(nombre);
FILTER activo = true;

-- NEW (M6)
SELECT UPPER(nombre) AS nombre, * FROM tabla WHERE activo = true;
```

---

## 📤 FASE 4: EXPORT & PARALLEL WRITES (7 días)

**Fecha:** 4-10 dic 2025

### 4.1 EXPORT Command

**Syntax:**
```sql
EXPORT (query) TO 'file.parquet' FORMAT PARQUET;
EXPORT (query) TO 'file.csv' FORMAT CSV;
EXPORT (query) TO 'file.json' FORMAT JSON;
```

### 4.2 Parallel Parquet Export (v2: Critical)

**Hallazgo v2**: `PER_THREAD_OUTPUT` is **mandatory** for large exports

```rust
impl DuckDBSource {
    pub fn export(&self, query: &str, path: &str, format: ExportFormat) -> Result<()> {
        let conn = self.conn.lock()?;

        let export_sql = match format {
            ExportFormat::Parquet => {
                // v2: ALWAYS use PER_THREAD_OUTPUT for Parquet
                format!(
                    "COPY ({}) TO '{}' (FORMAT PARQUET, COMPRESSION zstd, PER_THREAD_OUTPUT)",
                    query, path
                )
            },
            ExportFormat::Csv => {
                format!("COPY ({}) TO '{}' (FORMAT CSV, HEADER true)", query, path)
            },
            ExportFormat::Json => {
                format!("COPY ({}) TO '{}' (FORMAT JSON)", query, path)
            },
        };

        conn.execute(&export_sql, [])?;
        Ok(())
    }
}
```

---

## 🖥️ FASE 5: TUI/UX ENHANCEMENTS (7 días)

**Fecha:** 11-17 dic 2025

### 5.1 Dynamic Status Bar (v2)

```
Engine: 🦆 DuckDB │ I/O: 🌐 Remote (threads=24) │ Source: sales.parquet (1.2GB) │ Memory: 45MB/16GB │ 8ms
```

**Implementation:**
```rust
fn render_status_bar(&self) -> String {
    let io_icon = match self.engine.config.io_type {
        IOType::Local => "💾",
        IOType::Remote => "🌐",
        IOType::Adaptive => "🔄",
    };

    let threads = self.engine.get_config("threads")?;
    let memory_used = self.engine.get_memory_usage_mb();
    let memory_limit = self.engine.get_config("memory_limit")?;

    format!(
        "Engine: 🦆 DuckDB │ I/O: {} {} (threads={}) │ Memory: {}MB/{}",
        io_icon, self.config.io_type, threads, memory_used, memory_limit
    )
}
```

---

## 📚 FASE 6: RELEASE & DOCUMENTATION (7 días)

**Fecha:** 18-24 dic 2025

### 6.1 Documentation

**NEW FILES:**
```
docs/
├── M6_IMPLEMENTATION_PLAN_v2.md    # ← Este documento
├── RQL_EXTENSIONS.md                # ← Extensions reference
├── MIGRATION_M5_TO_M6.md            # ← Migration guide
├── PERFORMANCE_TUNING.md            # ← v2: Performance guide
└── BENCHMARKS.md                    # ← v2: Benchmark results
```

### 6.2 Performance Tuning Guide (NUEVO v2)

**Archivo:** `docs/PERFORMANCE_TUNING.md`

```markdown
# Noctra Performance Tuning Guide

## Thread Configuration

| Scenario | Recommendation | Configuration |
|----------|----------------|---------------|
| Local CSV/Parquet | threads = CPU cores | `base_threads = 8` |
| S3/HTTP Queries | threads = 3-5x cores | `remote_multiplier = 3.0` |
| Mixed Workload | Use `IOType::Adaptive` | `io_type = "adaptive"` |

## Memory Limits

- Development: `memory_limit = "4GB"`
- Production (16GB RAM): `memory_limit = "12GB"` (leave 4GB for OS)
- Large Aggregations: `memory_limit = "32GB"` (if available)

## ATTACH vs Materialization

```
SQLite Table Size Decision Tree:
├─ < 10K rows → Use ATTACH (virtual query)
├─ 10K-1M rows → Materialize if queried >3 times
└─ > 1M rows → ALWAYS materialize to DuckDB
```

## Export Strategies

- Small Results (<100MB): Single file CSV
- Large Results (>1GB): Parquet with `PER_THREAD_OUTPUT`
- Archival: Parquet with `COMPRESSION zstd`

## Prepared Statement Cache

```toml
[duckdb.performance]
stmt_cache_capacity = 100  # Default
# Increase for heavy query workloads
stmt_cache_capacity = 500
```
```

### 6.3 Benchmarks (v2)

**Performance Targets (Updated):**

| Operation | v1 Target | **v2 Target (Arrow)** | Actual |
|-----------|-----------|------------------------|--------|
| CSV 10MB load | <500ms | **<200ms** | TBD |
| Parquet 100MB | <1s | **<300ms** | TBD |
| SQLite ATTACH (100K) | <2s | <5s (virtual) / <500ms (mat) | TBD |
| Export 1GB Parquet | N/A | **<3s (PER_THREAD)** | TBD |
| Cross-source JOIN | N/A | **<1s (100K rows)** | TBD |

---

## ✅ CRITERIOS DE ÉXITO ACTUALIZADOS (v2)

### Funcionales

- ✅ `USE 'file.csv' AS t` → DuckDB `read_csv_auto()`
- ✅ `SELECT * FROM 'file.csv'` directo sin registro
- ✅ JOIN CSV + SQLite sin staging
- ✅ EXPORT a Parquet con `PER_THREAD_OUTPUT`
- ✅ Arrow integration con zero-copy
- ✅ `prepare_cached()` para todas las queries

### Performance (v2)

- ✅ CSV 10MB: **<200ms** (era <500ms)
- ✅ Parquet 100MB: **<300ms** (era <1s)
- ✅ Export 1GB: **<3s** con parallel writes
- ✅ Remote I/O: threads auto-scale 3-5x
- ✅ Memory: <200MB para 1GB CSV (streaming)

### Quality

- ✅ Test coverage: >85%
- ✅ Zero clippy warnings
- ✅ Arrow conversion tests
- ✅ Performance tuning guide
- ✅ Benchmark suite

---

## 🚨 BREAKING CHANGES

1. **`csv_backend.rs` ELIMINADO**: Migrar a DuckDB
2. **`MAP`/`FILTER` DEPRECATED**: Usar SQL estándar
3. **`IMPORT` cambia a `USE`**: Sintaxis actualizada
4. **Config file format**: Nuevos campos de performance

**Migration Path:**
```sql
-- M5 (OLD)
IMPORT 'data.csv' AS t;
MAP col = UPPER(col);
FILTER active = true;

-- M6 (NEW)
USE 'data.csv' AS t;
SELECT UPPER(col) AS col, * FROM t WHERE active = true;
```

---

## 📊 ESTIMACIONES DE ESFUERZO (Actualizado)

| Fase | Días | Complejidad | Riesgo |
|------|------|-------------|--------|
| 1. Fundación | 7 | High | Medium |
| **1.5. Performance Config** | **2** | **Medium** | **Low** |
| 2. Híbrido | 7 | High | High |
| 3. RQL 4GL | 7 | Medium | Low |
| 4. Export | 7 | Medium | Low |
| 5. TUI/UX | 7 | Low | Low |
| 6. Release | 7 | Low | Low |
| **TOTAL** | **44 días** | | |
| **→ 7 semanas** | | | |

---

## 🔗 REFERENCIAS

- [DuckDB Research Report](research/duckdb_research_2025_11.md)
- [Arrow Integration Guide](https://duckdb.org/docs/stable/guides/python/arrow)
- [Performance Tuning Guide](https://duckdb.org/docs/stable/guides/performance/how_to_tune_workloads)
- [M7_IMPLEMENTATION_PLAN.md](M7_IMPLEMENTATION_PLAN.md) - Next milestone

---

**Noctra(🦆) 2.0 "FABRIC" — Data Fabric Engine**
**"No importes datos. Consúltalos."**

**Versión del Plan:** 2.0
**Última Actualización:** 2025-11-13
**Autor:** Claude (Anthropic) + wirednil
