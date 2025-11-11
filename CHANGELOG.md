# Changelog

All notable changes to Noctra will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.2.0] - 2025-11-11 (Milestone 4 Fase 1) - **IN PROGRESS**

### Added - IMPORT/EXPORT Commands

#### IMPORT Command Implementation
- **Full CSV Import to SQLite**: Load CSV files directly into SQLite tables
  - `IMPORT 'file.csv' AS tablename OPTIONS (delimiter=',', header=true)`
  - Automatic table creation with column detection
  - Support for custom delimiters (`,`, `;`, `\t`, `|`)
  - Header detection (with/without headers)
  - Quote-aware parsing for complex CSV values
  - SQL injection prevention with parameterized inserts
  - Implemented in both TUI and REPL for consistency

#### EXPORT Command Implementation
- **Multi-Format Export**: Export query results or tables to files
  - **CSV Export**: `EXPORT tablename TO 'file.csv' FORMAT CSV OPTIONS (delimiter=',', header=true)`
    - Custom delimiters support
    - Optional header row
    - Proper CSV escaping (quotes, newlines, delimiters)
  - **JSON Export**: `EXPORT tablename TO 'file.json' FORMAT JSON`
    - Pretty-printed JSON arrays
    - Automatic type conversion (integers, floats, booleans, nulls)
  - **Query Support**: Export results from SELECT queries
    - `EXPORT (SELECT * FROM users WHERE active = true) TO 'active_users.csv' FORMAT CSV`
  - Implemented in both TUI and REPL

#### MAP & FILTER Command Stubs
- **MAP Command**: Placeholder for declarative transformations
  - Parser support implemented
  - Shows informative message directing users to use SELECT for transformations
  - Planned for M4 Fase 2 (complete pipeline implementation)
- **FILTER Command**: Placeholder for declarative filtering
  - Parser support implemented
  - Shows informative message directing users to use WHERE clauses
  - Planned for M4 Fase 2 (complete pipeline implementation)

### Technical Details

#### Code Changes
- **TUI Updates** (`noctra_tui.rs` - ~300 lines added)
  - `handle_import()`: Full CSV import logic with error handling
  - `handle_export()`: Multi-format export with CSV and JSON support
  - `handle_map()`: Informative placeholder
  - `handle_filter()`: Informative placeholder
  - Added serde_json dependency for JSON export

- **REPL Updates** (`repl.rs` - ~300 lines added)
  - Mirrored TUI implementation for consistency
  - Terminal-friendly output (println! instead of dialogs)
  - Same feature set as TUI

- **Parser** (`parser.rs`, `rql_ast.rs`)
  - Commands already defined in M3.5
  - Full AST support for IMPORT, EXPORT, MAP, FILTER
  - OPTIONS parsing with quote support

#### Statistics
- **Lines Added**: ~600 lines
- **Files Modified**: 3 (noctra_tui.rs, repl.rs, Cargo.toml)
- **Build Time**: ~67s (release mode)
- **Build Status**: ✅ Success
- **Warnings**: 2 minor (unused imports in core)

### Examples

**Import CSV to SQLite:**
```sql
IMPORT 'customers.csv' AS customers OPTIONS (delimiter=',', header=true);
SELECT * FROM customers;
```

**Export table to CSV:**
```sql
EXPORT customers TO 'customers_backup.csv' FORMAT CSV OPTIONS (delimiter=',', header=true);
```

**Export query results to JSON:**
```sql
EXPORT (SELECT name, email FROM customers WHERE active = true) TO 'active_customers.json' FORMAT JSON;
```

**Custom delimiter:**
```sql
IMPORT 'data.tsv' AS data OPTIONS (delimiter='\t', header=true);
EXPORT data TO 'data.csv' FORMAT CSV OPTIONS (delimiter=',', header=true);
```

### Known Limitations

- XLSX export not implemented (planned for M5)
- JSON import not implemented (planned for M5)
- MAP/FILTER require complete pipeline implementation (M4 Fase 2)
- No streaming support for very large files (>1GB)
- CSV parsing uses simplified algorithm (no RFC 4180 full compliance yet)

### Next Steps (M4 Fase 2)

- Enhanced CSV backend with WHERE, ORDER BY, LIMIT support
- Aggregations (COUNT, SUM, AVG, MIN, MAX) on CSV files
- Basic JOIN support between CSV sources
- Complete MAP/FILTER pipeline implementation
- Input validation and sandboxing
- Resource limits (max rows, query timeout)
- Query result caching with TTL

---

## [2.0.0] - TBD (Milestone 6 - "FABRIC") - **PLANNED**

> **"No importes datos. Consúltalos."**

### Vision

Transform Noctra into a **Data Fabric Engine** by integrating DuckDB as the primary ad hoc analytics engine. Query any file (CSV, JSON, Parquet) as native SQL tables without staging or mandatory databases.

### Planned Features

#### DuckDB Integration
- **New Crate**: `noctra-duckdb` with complete DuckDB integration
- **QueryEngine Modes**: SQLite, DuckDB, Hybrid (default)
  - Hybrid mode: Automatic routing between DuckDB (files) and SQLite (databases)
