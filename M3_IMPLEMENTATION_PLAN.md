# M3 Implementation Plan: Backend SQL/RQL Integration

**Milestone:** M3 - Backend SQL/RQL Integration
**Goal:** Connect NoctraTui to noctra-core Executor for real SQL query execution
**Estimated Effort:** 15 hours (~2-3 days)
**Priority:** 🔥 **CRITICAL** - Blocks all future features

---

## Overview

This plan outlines the step-by-step implementation to integrate the noctra-core executor with the NoctraTui interface, replacing simulated query results with real SQL execution.

---

## Implementation Phases

### ✅ Phase 0: Preparation (CURRENT)
- [x] Repository analysis complete
- [x] API compatibility assessment done
- [x] Implementation plan created
- [ ] Switch to development branch
- [ ] Commit analysis documents

---

### Phase 1: Add Backend Dependencies (Est: 1 hour)

#### 1.1 Update TUI Cargo.toml

**File:** `crates/tui/Cargo.toml`

**Changes:**
```toml
[dependencies]
# Core functionality
noctra-core = { path = "../core" }          # ✨ ADD THIS LINE
noctra-formlib = { path = "../formlib" }

# TUI framework
ratatui = { version = "0.24", features = ["macros"] }
crossterm = { version = "0.27" }
tui-textarea = "0.4"

# Serialization
serde = { workspace = true, features = ["derive"] }
serde_json = { workspace = true }
```

**Verification:**
```bash
cd crates/tui
cargo check
```

Expected: Clean compilation with new dependency

#### 1.2 Update TUI lib.rs Exports

**File:** `crates/tui/src/lib.rs`

**Add re-exports:**
```rust
// Re-export core types for convenience
pub use noctra_core::{Executor, Session, ResultSet, Value};
```

---

### Phase 2: Modify NoctraTui Structure (Est: 2 hours)

#### 2.1 Add Executor and Session Fields

**File:** `crates/tui/src/noctra_tui.rs`

**Add imports at top:**
```rust
use noctra_core::{Executor, Session, ResultSet};
use std::sync::Arc;
```

**Modify struct (around line 26):**
```rust
pub struct NoctraTui<'a> {
    /// Terminal de Ratatui
    terminal: Terminal<CrosstermBackend<Stdout>>,

    /// ✨ NEW: Backend executor
    executor: Arc<Executor>,

    /// ✨ NEW: Session de usuario
    session: Session,

    /// Modo actual de la interfaz
    mode: UiMode,

    // ... rest of fields unchanged
}
```

#### 2.2 Update Constructor

**File:** `crates/tui/src/noctra_tui.rs` (line 76)

**Replace `new()` method:**
```rust
/// Crear nueva instancia del TUI con base de datos en memoria
pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
    Self::with_executor(Arc::new(Executor::new_sqlite_memory()?))
}

/// Crear TUI con base de datos desde archivo
pub fn with_database<P: AsRef<str>>(db_path: P) -> Result<Self, Box<dyn std::error::Error>> {
    let executor = Executor::new_sqlite_file(db_path.as_ref())?;
    Self::with_executor(Arc::new(executor))
}

/// Crear TUI con executor personalizado
fn with_executor(executor: Arc<Executor>) -> Result<Self, Box<dyn std::error::Error>> {
    // Configurar terminal
    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen)?;

    let backend = CrosstermBackend::new(stdout);
    let terminal = Terminal::new(backend)?;

    // Crear editor de comandos
    let mut command_editor = TextArea::default();
    command_editor.set_block(
        Block::default()
            .borders(Borders::NONE)
            .style(Style::default()),
    );
    command_editor.set_cursor_line_style(Style::default());
    command_editor.set_cursor_style(Style::default().add_modifier(Modifier::REVERSED));

    // ✨ NEW: Crear sesión
    let session = Session::new();

    Ok(Self {
        terminal,
        executor,          // ✨ NEW
        session,           // ✨ NEW
        mode: UiMode::Command,
        command_editor,
        command_history: Vec::new(),
        command_number: 1,
        history_index: None,
        current_results: None,
        dialog_message: None,
        dialog_options: Vec::new(),
        dialog_selected: 0,
        should_quit: false,
    })
}
```

**Verification:**
```bash
cargo check --package noctra-tui
```

---

### Phase 3: Implement ResultSet Converter (Est: 1.5 hours)

