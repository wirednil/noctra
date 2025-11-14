# Noctra DuckDB Backend

DuckDB-powered multi-source query engine for Noctra, enabling native file queries and cross-database analytics.

## Features

### Core Capabilities
- **File-Native Queries**: Query CSV, JSON, and Parquet files directly without importing
- **Auto-Detection**: Automatic format detection and type inference
- **Full SQL Support**: Leverage DuckDB's complete SQL implementation
- **API Compatibility**: Drop-in replacement for existing Noctra data sources
- **Performance**: Optimized for analytical workloads with prepared statement caching

### New in v0.6.0-alpha2 (M6 Phase 2)

#### 🎯 Hybrid Query Engine
Unified interface for querying files and databases:
```rust
use noctra_duckdb::{DuckDBSource, QueryEngine};
use noctra_core::datasource::DataSource;
use noctra_core::types::Parameters;

let source = DuckDBSource::new_in_memory()?;
let mut engine = QueryEngine::new(source);

// Register files and attach databases
engine.register_file("sales.csv", "sales")?;
engine.attach_sqlite("warehouse.db", "wh")?;

// Query across sources
let result = engine.query(
    "SELECT * FROM sales s JOIN wh.inventory i ON s.product_id = i.product_id",
    &Parameters::new()
)?;
```

#### 🔍 RQL Parser (Extended SQL)
Data source management commands:
```rust
use noctra_duckdb::{RqlParser, Statement};

// USE - Register file as table (auto-detects format)
let stmt = RqlParser::parse("USE 'data.csv' AS sales")?;

// ATTACH - Connect external database
let stmt = RqlParser::parse("ATTACH 'warehouse.db' AS wh (TYPE sqlite)")?;

// DETACH - Remove source
let stmt = RqlParser::parse("DETACH sales")?;

// SHOW SOURCES - List registered sources
let stmt = RqlParser::parse("SHOW SOURCES")?;
```

#### ⚙️ Configuration API
Production-ready performance tuning:
```rust
use noctra_duckdb::{DuckDBConfig, DuckDBSource};

// Local I/O optimized (uses CPU core count)
let config = DuckDBConfig::local();

// Remote I/O optimized (3x cores for S3/HTTP latency)
let config = DuckDBConfig::remote();

// Custom configuration
let config = DuckDBConfig {
    memory_limit: Some("16GB".to_string()),
    threads: Some(8),
    ..Default::default()
};

let source = DuckDBSource::new_in_memory_with_config(config)?;
```

#### 💾 AttachmentRegistry
Persistent database attachments:
```rust
let mut source = DuckDBSource::new_in_memory()?;

// Attach database (persists in registry)
source.attach_sqlite("warehouse.db", "wh")?;

// Restore after restart
source.restore_attachments()?;
```

## Quick Start

### Basic File Query

```rust
use noctra_duckdb::DuckDBSource;
use noctra_core::datasource::DataSource;
use noctra_core::types::Parameters;

// Create in-memory database
let mut source = DuckDBSource::new_in_memory()?;

// Register files as virtual tables
source.register_file("sales.csv", "sales")?;
source.register_file("customers.json", "customers")?;

// Query across multiple sources
let result = source.query(
    "SELECT c.name, SUM(s.amount) as total
     FROM customers c
     JOIN sales s ON c.id = s.customer_id
     GROUP BY c.name
     ORDER BY total DESC",
    &noctra_core::types::Parameters::new()
)?;
```

### Cross-Database Analytics

```rust
use noctra_duckdb::{DuckDBSource, QueryEngine};

let source = DuckDBSource::new_in_memory()?;
let mut engine = QueryEngine::new(source);

// Register Parquet file
engine.register_file("events.parquet", "events")?;

// Attach SQLite database
engine.attach_sqlite("customers.db", "crm")?;

// JOIN across sources
let result = engine.query(
    "SELECT c.name, COUNT(e.event_id) as event_count
     FROM crm.customers c
     JOIN events e ON c.customer_id = e.customer_id
     GROUP BY c.name",
    &Parameters::new()
)?;
```

## Supported File Formats

- **CSV**: Comma-separated values with auto-detection of delimiters and headers
- **JSON**: Newline-delimited JSON (`.jsonl`, `.ndjson`) or JSON arrays (`.json`)
- **Parquet**: Columnar format with full schema preservation

## Examples

Run the comprehensive demo:

