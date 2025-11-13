# Milestone 6 — "FABRIC" Implementation Plan

> ⚠️ **DEPRECATION NOTICE**
>
> This document (v1) has been superseded by **[M6_IMPLEMENTATION_PLAN_v2.md](M6_IMPLEMENTATION_PLAN_v2.md)** which incorporates critical DuckDB research findings.
>
> **Key Changes in v2:**
> - Arrow integration moved from optional (Phase 5) to **MANDATORY** (Phase 1)
> - New Phase 1.5: Performance Configuration Layer (2 days)
> - Timeline extended from 6 to 7 weeks
> - Dynamic thread configuration (Local vs Remote I/O)
> - AttachmentRegistry for non-persistent ATTACH statements
> - Updated performance targets (CSV 10MB: 500ms→200ms)
> - Mandatory `PER_THREAD_OUTPUT` for Parquet exports
>
> **Please refer to [M6_IMPLEMENTATION_PLAN_v2.md](M6_IMPLEMENTATION_PLAN_v2.md) for the current implementation plan.**
>
> ---

**Noctra(🦆 DuckDB): Data Fabric Engine**
**Fecha de Inicio:** 11 de noviembre de 2025
**Duración:** 6 semanas (11 nov — 23 dic 2025) ~~OBSOLETO: Ver v2 para 7 semanas~~
**Versión Target:** v0.6.0
**Branch:** `claude/duckdb-integration-*`

---

## 🎯 OBJETIVO ESTRATÉGICO

> **Transformar Noctra de "entorno SQL interactivo" a "entorno 4GL de análisis de datos sobre DuckDB"**
> **Los archivos son tablas, el staging desaparece, y el análisis es instantáneo**

---

## 📋 PANORAMA GENERAL DEL MILESTONE

| Antes (Pre-M6) | Después (M6 - FABRIC) |
|----------------|------------------------|
| `IMPORT` → staging → query | `USE 'file.csv'` → query directo |
| CSV backend manual (900+ líneas) | **Eliminado** — DuckDB lo reemplaza |
| JOIN entre CSV imposible | JOIN nativo entre CSV, Parquet, SQLite |
| Máximo 100MB por archivo | Streaming ilimitado (zero-copy) |
| SQLite como motor único | **DuckDB como motor por defecto** |
| `MAP`, `FILTER` redundantes | **Deprecados** — SQL estándar |

---

## 🗓️ TIMELINE — 6 Semanas → 6 Fases

```
Noviembre 2025           Diciembre 2025
11  12  13  14  15  16   17  18  19  20  21  22  23
│   │   │   │   │   │    │   │   │   │   │   │   │
├───┼───┼───┼───┼───┼────┼───┼───┼───┼───┼───┼───┤
│ F1: FUNDACIÓN         │ F2: HÍBRIDO          │
├───────────────────────┼──────────────────────┤
│                       │ F3: RQL 4GL          │
├───────────────────────┼──────────────────────┤
│                       │ F4: EXPORT           │
├───────────────────────┼──────────────────────┤
│                       │ F5: TUI              │
├───────────────────────┼──────────────────────┤
│                       │ F6: RELEASE          │
└───────────────────────┴──────────────────────┘
```

---

## 📦 FASE 1: FUNDACIÓN — Integración DuckDB (Semana 1)

**Fecha:** 11-15 nov 2025
**Objetivo:** Reemplazar el backend CSV manual con DuckDB como motor universal.

### Tareas Técnicas

#### 1.1 Crear Crate `noctra-duckdb`

**Estructura:**
```
crates/noctra-duckdb/
  ├── Cargo.toml
  ├── build.rs (si necesario)
  └── src/
      ├── lib.rs          # Public API, re-exports
      ├── source.rs       # DuckDBSource impl DataSource
      ├── engine.rs       # Query execution, parameter binding
      ├── extensions.rs   # Parquet, JSON support
      └── error.rs        # Error types
```

**Cargo.toml:**
```toml
[package]
name = "noctra-duckdb"
version = "0.6.0"
edition = "2021"

[dependencies]
duckdb = { version = "1.1", features = ["bundled", "parquet", "json"] }
noctra-core = { path = "../noctra-core" }
anyhow = "1.0"
thiserror = "1.0"
log = "0.4"

[dev-dependencies]
tempfile = "3.0"
env_logger = "0.10"
```