#### 3.1 Add Conversion Method

**File:** `crates/tui/src/noctra_tui.rs`

**Add after `execute_command()` method (around line 547):**
```rust
/// Convertir ResultSet de noctra-core a QueryResults del TUI
fn convert_result_set(&self, result_set: ResultSet, command: &str) -> QueryResults {
    // Extraer nombres de columnas
    let columns: Vec<String> = result_set
        .columns
        .iter()
        .map(|col| col.name.clone())
        .collect();

    // Convertir valores a strings usando Display trait
    let rows: Vec<Vec<String>> = result_set
        .rows
        .iter()
        .map(|row| {
            row.values
                .iter()
                .map(|value| value.to_string())
                .collect()
        })
        .collect();

    // Construir mensaje de estado
    let status = if let Some(affected) = result_set.rows_affected {
        // Para INSERT/UPDATE/DELETE
        if let Some(rowid) = result_set.last_insert_rowid {
            format!(
                "{} fila(s) afectada(s) - Último ID insertado: {} - Comando: {}",
                affected,
                rowid,
                command.trim()
            )
        } else {
            format!(
                "{} fila(s) afectada(s) - Comando: {}",
                affected,
                command.trim()
            )
        }
    } else {
        // Para SELECT
        let row_count = result_set.row_count();
        if row_count == 0 {
            format!("Sin resultados - Comando: {}", command.trim())
        } else {
            format!("{} fila(s) retornada(s) - Comando: {}", row_count, command.trim())
        }
    };

    QueryResults {
        columns,
        rows,
        status,
    }
}
```

#### 3.2 Add Unit Tests

**File:** `crates/tui/src/noctra_tui.rs` (at end of file)

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use noctra_core::{Column, Row, Value};

    #[test]
    fn test_convert_result_set_select() {
        let tui = NoctraTui::new().unwrap();

        let mut result_set = ResultSet::new(vec![
            Column::new("id", "INTEGER", 0),
            Column::new("name", "TEXT", 1),
        ]);
        result_set.add_row(Row::new(vec![
            Value::Integer(1),
            Value::Text("Alice".to_string()),
        ]));
        result_set.add_row(Row::new(vec![
            Value::Integer(2),
            Value::Text("Bob".to_string()),
        ]));

        let query_results = tui.convert_result_set(result_set, "SELECT * FROM users");

        assert_eq!(query_results.columns, vec!["id", "name"]);
        assert_eq!(query_results.rows.len(), 2);
        assert_eq!(query_results.rows[0], vec!["1", "Alice"]);
        assert_eq!(query_results.rows[1], vec!["2", "Bob"]);
        assert!(query_results.status.contains("2 fila(s) retornada(s)"));
    }

    #[test]
    fn test_convert_result_set_insert() {
        let tui = NoctraTui::new().unwrap();

        let mut result_set = ResultSet::empty();
        result_set.rows_affected = Some(1);
        result_set.last_insert_rowid = Some(42);

        let query_results = tui.convert_result_set(result_set, "INSERT INTO users VALUES (...)");

        assert_eq!(query_results.columns.len(), 0);
        assert_eq!(query_results.rows.len(), 0);
        assert!(query_results.status.contains("1 fila(s) afectada(s)"));
        assert!(query_results.status.contains("Último ID insertado: 42"));
    }

    #[test]
    fn test_convert_result_set_empty() {
        let tui = NoctraTui::new().unwrap();

        let result_set = ResultSet::new(vec![
            Column::new("id", "INTEGER", 0),
        ]);

        let query_results = tui.convert_result_set(result_set, "SELECT * FROM empty_table");

        assert_eq!(query_results.columns, vec!["id"]);
        assert_eq!(query_results.rows.len(), 0);
        assert!(query_results.status.contains("Sin resultados"));
    }

    #[test]
    fn test_value_to_string_conversion() {
        let tui = NoctraTui::new().unwrap();

        let mut result_set = ResultSet::new(vec![
            Column::new("null_col", "NULL", 0),
            Column::new("int_col", "INTEGER", 1),
            Column::new("float_col", "REAL", 2),
            Column::new("bool_col", "BOOLEAN", 3),
            Column::new("text_col", "TEXT", 4),
        ]);
        result_set.add_row(Row::new(vec![
            Value::Null,
            Value::Integer(42),
            Value::Float(3.14),
            Value::Boolean(true),
            Value::Text("hello".to_string()),
        ]));

        let query_results = tui.convert_result_set(result_set, "SELECT ...");

        assert_eq!(query_results.rows[0][0], "NULL");
        assert_eq!(query_results.rows[0][1], "42");
        assert_eq!(query_results.rows[0][2], "3.14");
        assert_eq!(query_results.rows[0][3], "true");
        assert_eq!(query_results.rows[0][4], "hello");
    }
}
```

**Verification:**
```bash
cargo test --package noctra-tui convert_result_set
```

Expected: 4 tests pass

---

### Phase 4: Rewrite execute_command() (Est: 2 hours)

#### 4.1 Replace Simulated Logic

**File:** `crates/tui/src/noctra_tui.rs` (line 515)

**Replace entire `execute_command()` method:**
```rust
/// Ejecutar comando SQL actual
fn execute_command(&mut self) -> Result<(), Box<dyn std::error::Error>> {
    let command_text = self.command_editor.lines().join("\n");

    if command_text.trim().is_empty() {
        return Ok(());
    }

    // Agregar al historial
    self.command_history.push(command_text.clone());
    self.command_number += 1;

    // ✨ EJECUTAR SQL REAL
    match self.executor.execute_sql(&self.session, &command_text) {
        Ok(result_set) => {
            // Convertir ResultSet a QueryResults
            self.current_results = Some(self.convert_result_set(result_set, &command_text));

            // Cambiar a modo Result
            self.mode = UiMode::Result;
        }
        Err(e) => {
            // Mostrar error en Dialog Mode
            self.show_error_dialog(&format!("Error SQL: {}", e));
        }
    }

    // Limpiar editor para próximo comando
    self.clear_command_editor();

    Ok(())
}

