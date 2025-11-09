# Changelog

All notable changes to Noctra will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- CSV file support with automatic delimiter detection
- NQL (Noctra Query Language) basic commands
- Multi-source data management with SourceRegistry
- Complete CSV backend with type inference
- Session variable management (LET, UNSET, SHOW VARS)

### Changed
- NQL commands now return SQL-style tables instead of dialogs
- Status bar shows `source:table` format for better context
- Parser now handles quoted values in OPTIONS correctly

### Fixed
- "Failed to prepare" error when querying CSV files
- Tokio runtime panic in TUI and REPL
- OPTIONS parser now handles commas in quoted values

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