#### 1.2 Implementar `DuckDBSource`

**Archivo:** `crates/noctra-duckdb/src/source.rs`

```rust
use duckdb::{Connection, params};
use noctra_core::{DataSource, ResultSet, Parameters, Value, SourceType, TableInfo, ColumnInfo};
use anyhow::Result;

pub struct DuckDBSource {
    conn: Connection,
    name: String,
}

impl DuckDBSource {
    /// Create in-memory DuckDB connection
    pub fn new_in_memory() -> Result<Self> {
        Ok(Self {
            conn: Connection::open_in_memory()?,
            name: "duckdb".to_string(),
        })
    }

    /// Create file-based DuckDB connection
    pub fn new_with_path(path: &str) -> Result<Self> {
        Ok(Self {
            conn: Connection::open(path)?,
            name: path.to_string(),
        })
    }

    /// Register file as virtual table
    pub fn register_file(&mut self, path: &str, alias: &str) -> Result<()> {
        let ext = std::path::Path::new(path)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");

        let sql = match ext {
            "csv" | "tsv" => {
                // Auto-detect delimiters, headers, types
                format!("CREATE VIEW {} AS SELECT * FROM read_csv_auto('{}')", alias, path)
            },
            "json" => {
                format!("CREATE VIEW {} AS SELECT * FROM read_json_auto('{}')", alias, path)
            },
            "parquet" => {
                format!("CREATE VIEW {} AS SELECT * FROM read_parquet('{}')", alias, path)
            },
            _ => anyhow::bail!("Unsupported file type: {}", ext),
        };

        self.conn.execute(&sql, [])?;
        log::info!("Registered {} as virtual table '{}'", path, alias);
        Ok(())
    }
}

impl DataSource for DuckDBSource {
    fn query(&self, sql: &str, params: &Parameters) -> Result<ResultSet> {
        let mut stmt = self.conn.prepare(sql)?;

        // Convert Parameters to DuckDB format
        let duckdb_params = params_to_duckdb(params);

        // Execute and convert rows
        let rows = stmt.query_map(&duckdb_params[..], |row| {
            convert_duckdb_row(row)
        })?;

        Ok(ResultSet::from_rows(rows.collect()?))
    }

    fn schema(&self) -> Result<Vec<TableInfo>> {
        let sql = "SELECT table_name FROM information_schema.tables WHERE table_schema = 'main'";
        let mut stmt = self.conn.prepare(sql)?;

        let tables: Vec<String> = stmt
            .query_map([], |row| row.get(0))?
            .collect::<Result<_, _>>()?;

        tables.into_iter()
            .map(|table| {
                let columns = self.get_table_columns(&table)?;
                Ok(TableInfo {
                    name: table,
                    columns,
                })
            })
            .collect()
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn source_type(&self) -> SourceType {
        SourceType::DuckDB
    }
}
```

#### 1.3 Tests Básicos

**Archivo:** `crates/noctra-duckdb/src/lib.rs` (integration tests)

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;

    #[test]
    fn test_register_csv() {
        let mut csv = NamedTempFile::new().unwrap();
        writeln!(csv, "id,name,age").unwrap();
        writeln!(csv, "1,Alice,30").unwrap();
        writeln!(csv, "2,Bob,25").unwrap();
        csv.flush().unwrap();

        let mut source = DuckDBSource::new_in_memory().unwrap();
        source.register_file(csv.path().to_str().unwrap(), "users").unwrap();

        let result = source.query("SELECT * FROM users", &Parameters::default()).unwrap();
        assert_eq!(result.rows.len(), 2);
    }

    #[test]
    fn test_schema_introspection() {
        let mut source = DuckDBSource::new_in_memory().unwrap();
        source.conn.execute("CREATE TABLE test (id INTEGER, name TEXT)", []).unwrap();

        let schema = source.schema().unwrap();
        assert_eq!(schema.len(), 1);
        assert_eq!(schema[0].name, "test");
    }
}
```

#### 1.4 Eliminar `csv_backend.rs`

**Acción:**
```bash
git rm crates/core/src/csv_backend.rs
git rm -r crates/core/tests/csv_backend_tests.rs
```

**Deprecation Notice:**
```rust
// crates/core/src/lib.rs
#[deprecated(since = "0.6.0", note = "Use noctra-duckdb instead")]
pub mod csv_backend;
```

#### 1.5 Feature Flag

**Archivo:** `Cargo.toml` (workspace root)

```toml
[workspace.dependencies]
noctra-duckdb = { path = "crates/noctra-duckdb", optional = true }