```bash
# With DuckDB precompiled library
DUCKDB_LIB_DIR=/opt/duckdb \
LD_LIBRARY_PATH=/opt/duckdb \
cargo run --example hybrid_demo -p noctra-duckdb
```

The demo showcases:
- Configuration API with different presets
- Multi-format file registration (CSV, JSON)
- RQL parser for all commands
- Cross-source JOINs
- Complex analytics with CTEs

## Architecture

The `DuckDBSource` implements Noctra's `DataSource` trait, providing:

- `query()`: Execute SQL queries with parameter binding (uses prepared statement cache)
- `schema()`: Introspect table schemas and metadata
- `source_type()`: Identify the data source type
- `name()`: Get the source identifier

### Layered Design

```
QueryEngine (Hybrid Router)
    ↓
DuckDBSource (Config + AttachmentRegistry)
    ↓
DuckDB Engine
    ↓
├─ CSV Files
├─ JSON Files
├─ Parquet Files
└─ SQLite/PostgreSQL (via ATTACH)
```

## Build Configuration

### Using Precompiled DuckDB

Create `.envrc` (recommended with [direnv](https://direnv.net/)):

```bash
export DUCKDB_LIB_DIR=/opt/duckdb
export DUCKDB_INCLUDE_DIR=/opt/duckdb
export LD_LIBRARY_PATH=/opt/duckdb:$LD_LIBRARY_PATH
```

### Build from Source

If DuckDB is not precompiled:

```bash
cargo build -p noctra-duckdb  # Compiles DuckDB from source
```

## Testing

```bash
# Run all tests (unit + integration + doc)
DUCKDB_LIB_DIR=/opt/duckdb \
LD_LIBRARY_PATH=/opt/duckdb \
cargo test -p noctra-duckdb
```

**Test Coverage:**
- 42 unit tests (source, config, attachment, query_engine, parser)
- 11 integration tests (cross-source JOINs, multi-format, RQL commands)
- 8 documentation tests

The test suite includes:
- File registration and format detection
- Query execution with various SQL constructs
- Schema introspection
- Cross-source JOINs
- RQL command parsing
- Configuration management
- Error handling for unsupported formats

## Performance Characteristics

- **CSV Loading**: <500ms for typical files (<10MB)
- **Query Execution**: Sub-millisecond for simple queries
- **Memory Usage**: ~2-3x file size for in-memory operations
- **Concurrent Access**: Thread-safe for read operations
- **Prepared Statements**: 10-30% faster with automatic caching

### Performance Tips

1. Use configuration presets: `DuckDBConfig::local()` or `DuckDBConfig::remote()`
2. Prefer Parquet over CSV for large datasets
3. Create indexes in SQLite for frequent JOINs
4. Configure memory limits based on data size

## Migration from CSV Backend

This crate replaces the legacy `csv_backend.rs` with a more powerful DuckDB-based implementation:

```rust
// Old way (deprecated)
let csv_source = CsvDataSource::new("data.csv", "mydata", CsvOptions::default())?;

// New way (recommended)
let mut duckdb_source = DuckDBSource::new_in_memory()?;
duckdb_source.register_file("data.csv", "mydata")?;
```

## Dependencies

- `duckdb`: Core database engine with bundled extensions
- `noctra-core`: Noctra's core types and traits
- `anyhow`, `thiserror`: Error handling
- `log`: Structured logging
- `num_cpus`: CPU detection for thread configuration
- `serde`: Configuration serialization
- `regex`: RQL command parsing

## Roadmap

### Completed (v0.6.0-alpha2)
- ✅ Hybrid Query Engine
- ✅ RQL Parser (USE, ATTACH, DETACH, SHOW SOURCES)
- ✅ Configuration API
- ✅ AttachmentRegistry
- ✅ prepare_cached() migration

### Planned (v0.6.0-beta1)
- 🔲 SHOW SOURCES implementation
- 🔲 EXPORT/COPY TO functionality
- 🔲 Transaction API
- 🔲 Remote file support (S3, HTTP)

## Documentation

- [M6 Phase 2 Status Report](../../docs/M6_PHASE2_STATUS.md) - Complete implementation details
- [M6 Phase 2 Plan](../../docs/M6_PHASE2_PLAN.md) - Original implementation plan
- [DuckDB Blueprint Analysis](../../docs/M6_BLUEPRINT_ANALYSIS.md) - Architecture comparison

## License

MIT OR Apache-2.0
