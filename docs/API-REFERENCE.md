# Noctra API Reference

> **Version:** 1.0  
> **Last Updated:** 2025-01-05  
> **Status:** In Development (Milestone 1-2)

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

## Complete Example

See [GETTING_STARTED.md](GETTING_STARTED.md) for practical usage.

---

**Status:** This API is under active development. See [ROADMAP.md](ROADMAP.md) for implementation timeline.