[features]
default = ["duckdb-engine"]
duckdb-engine = ["noctra-duckdb"]
sqlite-fallback = []
```

### Entregables Fase 1

- [ ] Crate `noctra-duckdb` funcional
- [ ] `USE 'file.csv' AS alias` funciona con DuckDB
- [ ] Soporte CSV, JSON, Parquet
- [ ] `csv_backend.rs` eliminado
- [ ] Feature flag `duckdb-engine`
- [ ] Tests pasando (>5 tests básicos)

**Criterio de Éxito:**
```bash
cargo test --package noctra-duckdb
# 5+ tests passing
```

---

## 🔗 FASE 2: MOTOR HÍBRIDO — DuckDB + SQLite (Semana 2)

**Fecha:** 16-22 nov 2025
**Objetivo:** Modo híbrido por defecto: DuckDB para archivos, SQLite para persistencia.

### Tareas Técnicas

#### 2.1 Implementar `QueryEngine::Hybrid`

**Archivo:** `crates/core/src/engine.rs` (NUEVO)

```rust
use noctra_duckdb::DuckDBSource;
use crate::backend::SqliteBackend;

pub enum QueryEngine {
    Sqlite(Box<dyn DatabaseBackend>),
    DuckDB(DuckDBSource),
    Hybrid {
        duckdb: DuckDBSource,
        sqlite: SqliteBackend,
    },
}

impl QueryEngine {
    pub fn new_hybrid() -> Result<Self> {
        Ok(Self::Hybrid {
            duckdb: DuckDBSource::new_in_memory()?,
            sqlite: SqliteBackend::new_in_memory()?,
        })
    }

    pub fn execute(&mut self, nql: &NqlStatement) -> Result<ResultSet> {
        match self {
            Self::DuckDB(conn) => conn.execute_nql(nql),
            Self::Sqlite(backend) => backend.execute(nql),
            Self::Hybrid { duckdb, sqlite } => {
                // Routing logic
                match nql.source_type()? {
                    SourceType::Csv | SourceType::Json | SourceType::Parquet
                        => duckdb.execute_nql(nql),
                    SourceType::Sqlite
                        => sqlite.execute(nql),
                }
            },
        }
    }
}
```

#### 2.2 Routing Inteligente

**Lógica:**
```rust
impl QueryEngine {
    fn route_query(&self, source_path: &str) -> EngineType {
        let ext = Path::new(source_path)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");

        match ext {
            "csv" | "tsv" | "json" | "parquet" => EngineType::DuckDB,
            "db" | "sqlite" | "sqlite3" => EngineType::SQLite,
            _ => EngineType::DuckDB, // Default to DuckDB
        }
    }
}
```

#### 2.3 ATTACH Automático (SQLite en DuckDB)

**Archivo:** `crates/noctra-duckdb/src/attach.rs`

```rust
impl DuckDBSource {
    pub fn attach_sqlite(&mut self, db_path: &str, alias: &str) -> Result<()> {
        self.conn.execute(
            &format!("ATTACH '{}' AS {} (TYPE SQLITE)", db_path, alias),
            [],
        )?;
        log::info!("Attached SQLite database {} as '{}'", db_path, alias);
        Ok(())
    }
}
```

#### 2.4 Configuración TOML

**Archivo:** `~/.config/noctra/config.toml` (ejemplo)

```toml
[engine]
default = "hybrid"  # duckdb, sqlite, hybrid

[duckdb]
temp_dir = "/tmp/noctra-duckdb"
memory_limit = "2GB"
threads = 4
enable_profiling = false

[duckdb.extensions]
auto_install = true
enabled = ["parquet", "json"]