/// Limpiar el editor de comandos
fn clear_command_editor(&mut self) {
    self.command_editor = TextArea::default();
    self.command_editor.set_block(Block::default().borders(Borders::NONE));
    self.command_editor.set_cursor_line_style(Style::default());
    self.command_editor.set_cursor_style(Style::default().add_modifier(Modifier::REVERSED));
}

/// Mostrar diálogo de error
fn show_error_dialog(&mut self, message: &str) {
    self.dialog_message = Some(message.to_string());
    self.dialog_options = vec!["OK".to_string()];
    self.dialog_selected = 0;
    self.mode = UiMode::Dialog;
}
```

#### 4.2 Handle Dialog Confirmation

**File:** `crates/tui/src/noctra_tui.rs`

**Update `handle_dialog_keys()` (around line 497):**
```rust
fn handle_dialog_keys(&mut self, key: KeyEvent) -> Result<(), Box<dyn std::error::Error>> {
    match key.code {
        // Escape - Volver a Command
        KeyCode::Esc => {
            self.dialog_message = None;
            self.mode = UiMode::Command;
        }

        // Enter - Confirmar y volver a Command
        KeyCode::Enter => {
            self.dialog_message = None;
            self.mode = UiMode::Command;
        }

        // Flechas - Navegar opciones (si hay múltiples)
        KeyCode::Left => {
            if self.dialog_selected > 0 {
                self.dialog_selected -= 1;
            }
        }
        KeyCode::Right => {
            if self.dialog_selected < self.dialog_options.len().saturating_sub(1) {
                self.dialog_selected += 1;
            }
        }

        _ => {}
    }
    Ok(())
}
```

**Verification:**
```bash
cargo check --package noctra-tui
```

---

### Phase 5: Update CLI TUI Launcher (Est: 1 hour)

#### 5.1 Add --database Option

**File:** `crates/cli/src/cli.rs`

**Update TuiArgs:**
```rust
#[derive(Args, Debug, Clone, Default)]
pub struct TuiArgs {
    /// Cargar script SQL al inicio
    #[arg(short, long, value_name = "FILE")]
    pub load: Option<PathBuf>,

    /// Schema/base de datos a usar
    #[arg(short, long, value_name = "SCHEMA")]
    pub schema: Option<String>,