- **File-Native Queries**: Direct queries on CSV/JSON/Parquet without registration
  - `SELECT * FROM 'data.csv'` - query files directly
  - `USE 'data.csv' AS t` - instant table registration
  - `USE 'sales_*.csv' AS sales` - multi-file glob support
  - Compressed files: `USE 'data.csv.gz' AS data`

#### NQL 2.0 Extensions
- **EXPORT Command**: Multi-format export (CSV, JSON, Parquet, Excel)
  - `EXPORT ... TO 'file.parquet' FORMAT PARQUET`
  - Custom delimiters and headers
  - Column selection support
- **MAP Transformations**: Declarative column transformations
  - `MAP price = price * 1.1, category = UPPER(category)`
- **FILTER Operations**: Simplified filtering without WHERE
  - `FILTER date >= '2024-01-01' AND active = true`
- **Cross-Source JOINs**: Seamless joins between CSV and SQLite
  - `SELECT c.* FROM 'clients.csv' c JOIN db.orders o ON c.id = o.client_id`

#### TUI Enhancements
- **Dynamic Status Bar**: Shows engine type (DuckDB/SQLite/Hybrid) and active source
  - `Engine: DuckDB | Source: 'ventas.csv' | 12ms`
- **Source Type Indicators**: Visual icons for different source types
  - 🦆 CSV/JSON/Parquet (DuckDB)
  - 📦 SQLite databases
- **Export Shortcut**: Ctrl+E for quick data export
- **Engine Selection Dialog**: Switch between engines on-the-fly

#### CLI Enhancements
- **Ad Hoc Mode**: Launch without database
  - `noctra --engine duckdb --use 'data.csv'`
- **Hybrid Mode** (default): SQLite + DuckDB
  - `noctra --engine hybrid --db main.db --use 'extra.csv'`
- **Traditional Mode**: SQLite only (backward compatible)
  - `noctra --engine sqlite --db database.db`

#### Configuration System
- **Config File**: `~/.config/noctra/config.toml`
  - Default engine selection
  - DuckDB memory limits and thread count
  - CSV auto-detection settings
  - Export defaults

### Performance Targets

- CSV 10MB loads in <500ms
- 100K row aggregation in <1s
- Parquet read 10x faster than CSV
- Memory usage <100MB for typical workloads

### Breaking Changes

None. Full backward compatibility maintained.

### Migration Notes

- Existing SQLite workflows unchanged
- New `--engine` flag optional (defaults to hybrid mode)
- Configuration file optional (sensible defaults)

### Known Limitations

- DuckDB in-memory only (no persistent DuckDB databases in v2.0)
- No support for DuckDB extensions beyond bundled ones
- MAP/FILTER limited to single table operations
- No streaming mode for files >10GB (planned for v2.1)

---

## [0.2.0] - TBD (Milestone 4) - **PLANNED**

### Planned Features

#### Advanced NQL Commands
- **IMPORT Command**: Import data from various formats into SQLite
  - `IMPORT FROM 'data.csv' INTO table OPTIONS (...)`
  - Batch import with progress feedback
  - Support for CSV, JSON, TSV
- **Enhanced MAP/FILTER**: Chainable transformations
- **Advanced CSV Queries**: WHERE, ORDER BY, LIMIT on CSV files
  - Aggregations: COUNT, SUM, AVG, MIN, MAX on CSV

#### Security & Performance
- **Input Validation**: SQL injection prevention
- **File Path Sandboxing**: Prevent directory traversal
- **Resource Limits**: Max rows, query timeout, memory limits
- **Query Result Caching**: TTL-based cache for repeated queries
- **Lazy Loading**: Handle large datasets efficiently
- **Prepared Statement Pooling**: Connection pooling for backends

#### TUI Improvements
- **Virtual Scrolling**: Optimize rendering for large result sets
- **Memory Profiling**: Show memory usage in status bar
- **Query History**: Persistent query history with search

### Performance Targets

- Query execution: <100ms for simple queries
- Parser: <1ms for typical queries
- Table rendering: <50ms for 100 rows
- Memory usage: <50MB baseline
- CSV parsing: <500ms for 1MB files

---

## [0.1.0] - 2025-11-09 (Milestone 3.5)

### Added - CSV/NQL Support Hotfix

#### Core Features
- **CSV Backend** (`csv_backend.rs` - 420 lines)
  - Automatic delimiter detection for `,`, `;`, `\t`, `|`
  - Smart type inference (INTEGER, REAL, BOOLEAN, TEXT)
  - Header detection and column naming
  - Quote-aware CSV parsing
  - Schema introspection with `schema()` method

#### Multi-Source Architecture
- **DataSource Trait** (`datasource.rs` - 250 lines)
  - `SourceRegistry` for managing multiple data sources
  - Active source tracking and switching
  - `SourceType` enum (SQLite, CSV, JSON, Memory)
  - Query routing based on active source

#### NQL Commands
- `USE <path> AS <alias> OPTIONS (...)` - Load CSV/database files
  - Example: `USE './data.csv' AS csv OPTIONS (delimiter=',', header=true);`