[sqlite]
wal_mode = true
```

**Loader:** `crates/core/src/config.rs`

```rust
use serde::Deserialize;

#[derive(Deserialize)]
pub struct NoctraConfig {
    pub engine: EngineConfig,
    pub duckdb: DuckDBConfig,
    pub sqlite: SqliteConfig,
}

impl NoctraConfig {
    pub fn load() -> Result<Self> {
        let config_path = dirs::config_dir()
            .ok_or_else(|| anyhow!("Config directory not found"))?
            .join("noctra/config.toml");

        if !config_path.exists() {
            return Ok(Self::default());
        }

        let content = std::fs::read_to_string(&config_path)?;
        toml::from_str(&content).map_err(Into::into)
    }
}
```

### Entregables Fase 2

- [ ] `QueryEngine::Hybrid` funcional
- [ ] Routing automático (CSV → DuckDB, SQLite → SQLite)
- [ ] ATTACH de SQLite en DuckDB
- [ ] JOIN cross-source funciona
- [ ] Configuración TOML cargable
- [ ] Tests: routing, ATTACH, cross-source JOIN

**Criterio de Éxito:**
```sql
USE 'ventas.csv' AS v;
USE 'clientes.db' AS c;

SELECT c.nombre, v.total
FROM v JOIN c.clientes ON v.id = c.id;
-- Resultado: 10 filas
```

---

## 🛠️ FASE 3: RQL 4GL — Extensionalidad Nativa (Semana 3)

**Fecha:** 23-29 nov 2025
**Objetivo:** Consolidar las extensiones únicas de Noctra sobre DuckDB.

### Extensiones a Mantener

#### 3.1 `LET var = valor` — Variables de Sesión

**Estado:** ✅ Ya implementado
**Acción:** Validar compatibilidad con DuckDB

**Test:**
```sql
LET pais = 'AR';
SELECT * FROM 'ventas.csv' WHERE pais = #pais;
```

#### 3.2 `#var` en SQL — Interpolación

**Estado:** ✅ Ya implementado
**Acción:** Validar que funciona con DuckDB queries

#### 3.3 `SHOW VARS` — Tabla de Variables

**Estado:** ✅ Ya implementado
**Output:**
```
┌──────────┬────────┐
│ Variable │ Valor  │
├──────────┼────────┤
│ pais     │ AR     │
└──────────┴────────┘
```

#### 3.4 `SHOW SOURCES` — Catálogo Unificado

**Estado:** ✅ Ya implementado
**Acción:** Agregar columna `Engine` (DuckDB, SQLite)

**Output:**
```
┌──────────┬─────────┬────────────────┐
│ Alias    │ Engine  │ Path           │
├──────────┼─────────┼────────────────┤
│ ventas   │ DuckDB  │ ./ventas.csv   │
│ clientes │ SQLite  │ ./clientes.db  │
└──────────┴─────────┴────────────────┘
```

### Extensiones a Deprecar

#### 3.5 Deprecar `MAP`, `FILTER`

**Archivo:** `crates/parser/src/parser.rs`

```rust
#[deprecated(since = "0.6.0", note = "Use SQL SELECT with expressions instead")]
pub fn parse_map(&mut self) -> Result<RqlStatement> {
    eprintln!("WARNING: MAP is deprecated. Use SELECT with expressions instead.");
    // ... parsing lógica
}

#[deprecated(since = "0.6.0", note = "Use SQL WHERE clause instead")]
pub fn parse_filter(&mut self) -> Result<RqlStatement> {
    eprintln!("WARNING: FILTER is deprecated. Use WHERE clause instead.");
    // ... parsing lógica
}
```

**Documentación:** `docs/MIGRATION.md`

```markdown
## Migrating from MAP/FILTER to SQL Standard

### Before (Pre-M6):
```sql
USE 'datos.csv';
MAP nombre = UPPER(nombre);
FILTER edad > 25;
SELECT * FROM datos;
```

### After (M6+):
```sql
SELECT
    UPPER(nombre) AS nombre,
    *
FROM 'datos.csv'
WHERE edad > 25;
```
```

#### 3.6 Deprecar `OUTPUT TO`

**Reemplazo:** `EXPORT TO ... FORMAT ...`