    /// ✨ NEW: Archivo de base de datos SQLite
    #[arg(short, long, value_name = "DATABASE")]
    pub database: Option<PathBuf>,
}
```

#### 5.2 Update run_tui()

**File:** `crates/cli/src/cli.rs`

**Replace run_tui() implementation:**
```rust
async fn run_tui(self, args: TuiArgs) -> Result<(), Box<dyn std::error::Error>> {
    use noctra_tui::NoctraTui;

    println!("🖥️  Noctra TUI v0.1.0 - Modo Terminal Interactivo");

    // Mostrar información de la base de datos
    if let Some(ref db_path) = args.database {
        println!("📂 Base de datos: {}", db_path.display());
    } else {
        println!("💾 Base de datos: en memoria (temporal)");
    }

    if let Some(ref schema) = args.schema {
        println!("📊 Schema: {}", schema);
    }

    println!();
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

    // ✨ Crear TUI con base de datos si se especificó
    let mut tui = if let Some(db_path) = args.database {
        NoctraTui::with_database(db_path.to_string_lossy().to_string())?
    } else {
        NoctraTui::new()?
    };

    // TODO: Si se especificó --load, cargar script
    // TODO: Si se especificó --schema, cambiar schema por defecto

    tui.run()?;

    println!("\n👋 ¡Noctra finalizado correctamente!");
    Ok(())
}
```

**Verification:**
```bash
cargo build --release
./target/release/noctra tui --help
```

Expected output should include `--database <DATABASE>` option

---

### Phase 6: Integration Testing (Est: 3 hours)

#### 6.1 Create Integration Test File

**File:** `crates/tui/tests/integration_test.rs` (NEW FILE)

```rust
use noctra_tui::NoctraTui;

#[test]
fn test_tui_creation_memory() {
    let tui = NoctraTui::new();
    assert!(tui.is_ok(), "Failed to create TUI with in-memory database");
}

#[test]
fn test_tui_creation_file() {
    let temp_dir = tempfile::tempdir().unwrap();
    let db_path = temp_dir.path().join("test.db");

    let tui = NoctraTui::with_database(db_path.to_string_lossy().to_string());
    assert!(tui.is_ok(), "Failed to create TUI with file database");
}

// Note: Full interactive testing requires simulating keyboard input
// which is complex. These tests verify the basic structure works.
```

#### 6.2 Add tempfile Dependency

**File:** `crates/tui/Cargo.toml`

```toml
[dev-dependencies]
tempfile = { workspace = true }
```

#### 6.3 Manual Testing Script

**File:** `scripts/test_m3.sh` (NEW FILE)

```bash
#!/bin/bash
set -e

echo "=== M3 Integration Test Script ==="
echo

# Build release version
echo "1. Building Noctra..."
cargo build --release

# Test 1: TUI with in-memory database
echo
echo "2. Testing in-memory database..."
echo "   Commands to test:"
echo "   - CREATE TABLE users (id INTEGER, name TEXT)"
echo "   - INSERT INTO users VALUES (1, 'Alice')"
echo "   - SELECT * FROM users"
echo
echo "Press Enter to launch TUI (use Ctrl+C to exit after testing)"
read
./target/release/noctra tui

# Test 2: TUI with file database
echo
echo "3. Testing file-based database..."
rm -f test.db
echo "   Database will persist in test.db"
echo "Press Enter to launch TUI (use Ctrl+C to exit after testing)"
read
./target/release/noctra tui --database test.db

echo
echo "=== M3 Tests Complete ==="
echo "Check that:"
echo "  - CREATE TABLE worked"
echo "  - INSERT showed 'N fila(s) afectada(s)'"
echo "  - SELECT displayed actual data"
echo "  - SQL errors showed in Dialog mode"
echo "  - test.db file exists and persisted data"
```

**Make executable:**
```bash
chmod +x scripts/test_m3.sh
```

---

### Phase 7: Error Handling Improvements (Est: 2 hours)

#### 7.1 Add Specific Error Messages

**File:** `crates/tui/src/noctra_tui.rs`

**Improve error handling:**
```rust
fn execute_command(&mut self) -> Result<(), Box<dyn std::error::Error>> {
    let command_text = self.command_editor.lines().join("\n");

    if command_text.trim().is_empty() {
        return Ok(());
    }

    self.command_history.push(command_text.clone());
    self.command_number += 1;

    // Detectar tipo de comando para mejor manejo de errores
    let trimmed = command_text.trim().to_uppercase();
    let is_ddl = trimmed.starts_with("CREATE")
        || trimmed.starts_with("DROP")
        || trimmed.starts_with("ALTER");

    match self.executor.execute_sql(&self.session, &command_text) {
        Ok(result_set) => {
            self.current_results = Some(self.convert_result_set(result_set, &command_text));
            self.mode = UiMode::Result;
        }
        Err(e) => {
            // Construir mensaje de error más descriptivo
            let error_msg = if is_ddl {
                format!("❌ Error DDL: {}\n\nComando: {}", e, command_text.trim())
            } else {
                format!("❌ Error SQL: {}\n\nComando: {}", e, command_text.trim())
            };

            self.show_error_dialog(&error_msg);
        }
    }

    self.clear_command_editor();
    Ok(())
}
```

#### 7.2 Add Try-Catch for Panics

**File:** `crates/tui/src/noctra_tui.rs`

**Wrap risky operations:**
```rust
use std::panic;

