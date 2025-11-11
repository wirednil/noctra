# Noctra API Reference

> **Version:** 1.1
> **Last Updated:** 2025-11-09
> **Status:** In Development (Milestone 4 - NQL Implementation)

## 📚 Related Documentation

- **[Getting Started](GETTING_STARTED.md)** - Practical usage examples
- **[Design Document](DESIGN.md)** - Complete technical architecture (Section 3: Core Components)
- **[FDL2 Specification](FDL2-SPEC.md)** - Form API details
- **[RQL Extensions](RQL-EXTENSIONS.md)** - Query language reference
- **[Roadmap](ROADMAP.md)** - Implementation timeline and milestones

---

## noctra-core

### Executor

```rust
pub struct Executor {
    backend: Box<dyn DatabaseBackend>,
    session: Session,
}

impl Executor {
    pub async fn execute(&mut self, sql: &str) -> Result<ResultSet>;
    pub async fn execute_with_params(&mut self, sql: &str, params: Vec<Value>) -> Result<ResultSet>;
}
```

See: [DESIGN.md Section 3.1](DESIGN.md#31-noctra-core-noctra-core)

### Value Types

```rust
pub enum Value {
    Null,
    Integer(i64),
    Float(f64),
    Text(String),
    Boolean(bool),
    Date(NaiveDate),
    DateTime(NaiveDateTime),
}
```

### Session Management

```rust
pub struct Session {
    pub variables: HashMap<String, Value>,
    pub current_schema: Option<String>,
}
```

---

## noctra-parser

### RQL Parser

```rust
pub struct RqlParser;

impl RqlParser {
    pub fn parse(input: &str) -> Result<RqlAst>;
    pub fn extract_parameters(sql: &str) -> Vec<Parameter>;
}
```

See: [DESIGN.md Section 3.2](DESIGN.md#32-rql-parser-noctra-parser)  
Language Reference: [RQL-EXTENSIONS.md](RQL-EXTENSIONS.md)

---

## noctra-formlib

### Form Loading

```rust
pub fn load_form(path: &Path) -> Result<Form>;
```

See: [FDL2-SPEC.md](FDL2-SPEC.md)

### Form Compilation

```rust
pub struct FormCompiler;

impl FormCompiler {
    pub fn compile(&self, form: &Form) -> Result<CompiledAction>;
}
```

---

## noctra-tui

### Window Manager

```rust
pub struct NoctraWindowManager;

impl NoctraWindowManager {
    pub fn new() -> Result<Self>;
    pub fn render(&mut self) -> Result<()>;
}
```

See: [DESIGN.md Section 6](DESIGN.md#6-noctra-window-manager-nwm)

---

## NQL - Multi-Source Data Support (M4)

### DataSource Trait

```rust
pub trait DataSource: Send + Sync + Debug {
    /// Execute a query against the data source
    fn query(&self, sql: &str, parameters: &Parameters) -> Result<ResultSet>;

    /// Get schema information (tables/columns)
    fn schema(&self) -> Result<Vec<TableInfo>>;

    /// Get the type of this data source
    fn source_type(&self) -> SourceType;

    /// Get the name/identifier of this source
    fn name(&self) -> &str;

    /// Get metadata about this source
    fn metadata(&self) -> SourceMetadata;

    /// Close the data source (optional)
    fn close(&mut self) -> Result<()>;
}
```

### SourceType Enum

```rust
pub enum SourceType {
    SQLite { path: String },
    CSV {
        path: String,
        delimiter: char,
        has_header: bool,
        encoding: String,
    },
    JSON { path: String },
    Memory { capacity: usize },
}

impl SourceType {
    pub fn type_name(&self) -> &str;
    pub fn display_path(&self) -> String;
}
```

### SourceRegistry

```rust
pub struct SourceRegistry {
    sources: HashMap<String, Box<dyn DataSource>>,
    active_source: Option<String>,
}

impl SourceRegistry {
    pub fn new() -> Self;
    pub fn register(&mut self, alias: String, source: Box<dyn DataSource>) -> Result<()>;
    pub fn get(&self, alias: &str) -> Option<&dyn DataSource>;
    pub fn get_mut(&mut self, alias: &str) -> Option<&mut (dyn DataSource + '_)>;
    pub fn active(&self) -> Option<&dyn DataSource>;
    pub fn set_active(&mut self, alias: &str) -> Result<()>;
    pub fn list_sources(&self) -> Vec<(String, SourceType)>;
    pub fn remove(&mut self, alias: &str) -> Result<()>;
}
```

### CSV Backend

```rust
pub struct CsvDataSource {
    path: PathBuf,
    name: String,
    options: CsvOptions,
    schema: Vec<ColumnInfo>,
    data: Vec<Vec<Value>>,
}

impl CsvDataSource {
    /// Create a new CSV data source
    pub fn new<P: AsRef<Path>>(
        path: P,
        name: String,
        options: CsvOptions
    ) -> Result<Self>;
}

pub struct CsvOptions {
    pub delimiter: Option<char>,     // None = auto-detect
    pub has_header: bool,
    pub encoding: Option<String>,    // None = auto-detect
    pub quote: char,
    pub skip_rows: usize,
}

impl Default for CsvOptions {
    fn default() -> Self {
        Self {
            delimiter: None,
            has_header: true,
            encoding: None,
            quote: '"',
            skip_rows: 0,
        }
    }
}
```

### TableInfo & ColumnInfo

```rust
pub struct TableInfo {
    pub name: String,
    pub columns: Vec<ColumnInfo>,
    pub row_count: Option<usize>,
}

pub struct ColumnInfo {
    pub name: String,
    pub data_type: String,
    pub nullable: bool,
    pub default_value: Option<String>,
}
```

### Usage Example

```rust
use noctra_core::{CsvDataSource, CsvOptions, DataSource, SourceRegistry};

// Create CSV data source
let csv = CsvDataSource::new(
    "data/customers.csv",
    "customers".to_string(),
    CsvOptions::default()
)?;

// Create registry
let mut registry = SourceRegistry::new();
registry.register("customers".to_string(), Box::new(csv))?;

// Query the CSV
let source = registry.get("customers").unwrap();
let result = source.query("SELECT * FROM customers LIMIT 10", &Parameters::new())?;

// Inspect schema
let schema = source.schema()?;
for table in schema {
    println!("Table: {}", table.name);
    for col in table.columns {
        println!("  {}: {}", col.name, col.data_type);
    }
}
```

### NQL AST Extensions

```rust
pub enum RqlStatement {
    // ... existing SQL statements ...

    UseSource {
        path: String,
        alias: Option<String>,
        options: HashMap<String, String>,
    },

    ShowSources,

    ShowTables { source: Option<String> },

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

    Map { expressions: Vec<MapExpression> },

    Filter { condition: String },

    Unset { variables: Vec<String> },
}

pub struct MapExpression {
    pub expression: String,
    pub alias: Option<String>,
}

pub enum ExportFormat {
    Csv,
    Json,
    Xlsx,
}
```

---

## Complete Example

See [GETTING_STARTED.md](GETTING_STARTED.md) for practical usage.

For NQL usage examples and complete specification, see [NQL-SPEC.md](NQL-SPEC.md) and [DESIGN.md Section 11](DESIGN.md#11-nql---noctra-query-language-m4).

---

**Status:** This API is under active development. See [ROADMAP.md](ROADMAP.md) for implementation timeline.

**Current Progress:** M0-M3 Complete (100%) | M4 In Progress (25%) - NQL Foundation Implemented