**Parser:**
```rust
#[deprecated(since = "0.6.0", note = "Use EXPORT TO 'file' FORMAT format")]
pub fn parse_output_to(&mut self) -> Result<RqlStatement> {
    eprintln!("WARNING: OUTPUT TO is deprecated. Use EXPORT TO 'file' FORMAT format");
    // ...
}
```

### Entregables Fase 3

- [ ] `LET`, `#var`, `SHOW VARS` validados con DuckDB
- [ ] `SHOW SOURCES` con columna `Engine`
- [ ] `MAP`, `FILTER`, `OUTPUT TO` marcados como deprecated
- [ ] Warnings en consola al usar comandos deprecados
- [ ] `MIGRATION.md` documentado
- [ ] Tests actualizados

---

## 📤 FASE 4: EXPORT & OUTPUT — Unified Output Layer (Semana 4)

**Fecha:** 30 nov - 6 dic 2025
**Objetivo:** `EXPORT` como comando maestro, `OUTPUT TO` eliminado.

### Tareas Técnicas

#### 4.1 Implementar `EXPORT` Unificado

**Sintaxis:**
```sql
EXPORT (query) TO 'file.ext' FORMAT format [OPTIONS (...)];
EXPORT table TO 'file.ext' FORMAT format;
```

**Parser:** `crates/parser/src/parser.rs`

```rust
pub enum RqlStatement {
    // ...
    Export {
        source: ExportSource,      // Query or Table
        path: String,
        format: ExportFormat,
        options: HashMap<String, String>,
    },
}

pub enum ExportSource {
    Query(Box<RqlStatement>),
    Table(String),
}

pub enum ExportFormat {
    Csv,
    Json,
    Parquet,
}
```

**Traductor a DuckDB:**

```rust
impl DuckDBSource {
    pub fn export(&self, stmt: &ExportStatement) -> Result<()> {
        let format_str = match stmt.format {
            ExportFormat::Csv => "CSV",
            ExportFormat::Json => "JSON",
            ExportFormat::Parquet => "PARQUET",
        };

        let sql = match &stmt.source {
            ExportSource::Query(query) => {
                format!("COPY ({}) TO '{}' (FORMAT {})", query, stmt.path, format_str)
            },
            ExportSource::Table(table) => {
                format!("COPY {} TO '{}' (FORMAT {})", table, stmt.path, format_str)
            },
        };

        // Apply options
        let sql_with_options = self.apply_export_options(sql, &stmt.options);

        self.conn.execute(&sql_with_options, [])?;
        Ok(())
    }
}
```

#### 4.2 Soporte Multi-Formato

**CSV:**
```sql
EXPORT ventas TO 'out.csv' FORMAT CSV OPTIONS (delimiter=';', header=true);
```

**JSON:**
```sql
EXPORT (SELECT * FROM clientes WHERE activo = 1) TO 'activos.json' FORMAT JSON;
```

**Parquet:**
```sql
EXPORT datos TO 'backup.parquet' FORMAT PARQUET OPTIONS (compression='snappy');
```

### Entregables Fase 4

- [ ] `EXPORT` comando funcional
- [ ] Soporte CSV, JSON, Parquet
- [ ] OPTIONS configurables
- [ ] `OUTPUT TO` completamente deprecado
- [ ] Tests para cada formato
- [ ] Validación de paths (security)

**Criterio de Éxito:**
```bash
cargo test --package noctra-core -- export
# 10+ tests passing
```

---

## 🎨 FASE 5: TUI & UX — Data Fabric Experience (Semana 5)

**Fecha:** 7-13 dic 2025
**Objetivo:** Interfaz que refleje el nuevo poder de DuckDB.

### Tareas Técnicas

#### 5.1 Status Bar Dinámico

**Archivo:** `crates/tui/src/noctra_tui.rs`