fn execute_command(&mut self) -> Result<(), Box<dyn std::error::Error>> {
    // ... existing code ...

    // Catch panics during execution
    let result = panic::catch_unwind(panic::AssertUnwindSafe(|| {
        self.executor.execute_sql(&self.session, &command_text)
    }));

    match result {
        Ok(Ok(result_set)) => {
            // Success
            self.current_results = Some(self.convert_result_set(result_set, &command_text));
            self.mode = UiMode::Result;
        }
        Ok(Err(e)) => {
            // SQL error
            self.show_error_dialog(&format!("❌ Error SQL: {}", e));
        }
        Err(_) => {
            // Panic occurred
            self.show_error_dialog("❌ Error Crítico: El comando causó un fallo interno. Por favor reporta este bug.");
        }
    }

    // ... rest of code ...
}
```

---

### Phase 8: Documentation (Est: 1 hour)

#### 8.1 Update STATUS.md

**File:** `STATUS.md`

**Update M3 section:**
```markdown
## 🚀 Milestone 3 - Backend SQL/RQL Integration [EN PROGRESO]

### 🎯 Objetivos Alcanzados

#### 3.1 Query Execution Engine
- [x] Integrar noctra-core::Executor con NoctraTui ✅
- [x] Ejecutar queries reales desde Command Mode ✅
- [x] Mostrar resultados SQL en Result Mode ✅
- [x] Manejo de errores SQL en Dialog Mode ✅
- [ ] Soporte para transacciones (BEGIN/COMMIT/ROLLBACK)
- [ ] Connection pooling para múltiples bases de datos

#### 3.2 Schema Management
- [ ] Comando `use <schema>` para cambiar BD
- [ ] Mostrar esquema actual en header
- [ ] Listar tablas con `show tables`
- [ ] Describir tabla con `desc <table>`

### 📦 Archivos Modificados M3

```
crates/tui/
  ├── Cargo.toml - Added noctra-core dependency
  ├── src/noctra_tui.rs - Added Executor, Session, convert_result_set()
  └── tests/integration_test.rs - NEW: Integration tests

crates/cli/
  └── src/cli.rs - Added --database option to TuiArgs

scripts/
  └── test_m3.sh - NEW: Manual testing script
```

**Commit:** `git commit -m "feat(m3): Integrate noctra-core Executor with NoctraTui"`
```

#### 8.2 Update README Examples

**File:** `README.md`

**Add to Usage section:**
```markdown
### TUI Mode (Terminal Interface)

```bash
# In-memory database (data lost on exit)
noctra tui

# Persistent database file
noctra tui --database my_data.db

# Open existing database
noctra tui --database /path/to/existing.db
```

**Examples:**
```sql
-- Create a table
CREATE TABLE users (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL,
    email TEXT
);

-- Insert data
INSERT INTO users VALUES (1, 'Alice', 'alice@example.com');
INSERT INTO users VALUES (2, 'Bob', 'bob@example.com');

-- Query data
SELECT * FROM users;
SELECT name, email FROM users WHERE id = 1;

-- Update
UPDATE users SET email = 'newemail@example.com' WHERE id = 1;

-- Delete
DELETE FROM users WHERE id = 2;
```
```

---

### Phase 9: Final Testing & Validation (Est: 2 hours)

#### 9.1 Test Checklist

**Manual Tests:**
- [ ] `cargo test` - All tests pass (29 existing + 4 new = 33 tests)
- [ ] `cargo clippy` - No warnings
- [ ] `cargo build --release` - Clean build
- [ ] `noctra tui` - Launches successfully
- [ ] CREATE TABLE - Works and shows in Dialog or Result
- [ ] INSERT - Shows "1 fila(s) afectada(s)"
- [ ] SELECT - Displays actual data in table
- [ ] UPDATE - Shows affected rows
- [ ] DELETE - Shows affected rows
- [ ] Invalid SQL - Shows error in Dialog mode
- [ ] Empty SELECT - Shows "Sin resultados"
- [ ] ESC from Result mode - Returns to Command mode
- [ ] ESC from Dialog mode - Returns to Command mode
- [ ] F5 - Executes current command
- [ ] End - Exits TUI cleanly
- [ ] `noctra tui --database test.db` - Creates file
- [ ] Exit and reopen database - Data persists
- [ ] Multiple commands - History works (PageUp/PageDown)

**Automated Tests:**
```bash
# Run all tests
cargo test --all

