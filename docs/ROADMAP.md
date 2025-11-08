# Noctra Development Roadmap

> **Version:** 1.0
> **Last Updated:** 2025-01-05
> **Status:** Active Development

## Overview

This document outlines the development roadmap for Noctra, from initial setup through production-ready release. The roadmap is organized into milestones with clear deliverables, timelines, and success criteria.

**Current Status:** Milestone 0 (Setup & Foundation)

---

## Table of Contents

1. [Release Timeline](#release-timeline)
2. [Milestone 0: Foundation](#milestone-0-foundation)
3. [Milestone 1: Core MVP](#milestone-1-core-mvp)
4. [Milestone 2: Forms & TUI](#milestone-2-forms--tui)
5. [Milestone 3: Advanced Features](#milestone-3-advanced-features)
6. [Milestone 4: Production Ready](#milestone-4-production-ready)
7. [Milestone 5: Extended Capabilities](#milestone-5-extended-capabilities)
8. [Future Roadmap](#future-roadmap)
9. [Success Metrics](#success-metrics)

---

## Release Timeline

```
Milestone 0 ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ [████████████] 100%
           └─ Foundation & Setup (1 week)

Milestone 1 ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ [░░░░░░░░░░░░]   0%
           └─ Core MVP (3 weeks)                     ETA: 2025-02-15

Milestone 2 ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ [░░░░░░░░░░░░]   0%
           └─ Forms & TUI (3 weeks)                   ETA: 2025-03-08

Milestone 3 ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ [░░░░░░░░░░░░]   0%
           └─ Advanced Features (2 weeks)            ETA: 2025-03-22

Milestone 4 ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ [░░░░░░░░░░░░]   0%
           └─ Production Ready (3 weeks)             ETA: 2025-04-12

Milestone 5 ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ [░░░░░░░░░░░░]   0%
           └─ Extended Capabilities (ongoing)        ETA: 2025-06-01
```

**MVP Release:** End of Milestone 1 (February 2025)
**v0.1.0 Release:** End of Milestone 4 (April 2025)
**v1.0.0 Release:** End of Milestone 5 (June 2025)

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
**Status:** 🚧 In Progress
**Target Date:** 2025-02-15

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
**Status:** ⏸️ Not Started
**Target Date:** 2025-03-08

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

## Milestone 3: Advanced Features

**Duration:** 2 weeks
**Status:** ⏸️ Not Started
**Target Date:** 2025-03-22

### Objectives

Add advanced RQL features, multi-backend support, and enhanced functionality.

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

## Milestone 4: Production Ready

**Duration:** 3 weeks
**Status:** ⏸️ Not Started
**Target Date:** 2025-04-12

### Objectives

Production hardening, security, performance optimization, and comprehensive documentation.

### Week 1: Security & Validation

#### Tasks

- [ ] Implement SQL injection prevention
- [ ] Add input sanitization
- [ ] Implement query validation
- [ ] Add resource limits (row count, timeout)
- [ ] Create security audit
- [ ] Implement file path validation
- [ ] Add authentication for daemon mode

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

        // Check for dangerous patterns
        self.check_dangerous_keywords(query)?;

        Ok(())
    }
}
```

**Tests:**
- [ ] SQL injection prevention tests
- [ ] Path traversal prevention tests
- [ ] Resource limit tests
- [ ] Authentication tests

### Week 2: Performance Optimization

#### Tasks

- [ ] Implement query result caching
- [ ] Add prepared statement pooling
- [ ] Optimize table rendering
- [ ] Add lazy result loading
- [ ] Profile and optimize hot paths
- [ ] Implement connection pooling
- [ ] Add query plan caching

**Benchmarks:**
- [ ] Query execution benchmarks
- [ ] Parser benchmarks
- [ ] Rendering benchmarks
- [ ] End-to-end workflow benchmarks

**Performance Targets:**
- Query execution: <100ms for simple queries
- Parser: <1ms for typical queries
- Table rendering: <50ms for 100 rows
- Memory usage: <50MB baseline

### Week 3: Daemon & Documentation

#### Tasks

**noctra-srv (Daemon):**
- [ ] Implement HTTP server with axum
- [ ] Add JSON API endpoints
- [ ] Implement authentication
- [ ] Add request logging
- [ ] Create systemd service file
- [ ] Add health check endpoint

**Documentation:**
- [ ] Complete API reference
- [ ] Add migration guides
- [ ] Create tutorial series
- [ ] Document all RQL extensions
- [ ] Add troubleshooting guide
- [ ] Create deployment guide

**Code Example:**
```rust
async fn execute_query(
    State(executor): State<Arc<Mutex<Executor>>>,
    Json(request): Json<ExecRequest>
) -> Result<Json<ExecResponse>, ApiError> {
    let result = executor
        .lock()
        .await
        .execute_with_params(&request.sql, request.params)
        .await?;

    Ok(Json(ExecResponse {
        success: true,
        rows: result.rows,
        execution_time_ms: result.duration.as_millis(),
    }))
}
```

### Deliverables

- [ ] Security hardened codebase
- [ ] Performance optimized
- [ ] Daemon mode (noctrad)
- [ ] Complete documentation
- [ ] Migration tools
- [ ] Deployment guides
- [ ] v0.1.0 release

### Success Criteria

**Security:**
- No SQL injection vulnerabilities
- Input validation complete
- Security audit passed
- Resource limits enforced

**Performance:**
- All benchmarks meet targets
- Memory usage optimized
- Query caching functional
- Connection pooling working

**Documentation:**
- All features documented
- API reference complete
- Tutorials available
- Migration guides ready

**Release:**
- Binary builds for Linux, macOS, Windows
- Docker image available
- Package published to crates.io
- GitHub release created

---

## Milestone 5: Extended Capabilities

**Duration:** 6-8 weeks
**Status:** ⏸️ Not Started
**Target Date:** 2025-06-01

### Objectives

Extended database support, advanced features, and ecosystem integration.

### Phase 1: Additional Backends (2 weeks)

- [ ] MySQL/MariaDB backend
- [ ] DuckDB backend (analytics)
- [ ] Backend adapter documentation
- [ ] Cross-backend compatibility tests

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

- [ ] MySQL backend
- [ ] DuckDB backend
- [ ] Enhanced TUI features
- [ ] Data visualization
- [ ] Language bindings
- [ ] v1.0.0 release

### Success Criteria

- Multi-backend support verified
- Advanced UI features functional
- Language bindings tested
- Comprehensive integration tests
- Production deployment guide
- v1.0.0 release published

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

| Milestone | Target Duration | Buffer | Total |
|-----------|----------------|--------|-------|
| M0        | 1 week         | -      | 1 week |
| M1        | 3 weeks        | 1 week | 4 weeks |
| M2        | 3 weeks        | 1 week | 4 weeks |
| M3        | 2 weeks        | 1 week | 3 weeks |
| M4        | 3 weeks        | 1 week | 4 weeks |
| M5        | 6 weeks        | 2 weeks| 8 weeks |
| **Total** | **18 weeks**   | **6 weeks** | **24 weeks** |

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