```rust
fn render_status_bar(&self, area: Rect, buf: &mut Buffer) {
    let engine_icon = match &self.query_engine {
        QueryEngine::DuckDB(_) => "🦆",
        QueryEngine::Sqlite(_) => "📦",
        QueryEngine::Hybrid { .. } => "🔀",
    };

    let engine_name = match &self.query_engine {
        QueryEngine::DuckDB(_) => "DuckDB",
        QueryEngine::Sqlite(_) => "SQLite",
        QueryEngine::Hybrid { .. } => "Hybrid",
    };

    let source_info = self.active_source()
        .map(|s| format!("{} ({}, {}, {} rows)",
            s.name(),
            s.source_type(),
            format_size(s.size()?),
            format_rows(s.row_count()?)
        ))
        .unwrap_or_else(|| "No source".to_string());

    let memory_info = format!("{}MB", self.get_memory_usage_mb());
    let time_info = format!("{}ms", self.last_query_time.as_millis());

    let status = format!(
        " {} {} │ Source: {} │ Memory: {} │ {} ",
        engine_icon, engine_name, source_info, memory_info, time_info
    );

    // Render to status bar
    let status_widget = Paragraph::new(status)
        .style(Style::default().bg(Color::Blue).fg(Color::White));
    status_widget.render(area, buf);
}
```

**Output:**
```
Engine: 🦆 DuckDB │ Source: ventas.csv (CSV, 1.2GB, 1.2M rows) │ Memory: 45MB │ 8ms
```

#### 5.2 Panel `SOURCES`

**Nuevo Widget:** `crates/tui/src/widgets/sources_panel.rs`

```rust
pub struct SourcesPanel {
    sources: Vec<SourceInfo>,
}

impl Widget for SourcesPanel {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let rows: Vec<Row> = self.sources.iter()
            .map(|s| Row::new(vec![
                s.alias.clone(),
                get_source_icon(&s.source_type),
                format_size(s.size),
                format_rows(s.rows),
            ]))
            .collect();

        let table = Table::new(rows)
            .header(Row::new(vec!["Alias", "Type", "Size", "Rows"]))
            .widths(&[
                Constraint::Length(15),
                Constraint::Length(10),
                Constraint::Length(10),
                Constraint::Length(10),
            ]);

        table.render(area, buf);
    }
}
```

#### 5.3 CLI: `noctra 'file.csv'`

**Archivo:** `crates/cli/src/main.rs`

```rust
#[derive(Parser)]
struct Cli {
    /// File to open directly
    file: Option<String>,

    #[arg(long)]
    engine: Option<EngineType>,

    // ... otros args
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    if let Some(file) = cli.file {
        // Open with automatic USE
        let engine = QueryEngine::new_hybrid()?;
        engine.execute(&format!("USE '{}' AS data", file))?;

        // Launch TUI
        run_tui_with_engine(engine)?;
    } else {
        // Normal mode
        run_tui()?;
    }

    Ok(())
}
```

**Usage:**
```bash
noctra 'ventas.parquet'
# Equivalente a:
# noctra
# > USE 'ventas.parquet' AS v
```

### Entregables Fase 5

- [ ] Status bar dinámico funcional
- [ ] Panel SOURCES implementado
- [ ] `noctra 'file.csv'` funciona
- [ ] Autocomplete de tablas (básico)
- [ ] Icons por tipo de fuente (🦆, 📦)
- [ ] Tests UI (screenshots)

---

## 🚀 FASE 6: RELEASE & DOCUMENTACIÓN — v0.6.0 "FABRIC" (Semana 6)

**Fecha:** 14-23 dic 2025
**Objetivo:** Lanzamiento estable, documentación completa, migración clara.

### Tareas Técnicas

#### 6.1 Tag `v0.6.0`

```bash
git tag -a v0.6.0 -m "Release v0.6.0 - FABRIC (DuckDB Integration)"
git push origin v0.6.0
```

#### 6.2 Documentación

**Crear `docs/RQL_EXTENSIONS.md`:**