# Run TUI-specific tests
cargo test --package noctra-tui

# Run with output
cargo test -- --nocapture

# Check coverage (optional)
cargo tarpaulin --out Html
```

#### 9.2 Performance Benchmarks

**Create benchmark file:** `crates/tui/benches/query_benchmark.rs`

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use noctra_tui::NoctraTui;

fn benchmark_create_table(c: &mut Criterion) {
    c.bench_function("create_table", |b| {
        b.iter(|| {
            let tui = NoctraTui::new().unwrap();
            // Measure time to create table
            black_box(tui)
        });
    });
}

fn benchmark_select_query(c: &mut Criterion) {
    // Setup: Create table with 1000 rows
    // Measure: SELECT * FROM table
}

criterion_group!(benches, benchmark_create_table, benchmark_select_query);
criterion_main!(benches);
```

---

## Rollback Plan

If integration fails or causes critical issues:

### Rollback Steps:
1. Revert `crates/tui/src/noctra_tui.rs` to previous version
2. Remove `noctra-core` from `crates/tui/Cargo.toml`
3. Restore simulated query execution
4. Document failure reason in `M3_ROLLBACK_NOTES.md`

### Rollback Commands:
```bash
git diff HEAD~1 crates/tui/src/noctra_tui.rs > m3_changes.patch
git checkout HEAD~1 -- crates/tui/
cargo test --all  # Verify everything works again
```

---

## Success Criteria

**Definition of Done:**
- [ ] All existing 29 tests pass
- [ ] At least 4 new tests for ResultSet conversion
- [ ] `noctra tui` executes real SQL queries
- [ ] SQL errors display gracefully (no crashes)
- [ ] `--database` option works for persistent storage
- [ ] Documentation updated (STATUS.md, README.md)
- [ ] Code review completed (self-review minimum)
- [ ] No clippy warnings
- [ ] Manual testing checklist complete

---

## Timeline

| Phase | Duration | Dependencies | Status |
|-------|----------|--------------|--------|
| 0. Preparation | 30 min | - | ✅ DONE |
| 1. Dependencies | 1 hour | Phase 0 | 📋 Ready |
| 2. Modify Structure | 2 hours | Phase 1 | 📋 Ready |
| 3. Converter | 1.5 hours | Phase 2 | 📋 Ready |
| 4. execute_command() | 2 hours | Phase 3 | 📋 Ready |
| 5. CLI Launcher | 1 hour | Phase 4 | 📋 Ready |
| 6. Integration Tests | 3 hours | Phase 4 | 📋 Ready |
| 7. Error Handling | 2 hours | Phase 4 | 📋 Ready |
| 8. Documentation | 1 hour | All above | 📋 Ready |
| 9. Final Testing | 2 hours | All above | 📋 Ready |
| **TOTAL** | **15.5 hours** | | **~2-3 days** |

---

## Next Actions

**Immediate (Today):**
1. ✅ Commit analysis documents
2. Switch to M3 development branch
3. Execute Phase 1 (Add dependencies)
4. Execute Phase 2 (Modify structure)

**Tomorrow:**
5. Execute Phases 3-5 (Core logic)
6. Execute Phase 6 (Testing)

**Day 3:**
7. Execute Phases 7-9 (Polish & validation)
8. Create pull request
9. Merge to main

---

## Notes

- **Breaking Changes:** None - this is purely additive
- **Backward Compatibility:** In-memory mode maintains existing behavior
- **Performance:** Minimal impact (executor is already fast)
- **Security:** No new attack surface (SQLite is sandboxed)

---

**End of M3 Implementation Plan**