- `SHOW SOURCES` - List all registered data sources
- `SHOW TABLES [FROM source]` - List tables from specific or all sources
- `DESCRIBE source.table` - Show table schema with columns and types
- `SHOW VARS` - Display session variables
- `LET variable = value` - Set session variables
- `UNSET variable...` - Remove session variables

#### Parser Enhancements
- Enhanced OPTIONS parsing with quote support (`parser.rs` - 80 lines)
  - `split_options()` method respects quote boundaries
  - Handles: `OPTIONS (delimiter=',', header=true)`
  - Supports both single (`'`) and double (`"`) quotes

#### TUI Improvements
- RqlProcessor integration (`noctra_tui.rs` - 300 lines)
  - Thread-spawning parser to avoid Tokio runtime conflicts
  - All NQL commands return SQL-style ResultSet tables
  - Enhanced status bar showing `source:table` format
  - `extract_table_name()` helper for smart context display
- Added `noctra-parser` dependency to `tui/Cargo.toml`

#### REPL Improvements
- Thread-spawning parser to match TUI behavior
- Complete NQL command support
- Debug logging for troubleshooting

### Changed

- **Query Execution Flow**
  - `execute_rql()` now routes queries to active source first
  - Falls back to SQLite when no CSV source is active
  - TUI and REPL now use identical execution path

- **NQL Command Output**
  - Commands return SQL tables instead of dialog boxes
  - `SHOW SOURCES`: 3-column table (Alias, Tipo, Path)
  - `SHOW TABLES`: 1-column table (table)
  - `DESCRIBE`: 2-column table (Campos, Tipo)
  - `SHOW VARS`: 2-column table (Variable, Valor)

- **Status Bar Format**
  - Before: `── Fuente: csv ──`
  - After: `── Fuente: csv:clientes ──` (shows active table)

### Fixed

1. **"Failed to prepare" Error** (Commit `0438e65`)
   - **Problem**: SQL queries always routed to SQLite backend
   - **Solution**: Added query routing to active source in `execute_rql()`

2. **Tokio Runtime Panic** (Commits `ae57113`, `9e64243`)
   - **Problem**: "Cannot start a runtime from within a runtime"
   - **Solution**: Spawn dedicated thread with isolated runtime for parsing
   - **Applied to**: Both TUI and REPL

3. **OPTIONS Parser with Commas** (Commit `9e64243`)
   - **Problem**: `delimiter=','` broke parser (split on comma inside quotes)
   - **Solution**: `split_options()` respects quote boundaries

4. **TUI/REPL Parity** (Commit `5b9940e`)
   - **Problem**: TUI used `execute_sql()`, REPL used `execute_rql()`
   - **Solution**: Both now use RqlProcessor with consistent behavior

### Technical Details

#### Commits
- `0438e65` - fix: Route SQL queries to active data source
- `5b9940e` - fix: Integrate RqlProcessor into TUI
- `ae57113` - fix: Resolve Tokio runtime panic (TUI)
- `9e64243` - fix: Fix OPTIONS parsing and REPL runtime
- `b65ca95` - feat: Add complete NQL command support to TUI
- `dbddebc` - feat: Convert NQL commands to SQL-style tables

#### Statistics
- **Files Modified**: 8
- **Lines Added**: ~1,100
- **Build Time**: 8-18s
- **Test Status**: All passing (29 tests)
- **Warnings**: 0

#### Known Limitations
- CSV backend only supports `SELECT * FROM table`
- No support yet for WHERE, JOIN, GROUP BY, ORDER BY on CSV
- No INSERT/UPDATE/DELETE on CSV files
- Advanced SQL features require SQLite backend

### Examples

**Basic CSV Usage:**
```sql
-- Load CSV file
USE './examples/clientes.csv' AS csv OPTIONS (delimiter=',', header=true);

-- Query the data
SELECT * FROM clientes;

-- Inspect metadata
SHOW SOURCES;
SHOW TABLES FROM csv;
DESCRIBE csv.clientes;
```

**Multi-Source Management:**
```sql
-- Register multiple sources
USE './data1.csv' AS csv1 OPTIONS (delimiter=',', header=true);
USE './data2.csv' AS csv2 OPTIONS (delimiter=';', header=true);

-- List all sources
SHOW SOURCES;
```

**Session Variables:**
```sql
LET myvar = 'test value';
SHOW VARS;
UNSET myvar;
```

### Migration Notes

No breaking changes. All existing functionality preserved.

## [0.0.3] - 2025-11-08 (Milestone 3)

### Added
- Full SQL backend integration with noctra-core Executor
- Real SQL execution in TUI (previously simulated)
- In-memory and file-based database support
- Session state management

## [0.0.2] - 2025-11-07 (Milestone 2)

### Added
- Complete TUI with Ratatui
- FormLib declarative forms
- Interactive form executor
- Noctra Window Manager (NWM)

## [0.0.1] - 2025-11-06 (Milestone 1)

### Added
- Core SQL executor
- RQL parser
- CLI and REPL
- Basic SQLite backend

---

**For full details, see [PROJECT_STATUS.md](docs/PROJECT_STATUS.md)**