```markdown
# RQL Extensions Manual

## Variables de Sesión

### LET
Define una variable de sesión:
```sql
LET variable = valor;
```

### SHOW VARS
Lista todas las variables:
```sql
SHOW VARS;
```

### #var Interpolation
Usa variables en queries:
```sql
LET pais = 'AR';
SELECT * FROM ventas WHERE pais = #pais;
```

## Multi-Source Management

### USE
Registra una fuente de datos:
```sql
USE 'file.csv' AS alias;
USE 'database.db' AS alias;
```

### SHOW SOURCES
Lista todas las fuentes registradas:
```sql
SHOW SOURCES;
```

### DESCRIBE
Muestra el esquema de una tabla:
```sql
DESCRIBE source.table;
```

## Export

### EXPORT TO
Exporta datos a archivo:
```sql
EXPORT query TO 'file.ext' FORMAT format;
```

Formatos: CSV, JSON, PARQUET

## Deprecated Commands (v0.6.0)

- **MAP** → Use SQL SELECT with expressions
- **FILTER** → Use SQL WHERE clause
- **OUTPUT TO** → Use EXPORT TO
```

**Crear `docs/MIGRATION.md`:**

```markdown
# Migration Guide: v0.5.0 → v0.6.0

## Breaking Changes

### csv_backend.rs Removed

**Before:**
```rust
use noctra_core::csv_backend::CsvDataSource;
```

**After:**
```rust
use noctra_duckdb::DuckDBSource;
```

### MAP/FILTER Deprecated

**Before:**
```sql
USE 'datos.csv';
MAP nombre = UPPER(nombre);
FILTER edad > 25;
SELECT * FROM datos;
```

**After:**
```sql
SELECT
    UPPER(nombre) AS nombre,
    *
FROM 'datos.csv'
WHERE edad > 25;
```

### OUTPUT TO Deprecated

**Before:**
```sql
OUTPUT TO 'file.csv' FORMAT CSV;
SELECT * FROM tabla;
```

**After:**
```sql
EXPORT tabla TO 'file.csv' FORMAT CSV;
```

## New Features

### Direct File Queries

```sql
-- No need for USE anymore
SELECT * FROM 'ventas.csv' WHERE region = 'LATAM';
```

### Cross-Source JOINs

```sql
USE 'ventas.csv' AS v;
USE 'clientes.db' AS c;

SELECT c.nombre, v.total
FROM v JOIN c.clientes ON v.id = c.id;
```

### Parquet Support

```sql
SELECT * FROM 'datos.parquet';
EXPORT ventas TO 'backup.parquet' FORMAT PARQUET;
```
```

#### 6.3 Benchmarks

**Archivo:** `benches/duckdb_vs_sqlite.rs`

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn benchmark_csv_load(c: &mut Criterion) {
    c.bench_function("load_1gb_csv_duckdb", |b| {
        b.iter(|| {
            // Load 1GB CSV with DuckDB
        });
    });

    c.bench_function("load_1gb_csv_sqlite", |b| {
        b.iter(|| {
            // Load 1GB CSV with SQLite (IMPORT)
        });
    });
}

fn benchmark_cross_source_join(c: &mut Criterion) {
    c.bench_function("join_csv_sqlite_duckdb", |b| {
        b.iter(|| {
            // JOIN between CSV and SQLite using DuckDB
        });
    });
}

criterion_group!(benches, benchmark_csv_load, benchmark_cross_source_join);
criterion_main!(benches);
```

**Run:**
```bash
cargo bench --bench duckdb_vs_sqlite
```

**Expected Results:**
- CSV 1GB load: DuckDB <2s vs SQLite ~30s (15x faster)
- JOIN 100K rows: DuckDB <1s vs SQLite ~5s (5x faster)

#### 6.4 CHANGELOG.md

```markdown
# Changelog

## [0.6.0] - 2025-12-23 - "FABRIC"

### Added
- 🦆 **DuckDB Integration** as default query engine
- Parquet file support (read/write)
- Cross-source JOINs (CSV + SQLite + Parquet)
- `EXPORT TO 'file' FORMAT format` unified command
- Hybrid mode: DuckDB for files, SQLite for persistence
- Configuration file `~/.config/noctra/config.toml`
- Direct file queries: `SELECT * FROM 'file.csv'`

### Changed
- **BREAKING:** Removed `csv_backend.rs` (replaced by DuckDB)
- **BREAKING:** Default engine is now DuckDB (not SQLite)
- Status bar now shows engine type and source info

### Deprecated
- `MAP expression` → Use SQL SELECT with expressions
- `FILTER condition` → Use SQL WHERE clause
- `OUTPUT TO 'file'` → Use EXPORT TO 'file' FORMAT format

