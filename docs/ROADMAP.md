# Noctra Development Roadmap

> **Version:** 1.2
> **Last Updated:** 2025-11-09
> **Status:** Active Development - M3.5 Completed, M4 Planning

## Overview

This document outlines the development roadmap for Noctra, from initial setup through production-ready release. The roadmap is organized into milestones with clear deliverables, timelines, and success criteria.

**Current Status:** Milestone 3.5 Complete (v0.1.0 Released) - CSV/NQL Hotfix

---

## Table of Contents

1. [Release Timeline](#release-timeline)
2. [Milestone 0: Foundation](#milestone-0-foundation)
3. [Milestone 1: Core MVP](#milestone-1-core-mvp)
4. [Milestone 2: Forms & TUI](#milestone-2-forms--tui)
5. [Milestone 3: Backend Integration](#milestone-3-backend-integration)
6. [Milestone 3.5: CSV/NQL Hotfix](#milestone-35-csvnql-hotfix)
7. [Milestone 4: Advanced Features](#milestone-4-advanced-features--nql)
8. [Milestone 5: Extended Capabilities](#milestone-5-extended-capabilities)
9. [Milestone 6: Noctra 2.0 "FABRIC"](#milestone-6-noctra-20-fabric)
10. [Future Roadmap](#future-roadmap)
11. [Success Metrics](#success-metrics)

---

## Release Timeline

```
Milestone 0 ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ [████████████] 100%
           └─ Foundation & Setup                     ✅ COMPLETADO

Milestone 1 ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ [████████████] 100%
           └─ Core MVP (RQL Parser + Executor)       ✅ COMPLETADO

Milestone 2 ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ [████████████] 100%
           └─ Forms & TUI (FDL2 + NWM)               ✅ COMPLETADO

Milestone 3 ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ [████████████] 100%
           └─ Backend Integration                    ✅ COMPLETADO

Milestone 3.5 ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ [████████████] 100%
           └─ CSV/NQL Support Hotfix (v0.1.0)        ✅ COMPLETADO
              ├─ DataSource trait                    ✅ Completado
              ├─ CSV Backend                         ✅ Completado
              ├─ NQL Commands (USE, SHOW, etc.)      ✅ Completado
              ├─ Parser NQL básico                   ✅ Completado
              ├─ TUI/REPL Integration                ✅ Completado
              └─ ResultSet Tables                    ✅ Completado

Milestone 4 ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ [███░░░░░░░░░]  25%
           └─ Advanced Features (Enhanced NQL)       📋 PLANIFICADO
              ├─ IMPORT/EXPORT commands              📋 Pendiente
              ├─ MAP/FILTER transformations          📋 Pendiente
              ├─ Advanced CSV queries                📋 Pendiente
              ├─ Security features                   📋 Pendiente
              └─ Performance optimization            📋 Pendiente

Milestone 5 ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ [░░░░░░░░░░░░]   0%
           └─ Extended Capabilities                  📋 Planificado

Milestone 6 ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ [░░░░░░░░░░░░]   0%
           └─ Noctra 2.0 "FABRIC" (DuckDB)           🎯 REVOLUCIONARIO
              ├─ DuckDB como motor ad hoc            📋 Planificado
              ├─ NQL 2.0 (archivos nativos)          📋 Planificado
              ├─ EXPORT multi-formato                📋 Planificado
              ├─ Modo híbrido DuckDB+SQLite          📋 Planificado
              └─ Análisis sin base de datos          📋 Planificado
```

**MVP Release:** ✅ Completado (M1-M3)
**v0.1.0 Release:** ✅ Completado (M3.5 Hotfix)
**v0.2.0 Release:** 📋 Planificado (M4)
**v1.0.0 Release:** 📋 Planificado (M5)
**v2.0.0 "FABRIC" Release:** 🎯 Planificado (M6)

---

## Milestone 0: Foundation

**Duration:** 1 week
**Status:** ✅ Complete
**Target Date:** 2025-01-12

### Objectives

Establish project foundation with proper structure, build system, and documentation.

### Deliverables

#### ✅ Project Structure
- [x] Cargo workspace configured
- [x] All crate directories created
- [x] Initial `Cargo.toml` files
- [x] `.gitignore` and `.editorconfig`
- [x] License files (MIT + Apache 2.0)

#### ✅ CI/CD Pipeline
- [x] GitHub Actions workflow
- [x] Automated testing on push/PR
- [x] Multi-platform builds (Linux, macOS, Windows)
- [x] Clippy and rustfmt checks
- [x] Code coverage tracking

#### ✅ Documentation
- [x] README.md with project overview
- [x] DESIGN.md with technical architecture
- [x] GETTING_STARTED.md for new users
- [x] ROADMAP.md (this document)
- [x] CONTRIBUTING.md guidelines

#### ✅ Development Setup
- [x] VSCode/RustRover config files
- [x] Pre-commit hooks
- [x] Local development database setup
- [x] Example data fixtures

### Success Criteria

- ✅ `cargo build --workspace` succeeds
- ✅ `cargo test --workspace` passes (even with empty tests)
- ✅ CI pipeline runs successfully
- ✅ Documentation is comprehensive and accurate

---

## Milestone 1: Core MVP

**Duration:** 3 weeks
**Status:** ✅ Complete (100%)
**Completion Date:** 2025-08-15

### Objectives

Build the minimal viable product with core functionality: SQL execution against SQLite with basic REPL interface.

### Week 1: Core Engine

#### Tasks

**noctra-core:**
- [ ] Implement `Value` type with all variants
- [ ] Implement `Session` with variable management
- [ ] Implement `Executor` with basic execution
- [ ] Create SQLite backend wrapper
- [ ] Add parameter binding support
- [ ] Implement result set handling

**Code Example:**
```rust
pub struct Executor {
    backend: SqliteBackend,
    session: Session,
}

impl Executor {
    pub async fn execute(&mut self, sql: &str) -> Result<ResultSet> {
        // Implementation
    }

    pub async fn execute_with_params(
        &mut self,
        sql: &str,
        params: Vec<Value>
    ) -> Result<ResultSet> {
        // Implementation
    }
}
```

**Tests:**
- [ ] Unit tests for `Value` type conversions
- [ ] Unit tests for `Session` variable management
- [ ] Integration tests with in-memory SQLite
- [ ] Parameter binding tests

### Week 2: Parser & REPL

#### Tasks

**noctra-parser:**
- [ ] Integrate `sqlparser-rs`
- [ ] Implement basic RQL parser
- [ ] Add positional parameter support (`$1`, `$2`)
- [ ] Add named parameter support (`:name`)
- [ ] Implement `USE` command parser
- [ ] Implement `LET` command parser

**noctra-cli:**
- [ ] Setup `rustyline` for REPL
- [ ] Implement basic command loop
- [ ] Add command history
- [ ] Add history persistence
- [ ] Implement basic error handling
- [ ] Add colored output with `colored` crate

**Code Example:**
```rust
pub struct Repl {
    executor: Arc<Mutex<Executor>>,
    editor: Editor<()>,
    history_file: PathBuf,
}

impl Repl {
    pub async fn run(&mut self) -> Result<()> {
        loop {
            let line = self.editor.readline("noctra> ")?;
            self.handle_line(&line).await?;
        }
    }
}
```

**Tests:**
- [ ] Parser tests for SQL statements
- [ ] Parser tests for RQL extensions
- [ ] Parameter extraction tests
- [ ] REPL simulation tests

### Week 3: Results Display & Integration

#### Tasks

**noctra-cli:**
- [ ] Implement table formatter
- [ ] Add ASCII box drawing
- [ ] Implement column width calculation
- [ ] Add number formatting
- [ ] Add execution time display
- [ ] Implement batch mode (`-c` flag)

**Integration:**
- [ ] End-to-end workflow tests
- [ ] Sample database creation script
- [ ] Example queries documentation
- [ ] Performance benchmarks

**Code Example:**
```rust
pub fn format_table(result: &ResultSet) -> String {
    let mut output = String::new();

    // Header
    output.push_str("┌");
    for (i, col) in result.columns.iter().enumerate() {
        output.push_str(&"─".repeat(col.width));
        if i < result.columns.len() - 1 {
            output.push_str("┬");
        }
    }
    output.push_str("┐\n");

    // ... rows and footer

    output
}
```

**Tests:**
- [ ] Table formatting tests
- [ ] Wide column handling tests
- [ ] Empty result set tests
- [ ] Large result set tests (pagination)

### Deliverables

- [ ] Working `noctra` CLI binary
- [ ] SQLite backend fully functional
- [ ] Basic RQL support (SQL + parameters)
- [ ] REPL with history
- [ ] Table output formatting
- [ ] Batch execution mode
- [ ] Test suite with >70% coverage

### Success Criteria

**Functional:**
```bash
$ noctra
noctra> USE demo;
Schema changed to: demo

noctra> LET dept = 'SALES';
Variable dept = "SALES"

noctra> SELECT * FROM employees WHERE dept = :dept;
┌──────┬──────────────┬────────┬──────────┐
│ id   │ name         │ dept   │ salary   │
├──────┼──────────────┼────────┼──────────┤
│ 1001 │ John Smith   │ SALES  │ 75000.00 │
│ 1002 │ Mary Johnson │ SALES  │ 68000.00 │
└──────┴──────────────┴────────┴──────────┘
(2 rows, 12.3ms)

noctra> .exit
```

**Technical:**
- All tests pass
- No clippy warnings
- Code formatted with rustfmt
- Documentation up to date

---

## Milestone 2: Forms & TUI

**Duration:** 3 weeks
**Status:** ✅ Complete (100%)
**Completion Date:** 2025-09-20

### Objectives

Implement FDL2 form system and Noctra Window Manager (NWM) for professional terminal UI.

### Week 1: Form Library

#### Tasks

**noctra-formlib:**
- [ ] Implement TOML loader with `serde`
- [ ] Define `Form` struct hierarchy
- [ ] Implement field type system
- [ ] Add field validation engine
- [ ] Create template processor
- [ ] Implement SQL compiler

**Code Example:**
```rust
#[derive(Debug, Deserialize)]
pub struct Form {
    pub title: String,
    pub fields: HashMap<String, Field>,
    pub actions: HashMap<String, Action>,
}

pub fn load_form(path: &Path) -> Result<Form> {
    let content = fs::read_to_string(path)?;
    let form: Form = toml::from_str(&content)?;
    Ok(form)
}
```

**Tests:**
- [ ] TOML parsing tests
- [ ] Field validation tests
- [ ] Template compilation tests
- [ ] SQL generation tests

### Week 2: Noctra Window Manager (NWM)

#### Tasks

**noctra-tui:**
- [ ] Setup `ncurses` bindings
- [ ] Implement window management system
- [ ] Create header/footer bars
- [ ] Implement command mode
- [ ] Implement result mode
- [ ] Add keyboard event handling

**Code Example:**
```rust
pub struct NoctraWindowManager {
    screen: Window,
    header: Window,
    main_area: Window,
    footer: Window,
    mode: UiMode,
}

pub enum UiMode {
    Command,
    Result,
    Form,
    Dialog,
}
```

**Tests:**
- [ ] Window creation tests
- [ ] Mode switching tests
- [ ] Keyboard handling tests
- [ ] Layout calculation tests

### Week 3: Form Rendering & Integration

#### Tasks

**noctra-tui:**
- [ ] Implement form renderer
- [ ] Add field input widgets
- [ ] Implement dropdown/enum widgets
- [ ] Add form validation display
- [ ] Implement dialog boxes
- [ ] Add form submission handling

**noctra-cli:**
- [ ] Integrate NWM with REPL
- [ ] Add `FORM LOAD` command
- [ ] Add `FORM EXECUTE` command
- [ ] Implement form-to-query binding

**Tests:**
- [ ] Form rendering tests
- [ ] Input validation tests
- [ ] Full form workflow tests

### Deliverables

- [ ] FDL2 loader functional
- [ ] Form validation system
- [ ] NWM with all modes
- [ ] Form rendering in TUI
- [ ] Example forms library
- [ ] Form documentation

### Success Criteria

**Functional:**
```bash
$ noctra
noctra> FORM LOAD 'examples/employees.toml';

┌─────────────────────────────────────────────┐
│         Employee Search                     │
├─────────────────────────────────────────────┤
│ Employee ID: [     ]                        │
│ Name:        [                    ]         │
│ Department:  [SALES      ▼]                 │
│                                             │
│   [ Search ]  [ Clear ]  [ Cancel ]        │
└─────────────────────────────────────────────┘

[User fills form and presses Search]

┌──────┬──────────────┬────────┬──────────┐
│ id   │ name         │ dept   │ salary   │
├──────┼──────────────┼────────┼──────────┤
│ 1001 │ John Smith   │ SALES  │ 75000.00 │
└──────┴──────────────┴────────┴──────────┘
```

**Technical:**
- Forms load and validate correctly
- TUI renders properly on 80x24 terminal
- All keyboard shortcuts work
- No rendering artifacts
- Test coverage >75%

---

## Milestone 3: Backend Integration

**Duration:** 2 weeks
**Status:** ✅ Complete (100%)
**Completion Date:** 2025-10-30

### Objectives

Integrate TUI with backend, add advanced RQL features, and enhance functionality.

### Week 1: RQL Extensions

#### Tasks

**noctra-parser:**
- [ ] Implement `SHOW` command variants
- [ ] Add `OUTPUT TO` command
- [ ] Implement template conditionals
- [ ] Add running aggregates support
- [ ] Implement query macros

**Running Aggregates:**
```sql
-- Support window function emulation
SELECT
    date,
    sales,
    RUNSUM(sales) as cumulative_sales,
    RUNAVG(sales) as moving_average
FROM daily_sales
ORDER BY date;

-- Translates to:
SELECT
    date,
    sales,
    SUM(sales) OVER (ORDER BY date ROWS UNBOUNDED PRECEDING) as cumulative_sales,
    AVG(sales) OVER (ORDER BY date ROWS UNBOUNDED PRECEDING) as moving_average
FROM daily_sales
ORDER BY date;
```

**Tasks:**
- [ ] Parse `RUNSUM`, `RUNCOUNT`, `RUNAVG`, `RUNMIN`, `RUNMAX`
- [ ] Translate to window functions for modern backends
- [ ] Implement manual accumulation fallback
- [ ] Add aggregation tests

**Code Example:**
```rust
pub enum RunningAggregate {
    RunSum(Expr),
    RunCount,
    RunAvg(Expr),
    RunMin(Expr),
    RunMax(Expr),
}

impl RunningAggregate {
    pub fn to_window_function(&self) -> WindowFunction {
        match self {
            RunSum(expr) => WindowFunction {
                func: AggregateFunction::Sum(expr.clone()),
                order_by: vec![],
                frame: WindowFrame::unbounded_preceding(),
            },
            // ...
        }
    }
}
```

**noctra-core:**
- [ ] Implement output formatters (CSV, JSON)
- [ ] Add file output support
- [ ] Implement query timeout handling
- [ ] Add result streaming for large datasets

**Tests:**
- [ ] Running aggregate translation tests
- [ ] Template processing tests
- [ ] Output format tests
- [ ] Streaming tests

### Week 2: Multi-Backend Support

#### Tasks

**noctra-core:**
- [ ] Abstract `DatabaseBackend` trait
- [ ] Implement connection pooling
- [ ] Add backend feature detection
- [ ] Create PostgreSQL backend
- [ ] Add backend-specific optimizations

**Code Example:**
```rust
#[async_trait]
pub trait DatabaseBackend: Send + Sync {
    async fn connect(&self, config: &ConnectionConfig)
        -> Result<Box<dyn Connection>>;

    fn features(&self) -> BackendFeatures;
    fn name(&self) -> &str;
}

pub struct PostgresBackend {
    pool: PgPool,
}
```

**Configuration:**
- [ ] Add database configuration system
- [ ] Implement connection string parsing
- [ ] Add environment variable support
- [ ] Create config file loader

**Tests:**
- [ ] Backend abstraction tests
- [ ] PostgreSQL integration tests
- [ ] Connection pooling tests
- [ ] Multi-backend compatibility tests

### Deliverables

- [ ] Running aggregates support
- [ ] Template processing
- [ ] CSV/JSON output
- [ ] PostgreSQL backend
- [ ] Connection pooling
- [ ] Configuration system

### Success Criteria

**Running Aggregates:**
```sql
noctra> SELECT month, sales, RUNSUM(sales) FROM monthly_sales;
┌───────┬──────────┬─────────┐
│ month │ sales    │ runsum  │
├───────┼──────────┼─────────┤
│ Jan   │ 10000.00 │ 10000.00│
│ Feb   │ 15000.00 │ 25000.00│
│ Mar   │ 12000.00 │ 37000.00│
└───────┴──────────┴─────────┘
```

**Multi-Backend:**
```bash
$ noctra --backend postgresql --db postgres://localhost/mydb
noctra> SELECT version();
PostgreSQL 15.1 on x86_64-linux-gnu
```

---

## Milestone 3.5: CSV/NQL Hotfix

**Duration:** 1 week
**Status:** ✅ Complete (100%)
**Start Date:** 2025-11-08
**Completion Date:** 2025-11-09
**Version:** v0.1.0

### Objectives

Emergency hotfix to implement CSV file support and basic NQL commands. This milestone delivers ~40% of M4 objectives early to address critical user needs for multi-source data support.

### Background

This hotfix was triggered by a "Failed to prepare" error when attempting to query CSV files in the TUI. The fix evolved into a comprehensive CSV/NQL implementation that bridges M3 and M4.

### Deliverables

#### ✅ Multi-Source Architecture
- [x] `DataSource` trait abstraction (`datasource.rs` - 250 lines)
  - Unified interface for different data sources
  - `query()` method for SQL execution
  - `schema()` method for metadata introspection
  - `name()` and `source_type()` accessors

- [x] `SourceRegistry` for managing multiple sources
  - Active source tracking and switching
  - HashMap-based source storage
  - Thread-safe implementation (Send + Sync)

- [x] `SourceType` enum (SQLite, CSV, JSON, Memory)

#### ✅ CSV Backend Implementation
- [x] Complete CSV data source (`csv_backend.rs` - 420 lines)
  - Automatic delimiter detection (`,`, `;`, `\t`, `|`)
  - Smart type inference (INTEGER, REAL, BOOLEAN, TEXT)
  - Header detection and column naming
  - Quote-aware CSV parsing
  - Schema introspection support
  - Full ResultSet integration

#### ✅ NQL Command Support
- [x] `USE <path> AS <alias> OPTIONS (...)` - Load data sources
  - Example: `USE './data.csv' AS csv OPTIONS (delimiter=',', header=true);`

- [x] `SHOW SOURCES` - List all registered data sources
  - Returns 3-column table: (Alias, Tipo, Path)

- [x] `SHOW TABLES [FROM source]` - List tables from sources
  - Returns 1-column table: (table)

- [x] `DESCRIBE source.table` - Show table schema
  - Returns 2-column table: (Campos, Tipo)

- [x] `SHOW VARS` - Display session variables
  - Returns 2-column table: (Variable, Valor)

- [x] `LET variable = value` - Set session variables

- [x] `UNSET variable...` - Remove session variables

#### ✅ Parser Enhancements
- [x] Enhanced OPTIONS parsing (`parser.rs`)
  - `split_options()` method respects quote boundaries
  - Handles: `delimiter=','` without breaking on internal commas
  - Supports both single (`'`) and double (`"`) quotes
  - Quote-aware tokenization

#### ✅ TUI/REPL Integration
- [x] RqlProcessor integration in TUI (`noctra_tui.rs` - 300 lines)
  - Thread-spawning parser to avoid Tokio runtime conflicts
  - All NQL commands return SQL-style ResultSet tables
  - Enhanced status bar showing `source:table` format
  - `extract_table_name()` helper for context display

- [x] REPL parity with TUI
  - Same thread-spawning pattern
  - Identical command handling
  - Consistent output formatting

- [x] Query routing in `execute_rql()`
  - Check active source first
  - Fallback to SQLite backend
  - Parameter passing preserved

### Technical Challenges Solved

**Challenge 1: "Failed to prepare" Error**
- **Problem**: SQL queries always routed to SQLite backend, ignoring CSV sources
- **Solution**: Added source-aware query routing in `execute_rql()`
- **Commit**: `0438e65`

**Challenge 2: Tokio Runtime Panic**
- **Problem**: "Cannot start a runtime from within a runtime"
- **Root Cause**: RqlProcessor creating new runtime inside TUI's existing runtime
- **Solution**: Spawn dedicated thread with isolated runtime for parsing
- **Applied To**: Both TUI and REPL
- **Commits**: `ae57113` (TUI), `9e64243` (REPL)

**Challenge 3: OPTIONS Parser with Quoted Delimiters**
- **Problem**: `delimiter=','` broke parser (split on comma inside quotes)
- **Solution**: Implemented `split_options()` with quote-aware state machine
- **Commit**: `9e64243`

**Challenge 4: TUI/REPL Parity**
- **Problem**: TUI used `execute_sql()`, REPL used `execute_rql()`
- **Solution**: Both now use RqlProcessor with consistent behavior
- **Commit**: `5b9940e`

**Challenge 5: NQL Display Format**
- **Problem**: NQL commands showing as dialog boxes instead of SQL tables
- **Solution**: Converted all handlers to build and return `ResultSet`
- **Impact**: Unified display for SQL and NQL commands
- **Commit**: `dbddebc`

### Commit History

| Commit | Date | Description | Files | Lines |
|--------|------|-------------|-------|-------|
| `0438e65` | 2025-11-08 | fix: Route SQL queries to active data source | 1 | +15 -3 |
| `5b9940e` | 2025-11-08 | fix: Integrate RqlProcessor into TUI | 2 | +120 -45 |
| `ae57113` | 2025-11-08 | fix: Resolve Tokio runtime panic (TUI) | 1 | +35 -20 |
| `9e64243` | 2025-11-09 | fix: Fix OPTIONS parsing and REPL runtime | 2 | +80 -30 |
| `b65ca95` | 2025-11-09 | feat: Add complete NQL command support to TUI | 1 | +250 -50 |
| `dbddebc` | 2025-11-09 | feat: Convert NQL commands to SQL-style tables | 1 | +180 -120 |

### Success Criteria

**Functional:**
- ✅ Load CSV files with `USE` command
- ✅ Query CSV data with `SELECT * FROM table`
- ✅ All NQL commands functional (SHOW, DESCRIBE, LET, UNSET)
- ✅ Multi-source management working
- ✅ TUI and REPL have identical behavior
- ✅ Status bar shows `source:table` format

**Technical:**
- ✅ All tests pass (29 tests)
- ✅ Zero warnings on build
- ✅ No Tokio runtime conflicts
- ✅ Thread-safe implementation
- ✅ Clean separation of concerns

**Performance:**
- ✅ Build time: 8-18s
- ✅ CSV parsing: <100ms for typical files
- ✅ No memory leaks detected

### Known Limitations

- CSV backend only supports `SELECT * FROM table`
- No support for WHERE, JOIN, GROUP BY, ORDER BY on CSV
- No INSERT/UPDATE/DELETE on CSV files
- Advanced SQL features require SQLite backend
- Large CSV files (>10MB) not optimized

### Impact on M4

**Work Completed Early (~40% of M4.10):**
- ✅ DataSource trait architecture
- ✅ CSV backend implementation
- ✅ Basic NQL commands (USE, SHOW, DESCRIBE)
- ✅ Parser OPTIONS support
- ✅ TUI integration

**Remaining for M4:**
- IMPORT/EXPORT commands
- MAP/FILTER transformations
- Advanced CSV queries (WHERE, JOIN, etc.)
- Security features
- Performance optimization
- Daemon mode

### Documentation

- [x] CHANGELOG.md created with v0.1.0 release notes
- [x] PROJECT_STATUS.md updated with M3.5 section
- [x] GETTING_STARTED.md updated with CSV examples
- [x] ROADMAP.md updated (this document)

### Statistics

- **Files Modified**: 8
- **Lines Added**: ~1,100
- **New Files**: 2 (csv_backend.rs, datasource.rs)
- **Test Coverage**: 29 tests passing
- **Build Status**: ✅ Clean (0 warnings)

### Example Usage

```sql
-- Load CSV file
USE './examples/clientes.csv' AS csv OPTIONS (delimiter=',', header=true);

-- Query the data
SELECT * FROM clientes;

-- Inspect metadata
SHOW SOURCES;
SHOW TABLES FROM csv;
DESCRIBE csv.clientes;

-- Session variables
LET myvar = 'test value';
SHOW VARS;
UNSET myvar;
```

---

## Milestone 4: Advanced Features + NQL

**Duration:** 3-4 weeks
**Status:** 📋 Planning (M3.5 completed ~40% of objectives)
**Start Date:** 2025-11-10 (Planned)
**Target Date:** 2025-12-08

### Objectives

Extend NQL capabilities with advanced features, security hardening, and performance optimization. M3.5 hotfix completed the foundation, so M4 focuses on advanced functionality.

**Note:** M3.5 completed DataSource trait, CSV backend, basic NQL commands, and TUI integration.

### Advanced NQL Commands (Week 1-2)

#### Tasks

- [ ] **IMPORT Command**
  - [ ] `IMPORT FROM 'file.csv' INTO table OPTIONS (...)`
  - [ ] Support multiple formats (CSV, JSON, TSV)
  - [ ] Batch import with progress feedback
  - [ ] Error handling and validation

- [ ] **EXPORT Command**
  - [ ] `EXPORT table TO 'file.csv' OPTIONS (...)`
  - [ ] Multiple output formats
  - [ ] Column selection support
  - [ ] Custom delimiters and headers

- [ ] **MAP/FILTER Transformations**
  - [ ] `MAP expression OVER table`
  - [ ] `FILTER condition FROM table`
  - [ ] Chainable transformations
  - [ ] Type-safe operations

**Code Example:**
```sql
-- Import data
IMPORT FROM 'data.csv' INTO customers OPTIONS (delimiter=',', skip_rows=1);

-- Export with custom format
EXPORT sales_2023 TO 'report.csv' OPTIONS (delimiter=';', header=true);

-- Transform data
MAP price * 1.1 OVER products;
FILTER sales > 1000 FROM transactions;
```

### Enhanced CSV Support (Week 2)

#### Tasks

- [ ] **Advanced CSV Queries**
  - [ ] WHERE clause support
  - [ ] ORDER BY implementation
  - [ ] LIMIT/OFFSET support
  - [ ] Basic JOIN support (single table joins)
  - [ ] Aggregations (COUNT, SUM, AVG, MIN, MAX)

- [ ] **CSV Optimizations**
  - [ ] Lazy loading for large files
  - [ ] Index creation for frequently queried columns
  - [ ] Query result caching
  - [ ] Memory-mapped file support for >10MB files

### Security & Performance (Week 3-4)

#### Security Tasks

- [ ] **Input Validation**
  - [ ] SQL injection prevention
  - [ ] File path validation and sandboxing
  - [ ] Resource limits (max rows, timeout)
  - [ ] Query complexity analysis

- [ ] **Authentication & Authorization**
  - [ ] Basic authentication for daemon mode
  - [ ] Token-based session management
  - [ ] Role-based access control (basic)
  - [ ] Audit logging

**Code Example:**
```rust
pub struct SecurityValidator {
    max_rows: usize,
    query_timeout: Duration,
    allowed_paths: Vec<PathBuf>,
}

impl SecurityValidator {
    pub fn validate_query(&self, query: &str) -> Result<()> {
        if query.len() > self.max_query_length {
            return Err(Error::QueryTooLong);
        }
        self.check_dangerous_keywords(query)?;
        Ok(())
    }

    pub fn validate_path(&self, path: &Path) -> Result<()> {
        let canonical = path.canonicalize()?;
        if !self.allowed_paths.iter().any(|p| canonical.starts_with(p)) {
            return Err(Error::PathNotAllowed);
        }
        Ok(())
    }
}
```

#### Performance Tasks

- [ ] **Query Optimization**
  - [ ] Query result caching with TTL
  - [ ] Prepared statement pooling
  - [ ] Query plan caching
  - [ ] Lazy result loading for large datasets

- [ ] **TUI Optimization**
  - [ ] Optimize table rendering (virtual scrolling)
  - [ ] Reduce allocations in hot paths
  - [ ] Profile and optimize parser
  - [ ] Connection pooling for backends

**Performance Targets:**
- Query execution: <100ms for simple queries
- Parser: <1ms for typical queries
- Table rendering: <50ms for 100 rows
- Memory usage: <50MB baseline
- CSV parsing: <500ms for 1MB files

**Benchmarks:**
- [ ] Query execution benchmarks
- [ ] Parser benchmarks
- [ ] Rendering benchmarks
- [ ] End-to-end workflow benchmarks

### Deliverables

- [ ] IMPORT/EXPORT commands functional
- [ ] MAP/FILTER transformations working
- [ ] Advanced CSV queries (WHERE, ORDER BY, etc.)
- [ ] Security validation framework
- [ ] Performance optimizations applied
- [ ] Comprehensive test suite (>80% coverage)
- [ ] Updated documentation
- [ ] v0.2.0 release

### Success Criteria

**Advanced NQL:**
- ✅ IMPORT/EXPORT commands working for CSV/JSON
- ✅ MAP/FILTER transformations functional
- ✅ WHERE/ORDER BY/LIMIT work on CSV files
- ✅ All commands tested and documented

**Security:**
- ✅ No SQL injection vulnerabilities
- ✅ Input validation complete
- ✅ File path sandboxing working
- ✅ Resource limits enforced

**Performance:**
- ✅ All benchmarks meet targets
- ✅ Memory usage optimized (<50MB baseline)
- ✅ CSV files >10MB handled efficiently
- ✅ Query result caching working

**Testing:**
- ✅ Test coverage >80%
- ✅ All integration tests passing
- ✅ Performance benchmarks established
- ✅ Security tests comprehensive

---

## Milestone 5: Extended Capabilities

**Duration:** 4-6 weeks
**Status:** ⏸️ Not Started
**Target Date:** 2026-01-15

### Objectives

Advanced UI features, data visualization, and ecosystem integration. Focus on user experience and integrations.

**Note:** DuckDB integration moved to M6 (Noctra 2.0 FABRIC)

### Phase 1: Additional Backends (2 weeks)

- [ ] MySQL/MariaDB backend
- [ ] Backend adapter documentation
- [ ] Cross-backend compatibility tests
- [ ] Connection string standardization

### Phase 2: Advanced UI (2 weeks)

- [ ] Enhanced table navigation
- [ ] Visual query builder (basic)
- [ ] Form designer tool
- [ ] Syntax highlighting
- [ ] Auto-completion
- [ ] Query history search

### Phase 3: Data Visualization (2 weeks)

- [ ] ASCII chart rendering
- [ ] Histogram support
- [ ] Bar chart support
- [ ] Export to Gnuplot format
- [ ] Integration with plotters-rs (optional)

**Example:**
```sql
noctra> SELECT dept, COUNT(*) FROM employees GROUP BY dept CHART BAR;

IT        ████████████████ 45
SALES     ██████████████████████ 62
HR        ████████ 23
FINANCE   ████████████ 34
```

### Phase 4: Integrations (2 weeks)

- [ ] Python bindings (PyO3)
- [ ] JavaScript/WASM version
- [ ] C FFI library
- [ ] REST API client library
- [ ] Excel export support
- [ ] VS Code extension

### Deliverables

- [ ] MySQL/MariaDB backend
- [ ] Enhanced TUI features (navigation, query builder)
- [ ] Data visualization (ASCII charts)
- [ ] Language bindings (Python, JS/WASM, C FFI)
- [ ] VS Code extension
- [ ] v1.0.0 release

### Success Criteria

- MySQL backend fully functional
- Advanced UI features working
- At least 2 language bindings available
- Data visualization rendering correctly
- VS Code extension published
- Production deployment guide complete
- v1.0.0 release published

---

## Milestone 6: Noctra 2.0 "FABRIC"

**Duration:** 7 weeks (Extended from 6 weeks, v2 plan)
**Status:** 📋 Planning Phase (Fase 1 en progreso)
**Target Date:** 2025-12-23 (Updated from 2026-03-01)
**Version:** v2.0.0 (v0.6.0)
**Implementation Plan:** [M6_IMPLEMENTATION_PLAN_v2.md](M6_IMPLEMENTATION_PLAN_v2.md)

> ℹ️ **NOTE:** M6 timeline extended to 7 weeks to incorporate critical DuckDB research findings including:
> - Mandatory Arrow integration (Phase 1, not Phase 5)
> - New Phase 1.5: Performance Configuration Layer (2 days)
> - Dynamic thread configuration based on I/O type (Local vs Remote)
> - AttachmentRegistry for non-persistent ATTACH statements
> - Updated performance targets (CSV 10MB: 500ms→200ms)
> - Mandatory `PER_THREAD_OUTPUT` for parallel Parquet exports
>
> See [M6_IMPLEMENTATION_PLAN_v2.md](M6_IMPLEMENTATION_PLAN_v2.md) for detailed research-driven implementation plan.

### Vision Statement

> **"No importes datos. Consúltalos."**
> **"Un archivo. Una tabla. Un lenguaje."**
> **"Noctra no necesita una base de datos. Tú sí."**

### Objectives

Transform Noctra into a **Data Fabric Engine** by integrating DuckDB as the primary ad hoc analytics engine. Enable querying any file (CSV, JSON, Parquet) as native SQL tables without staging, imports, or mandatory databases.

**Key Innovation:** Files become tables. Queries become instant. Databases become optional.

### Current State Analysis

| Área | Estado actual | Observación |
|------|---------------|-------------|
| **Arquitectura** | Modular, escalable, con crates | ✅ Excelente |
| **Motor SQL** | RQL + backends (SQLite, PG, MySQL) | ✅ Sólido |
| **TUI (NWM)** | Ncurses profesional, modos, temas | ✅ De referencia |
| **Forms (FDL2)** | Declarativas, compilables | ✅ Únicas en su tipo |
| **NQL (M3.5/M4)** | Multi-fuente, CSV directo, `USE` | ✅ El futuro |
| **DuckDB** | No integrado | 🎯 **La pieza que falta** |

### Phase 1: NQL 2.0 - File-Native Queries (Week 1)

#### 1.1 `USE 'archivo.csv'` → Tabla lógica inmediata

**Syntax:**
```sql
USE 'ventas_2024.csv' AS ventas;
-- → DuckDB registra el CSV como tabla virtual
-- → Inferencia automática: tipos, delimitador, header
```

**Implementation:**
- DuckDB `read_csv_auto()` function
- No `IMPORT` required
- No SQLite staging
- Instant table registration

**Behavior:**
- `DESCRIBE ventas` → shows inferred types
- `SELECT * FROM ventas` → executes on DuckDB
- Multi-file support: `USE 'sales_*.csv' AS sales`
- Compressed files: `USE 'data.csv.gz' AS data`

#### 1.2 Direct `SELECT` on Any Source

**Syntax:**
```sql
SELECT pais, SUM(total)
FROM 'clientes.csv'
WHERE edad > 30
GROUP BY pais;
```

**Engine:** DuckDB (`read_csv_auto`)

**Features:**
- No pre-registration needed
- Automatic type inference
- Full SQL support (WHERE, GROUP BY, HAVING, ORDER BY, LIMIT)

#### 1.3 Cross-Source `JOIN`

**Syntax:**
```sql
USE 'clientes.csv' AS csv;
USE 'pedidos.db' AS db;

SELECT c.nombre, p.total
FROM csv.clientes c
JOIN db.pedidos p ON c.id = p.cliente_id;
```

**Engine:** DuckDB + SQLite (via `ATTACH DATABASE`)

**Implementation:**
```rust
// Attach SQLite database to DuckDB
duckdb.execute(&format!(
    "ATTACH '{}' AS {} (TYPE SQLITE)",
    db_path, alias
))?;
```

#### 1.4 Multi-Format `EXPORT`

**Syntax:**
```sql
EXPORT (SELECT * FROM 'ventas.csv' WHERE pais = 'AR')
TO 'argentinos.json' FORMAT JSON;

EXPORT ventas TO 'backup.parquet' FORMAT PARQUET;
```

**Engine:** DuckDB `COPY TO`

**Supported Formats:**
- CSV (with custom delimiters)
- JSON (array or newline-delimited)
- Parquet (columnar, compressed)
- Excel (via extension)

#### 1.5 `MAP` and `FILTER` Transformations

**Syntax:**
```sql
USE 'datos.csv';
MAP nombre = UPPER(nombre),
    categoria = CASE WHEN precio > 1000 THEN 'Premium' ELSE 'Standard' END;
FILTER activo = true;
SELECT * FROM datos;
```

**Engine:** DuckDB + CTEs

**Translation:**
```sql
-- Translates to:
WITH transformed AS (
    SELECT
        UPPER(nombre) AS nombre,
        CASE WHEN precio > 1000 THEN 'Premium' ELSE 'Standard' END AS categoria,
        *
    FROM datos
    WHERE activo = true
)
SELECT * FROM transformed;
```

### Phase 2: DuckDB Integration Architecture (Week 1)

#### 2.1 New Crate: `noctra-duckdb`

**Structure:**
```
noctra/
├── crates/
│   ├── noctra-core/           # + QueryEngine::DuckDB
│   ├── noctra-parser/         # + NQL 2.0 extensions
│   ├── noctra-duckdb/         # ← NUEVO
│   │   ├── src/
│   │   │   ├── lib.rs         # Main entry point
│   │   │   ├── source.rs      # DuckDBSource impl
│   │   │   ├── engine.rs      # Query engine
│   │   │   └── extensions.rs  # DuckDB extensions (JSON, Parquet)
│   │   └── Cargo.toml
│   ├── noctra-tui/            # + barra de fuente dinámica
│   └── noctra-cli/            # + --engine duckdb
```

**Dependencies:**
```toml
# crates/noctra-duckdb/Cargo.toml
[package]
name = "noctra-duckdb"
version = "2.0.0"

[dependencies]
duckdb = { version = "1.1", features = ["bundled", "parquet", "json"] }
noctra-core = { path = "../noctra-core" }
anyhow = "1.0"
```

#### 2.2 QueryEngine Enum Extension

**Code:**
```rust
// noctra-core/src/engine.rs
pub enum QueryEngine {
    Sqlite(Box<dyn DatabaseBackend>),
    DuckDB(DuckDBConnection),        // ← NUEVO
    Hybrid {
        duckdb: DuckDBConnection,
        sqlite: SqliteConnection
    },
}

impl QueryEngine {
    pub async fn execute(&mut self, nql: &NqlStatement) -> Result<ResultSet> {
        match self {
            Self::DuckDB(conn) => conn.execute_nql(nql).await,
            Self::Hybrid { duckdb, sqlite } => {
                // Route to appropriate engine based on source type
                match nql.source_type()? {
                    SourceType::Csv | SourceType::Json | SourceType::Parquet
                        => duckdb.execute_nql(nql).await,
                    SourceType::Sqlite
                        => sqlite.execute_nql(nql).await,
                }
            },
            Self::Sqlite(backend) => backend.execute(nql).await,
        }
    }

    pub fn new_hybrid() -> Result<Self> {
        Ok(Self::Hybrid {
            duckdb: DuckDBConnection::new_in_memory()?,
            sqlite: SqliteConnection::new_in_memory()?,
        })
    }
}
```

#### 2.3 DuckDBSource Implementation

**Code:**
```rust
// noctra-duckdb/src/source.rs
use duckdb::{Connection, params};
use noctra_core::{DataSource, ResultSet, Parameters, Value, SourceType};

pub struct DuckDBSource {
    conn: Connection,
    name: String,
    source_type: SourceType,
}

impl DuckDBSource {
    pub fn new_in_memory() -> Result<Self> {
        Ok(Self {
            conn: Connection::open_in_memory()?,
            name: "duckdb".to_string(),
            source_type: SourceType::DuckDB,
        })
    }

    pub fn register_file(&mut self, path: &str, alias: &str) -> Result<()> {
        let extension = std::path::Path::new(path)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");

        let sql = match extension {
            "csv" | "tsv" => {
                format!("CREATE VIEW {} AS SELECT * FROM read_csv_auto('{}')", alias, path)
            },
            "json" => {
                format!("CREATE VIEW {} AS SELECT * FROM read_json_auto('{}')", alias, path)
            },
            "parquet" => {
                format!("CREATE VIEW {} AS SELECT * FROM read_parquet('{}')", alias, path)
            },
            _ => return Err(anyhow::anyhow!("Unsupported file type: {}", extension)),
        };

        self.conn.execute(&sql, [])?;
        Ok(())
    }

    pub fn attach_sqlite(&mut self, db_path: &str, alias: &str) -> Result<()> {
        self.conn.execute(
            &format!("ATTACH '{}' AS {} (TYPE SQLITE)", db_path, alias),
            [],
        )?;
        Ok(())
    }
}

impl DataSource for DuckDBSource {
    fn query(&self, sql: &str, params: &Parameters) -> Result<ResultSet> {
        let mut stmt = self.conn.prepare(sql)?;

        // Convert noctra Parameters to duckdb params
        let duckdb_params = params.iter().map(|v| match v {
            Value::Integer(i) => params![i],
            Value::Real(r) => params![r],
            Value::Text(s) => params![s],
            Value::Boolean(b) => params![b],
            Value::Null => params![None::<i64>],
        }).collect::<Vec<_>>();

        let rows = stmt.query_map(&duckdb_params[..], |row| {
            // Convert DuckDB row to noctra ResultSet
            // ... implementation
        })?;

        Ok(ResultSet::from_rows(rows))
    }

    fn schema(&self) -> Result<Vec<TableInfo>> {
        let sql = "SELECT table_name FROM information_schema.tables WHERE table_schema = 'main'";
        let mut stmt = self.conn.prepare(sql)?;
        let tables = stmt.query_map([], |row| {
            let name: String = row.get(0)?;
            Ok(name)
        })?;

        tables.into_iter()
            .map(|table| {
                let columns = self.get_table_columns(&table?)?;
                Ok(TableInfo {
                    name: table?,
                    columns,
                })
            })
            .collect()
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn source_type(&self) -> &SourceType {
        &self.source_type
    }
}
```

### Phase 3: NQL 2.0 Extensions (Week 1)

#### NQL Command Mappings

| NQL Command | DuckDB Implementation |
|-------------|----------------------|
| `USE 'file.csv' AS t` | `CREATE VIEW t AS SELECT * FROM read_csv_auto('file.csv')` |
| `USE 'data/*.csv'` | `SELECT * FROM read_csv('data/sales_*.csv', AUTO_DETECT=TRUE)` |
| `USE 'data.csv.gz'` | Automatic compression detection |
| `DESCRIBE t` | `PRAGMA table_info(t)` or `information_schema.columns` |
| `EXPORT ... TO 'file.json'` | `COPY (...) TO 'file.json' (FORMAT JSON)` |
| `EXPORT ... TO 'file.parquet'` | `COPY (...) TO 'file.parquet' (FORMAT PARQUET)` |

#### Parser Extensions

```rust
// noctra-parser/src/nql.rs
pub enum NqlStatement {
    // ... existing variants

    // New NQL 2.0 variants
    Export {
        query: Box<NqlStatement>,
        path: String,
        format: ExportFormat,
        options: HashMap<String, String>,
    },
    Map {
        transformations: Vec<MapTransform>,
        table: String,
    },
    Filter {
        condition: Expr,
        table: String,
    },
}

pub enum ExportFormat {
    Csv,
    Json,
    Parquet,
    Excel,
}

pub struct MapTransform {
    pub column: String,
    pub expression: Expr,
}
```

### Phase 4: TUI Enhancements (Week 2)

#### Dynamic Status Bar

**Design:**
```
──( RESULT ) Noctra 2.0 ───── Engine: DuckDB ─── Source: 'ventas.csv' ─── 12ms
┌──────┬────────┬───────┐
│ pais │ total  │ cnt   │
├──────┼────────┼───────┤
│ AR   │ 125034 │ 342   │
│ MX   │ 98723  │ 287   │
│ CL   │ 76234  │ 198   │
└──────┴────────┴───────┘
3 filas | Engine: DuckDB | Memory: 45MB | F5:Run | Ctrl+E:Export
```

**Implementation:**
```rust
// noctra-tui/src/noctra_tui.rs
fn render_status_bar(&self, area: Rect, buf: &mut Buffer) {
    let engine = match &self.query_engine {
        QueryEngine::DuckDB(_) => "DuckDB",
        QueryEngine::Sqlite(_) => "SQLite",
        QueryEngine::Hybrid { .. } => "Hybrid",
    };

    let source_info = self.active_source()
        .map(|s| format!("Source: '{}' ({})", s.name(), s.source_type()))
        .unwrap_or_else(|| "No source".to_string());

    let status = format!(
        " Engine: {} │ {} │ {}ms ",
        engine,
        source_info,
        self.last_query_time.as_millis()
    );

    // Render to status bar...
}
```

#### Source Type Indicators

```
┌─────────────────────────────────────────────────┐
│ 📊 ACTIVE SOURCES                               │
├──────────┬─────────┬──────────────────────────┤
│ Alias    │ Type    │ Path                      │
├──────────┼─────────┼──────────────────────────┤
│ ventas   │ 🦆 CSV  │ ./data/ventas_2024.csv    │
│ clientes │ 🦆 JSON │ ./data/clientes.json      │
│ main     │ 📦 SQLite│ ./database.db           │
└──────────┴─────────┴──────────────────────────┘
```

### Phase 5: Ad Hoc Mode (Week 2)

#### CLI Flags

```bash
# Launch without database (DuckDB only)
noctra --engine duckdb --use 'ventas.csv'

# Hybrid mode (default)
noctra --engine hybrid --db main.db --use 'extra_data.csv'

# Traditional mode (SQLite only)
noctra --engine sqlite --db database.db
```

**Implementation:**
```rust
// noctra-cli/src/main.rs
#[derive(Parser)]
struct Cli {
    #[arg(long, default_value = "hybrid")]
    engine: EngineType,

    #[arg(long)]
    db: Option<String>,

    #[arg(long)]
    use_file: Option<String>,
}

enum EngineType {
    Sqlite,
    DuckDB,
    Hybrid,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let engine = match cli.engine {
        EngineType::DuckDB => {
            let mut duck = DuckDBConnection::new_in_memory()?;
            if let Some(file) = cli.use_file {
                duck.register_file(&file, "data")?;
            }
            QueryEngine::DuckDB(duck)
        },
        EngineType::Sqlite => {
            QueryEngine::Sqlite(Box::new(SqliteBackend::open(&cli.db.unwrap())?))
        },
        EngineType::Hybrid => {
            QueryEngine::new_hybrid()?
        },
    };

    // Launch TUI/REPL with engine...
}
```

### Phase 6: Configuration System (Week 2)

#### Config File

**Location:** `~/.config/noctra/config.toml`

**Content:**
```toml
[engine]
default = "hybrid"  # or "duckdb" or "sqlite"

[duckdb]
temp_dir = "/tmp/noctra-duckdb"
memory_limit = "2GB"
threads = 4
enable_profiling = false

[duckdb.extensions]
auto_install = true
enabled = ["parquet", "json", "excel"]

[csv]
auto_detect = true
sample_rows = 100
null_values = ["NA", "", "NULL"]
delimiter_candidates = [",", ";", "\t", "|"]

[export]
default_format = "csv"
compression = "auto"  # auto, gzip, none

[performance]
query_cache_size = "500MB"
max_result_rows = 10000
streaming_threshold = 1000  # rows
```

**Implementation:**
```rust
// noctra-core/src/config.rs
#[derive(Deserialize)]
pub struct NoctraConfig {
    pub engine: EngineConfig,
    pub duckdb: DuckDBConfig,
    pub csv: CsvConfig,
    pub export: ExportConfig,
    pub performance: PerformanceConfig,
}

impl NoctraConfig {
    pub fn load() -> Result<Self> {
        let config_path = dirs::config_dir()
            .ok_or_else(|| anyhow!("Could not find config directory"))?
            .join("noctra/config.toml");

        if !config_path.exists() {
            return Ok(Self::default());
        }

        let content = std::fs::read_to_string(&config_path)?;
        toml::from_str(&content).map_err(Into::into)
    }
}
```

### Deliverables

**Core:**
- [x] `noctra-duckdb` crate fully implemented
- [x] `QueryEngine` enum with DuckDB, SQLite, Hybrid modes
- [x] NQL 2.0 parser extensions (EXPORT, MAP, FILTER)
- [x] DuckDB file registration (`USE 'file.csv'`)
- [x] Cross-source JOIN support
- [x] Multi-format EXPORT (CSV, JSON, Parquet)

**TUI:**
- [x] Dynamic status bar with engine indicator
- [x] Source type indicators in SHOW SOURCES
- [x] Export shortcut (Ctrl+E)
- [x] Engine selection dialog

**CLI:**
- [x] `--engine` flag (duckdb, sqlite, hybrid)
- [x] `--use` flag for immediate file loading
- [x] Ad hoc mode (no database required)

**Configuration:**
- [x] `~/.config/noctra/config.toml` support
- [x] Engine preferences
- [x] DuckDB memory limits
- [x] CSV auto-detection settings

**Documentation:**
- [x] NQL 2.0 language reference
- [x] DuckDB integration guide
- [x] Migration guide from v1.0
- [x] Performance tuning guide
- [x] Example workflows

### Success Criteria

**Functional:**
- ✅ Load CSV/JSON/Parquet with `USE 'file.ext' AS alias`
- ✅ Query files directly: `SELECT * FROM 'data.csv'`
- ✅ JOIN between CSV and SQLite
- ✅ EXPORT to multiple formats
- ✅ MAP/FILTER transformations working
- ✅ Ad hoc mode launches without database

**Performance:**
- ✅ 10MB CSV loads in <500ms
- ✅ 100K row aggregation in <1s
- ✅ Parquet read 10x faster than CSV
- ✅ Memory usage <100MB for typical workloads

**Quality:**
- ✅ All tests pass (>90% coverage)
- ✅ Zero clippy warnings
- ✅ Documentation complete
- ✅ Example workflows validated

**User Experience:**
- ✅ TUI shows engine and source context
- ✅ Error messages are clear and actionable
- ✅ Configuration is intuitive
- ✅ Migration from v1.0 is seamless

### Example Workflows

**Workflow 1: Ad Hoc CSV Analysis**
```bash
# No database needed!
$ noctra --engine duckdb --use 'sales_2024.csv'
noctra> DESCRIBE sales_2024;
noctra> SELECT product, SUM(amount) FROM sales_2024 GROUP BY product ORDER BY 2 DESC LIMIT 10;
noctra> EXPORT (SELECT * FROM sales_2024 WHERE region = 'LATAM') TO 'latam_sales.json';
```

**Workflow 2: Hybrid Analytics**
```bash
$ noctra --engine hybrid --db warehouse.db --use 'recent_sales.csv'
noctra> USE 'customers.json' AS customers;
noctra> SELECT c.name, s.total
        FROM customers c
        JOIN recent_sales s ON c.id = s.customer_id
        JOIN warehouse.products p ON s.product_id = p.id
        WHERE p.category = 'Electronics';
```

**Workflow 3: Data Pipeline**
```sql
-- Load multiple sources
USE 'raw_data/*.csv' AS raw;
USE 'reference.db' AS ref;

-- Transform
MAP
    date = CAST(fecha AS DATE),
    amount = CAST(monto AS DECIMAL(10,2)),
    category = UPPER(categoria);

-- Filter
FILTER date >= '2024-01-01' AND amount > 0;

-- Enrich
SELECT r.*, ref.category_name
FROM raw r
LEFT JOIN ref.categories ref ON r.category = ref.code;

-- Export
EXPORT (SELECT * FROM raw) TO 'processed.parquet' FORMAT PARQUET;
```

### Known Limitations (v2.0.0)

- DuckDB in-memory only (no persistence of DuckDB databases)
- No support for DuckDB extensions beyond bundled ones
- MAP/FILTER limited to single table operations
- EXPORT limited to single query (no multi-table exports)
- No support for streaming very large files (>10GB)

### Future Enhancements (v2.1+)

- Persistent DuckDB databases
- DuckDB extension marketplace integration
- Streaming mode for files >10GB
- Delta Lake support
- Cloud storage integration (S3, GCS, Azure Blob)
- Remote Parquet files via HTTP
- Query optimization hints
- Materialized views in DuckDB

---

## Future Roadmap

### v1.1.x - Enterprise Features
- Multi-user support
- Role-based access control
- Query audit logging
- Distributed execution
- High availability setup

### v1.2.x - Advanced Analytics
- Machine learning integration
- Time series analysis
- Statistical functions
- Data profiling tools
- Advanced aggregations

### v1.3.x - Cloud Native
- Kubernetes deployment
- Cloud database support (RDS, CloudSQL, etc.)
- Serverless mode
- Auto-scaling
- Observability integration

### v2.0.x - Platform Evolution
- Web-based UI
- Collaborative features
- Scheduled queries
- Report generation
- Email notifications
- Slack/Teams integration

---

## Success Metrics

### Development Velocity

| Milestone | Target Duration | Buffer | Total | Status |
|-----------|----------------|--------|-------|--------|
| M0        | 1 week         | -      | 1 week | ✅ Complete |
| M1        | 3 weeks        | 1 week | 4 weeks | ✅ Complete |
| M2        | 3 weeks        | 1 week | 4 weeks | ✅ Complete |
| M3        | 2 weeks        | 1 week | 3 weeks | ✅ Complete |
| M3.5      | 1 week         | -      | 1 week | ✅ Complete |
| M4        | 3 weeks        | 1 week | 4 weeks | 📋 Planning |
| M5        | 4 weeks        | 1 week | 5 weeks | ⏸️ Pending |
| M6        | 2 weeks        | 1 week | 3 weeks | 📋 Planning |
| **Total** | **19 weeks**   | **7 weeks** | **26 weeks** | **~31% Complete** |

### Quality Metrics

**Code Quality:**
- Test coverage: >80% for all milestones
- Zero critical clippy warnings
- All code formatted with rustfmt
- Documentation coverage: >90%

**Performance:**
- Query execution: <100ms (simple)
- Parser overhead: <1ms
- Memory usage: <100MB (typical workload)
- Startup time: <500ms

**Security:**
- Zero high-severity vulnerabilities
- Regular dependency updates
- Security audit passed

### User Adoption

**MVP (M1):**
- 10 early adopters
- Basic use cases validated
- Initial feedback collected

**v0.1.0 (M4):**
- 100+ downloads
- 5+ GitHub stars
- 2+ external contributions
- Documentation completeness >90%

**v1.0.0 (M5):**
- 1000+ downloads
- 50+ GitHub stars
- 10+ external contributions
- Production deployment examples
- Community engagement active

**v2.0.0 "FABRIC" (M6):**
- 5000+ downloads
- 200+ GitHub stars
- 25+ external contributions
- Featured in data engineering blogs/podcasts
- Enterprise pilot deployments
- Active community forum
- Integration examples with popular tools

---

## Risk Management

### Technical Risks

| Risk | Impact | Probability | Mitigation |
|------|--------|-------------|------------|
| ncurses compatibility issues | High | Medium | Early testing on multiple platforms |
| Performance bottlenecks | Medium | Medium | Regular profiling, benchmarks |
| Backend abstraction complexity | High | Low | Prototype early, iterate |
| Security vulnerabilities | Critical | Low | Security audit, regular updates |

### Schedule Risks

| Risk | Impact | Probability | Mitigation |
|------|--------|-------------|------------|
| Milestone delays | Medium | High | Built-in buffer time |
| Scope creep | High | Medium | Strict milestone definitions |
| Dependency issues | Low | Medium | Pin versions, regular updates |

### Resource Risks

| Risk | Impact | Probability | Mitigation |
|------|--------|-------------|------------|
| Developer availability | High | Medium | Clear documentation, modular design |
| Infrastructure costs | Low | Low | Use free tiers, optimize early |

---

## Change Log

### 2025-01-05
- Initial roadmap created
- Milestones 0-5 defined
- Success criteria established
- Timeline estimates added

---

## Appendix A: Milestone Dependencies

```
M0 (Foundation)
  │
  ├─> M1 (Core MVP)
  │     │
  │     ├─> M2 (Forms & TUI)
  │     │     │
  │     │     ├─> M4 (Production)
  │     │     │
  │     │     └─> M5 (Extended)
  │     │
  │     └─> M3 (Advanced Features)
  │           │
  │           ├─> M4 (Production)
  │           │
  │           └─> M5 (Extended)
```

**Critical Path:** M0 → M1 → M2 → M4 → M5

**Parallel Work:** M3 can run in parallel with M2 after M1 completion

---

## Appendix B: Feature Checklist

### MVP Features (M1)
- [x] Workspace structure
- [ ] SQLite backend
- [ ] Basic SQL execution
- [ ] REPL interface
- [ ] Parameter binding
- [ ] Table output
- [ ] Batch mode

### v0.1.0 Features (M4)
- [ ] Forms system
- [ ] Terminal UI (NWM)
- [ ] Running aggregates
- [ ] PostgreSQL backend
- [ ] CSV/JSON output
- [ ] Daemon mode
- [ ] Security hardening

### v1.0.0 Features (M5)
- [ ] MySQL backend
- [ ] DuckDB backend
- [ ] Data visualization
- [ ] Language bindings
- [ ] Advanced UI features
- [ ] Production deployment

---

**Document Maintained By:** Noctra Development Team
**Review Schedule:** Weekly during active development
**Next Review:** 2025-01-12
