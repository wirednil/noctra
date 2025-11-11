# Noctra - Technical Design Document

> **Version:** 1.2
> **Date:** 2025-11-09
> **Status:** Active Development - M3.5 Complete, Planning M4 & M6 (Noctra 2.0)

## Table of Contents

1. [Executive Summary](#1-executive-summary)
2. [System Architecture](#2-system-architecture)
3. [Core Components](#3-core-components)
4. [RQL Language Specification](#4-rql-language-specification)
5. [FDL2 Form Definition Language](#5-fdl2-form-definition-language)
6. [Noctra Window Manager (NWM)](#6-noctra-window-manager-nwm)
7. [Database Backend Architecture](#7-database-backend-architecture)
8. [Security & Performance](#8-security--performance)
9. [Testing Strategy](#9-testing-strategy)
10. [Deployment & Operations](#10-deployment--operations)
11. [NQL - Noctra Query Language](#11-nql---noctra-query-language-m4)
12. [Noctra 2.0 FABRIC - DuckDB Integration](#12-noctra-20-fabric---duckdb-integration)

---

## 1. Executive Summary

### 1.1 Vision

**Noctra** is a modern interactive SQL environment and declarative forms framework built in Rust. It combines the productivity of 4GL environments with contemporary technology standards.

**Core Philosophy:**
- Text-first: Everything is text (queries, forms, configs)
- SQL-first: Direct SQL access without ORM overhead
- Declarative forms: TOML-based form definitions
- Terminal UI: Professional ncurses-based interface
- Batch-capable: Script automation support

### 1.2 Key Features

- **Interactive REPL** with command history and editing
- **Extended SQL (RQL)** with named/positional parameters
- **Declarative Forms (FDL2)** in TOML format
- **Terminal UI (NWM)** - Noctra Window Manager
- **Multiple backends** (SQLite, PostgreSQL, MySQL)
- **Batch mode** for automation
- **Optional daemon** (noctrad) for remote access

### 1.3 Target Users

- Database administrators
- Data analysts
- Application developers
- Business users (via forms)
- DevOps engineers (via batch scripts)

---

## 2. System Architecture

### 2.1 High-Level Architecture

```
┌────────────────────────────────────────────────┐
│              User Interface                    │
│  ┌──────────────┐      ┌──────────────┐       │
│  │  CLI / REPL  │      │  TUI (NWM)   │       │
│  └──────┬───────┘      └──────┬───────┘       │
│         │                     │                │
│         └──────────┬──────────┘                │
│                    │                           │
└────────────────────┼───────────────────────────┘
                     │
┌────────────────────▼───────────────────────────┐
│           Noctra Core Engine                   │
│  ┌──────────────┐      ┌──────────────┐       │
│  │  RQL Parser  │      │   Executor   │       │
│  └──────┬───────┘      └──────┬───────┘       │
│         │                     │                │
│         └──────────┬──────────┘                │
│                    │                           │
└────────────────────┼───────────────────────────┘
                     │
┌────────────────────▼───────────────────────────┐
│         Backend Layer                          │
│  ┌─────────┐  ┌─────────┐  ┌─────────┐        │
│  │ SQLite  │  │Postgres │  │  MySQL  │        │
│  └─────────┘  └─────────┘  └─────────┘        │
└────────────────────────────────────────────────┘
```

### 2.2 Workspace Structure

```
noctra/
├── Cargo.toml              # Workspace root
├── README.md
├── docs/
│   ├── DESIGN.md          # This document
│   ├── GETTING_STARTED.md
│   ├── FDL2-SPEC.md
│   └── RQL-EXTENSIONS.md
├── crates/
│   ├── noctra-core/       # Runtime engine
│   ├── noctra-parser/     # RQL parser
│   ├── noctra-cli/        # CLI/REPL
│   ├── noctra-tui/        # Terminal UI
│   ├── noctra-srv/        # Daemon server
│   ├── noctra-formlib/    # Forms library
│   └── noctra-ffi/        # C bindings
├── examples/
│   ├── forms/
│   ├── scripts/
│   └── sample.db
└── tests/
    ├── integration/
    └── fixtures/
```

### 2.3 Data Flow

**Query Execution Flow:**

```
User Input (SQL/RQL)
  │
  ├─> RQL Parser
  │     └─> Abstract Syntax Tree (AST)
  │
  ├─> Executor
  │     ├─> Parameter Binding
  │     ├─> Template Processing
  │     └─> Backend Selection
  │
  ├─> Database Backend
  │     ├─> Query Translation
  │     ├─> Execution
  │     └─> Result Set
  │
  └─> Formatter
        ├─> Table Rendering (TUI)
        ├─> CSV/JSON Export
        └─> User Display
```

---

## 3. Core Components

### 3.1 Noctra Core (`noctra-core`)

**Purpose:** Central runtime engine with execution logic, type system, and session management.

#### Type System

```rust
/// Core value type representing all SQL data types
pub enum Value {
    Null,
    Boolean(bool),
    Integer(i64),
    Float(f64),
    Text(String),
    Bytes(Vec<u8>),
    Date(NaiveDate),
    DateTime(NaiveDateTime),
    Decimal(Decimal),
}

impl Value {
    pub fn type_name(&self) -> &str;
    pub fn is_null(&self) -> bool;
    pub fn coerce_to(&self, target: ValueType) -> Result<Value>;
}
```

#### Executor

```rust
pub struct Executor {
    backend: Box<dyn DatabaseBackend>,
    session: Session,
    config: ExecutorConfig,
}

impl Executor {
    /// Execute a single RQL statement
    pub async fn execute(&mut self, stmt: &RqlStatement)
        -> Result<ExecutionResult>;

    /// Execute a batch of statements
    pub async fn execute_batch(&mut self, statements: Vec<RqlStatement>)
        -> Result<Vec<ExecutionResult>>;

    /// Execute with timeout
    pub async fn execute_with_timeout(
        &mut self,
        stmt: &RqlStatement,
        timeout: Duration
    ) -> Result<ExecutionResult>;
}

pub struct Session {
    variables: HashMap<String, Value>,
    current_schema: Option<String>,
    transaction_state: TransactionState,
    query_history: VecDeque<QueryHistoryEntry>,
}
```

#### Configuration

```rust
pub struct ExecutorConfig {
    pub max_rows: Option<usize>,
    pub query_timeout: Duration,
    pub auto_commit: bool,
    pub result_format: ResultFormat,
}

pub enum ResultFormat {
    Table,
    Csv { delimiter: char },
    Json { pretty: bool },
    Custom(Box<dyn Formatter>),
}
```

### 3.2 RQL Parser (`noctra-parser`)

**Purpose:** Parse extended SQL (RQL) with parameter support and custom commands.

#### Architecture

```rust
pub struct RqlParser {
    base_parser: SqlParser,  // sqlparser-rs wrapper
    extensions: ExtensionRegistry,
}

pub struct RqlAst {
    pub statements: Vec<RqlStatement>,
    pub parameters: Vec<Parameter>,
}

pub enum RqlStatement {
    // Standard SQL
    Select(SelectStatement),
    Insert(InsertStatement),
    Update(UpdateStatement),
    Delete(DeleteStatement),

    // DDL
    CreateTable(CreateTableStatement),
    DropTable(DropTableStatement),

    // RQL Extensions
    Use { schema: String },
    Let { name: String, value: Value },
    Show { target: ShowTarget },
    FormLoad { path: String },
    OutputTo { destination: OutputDestination },
}

pub enum Parameter {
    Positional(usize),           // $1, $2, ...
    Named(String),               // :name
}
```

#### Extension Points

```rust
pub trait ParserExtension {
    fn keyword(&self) -> &str;
    fn parse(&self, tokens: &mut TokenStream) -> Result<RqlStatement>;
}

// Example: USE command
pub struct UseExtension;
impl ParserExtension for UseExtension {
    fn keyword(&self) -> &str { "USE" }
    fn parse(&self, tokens: &mut TokenStream) -> Result<RqlStatement> {
        let schema = tokens.expect_identifier()?;
        Ok(RqlStatement::Use { schema })
    }
}
```

### 3.3 Form Library (`noctra-formlib`)

**Purpose:** Load, validate, and compile declarative forms.

#### Form Structure

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Form {
    pub title: String,
    pub schema: Option<String>,
    pub fields: HashMap<String, Field>,
    pub actions: HashMap<String, Action>,
    pub validations: Vec<ValidationRule>,
    pub views: HashMap<String, View>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Field {
    pub label: String,
    pub field_type: FieldType,
    pub required: bool,
    pub default: Option<Value>,
    pub width: Option<u32>,
    pub validations: Vec<FieldValidation>,
}

pub enum FieldType {
    Text { max_length: Option<usize> },
    Integer { min: Option<i64>, max: Option<i64> },
    Float { precision: u8 },
    Boolean,
    Date { format: String },
    Enum { options: Vec<String> },
    Password,
}
```

#### Form Compiler

```rust
pub struct FormCompiler {
    template_engine: TemplateEngine,
}

impl FormCompiler {
    /// Compile form to executable action
    pub fn compile(&self, form: &Form, action_name: &str)
        -> Result<CompiledAction>;

    /// Bind parameters from form values
    pub fn bind_parameters(
        &self,
        action: &CompiledAction,
        values: HashMap<String, Value>
    ) -> Result<BoundQuery>;
}

pub struct CompiledAction {
    sql_template: String,
    parameters: Vec<Parameter>,
    conditions: Vec<ConditionalBlock>,
}
```

### 3.4 Terminal UI (`noctra-tui`)

**Purpose:** Professional ncurses-based interface (Noctra Window Manager).

See [Section 6: Noctra Window Manager](#6-noctra-window-manager-nwm) for detailed specification.

### 3.5 CLI / REPL (`noctra-cli`)

**Purpose:** Interactive command-line interface.

```rust
pub struct Repl {
    executor: Arc<Mutex<Executor>>,
    editor: Editor<ReplHelper>,
    history: HistoryManager,
    config: ReplConfig,
}

impl Repl {
    pub async fn run(&mut self) -> Result<()> {
        loop {
            let line = self.editor.readline("noctra> ")?;

            if line.trim().is_empty() {
                continue;
            }

            self.history.add(&line);

            match self.handle_line(&line).await {
                Ok(_) => {},
                Err(e) => eprintln!("Error: {}", e),
            }
        }
    }

    async fn handle_line(&mut self, line: &str) -> Result<()> {
        // Meta-commands
        if line.starts_with('.') {
            return self.handle_meta_command(line).await;
        }

        // RQL execution
        let ast = self.parser.parse(line)?;
        let result = self.executor.lock().await.execute(&ast).await?;

        self.display_result(result)?;
        Ok(())
    }
}
```

---

## 4. RQL Language Specification

### 4.1 Core SQL Support

RQL is a superset of standard SQL. All valid SQL queries are valid RQL queries.

**Supported SQL Features:**
- SELECT with JOINs, GROUP BY, HAVING, ORDER BY
- INSERT, UPDATE, DELETE
- CREATE TABLE, DROP TABLE, ALTER TABLE
- CREATE INDEX, DROP INDEX
- Transactions (BEGIN, COMMIT, ROLLBACK)
- Subqueries and CTEs

### 4.2 Parameter Syntax

#### Positional Parameters

```sql
SELECT * FROM employees WHERE dept = $1 AND salary > $2;
```

Parameters are bound by position:
```rust
executor.execute_with_params(query, vec![
    Value::Text("SALES".into()),
    Value::Integer(50000),
])?;
```

#### Named Parameters

```sql
SELECT * FROM employees WHERE dept = :dept AND salary > :min_salary;
```

Parameters are bound by name:
```rust
let params = HashMap::from([
    ("dept".to_string(), Value::Text("SALES".into())),
    ("min_salary".to_string(), Value::Integer(50000)),
]);
executor.execute_with_params(query, params)?;
```

### 4.3 RQL Extensions

#### USE Command

Switch active schema/database:

```sql
USE payroll;
USE demo;
```

#### LET Command

Define session variables:

```sql
LET dept = 'SALES';
LET min_salary = 50000;
LET active = true;

SELECT * FROM employees WHERE dept = :dept AND salary > :min_salary;
```

#### SHOW Command

Display metadata:

```sql
SHOW TABLES;
SHOW COLUMNS FROM employees;
SHOW INDEXES FROM employees;
SHOW DATABASES;
```

#### OUTPUT Command

Redirect query results:

```sql
OUTPUT TO 'results.csv' FORMAT CSV;
SELECT * FROM employees;

OUTPUT TO 'report.json' FORMAT JSON;
SELECT * FROM sales WHERE year = 2023;

OUTPUT TO TERMINAL;  -- Reset to terminal output
```

#### FORM Commands

Load and execute forms:

```sql
FORM LOAD 'employees.toml';
FORM EXECUTE 'employees.toml' WITH dept = 'SALES';
```

### 4.4 Template Processing

RQL supports conditional SQL generation:

```sql
SELECT * FROM employees
WHERE 1=1
{{#if dept}} AND dept = :dept {{/if}}
{{#if min_salary}} AND salary >= :min_salary {{/if}}
{{#if active}} AND status = 'ACTIVE' {{/if}}
ORDER BY name;
```

**Template Syntax:**
- `{{#if var}} ... {{/if}}` - Conditional inclusion
- `{{#unless var}} ... {{/unless}}` - Negative conditional
- `{{var}}` - Variable interpolation

---

## 5. FDL2 Form Definition Language

### 5.1 Overview

FDL2 (Form Definition Language 2) is a TOML-based declarative language for defining data entry forms and their associated database operations.

### 5.2 Complete Example

```toml
title = "Employee Management"
schema = "payroll"
description = "Manage employee records"

# Field Definitions
[fields.employee_id]
label = "Employee ID"
field_type = "integer"
required = true
readonly = false
width = 10
validations = [
    { type = "min", value = 1 },
    { type = "max", value = 99999 }
]

[fields.name]
label = "Full Name"
field_type = "text"
required = true
width = 40
validations = [
    { type = "min_length", value = 3 },
    { type = "max_length", value = 100 },
    { type = "regex", pattern = "^[A-Za-z ]+$" }
]

[fields.department]
label = "Department"
field_type = "enum"
required = true
options = ["SALES", "MARKETING", "IT", "HR", "FINANCE"]
default = "IT"

[fields.salary]
label = "Annual Salary"
field_type = "float"
required = true
validations = [
    { type = "min", value = 0 },
    { type = "max", value = 999999.99 }
]
format = "currency"

[fields.hire_date]
label = "Hire Date"
field_type = "date"
required = true
format = "%Y-%m-%d"
default = "today"

[fields.active]
label = "Active"
field_type = "boolean"
default = true

# Actions
[actions.search]
type = "query"
sql = """
SELECT employee_id, name, department, salary, hire_date
FROM employees
WHERE 1=1
{{#if employee_id}} AND employee_id = :employee_id {{/if}}
{{#if name}} AND name LIKE '%' || :name || '%' {{/if}}
{{#if department}} AND department = :department {{/if}}
{{#if active}} AND active = :active {{/if}}
ORDER BY name;
"""
params = ["employee_id", "name", "department", "active"]
on_success = "display_results"
on_error = "show_error"

[actions.insert]
type = "insert"
table = "employees"
mapping = {
    employee_id = "employee_id",
    name = "name",
    department = "department",
    salary = "salary",
    hire_date = "hire_date",
    active = "active"
}
on_success = "show_confirmation"
on_error = "show_error"

[actions.update]
type = "update"
table = "employees"
where = "employee_id = :employee_id"
mapping = {
    name = "name",
    department = "department",
    salary = "salary",
    active = "active"
}
on_success = "show_confirmation"

# Views
[views.display_results]
type = "table"
title = "Search Results"
columns = ["employee_id", "name", "department", "salary", "hire_date"]
pager = true
max_rows = 100

[views.show_confirmation]
type = "message"
title = "Success"
message = "Operation completed successfully"

[views.show_error]
type = "message"
title = "Error"
message = "Error: {{error_message}}"
```

### 5.3 Field Types Reference

| Type | Description | Parameters |
|------|-------------|------------|
| `text` | String input | `max_length` |
| `integer` | Integer number | `min`, `max` |
| `float` | Decimal number | `min`, `max`, `precision` |
| `boolean` | True/false | - |
| `date` | Date value | `format` |
| `datetime` | Date and time | `format` |
| `enum` | Selection from list | `options` |
| `password` | Hidden text | `min_length` |

### 5.4 Validation Rules

```toml
validations = [
    { type = "required" },
    { type = "min", value = 0 },
    { type = "max", value = 100 },
    { type = "min_length", value = 3 },
    { type = "max_length", value = 50 },
    { type = "regex", pattern = "^[A-Za-z]+$" },
    { type = "email" },
    { type = "url" },
    { type = "range", min = 18, max = 65 },
]
```

### 5.5 Action Types

**Query Action:**
```toml
[actions.search]
type = "query"
sql = "SELECT * FROM table WHERE condition"
params = ["param1", "param2"]
```

**Insert Action:**
```toml
[actions.create]
type = "insert"
table = "employees"
mapping = { field1 = "col1", field2 = "col2" }
```

**Update Action:**
```toml
[actions.update]
type = "update"
table = "employees"
where = "id = :id"
mapping = { field1 = "col1" }
```

**Delete Action:**
```toml
[actions.delete]
type = "delete"
table = "employees"
where = "id = :id"
confirm = true
```

**Custom Action:**
```toml
[actions.custom]
type = "custom"
sql = """
UPDATE employees SET salary = salary * 1.05
WHERE department = :dept
"""
```

---

## 6. Noctra Window Manager (NWM)

### 6.1 Overview

The Noctra Window Manager (NWM) is a professional terminal-based user interface built on ncurses, providing a consistent and productive environment for database interaction.

### 6.2 Screen Layout

```
+--------------------------------------------------------------------------------+
|                                                                                |
|──( COMMAND ) Noctra 0.1.0 ────────────────────────────────────── Line: 1 ────|
|                                                                                |
|                                                                                |
|                    [Main Content Area - Dynamic]                               |
|                                                                                |
|                                                                                |
|────────────────────────────────────────────────────────────────────────────────|
| F5:Execute      F1:Help         F8:Cancel       End:Exit                      |
| PgUp:Prev Query PgDn:Next Query Insert:Mode     Delete:Clear                  |
| Alt+R:Load File Alt+W:Save File Ctrl+L:Redraw                                 |
|                                                                                |
+--------------------------------------------------------------------------------+
```

**Components:**
1. **Header Bar** - Shows mode, version, and line number
2. **Main Area** - Dynamic content (command input, results, forms, dialogs)
3. **Separator Line** - Visual boundary
4. **Footer Bar** - Key bindings reference (context-sensitive)

### 6.3 Operating Modes

#### Command Mode

Default mode for entering SQL/RQL queries.

```
┌────────────────────────────────────────────────────────┐
│                                                        │
│──( COMMAND ) Noctra 0.1.0 ──────────────── Line: 1 ───│
│                                                        │
│ SELECT * FROM employees WHERE dept = 'SALES'▊         │
│                                                        │
│                                                        │
│────────────────────────────────────────────────────────│
│ F5:Execute  F1:Help  PgUp:History  Alt+R:Load         │
└────────────────────────────────────────────────────────┘
```

#### Result Mode

Displays query results in tabular format.

```
┌────────────────────────────────────────────────────────┐
│                                                        │
│──( RESULT ) Noctra 0.1.0 ──────────────────────────────│
│                                                        │
│ ┌──────┬──────────────┬────────────┬──────────┐       │
│ │ ID   │ Name         │ Department │ Salary   │       │
│ ├──────┼──────────────┼────────────┼──────────┤       │
│ │ 1001 │ John Smith   │ SALES      │ 75000.00 │       │
│ │ 1002 │ Mary Johnson │ SALES      │ 68000.00 │       │
│ │ 1003 │ Bob Williams │ SALES      │ 82000.00 │       │
│ └──────┴──────────────┴────────────┴──────────┘       │
│ (3 rows, 15.2ms)                                       │
│────────────────────────────────────────────────────────│
│ PgUp:Scroll Up  PgDn:Scroll Down  Q:Quit Result       │
└────────────────────────────────────────────────────────┘
```

#### Form Mode

Renders interactive forms for data entry.

```
┌────────────────────────────────────────────────────────┐
│                                                        │
│──( FORM ) Employee Search ──────────────────────────────│
│                                                        │
│  Employee ID: [     ]                                  │
│  Name:        [                              ]         │
│  Department:  [SALES     ▼]                            │
│  Active:      [✓] Yes  [ ] No                          │
│                                                        │
│  [  Search  ]  [  Cancel  ]                            │
│────────────────────────────────────────────────────────│
│ Tab:Next Field  Shift+Tab:Prev  Enter:Submit          │
└────────────────────────────────────────────────────────┘
```

#### Dialog Mode

Modal dialogs for confirmations and messages.

```
┌────────────────────────────────────────────────────────┐
│                                                        │
│──( COMMAND ) Noctra 0.1.0 ──────────────────────────────│
│                                                        │
│         ┌──────────────────────────────────┐           │
│         │        Exit Noctra?              │           │
│         │                                  │           │
│         │  [ Yes ]  [ No ]  [ Cancel ]    │           │
│         └──────────────────────────────────┘           │
│                                                        │
│────────────────────────────────────────────────────────│
│ Tab:Select  Enter:Confirm  Esc:Cancel                 │
└────────────────────────────────────────────────────────┘
```

### 6.4 Key Bindings

| Key | Action | Context |
|-----|--------|---------|
| **F5** | Execute query | Command mode |
| **F1** | Show help | All modes |
| **F8** | Cancel/Interrupt | Execution |
| **End** | Exit application | All modes |
| **PgUp** | Previous query/scroll up | Command/Result |
| **PgDn** | Next query/scroll down | Command/Result |
| **Insert** | Toggle insert mode | Command |
| **Delete** | Delete character | Command |
| **Alt+R** | Load file | Command |
| **Alt+W** | Save to file | Command |
| **Ctrl+L** | Redraw screen | All modes |
| **Tab** | Next field/option | Form/Dialog |
| **Shift+Tab** | Previous field | Form/Dialog |
| **Enter** | Confirm/Submit | Form/Dialog |
| **Esc** | Cancel | Form/Dialog |

### 6.5 Color Scheme

**Classic Theme (Default):**
- Background: Black (#000000)
- Text: Phosphor Green (#00FF00)
- Highlight: Cyan (#00FFFF)
- Warning: Yellow (#FFFF00)
- Error: Red (#FF0000)
- Border: Dim White (#AAAAAA)

**Modern Theme:**
- Background: Dark Gray (#1E1E1E)
- Text: Light Gray (#D4D4D4)
- Highlight: Blue (#569CD6)
- Warning: Orange (#CE9178)
- Error: Red (#F44747)
- Border: Medium Gray (#808080)

### 6.6 Table Rendering

**ASCII Box Drawing:**
```
┌───────┬──────────────┬────────────┬──────────┐
│ ID    │ Name         │ Department │ Salary   │
├───────┼──────────────┼────────────┼──────────┤
│ 1001  │ John Smith   │ SALES      │ 75000.00 │
│ 1002  │ Mary Johnson │ SALES      │ 68000.00 │
└───────┴──────────────┴────────────┴──────────┘
```

**Features:**
- Automatic column width calculation
- Header alignment
- Number right-alignment
- Text left-alignment
- Pagination for large result sets
- Scrolling support

---

## 7. Database Backend Architecture

### 7.1 Backend Trait

All database backends implement the `DatabaseBackend` trait:

```rust
#[async_trait]
pub trait DatabaseBackend: Send + Sync {
    /// Connect to database
    async fn connect(&self, config: &ConnectionConfig)
        -> Result<Box<dyn Connection>>;

    /// Query capabilities
    fn features(&self) -> BackendFeatures;

    /// Backend name
    fn name(&self) -> &str;
}

#[async_trait]
pub trait Connection: Send + Sync {
    /// Execute query and return results
    async fn execute(&mut self, query: &str, params: &[Value])
        -> Result<ResultSet>;

    /// Prepare statement for reuse
    async fn prepare(&mut self, query: &str)
        -> Result<Box<dyn PreparedStatement>>;

    /// Start transaction
    async fn begin_transaction(&mut self) -> Result<()>;

    /// Commit transaction
    async fn commit(&mut self) -> Result<()>;

    /// Rollback transaction
    async fn rollback(&mut self) -> Result<()>;

    /// Get table metadata
    async fn table_info(&mut self, table: &str)
        -> Result<TableMetadata>;
}

pub struct BackendFeatures {
    pub supports_transactions: bool,
    pub supports_savepoints: bool,
    pub supports_returning: bool,
    pub supports_cte: bool,
    pub supports_window_functions: bool,
    pub max_parameter_count: Option<usize>,
}
```

### 7.2 SQLite Backend

```rust
pub struct SqliteBackend {
    pool: Pool<Sqlite>,
}

impl SqliteBackend {
    pub async fn new(path: &str) -> Result<Self> {
        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect(path)
            .await?;

        Ok(Self { pool })
    }
}

#[async_trait]
impl DatabaseBackend for SqliteBackend {
    async fn connect(&self, _config: &ConnectionConfig)
        -> Result<Box<dyn Connection>> {
        let conn = self.pool.acquire().await?;
        Ok(Box::new(SqliteConnection::new(conn)))
    }

    fn features(&self) -> BackendFeatures {
        BackendFeatures {
            supports_transactions: true,
            supports_savepoints: true,
            supports_returning: true,
            supports_cte: true,
            supports_window_functions: true,
            max_parameter_count: Some(32766),
        }
    }

    fn name(&self) -> &str { "sqlite" }
}
```

### 7.3 PostgreSQL Backend

```rust
pub struct PostgresBackend {
    pool: Pool<Postgres>,
}

impl PostgresBackend {
    pub async fn new(connection_string: &str) -> Result<Self> {
        let pool = PgPoolOptions::new()
            .max_connections(10)
            .connect(connection_string)
            .await?;

        Ok(Self { pool })
    }
}

#[async_trait]
impl DatabaseBackend for PostgresBackend {
    async fn connect(&self, _config: &ConnectionConfig)
        -> Result<Box<dyn Connection>> {
        let conn = self.pool.acquire().await?;
        Ok(Box::new(PostgresConnection::new(conn)))
    }

    fn features(&self) -> BackendFeatures {
        BackendFeatures {
            supports_transactions: true,
            supports_savepoints: true,
            supports_returning: true,
            supports_cte: true,
            supports_window_functions: true,
            max_parameter_count: Some(65535),
        }
    }

    fn name(&self) -> &str { "postgresql" }
}
```

### 7.4 Connection Pooling

```rust
pub struct ConnectionPool {
    backend: Arc<dyn DatabaseBackend>,
    active: Arc<Mutex<Vec<Box<dyn Connection>>>>,
    max_size: usize,
}

impl ConnectionPool {
    pub async fn acquire(&self) -> Result<PooledConnection> {
        // Try to get existing connection
        if let Some(conn) = self.active.lock().await.pop() {
            return Ok(PooledConnection::new(conn, self.clone()));
        }

        // Create new connection if under limit
        if self.active.lock().await.len() < self.max_size {
            let conn = self.backend.connect(&ConnectionConfig::default()).await?;
            return Ok(PooledConnection::new(conn, self.clone()));
        }

        // Wait for available connection
        // ... (implementation)
    }
}
```

---

## 8. Security & Performance

### 8.1 Security Measures

#### SQL Injection Prevention

**Always use parameterized queries:**

```rust
// ❌ UNSAFE - Direct string concatenation
let query = format!("SELECT * FROM users WHERE name = '{}'", user_input);

// ✅ SAFE - Parameterized query
let query = "SELECT * FROM users WHERE name = $1";
executor.execute_with_params(query, vec![Value::Text(user_input)])?;
```

**Input validation:**

```rust
pub struct InputValidator {
    allowed_tables: HashSet<String>,
    max_query_length: usize,
}

impl InputValidator {
    pub fn validate_table_name(&self, name: &str) -> Result<()> {
        if !self.allowed_tables.contains(name) {
            return Err(Error::UnauthorizedTable(name.to_string()));
        }
        Ok(())
    }

    pub fn validate_query(&self, query: &str) -> Result<()> {
        if query.len() > self.max_query_length {
            return Err(Error::QueryTooLong);
        }
        Ok(())
    }
}
```

#### Resource Limits

```rust
pub struct ResourceLimits {
    pub max_rows: usize,
    pub query_timeout: Duration,
    pub max_memory: usize,
}

impl Executor {
    pub async fn execute_with_limits(
        &mut self,
        query: &str,
        limits: &ResourceLimits
    ) -> Result<ResultSet> {
        // Set timeout
        let result = timeout(limits.query_timeout, async {
            let mut result = self.backend.execute(query, &[]).await?;

            // Limit rows
            if result.rows.len() > limits.max_rows {
                result.rows.truncate(limits.max_rows);
                result.truncated = true;
            }

            Ok(result)
        }).await??;

        Ok(result)
    }
}
```

#### File Operations Security

```rust
pub struct FileValidator {
    allowed_paths: Vec<PathBuf>,
}

impl FileValidator {
    pub fn validate_path(&self, path: &Path) -> Result<()> {
        let canonical = path.canonicalize()?;

        // Check if path is under allowed directories
        for allowed in &self.allowed_paths {
            if canonical.starts_with(allowed) {
                return Ok(());
            }
        }

        Err(Error::UnauthorizedPath(path.to_path_buf()))
    }
}
```

### 8.2 Performance Optimization

#### Query Caching

```rust
pub struct QueryCache {
    cache: Arc<Mutex<LruCache<String, ResultSet>>>,
    ttl: Duration,
}

impl QueryCache {
    pub async fn get_or_execute<F, Fut>(
        &self,
        query: &str,
        executor: F
    ) -> Result<ResultSet>
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output = Result<ResultSet>>,
    {
        // Check cache
        if let Some(cached) = self.cache.lock().await.get(query) {
            if !cached.is_expired(self.ttl) {
                return Ok(cached.clone());
            }
        }

        // Execute and cache
        let result = executor().await?;
        self.cache.lock().await.put(query.to_string(), result.clone());

        Ok(result)
    }
}
```

#### Prepared Statement Pool

```rust
pub struct PreparedStatementPool {
    statements: Arc<Mutex<HashMap<String, PreparedStatement>>>,
    max_size: usize,
}

impl PreparedStatementPool {
    pub async fn get_or_prepare(
        &self,
        query: &str,
        conn: &mut dyn Connection
    ) -> Result<PreparedStatement> {
        let mut statements = self.statements.lock().await;

        if let Some(stmt) = statements.get(query) {
            return Ok(stmt.clone());
        }

        // Evict oldest if at capacity
        if statements.len() >= self.max_size {
            // LRU eviction logic
        }

        let stmt = conn.prepare(query).await?;
        statements.insert(query.to_string(), stmt.clone());

        Ok(stmt)
    }
}
```

#### Memory Management

```rust
pub struct ResultSetStreamer {
    chunk_size: usize,
}

impl ResultSetStreamer {
    pub async fn stream_results(
        &self,
        query: &str,
        conn: &mut dyn Connection
    ) -> impl Stream<Item = Result<Vec<Row>>> {
        async_stream::try_stream! {
            let mut cursor = conn.execute_cursor(query).await?;

            loop {
                let chunk = cursor.fetch_many(self.chunk_size).await?;
                if chunk.is_empty() {
                    break;
                }
                yield chunk;
            }
        }
    }
}
```

---

## 9. Testing Strategy

### 9.1 Test Pyramid

```
        /\
       /  \      E2E Tests (5%)
      /    \     - Full workflows
     /------\    - CLI integration
    /        \   - Cross-platform
   /----------\
  / Integration \ Integration Tests (25%)
 /    Tests      \ - Backend testing
/________________\ - REPL simulation
    Unit Tests      Unit Tests (70%)
                    - Parser
                    - Core logic
                    - Form validation
```

### 9.2 Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parameter_extraction() {
        let query = "SELECT * FROM users WHERE id = $1 AND name = :name";
        let params = extract_parameters(query);

        assert_eq!(params.len(), 2);
        assert!(matches!(params[0], Parameter::Positional(1)));
        assert!(matches!(params[1], Parameter::Named(ref n) if n == "name"));
    }

    #[test]
    fn test_value_coercion() {
        let value = Value::Integer(42);
        let coerced = value.coerce_to(ValueType::Float).unwrap();

        assert!(matches!(coerced, Value::Float(f) if f == 42.0));
    }

    #[tokio::test]
    async fn test_query_execution() {
        let mut executor = Executor::new_with_memory_backend();

        let result = executor.execute("SELECT 1 as num").await.unwrap();

        assert_eq!(result.rows.len(), 1);
        assert_eq!(result.rows[0].get("num"), Some(&Value::Integer(1)));
    }
}
```

### 9.3 Integration Tests

```rust
#[tokio::test]
async fn test_full_workflow() {
    // Setup test database
    let db_path = tempfile::NamedTempFile::new().unwrap();
    let backend = SqliteBackend::new(db_path.path()).await.unwrap();
    let mut executor = Executor::new(backend);

    // Create table
    executor.execute("
        CREATE TABLE users (
            id INTEGER PRIMARY KEY,
            name TEXT NOT NULL,
            email TEXT UNIQUE
        )
    ").await.unwrap();

    // Insert data
    executor.execute_with_params(
        "INSERT INTO users (id, name, email) VALUES ($1, $2, $3)",
        vec![
            Value::Integer(1),
            Value::Text("Alice".into()),
            Value::Text("alice@example.com".into()),
        ]
    ).await.unwrap();

    // Query data
    let result = executor.execute(
        "SELECT * FROM users WHERE name = 'Alice'"
    ).await.unwrap();

    assert_eq!(result.rows.len(), 1);
    assert_eq!(result.rows[0].get("email"),
               Some(&Value::Text("alice@example.com".into())));
}
```

### 9.4 Property-Based Testing

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_value_roundtrip(value in any::<Value>()) {
        let serialized = value.to_sql();
        let deserialized = Value::from_sql(&serialized)?;
        prop_assert_eq!(value, deserialized);
    }

    #[test]
    fn test_parameter_parsing(
        query in "[A-Z ]+ WHERE [a-z]+ = (\\$[0-9]+|:[a-z]+)"
    ) {
        let params = extract_parameters(&query);
        prop_assert!(!params.is_empty());
    }
}
```

### 9.5 Benchmark Tests

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn benchmark_query_parsing(c: &mut Criterion) {
    let query = "SELECT * FROM users WHERE id = $1 AND name = :name";

    c.bench_function("parse_query", |b| {
        b.iter(|| {
            let parser = RqlParser::new();
            parser.parse(black_box(query))
        });
    });
}

fn benchmark_result_formatting(c: &mut Criterion) {
    let result = create_sample_result_set(1000);

    c.bench_function("format_table", |b| {
        b.iter(|| {
            format_as_table(black_box(&result))
        });
    });
}

criterion_group!(benches, benchmark_query_parsing, benchmark_result_formatting);
criterion_main!(benches);
```

---

## 10. Deployment & Operations

### 10.1 Installation

**From Source:**
```bash
git clone https://github.com/noctra/noctra.git
cd noctra
cargo build --release
cargo install --path crates/noctra-cli
```

**Pre-built Binaries:**
```bash
# Linux
curl -LO https://github.com/noctra/noctra/releases/latest/download/noctra-linux-x64.tar.gz
tar xzf noctra-linux-x64.tar.gz
sudo mv noctra /usr/local/bin/

# macOS
brew install noctra

# Windows
choco install noctra
```

### 10.2 Configuration

**Config File Location:**
- Linux/macOS: `~/.config/noctra/config.toml`
- Windows: `%APPDATA%\Noctra\config.toml`

**Example Configuration:**

```toml
[general]
theme = "classic"          # classic, modern, custom
log_level = "info"         # error, warn, info, debug, trace
history_file = "~/.noctra_history"
history_size = 10000

[database]
default_backend = "sqlite"
connection_timeout = 30    # seconds
query_timeout = 300        # seconds
max_rows = 10000

[database.sqlite]
default_path = "./noctra.db"
wal_mode = true

[database.postgresql]
host = "localhost"
port = 5432
# username and password via env vars

[ui]
color_scheme = "classic"
table_style = "unicode"    # unicode, ascii
pager_enabled = true
confirm_exit = true

[security]
allowed_output_dirs = ["/tmp", "~/noctra-output"]
max_query_length = 1000000
```

### 10.3 Environment Variables

```bash
# Database connection
export NOCTRA_DB_TYPE=postgresql
export NOCTRA_DB_HOST=localhost
export NOCTRA_DB_PORT=5432
export NOCTRA_DB_NAME=mydb
export NOCTRA_DB_USER=admin
export NOCTRA_DB_PASSWORD=secret

# Behavior
export NOCTRA_THEME=classic
export NOCTRA_LOG_LEVEL=debug
export NOCTRA_CONFIG_FILE=/etc/noctra/config.toml

# Security
export NOCTRA_MAX_ROWS=5000
export NOCTRA_QUERY_TIMEOUT=60
```

### 10.4 Daemon Mode (noctrad)

**Start Daemon:**
```bash
noctrad \
    --bind 0.0.0.0:7100 \
    --db sqlite:///var/lib/noctra/data.db \
    --auth-token-file /etc/noctra/token \
    --log-file /var/log/noctrad.log \
    --daemonize
```

**Systemd Service:**
```ini
[Unit]
Description=Noctra Database Daemon
After=network.target

[Service]
Type=simple
User=noctra
Group=noctra
ExecStart=/usr/local/bin/noctrad --config /etc/noctra/noctrad.conf
Restart=on-failure
RestartSec=10s

[Install]
WantedBy=multi-user.target
```

**API Usage:**
```bash
# Execute query
curl -X POST http://localhost:7100/api/execute \
    -H "Authorization: Bearer ${TOKEN}" \
    -H "Content-Type: application/json" \
    -d '{
      "sql": "SELECT * FROM users WHERE id = $1",
      "params": [42]
    }'

# Load form
curl -X POST http://localhost:7100/api/form/load \
    -H "Authorization: Bearer ${TOKEN}" \
    -H "Content-Type: application/json" \
    -d '{
      "form_path": "forms/users.toml",
      "values": {"dept": "SALES"}
    }'
```

### 10.5 Monitoring & Observability

**Metrics Export:**
```rust
pub struct Metrics {
    queries_executed: Counter,
    query_duration: Histogram,
    active_connections: Gauge,
    errors_total: Counter,
}

impl Executor {
    pub async fn execute_with_metrics(
        &mut self,
        query: &str
    ) -> Result<ResultSet> {
        let start = Instant::now();
        self.metrics.queries_executed.inc();

        let result = self.execute(query).await;

        self.metrics.query_duration.observe(start.elapsed().as_secs_f64());

        if result.is_err() {
            self.metrics.errors_total.inc();
        }

        result
    }
}
```

**Health Check Endpoint:**
```rust
async fn health_check(State(executor): State<Arc<Executor>>) -> impl IntoResponse {
    let health = Health {
        status: "healthy",
        database: executor.check_connection().await.is_ok(),
        uptime: get_uptime(),
        version: env!("CARGO_PKG_VERSION"),
    };

    Json(health)
}
```

### 10.6 Backup & Recovery

**Automated Backup:**
```bash
#!/bin/bash
# backup-noctra.sh

BACKUP_DIR="/var/backups/noctra"
DATE=$(date +%Y%m%d_%H%M%S)

# Backup SQLite database
sqlite3 /var/lib/noctra/data.db ".backup $BACKUP_DIR/noctra_$DATE.db"

# Compress old backups
find $BACKUP_DIR -name "*.db" -mtime +7 -exec gzip {} \;

# Remove very old backups
find $BACKUP_DIR -name "*.db.gz" -mtime +30 -delete
```

**Restore:**
```bash
# Stop daemon
sudo systemctl stop noctrad

# Restore database
cp /var/backups/noctra/noctra_20250105_120000.db /var/lib/noctra/data.db

# Start daemon
sudo systemctl start noctrad
```

---

## 11. NQL - Noctra Query Language (M4)

### 11.1 Overview

**NQL (Noctra Query Language)** extends RQL to support **multi-source data operations**. With NQL, users can query CSV files, SQLite databases, JSON files, and in-memory datasets using the same unified SQL-like syntax.

**Key Features:**
- Unified query interface across different data sources
- Automatic CSV delimiter detection and type inference
- Declarative data transformations (MAP, FILTER)
- Import/Export between formats (CSV ↔ SQLite ↔ JSON)
- Multiple active sources with aliasing

### 11.2 DataSource Architecture

```rust
/// Core trait for all data sources
pub trait DataSource: Send + Sync + Debug {
    /// Execute a query against the data source
    fn query(&self, sql: &str, parameters: &Parameters) -> Result<ResultSet>;

    /// Get schema information (tables/columns)
    fn schema(&self) -> Result<Vec<TableInfo>>;

    /// Get the type of this data source
    fn source_type(&self) -> SourceType;

    /// Get the name/identifier of this source
    fn name(&self) -> &str;
}

/// Type of data source
pub enum SourceType {
    SQLite { path: String },
    CSV { path: String, delimiter: char, has_header: bool, encoding: String },
    JSON { path: String },
    Memory { capacity: usize },
}

/// Registry of named data sources
pub struct SourceRegistry {
    sources: HashMap<String, Box<dyn DataSource>>,
    active_source: Option<String>,
}
```

### 11.3 CSV Backend Implementation

**Features:**
- **Auto-detection**: Analyzes first 5 rows to detect delimiter (`,` `;` `\t` `|`)
- **Type Inference**: Samples up to 100 rows to infer column types (BOOLEAN, INTEGER, REAL, TEXT)
- **Quote Handling**: Proper handling of quoted fields
- **Encoding**: Support for different encodings (UTF-8, Latin1, etc.)

```rust
pub struct CsvDataSource {
    path: PathBuf,
    name: String,
    options: CsvOptions,
    schema: Vec<ColumnInfo>,
    data: Vec<Vec<Value>>,
}

impl CsvDataSource {
    pub fn new<P: AsRef<Path>>(
        path: P,
        name: String,
        options: CsvOptions
    ) -> Result<Self>;

    fn detect_delimiter(path: &Path) -> Result<char>;
    fn infer_column_type(data: &[Vec<String>], col_idx: usize) -> String;
}
```

### 11.4 NQL Commands

#### Source Management

```sql
-- Load CSV file
USE 'clientes.csv' AS csv;

-- Load SQLite database
USE 'demo.db' AS demo OPTIONS (mode=readonly);

-- List all sources
SHOW SOURCES;

-- Show tables from specific source
SHOW TABLES FROM csv;

-- Describe table structure
DESCRIBE csv.clientes;
```

#### Data Import/Export

```sql
-- Import CSV to SQLite
USE 'demo.db';
IMPORT 'datos.csv' AS staging;
INSERT INTO clientes SELECT * FROM staging;

-- Export query results to CSV
EXPORT (SELECT * FROM empleados WHERE activo = 1)
TO 'export.csv'
FORMAT CSV
OPTIONS (delimiter=';', header=true);

-- Export to JSON
EXPORT empleados TO 'data.json' FORMAT JSON;
```

#### Transformations

```sql
-- Use MAP for column transformations
USE 'data.csv';
MAP UPPER(nombre) AS nombre_upper,
    CONCAT(apellido, ', ', nombre) AS nombre_completo;
SELECT * FROM data;

-- Use FILTER for row filtering
USE 'clientes.csv';
FILTER pais IN ('AR', 'UY', 'CL');
SELECT * FROM clientes;
```

#### Session Variables

```sql
-- Define variables
LET min_age = 18;
LET country = 'AR';

-- Use in queries
SELECT * FROM clientes
WHERE edad >= :min_age AND pais = :country;

-- Show all variables
SHOW VARS;

-- Remove variable
UNSET min_age;
```

### 11.5 NQL AST Extensions

New statement types added to `RqlStatement`:

```rust
pub enum RqlStatement {
    // ... existing SQL statements ...

    // NQL Extensions
    UseSource {
        path: String,
        alias: Option<String>,
        options: HashMap<String, String>,
    },

    ShowSources,

    ShowTables {
        source: Option<String>,
    },

    ShowVars,

    Describe {
        source: Option<String>,
        table: String,
    },

    Import {
        file: String,
        table: String,
        options: HashMap<String, String>,
    },

    Export {
        query: String,
        file: String,
        format: ExportFormat,
        options: HashMap<String, String>,
    },

    Map {
        expressions: Vec<MapExpression>,
    },

    Filter {
        condition: String,
    },

    Unset {
        variables: Vec<String>,
    },
}

pub enum ExportFormat {
    Csv,
    Json,
    Xlsx,
}
```

### 11.6 Usage Examples

**Example 1: CSV Analysis**
```sql
-- Load CSV
USE 'sales_2024.csv' AS sales;

-- Inspect structure
DESCRIBE sales.sales_2024;

-- Query with aggregation
SELECT
    product,
    SUM(amount) as total_sales,
    COUNT(*) as transactions
FROM sales
GROUP BY product
ORDER BY total_sales DESC;

-- Export results
EXPORT sales TO 'summary.json' FORMAT JSON;
```

**Example 2: Data Migration**
```sql
-- Load legacy CSV
USE 'legacy_data.csv' AS legacy;

-- Load target database
USE 'new_system.db' AS target;

-- Migrate data
IMPORT 'legacy_data.csv' AS staging;
INSERT INTO target.customers
SELECT
    id,
    UPPER(name) as name,
    email,
    CURRENT_DATE as migrated_at
FROM staging
WHERE active = true;
```

**Example 3: Multi-Source JOIN**
```sql
-- Load both sources
USE 'customers.csv' AS csv_customers;
USE 'orders.db' AS db_orders;

-- Join across sources (requires staging)
IMPORT 'customers.csv' AS customers_staging;

SELECT
    c.customer_name,
    o.order_id,
    o.total
FROM customers_staging c
JOIN db_orders.orders o ON c.customer_id = o.customer_id
WHERE o.order_date >= '2024-01-01';
```

### 11.7 Implementation Status

**✅ Completed (M4 - Week 1-2):**
- DataSource trait and architecture
- SourceRegistry for managing multiple sources
- CSV backend with auto-detection
- Type inference system
- AST extensions for all NQL commands
- Test coverage for core functionality

**📋 Pending (M4 - Week 3-6):**
- NQL parser implementation
- Executor integration
- TUI contextual features (show active source)
- JSON backend
- Memory backend
- Advanced CSV options (encoding detection)

**📚 Documentation:**
- Complete NQL specification: [docs/NQL-SPEC.md](NQL-SPEC.md)
- Project status with M4 details: [docs/PROJECT_STATUS.md](PROJECT_STATUS.md)

---

## 12. Noctra 2.0 FABRIC - DuckDB Integration

### 12.1 Vision & Objectives

**Mantra:** *"No importes datos. Consúltalos."*

Noctra 2.0 "FABRIC" transforms Noctra into a **Data Fabric Engine** by integrating DuckDB as the primary ad hoc analytics engine, enabling direct queries on files (CSV, JSON, Parquet) without staging or mandatory databases.

**Core Innovation:**
- **Files → Tables**: Any file becomes a queryable table instantly
- **Zero-Copy Analytics**: Query files directly without imports
- **Optional Databases**: Databases become optional, not required

### 12.2 Architecture Overview

#### Query Engine Evolution

```rust
// noctra-core/src/engine.rs
pub enum QueryEngine {
    Sqlite(Box<dyn DatabaseBackend>),
    DuckDB(DuckDBConnection),        // ← NEW: Pure DuckDB mode
    Hybrid {                          // ← NEW: Default mode
        duckdb: DuckDBConnection,     // Handles files (CSV, JSON, Parquet)
        sqlite: SqliteConnection,     // Handles SQLite databases
    },
}

impl QueryEngine {
    /// Route queries to appropriate engine based on source type
    pub async fn execute(&mut self, nql: &NqlStatement) -> Result<ResultSet> {
        match self {
            Self::DuckDB(conn) => conn.execute_nql(nql).await,
            Self::Hybrid { duckdb, sqlite } => {
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

    /// Create hybrid engine (default)
    pub fn new_hybrid() -> Result<Self> {
        Ok(Self::Hybrid {
            duckdb: DuckDBConnection::new_in_memory()?,
            sqlite: SqliteConnection::new_in_memory()?,
        })
    }
}
```

#### Routing Logic

| Source Type | Engine | Implementation |
|-------------|--------|----------------|
| CSV files | DuckDB | `read_csv_auto('file.csv')` |
| JSON files | DuckDB | `read_json_auto('file.json')` |
| Parquet files | DuckDB | `read_parquet('file.parquet')` |
| SQLite databases | SQLite | Native connection |
| Cross-source JOINs | DuckDB | `ATTACH 'db.sqlite' AS alias (TYPE SQLITE)` |

### 12.3 New Crate: `noctra-duckdb`

#### Structure

```
noctra/
├── crates/
│   ├── noctra-core/
│   │   └── src/
│   │       ├── engine.rs           # + QueryEngine::DuckDB, Hybrid
│   │       └── datasource.rs       # (existing)
│   ├── noctra-parser/
│   │   └── src/
│   │       └── nql.rs              # + EXPORT, MAP, FILTER variants
│   ├── noctra-duckdb/              # ← NEW CRATE
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs              # Public API
│   │       ├── source.rs           # DuckDBSource (DataSource impl)
│   │       ├── engine.rs           # Query execution logic
│   │       ├── extensions.rs       # Format support (JSON, Parquet)
│   │       └── attach.rs           # SQLite ATTACH logic
│   ├── noctra-tui/
│   │   └── src/
│   │       └── noctra_tui.rs       # + Engine indicator, source icons
│   └── noctra-cli/
│       └── src/
│           └── main.rs             # + --engine flag
```

#### Dependencies

```toml
# crates/noctra-duckdb/Cargo.toml
[package]
name = "noctra-duckdb"
version = "2.0.0"
edition = "2021"

[dependencies]
duckdb = { version = "1.1", features = ["bundled", "parquet", "json"] }
noctra-core = { path = "../noctra-core" }
anyhow = "1.0"
log = "0.4"

[dev-dependencies]
tempfile = "3.0"
```

#### Core Implementation

```rust
// noctra-duckdb/src/source.rs
use duckdb::{Connection, params};
use noctra_core::{DataSource, ResultSet, Parameters, Value, SourceType, TableInfo};

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
            _ => return Err(anyhow!("Unsupported file type: {}", ext)),
        };

        self.conn.execute(&sql, [])?;
        log::info!("Registered {} as virtual table '{}'", path, alias);
        Ok(())
    }

    /// Attach SQLite database for cross-source queries
    pub fn attach_sqlite(&mut self, db_path: &str, alias: &str) -> Result<()> {
        self.conn.execute(
            &format!("ATTACH '{}' AS {} (TYPE SQLITE)", db_path, alias),
            [],
        )?;
        log::info!("Attached SQLite database {} as '{}'", db_path, alias);
        Ok(())
    }
}

impl DataSource for DuckDBSource {
    fn query(&self, sql: &str, params: &Parameters) -> Result<ResultSet> {
        let mut stmt = self.conn.prepare(sql)?;

        // Convert Parameters to DuckDB format
        let duckdb_params = params.to_duckdb_params();

        let rows = stmt.query_map(&duckdb_params[..], |row| {
            // Convert DuckDB row to noctra ResultSet
            convert_row(row)
        })?;

        Ok(ResultSet::from_rows(rows))
    }

    fn schema(&self) -> Result<Vec<TableInfo>> {
        let sql = "SELECT table_name FROM information_schema.tables WHERE table_schema = 'main'";
        // ... implementation
    }

    fn name(&self) -> &str { &self.name }

    fn source_type(&self) -> &SourceType { &SourceType::DuckDB }
}
```

### 12.4 NQL 2.0 Extensions

#### Command Mappings

| NQL Command | DuckDB SQL | Purpose |
|-------------|------------|---------|
| `USE 'file.csv' AS t` | `CREATE VIEW t AS SELECT * FROM read_csv_auto('file.csv')` | Register file as table |
| `USE 'data/*.csv'` | `SELECT * FROM read_csv('data/sales_*.csv', AUTO_DETECT=TRUE)` | Multi-file glob |
| `USE 'data.csv.gz'` | Automatic decompression | Compressed file support |
| `DESCRIBE t` | `PRAGMA table_info(t)` | Show schema |
| `EXPORT ... TO 'f.json'` | `COPY (...) TO 'f.json' (FORMAT JSON)` | Export to JSON |
| `EXPORT ... TO 'f.parquet'` | `COPY (...) TO 'f.parquet' (FORMAT PARQUET)` | Export to Parquet |

#### EXPORT Command

```rust
// noctra-parser/src/nql.rs
pub enum NqlStatement {
    // ... existing variants

    Export {
        query: Box<NqlStatement>,  // Query to export
        path: String,               // Destination file
        format: ExportFormat,       // CSV, JSON, Parquet, Excel
        options: HashMap<String, String>,
    },
}

pub enum ExportFormat {
    Csv,
    Json,
    Parquet,
    Excel,
}
```

**Usage:**
```sql
EXPORT (SELECT * FROM 'sales.csv' WHERE region = 'LATAM')
TO 'latam_sales.parquet' FORMAT PARQUET;
```

**Translation:**
```sql
COPY (SELECT * FROM 'sales.csv' WHERE region = 'LATAM')
TO 'latam_sales.parquet' (FORMAT PARQUET);
```

#### MAP/FILTER Commands

```rust
pub enum NqlStatement {
    Map {
        transformations: Vec<MapTransform>,
        table: String,
    },
    Filter {
        condition: Expr,
        table: String,
    },
}

pub struct MapTransform {
    pub column: String,
    pub expression: Expr,
}
```

**Usage:**
```sql
USE 'data.csv';
MAP
    price = price * 1.1,
    category = UPPER(category),
    is_premium = CASE WHEN price > 1000 THEN true ELSE false END;
FILTER active = true AND date >= '2024-01-01';
SELECT * FROM data;
```

**Translation (CTE):**
```sql
WITH transformed AS (
    SELECT
        price * 1.1 AS price,
        UPPER(category) AS category,
        CASE WHEN price > 1000 THEN true ELSE false END AS is_premium,
        *
    FROM data
    WHERE active = true AND date >= '2024-01-01'
)
SELECT * FROM transformed;
```

### 12.5 TUI Enhancements

#### Dynamic Status Bar

```rust
// noctra-tui/src/noctra_tui.rs
fn render_status_bar(&self, area: Rect, buf: &mut Buffer) {
    let engine_name = match &self.query_engine {
        QueryEngine::DuckDB(_) => "🦆 DuckDB",
        QueryEngine::Sqlite(_) => "📦 SQLite",
        QueryEngine::Hybrid { .. } => "🔀 Hybrid",
    };

    let source_info = self.active_source()
        .map(|s| format!("Source: '{}' ({})", s.name(), s.source_type()))
        .unwrap_or_else(|| "No source".to_string());

    let memory_info = format!("Memory: {}MB", self.get_memory_usage_mb());

    let status = format!(
        " Engine: {} │ {} │ {} │ {}ms ",
        engine_name,
        source_info,
        memory_info,
        self.last_query_time.as_millis()
    );

    // Render...
}
```

**Visual Output:**
```
──( RESULT ) Noctra 2.0 ─── Engine: 🦆 DuckDB ─── Source: 'ventas.csv' (CSV) ─── Memory: 45MB ─── 12ms
```

#### Source Type Icons

```rust
fn get_source_icon(source_type: &SourceType) -> &'static str {
    match source_type {
        SourceType::Csv => "🦆",
        SourceType::Json => "🦆",
        SourceType::Parquet => "🦆",
        SourceType::Sqlite => "📦",
        SourceType::Memory => "💾",
        SourceType::DuckDB => "🦆",
    }
}
```

### 12.6 CLI Modes

#### Ad Hoc Mode (DuckDB Only)

```bash
noctra --engine duckdb --use 'sales_2024.csv'
```

- No SQLite database required
- Pure file-based analytics
- Lightweight and fast startup

#### Hybrid Mode (Default)

```bash
noctra --engine hybrid --db warehouse.db --use 'recent_sales.csv'
```

- SQLite for persistent data
- DuckDB for file queries
- Automatic routing

#### Traditional Mode (SQLite Only)

```bash
noctra --engine sqlite --db database.db
```

- Backward compatible
- Original Noctra behavior
- No DuckDB dependency

### 12.7 Configuration System

**Location:** `~/.config/noctra/config.toml`

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
streaming_threshold = 1000
```

**Loading:**
```rust
use serde::Deserialize;

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

### 12.8 Performance Characteristics

| Operation | Target | DuckDB vs SQLite |
|-----------|--------|------------------|
| CSV 10MB load | <500ms | 3-5x faster |
| 100K row aggregation | <1s | 5-10x faster |
| Parquet read | <100ms | 10-20x faster |
| JOIN (CSV + SQLite) | <2s | Comparable |
| Memory usage | <100MB | Similar |

**Optimizations:**
- **Zero-copy reads**: DuckDB reads files without staging
- **Columnar storage**: Parquet optimized for analytics
- **Parallel processing**: Multi-threaded execution
- **Lazy evaluation**: Only read required data

### 12.9 Security Considerations

**File Path Validation:**
```rust
pub fn validate_file_path(path: &Path, allowed_dirs: &[PathBuf]) -> Result<()> {
    let canonical = path.canonicalize()
        .map_err(|_| anyhow!("Invalid path: {:?}", path))?;

    if !allowed_dirs.iter().any(|dir| canonical.starts_with(dir)) {
        return Err(anyhow!("Path not in allowed directories: {:?}", canonical));
    }

    Ok(())
}
```

**Resource Limits:**
- Max file size: 10GB (configurable)
- Max result rows: 10,000 (configurable)
- Query timeout: 30s (configurable)
- Memory limit: 2GB (configurable)

### 12.10 Implementation Timeline

**Duration:** 2 weeks
**Target:** 2026-03-01
**Version:** v2.0.0

| Week | Phase | Deliverables |
|------|-------|--------------|
| **1** | Core Integration | - `noctra-duckdb` crate<br>- `QueryEngine` enum<br>- DuckDB `DataSource` impl<br>- File registration (`USE 'file'`)<br>- Parser extensions (EXPORT, MAP, FILTER) |
| **2** | Features & UI | - EXPORT multi-format<br>- TUI status bar updates<br>- CLI `--engine` flag<br>- Configuration system<br>- Ad hoc mode<br>- Documentation |

### 12.11 Success Criteria

**Functional:**
- ✅ Load CSV/JSON/Parquet with `USE 'file.ext' AS alias`
- ✅ Direct queries: `SELECT * FROM 'data.csv'`
- ✅ Cross-source JOINs (CSV + SQLite)
- ✅ EXPORT to multiple formats
- ✅ MAP/FILTER transformations
- ✅ Ad hoc mode (no database)

**Performance:**
- ✅ CSV 10MB: <500ms
- ✅ 100K aggregation: <1s
- ✅ Parquet 10x faster than CSV
- ✅ Memory: <100MB (typical)

**Quality:**
- ✅ Test coverage: >90%
- ✅ Zero clippy warnings
- ✅ Complete documentation
- ✅ Migration guide from v1.0

### 12.12 Migration Path from v1.0

**Backward Compatibility:**
- All v1.0 SQLite workflows unchanged
- Hybrid mode default (SQLite + DuckDB)
- New `--engine` flag optional
- Configuration file optional

**Recommended Migration:**
```bash
# Before (v1.0): SQLite only
noctra --db warehouse.db

# After (v2.0): Hybrid mode (auto-enabled)
noctra --db warehouse.db  # Same command, now supports files too!

# Query CSV without import
USE 'new_data.csv' AS csv;
SELECT * FROM csv;

# JOIN with existing SQLite tables
SELECT c.*, o.total
FROM csv.customers c
JOIN warehouse.orders o ON c.id = o.customer_id;
```

**No Breaking Changes:** All existing functionality preserved.

---

## Appendix A: API Reference

Complete API documentation is available at `docs/API-REFERENCE.md`.

## Appendix B: Migration Guide

Migration guide from other systems is available at `docs/MIGRATION.md`.

## Appendix C: Contributing

Contribution guidelines are available in `CONTRIBUTING.md`.

---

**Document Version:** 1.2
**Last Updated:** 2025-11-09
**Status:** Complete (M3.5), Planning (M4, M6)
**Authors:** Noctra Development Team