### Fixed
- Performance: 10x faster CSV loading
- Memory: Streaming for large files (no 100MB limit)

### Migration Guide
See `docs/MIGRATION.md` for detailed migration instructions.
```

### Entregables Fase 6

- [ ] Tag `v0.6.0` pushed
- [ ] `RQL_EXTENSIONS.md` completo
- [ ] `MIGRATION.md` completo
- [ ] Benchmarks ejecutados y documentados
- [ ] CHANGELOG.md actualizado
- [ ] Feature flags documentados
- [ ] Tests de regresión completos (>85% coverage)

---

## ✅ CRITERIOS DE ÉXITO GLOBALES

### Funcionales

- ✅ `USE 'file.csv' AS alias` carga archivo sin staging
- ✅ `SELECT * FROM 'file.csv'` funciona directamente
- ✅ JOIN entre CSV y SQLite sin IMPORT
- ✅ EXPORT a CSV, JSON, Parquet
- ✅ Modo híbrido por defecto (DuckDB + SQLite)
- ✅ `LET`, `#var`, `SHOW VARS` funcionan con DuckDB
- ✅ `csv_backend.rs` eliminado
- ✅ `MAP`, `FILTER`, `OUTPUT TO` deprecados

### Performance

- ✅ CSV 1GB carga en <2s (vs ~30s con csv_backend)
- ✅ JOIN 100K rows: <1s
- ✅ GROUP BY con agregaciones: <500ms
- ✅ Memoria: <200MB para 1GB CSV (streaming)
- ✅ Parquet 10x más rápido que CSV

### Calidad

- ✅ Test coverage: >85%
- ✅ Zero clippy warnings
- ✅ Documentación completa (RQL_EXTENSIONS.md, MIGRATION.md)
- ✅ Benchmarks publicados
- ✅ Feature flags documentados

---

## 🚧 DEPENDENCIAS Y RIESGOS

### Dependencias Técnicas

| Dependencia | Versión | Criticidad | Notas |
|-------------|---------|------------|-------|
| DuckDB | 1.1+ | **CRITICAL** | Motor principal |
| Rust | 1.70+ | High | MSRV |
| Ratatui | 0.29+ | Medium | TUI framework |
| Tokio | 1.48+ | Medium | Async runtime |

### Riesgos Identificados

| Riesgo | Probabilidad | Impacto | Mitigación |
|--------|--------------|---------|------------|
| DuckDB API breaking changes | Low | High | Pin version to 1.1.x |
| Performance regression | Medium | High | Benchmarks continuos |
| Migration pain (users) | Medium | Medium | MIGRATION.md detallado |
| Feature flag complexity | Low | Medium | Documentación clara |

---

## 📞 RECURSOS Y REFERENCIAS

### Documentación DuckDB

- [DuckDB SQL Reference](https://duckdb.org/docs/sql/introduction)
- [DuckDB CSV Reader](https://duckdb.org/docs/data/csv/overview)
- [DuckDB Parquet](https://duckdb.org/docs/data/parquet/overview)
- [DuckDB ATTACH DATABASE](https://duckdb.org/docs/sql/statements/attach)

### Noctra Docs

- [PROJECT_STATUS.md](PROJECT_STATUS.md)
- [ROADMAP.md](ROADMAP.md)
- [DESIGN.md](DESIGN.md)
- [RQL-EXTENSIONS.md](RQL-EXTENSIONS.md)

---

## 🎯 PRÓXIMOS PASOS (Post-M6)

Ver **Milestone 7 - "SCRIPT"** para extensiones de scripting:

- `IF/THEN`, `FOR` loops en RQL
- `MACRO`, `CALL` para reutilización de queries
- `RUNSUM()`, `RUNAVG()` funciones de ventana
- `GRAPH BAR`, `GRAPH LINE` visualización ASCII
- `SAVE SESSION`, `LOAD SESSION` persistencia de estado

---

**Noctra(🦆) — Data Fabric Engine**
**"Los archivos son tablas. El staging desaparece. El análisis es instantáneo."**

---

**Última actualización:** 2025-11-11
**Autor:** Claude (Anthropic) + wirednil
**Versión del Plan:** 1.0
