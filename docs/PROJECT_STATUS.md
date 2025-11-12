# Estado del Proyecto Noctra

**Última actualización:** 2025-11-11
**Branch activo:** `claude/duckdb-integration-analysis-011CV2uozdTvq4uXr2JRRb68`
**Versión:** 0.2.0 (M5 Completado)
**Próximo Release:** v0.6.0 "FABRIC" (M6 - En Progreso)

---

## 📊 Resumen Ejecutivo

Noctra es un entorno SQL interactivo moderno escrito en Rust con filosofía 4GL, proporcionando una experiencia profesional de consulta SQL con formularios declarativos y TUI avanzado.

**Progreso General:** M1 ✅ | M2 ✅ | M3 ✅ | M3.5 ✅ | M4 ✅ | M5 ✅ | **M6 🚧** | M7 📋

| Milestone | Estado | Progreso | Fecha/Commit |
|-----------|--------|----------|---------------|
| **M0: Foundation** | ✅ Completado | 100% | 2025-01-12 |
| **M1: Core + Parser** | ✅ Completado | 100% | 88805e8 |
| **M2: Forms + TUI** | ✅ Completado | 100% | fa43a74 |
| **M3: Backend SQL/RQL** | ✅ Completado | 100% | a64a72c |
| **M3.5: CSV/NQL Hotfix** | ✅ Completado | 100% | dbddebc |
| **M4: Advanced Features** | ✅ Completado | 100% | 83b100d |
| **M5: Extended Capabilities** | ✅ Completado | 100% | 2025-11-11 |
| **M6: Noctra(🦆) "FABRIC"** | 🚧 En Progreso | 0% | 2025-11-11 (Fase 1) |
| **M7: "SCRIPT" (Opcional)** | 📋 Planificado | 0% | Post-M6 |

**Total Tests:** 29 pasando (100%)
**Build:** Release OK (3 warnings menores en core)
**Clippy:** 0 warnings
**Estado:** ✅ **M4 Completado - CSV Backend Avanzado + Seguridad**

### 🆕 Extensión Conceptual: NQL (Noctra Query Language)

**Visión M4+**: Noctra evolucionará de un entorno SQL puro a un **sistema de consultas multi-fuente** mediante NQL, permitiendo:

- 📄 **Consultar archivos CSV** como si fueran bases de datos
- 🔄 **Importar/Exportar** entre diferentes formatos (CSV ↔ SQLite ↔ JSON)
- 🎯 **Sintaxis unificada** para todas las fuentes de datos
- 🛠️ **Transformaciones declarativas** con MAP y FILTER
- 📊 **Administración de múltiples fuentes** simultáneas

**Ejemplo de uso futuro:**
```sql
USE 'clientes.csv' AS csv;          -- Cargar CSV
SELECT * FROM csv WHERE pais = 'AR'; -- Consultar como SQL
EXPORT csv TO 'filtrado.json';      -- Exportar a JSON
```

Ver [M4.10 - NQL](#410-nql---noctra-query-language-extensión-conceptual) para detalles completos.

---

## ✅ Milestone 0 - Foundation [COMPLETADO]

### Objetivos Alcanzados
- [x] Workspace Cargo configurado (6 crates activos) ✅
- [x] Estructura de proyecto definida ✅
- [x] CI/CD básico configurado ✅
- [x] Documentación inicial ✅
- [x] Licencias (MIT + Apache 2.0) ✅

---

## ✅ Milestone 1 - Core + Parser [COMPLETADO]

### Objetivos Alcanzados

#### 1.1 Core Runtime
- [x] `core::Executor` con SQLite backend ✅
- [x] Sistema de tipos `Value` completo ✅
- [x] `Session` con manejo de variables ✅
- [x] CRUD operations (SELECT/INSERT/UPDATE/DELETE) ✅
- [x] Manejo de parámetros (posicionales y nombrados) ✅
- [x] Tests: 10 core tests ✅

#### 1.2 Parser RQL
- [x] Parser RQL completo con extensiones SQL ✅
- [x] Soporte para parámetros `$1`, `:name` ✅
- [x] Comandos extendidos (USE, LET, SHOW) ✅
- [x] Templates condicionales ✅
- [x] Tests: Parser completo ✅

#### 1.3 CLI REPL
- [x] REPL interactivo con rustyline ✅
- [x] Historial de comandos ✅
- [x] Ejecución de queries ✅
- [x] Formateo de resultados ✅

### Archivos Clave M1
```
crates/core/src/
  ├── executor.rs (450 líneas) - Executor principal
  ├── backend.rs (350 líneas) - Backend SQLite
  ├── session.rs (200 líneas) - Gestión de sesión
  └── types.rs (300 líneas) - Sistema de tipos

crates/parser/src/
  ├── parser.rs (600 líneas) - Parser RQL
  └── ast.rs (400 líneas) - AST definitions

crates/cli/src/
  ├── cli.rs (500 líneas) - CLI commands
  └── repl.rs (400 líneas) - REPL loop
```

**Commit final:** `88805e8 - Milestone 1 Completado`

---

## ✅ Milestone 2 - Forms & TUI Completo [COMPLETADO]

### Objetivos Alcanzados

#### 2.1 Capa Declarativa (FormLib)

**Estructuras Core:**
- [x] `Form` struct con tipos de campo completos ✅
- [x] Parser TOML/JSON con serde ✅
- [x] `FormValidator` con validación completa ✅
  - Tipos: text, int, float, bool, date, datetime, email, password
  - Validaciones: required, min/max, length, regex, allowed values
- [x] `FormGraph` para navegación jerárquica ✅
  - Detección de ciclos
  - Path validation
  - Node search
- [x] `GraphNavigator` con historial ✅
  - Stack LIFO de ventanas
  - Breadcrumbs
  - go_back/go_forward/go_home

#### 2.2 Capa TUI (Ratatui)

**FormRenderer:**
- [x] Widget FormRenderer profesional con Ratatui ✅
  - Layout automático: Header/Fields/Actions/Help
  - Widgets: List, Paragraph, Block, Borders
  - Manejo correcto de unicode y box-drawing
  - 100% responsivo sin cálculos manuales
  - render() para TUI y render_to_string() para preview

**InteractiveFormExecutor:**
- [x] Executor interactivo completo ✅
  - Terminal<CrosstermBackend> con raw mode
  - Event loop: terminal.draw(|frame| ...)
  - Navegación TAB/Shift+TAB
  - Edición en tiempo real
  - Validación durante entrada
  - Drop trait para limpieza segura

**NoctraTui (TUI Completo):**
- [x] Layout fijo estilo 4GL retro ✅
  - Header: `──( MODE ) SQL Noctra 0.1.0───────── Cmd: N───`
  - Workspace: Área dinámica según modo
  - Separator: Línea divisoria
  - Shortcuts: Barra con F1-F8, Alt+R/W

- [x] **Modo Command (INSERTAR)** ✅
  - Editor SQL/RQL con tui-textarea
  - Historial navegable (PageUp/PageDown)
  - F5 ejecuta comando
  - Contador de comandos automático

- [x] **Modo Result (RESULTADO)** ✅
  - Table widget con bordes ASCII
  - Columnas y filas dinámicas
  - Mensaje de estado
  - ESC vuelve a Command

- [x] **Modo Dialog (DIÁLOGO)** ✅
  - Ventana modal centrada
  - Confirmaciones: SI/NO/CANCELAR
  - Navegación con flechas
  - Enter ejecuta acción

- [x] **Modo Form** ✅
  - Integración con FormRenderer

#### 2.3 Integración CLI

- [x] Comando `noctra form load <file>` ✅
- [x] Comando `noctra form exec <file>` ✅
  - Modo interactivo con Ratatui completo
  - Modo batch con --non-interactive
- [x] Comando `noctra form preview <file>` ✅
- [x] **Comando `noctra tui`** ✅
  - TUI completo estilo 4GL
  - Opciones: --load, --schema

### Archivos Clave M2

```
crates/formlib/
  ├── src/forms.rs (600 líneas) - Form, FormField, FormAction
  ├── src/validation.rs (350 líneas) - FormValidator
  └── src/graph.rs (400 líneas) - FormGraph, GraphNavigator

crates/tui/
  ├── src/form_renderer.rs (585 líneas) - Ratatui FormRenderer
  ├── src/noctra_tui.rs (640 líneas) - TUI completo
  ├── src/nwm.rs (450 líneas) - Noctra Window Manager
  └── src/layout.rs (300 líneas) - Layout Manager

crates/cli/
  ├── src/cli.rs - TuiArgs, run_tui()
  └── src/interactive_form.rs (220 líneas) - InteractiveFormExecutor
```

### Lecciones Aprendidas M2

1. **Arquitectura en capas:** FormLib → TUI → CLI funciona perfectamente
2. **Declarativo > Imperativo:** TOML para formularios es mantenible
3. **Validación temprana:** Detecta errores antes del runtime
4. **Stack LIFO:** Simplifica navegación entre ventanas
5. **Tests desde inicio:** Detectan bugs temprano
6. **Interactividad real:** Raw mode + event loop necesario
7. **Ratatui FTW:** Evita todos los problemas de renderizado manual ⭐

**Commit final M2:** `fa43a74 - feat: Implementar TUI completo de Noctra con Ratatui`

---

## ✅ Milestone 3 - Backend SQL/RQL Integration [COMPLETADO]

### Objetivos Alcanzados

#### 3.1 Query Execution Engine
- [x] Integrar noctra-core::Executor con NoctraTui ✅
- [x] Ejecutar queries reales desde Command Mode ✅
- [x] Mostrar resultados SQL en Result Mode ✅
- [x] Manejo de errores SQL en Dialog Mode ✅
- [x] Soporte para in-memory y file-based databases ✅
- [ ] Soporte para transacciones (BEGIN/COMMIT/ROLLBACK) - **Pendiente M4**
- [ ] Connection pooling para múltiples bases de datos - **Pendiente M4**

#### 3.2 Integración Completa

**Cambios en NoctraTui:**
- Agregado `executor: Arc<Executor>` para ejecución SQL
- Agregado `session: Session` para estado de sesión
- Nuevo método `convert_result_set()` para mapear ResultSet → QueryResults
- Método `execute_command()` reescrito para SQL real (no simulado)
- Constructores: `new()` (in-memory), `with_database()` (file-based)

**Cambios en CLI:**
- Agregado `--database <PATH>` option en `noctra tui`
- Banner informativo mostrando tipo de BD al iniciar
- Soporte para bases de datos persistentes

**Mapeo de Tipos:**
| Backend (ResultSet)              | TUI (QueryResults)      |
|----------------------------------|-------------------------|
| `Vec<Column>`                    | `Vec<String>` (names)   |
| `Vec<Row{values: Vec<Value>}>`  | `Vec<Vec<String>>`      |
| `rows_affected: Option<u64>`     | status message          |
| `last_insert_rowid: Option<i64>` | status message          |

### Archivos Modificados M3

```
crates/tui/
  └── src/noctra_tui.rs - Added Executor, Session, convert_result_set()

crates/cli/
  └── src/cli.rs - Added --database option to TuiArgs
```

### Funcionalidad M3

**Antes (M2 - Simulado):**
```rust
// Datos hardcodeados
self.current_results = Some(QueryResults {
    columns: vec!["id", "nombre", "email"],
    rows: vec![vec!["1", "Juan", "juan@example.com"]],
    status: "3 filas retornadas",
});
```

**Después (M3 - Real):**
```rust
// Ejecución SQL real
match self.executor.execute_sql(&self.session, &command_text) {
    Ok(result_set) => {
        self.current_results = Some(self.convert_result_set(result_set, &command_text));
        self.mode = UiMode::Result;
    }
    Err(e) => self.show_error_dialog(&format!("❌ Error SQL: {}", e)),
}
```

### Ejemplos de Uso

```bash
# Base de datos en memoria (se pierde al salir)
noctra tui

# Base de datos persistente
noctra tui --database mydata.db

# Abrir base de datos existente
noctra tui -d /path/to/existing.db
```

**Dentro del TUI:**
```sql
-- Crear tabla
CREATE TABLE empleados (
    id INTEGER PRIMARY KEY,
    nombre TEXT NOT NULL,
    departamento TEXT,
    salario REAL
);

-- Insertar datos
INSERT INTO empleados VALUES (1, 'Ana García', 'IT', 75000);
INSERT INTO empleados VALUES (2, 'Carlos López', 'Ventas', 65000);

-- Consultar
SELECT * FROM empleados WHERE departamento = 'IT';

-- Actualizar
UPDATE empleados SET salario = 80000 WHERE id = 1;

-- Eliminar
DELETE FROM empleados WHERE id = 2;
```

**Commit final M3:** `a64a72c - feat(m3): Integrate noctra-core Executor with NoctraTui`
**Fecha:** 2025-11-08

---

## ✅ Milestone 3.5 - CSV/NQL Support Hotfix [COMPLETADO]

### Contexto

Hotfix intermedio entre M3 y M4 que implementa soporte completo para archivos CSV y comandos NQL básicos. Este trabajo acelera la implementación de la sección 4.10 (NQL) del Milestone 4.

**Branch:** `claude/fix-csv-prepare-error-011CUwdxvbzoQoC1JawsGqpg`
**Fecha:** 2025-11-09
**Commits:** 6 commits (0438e65 → dbddebc)

### Objetivos Alcanzados

#### 3.5.1 CSV Backend Implementation
- [x] `CsvDataSource` trait implementation ✅
- [x] Automatic delimiter detection (`,`, `;`, `\t`, `|`) ✅
- [x] Type inference (INTEGER, REAL, BOOLEAN, TEXT) ✅
- [x] Header detection and column naming ✅
- [x] CSV parsing with quote handling ✅
- [x] Schema introspection ✅

#### 3.5.2 Multi-Source Data Routing
- [x] `SourceRegistry` for managing multiple data sources ✅
- [x] Active source tracking and switching ✅
- [x] Query routing to active source in `execute_rql()` ✅
- [x] Fallback to SQLite when no CSV source active ✅

#### 3.5.3 NQL Commands - Basic Set
- [x] `USE <path> AS <alias> OPTIONS (...)` - Load CSV files ✅
- [x] `SHOW SOURCES` - List registered sources ✅
- [x] `SHOW TABLES [FROM source]` - List tables/datasets ✅
- [x] `DESCRIBE source.table` - Show table schema ✅
- [x] `SHOW VARS` - Display session variables ✅
- [x] `LET variable = value` - Set session variables ✅
- [x] `UNSET variable...` - Remove session variables ✅

#### 3.5.4 OPTIONS Parser Enhancement
- [x] Quote handling in OPTIONS values ✅
- [x] Support for quoted delimiters: `delimiter=','` ✅
- [x] Single and double quote support ✅
- [x] Proper comma splitting respecting quotes ✅

#### 3.5.5 TUI Integration
- [x] RqlProcessor integration in TUI ✅
- [x] Thread-spawning parser to avoid Tokio conflicts ✅
- [x] NQL commands return SQL-style tables ✅
- [x] Status bar shows `source:table` format ✅
- [x] Table extraction from SQL commands ✅

#### 3.5.6 REPL Parity
- [x] Same thread-spawning fix for REPL ✅
- [x] All NQL commands work in REPL ✅
- [x] Debug logging throughout ✅

### Technical Implementation

#### Files Created/Modified (15 files)

**Core Changes:**
```
crates/core/src/
  ├── executor.rs - Added query routing to active source
  ├── datasource.rs - DataSource trait, SourceRegistry, SourceType
  └── csv_backend.rs - Complete CSV backend implementation
```

**Parser Changes:**
```
crates/parser/src/
  └── parser.rs - Enhanced OPTIONS parsing with quote support
```

**TUI Changes:**
```
crates/tui/src/
  └── noctra_tui.rs - RqlProcessor integration, NQL handlers, status bar
crates/tui/
  └── Cargo.toml - Added noctra-parser dependency
```

**REPL Changes:**
```
crates/cli/src/
  └── repl.rs - Thread-spawning parser, debug logging
```

**Examples:**
```
examples/
  └── clientes.csv - Test CSV file
```

### Commit History

| # | Commit | Description |
|---|--------|-------------|
| 1 | `0438e65` | Query routing in execute_rql() |
| 2 | `5b9940e` | RqlProcessor integration in TUI |
| 3 | `ae57113` | Fix Tokio runtime panic (TUI) |
| 4 | `9e64243` | OPTIONS parser + REPL runtime fix |
| 5 | `b65ca95` | Complete NQL command support in TUI |
| 6 | `dbddebc` | NQL commands as SQL-style tables |

### Features Demonstrated

**CSV Loading and Querying:**
```sql
-- Load CSV with options
USE './examples/clientes.csv' AS csv OPTIONS (delimiter=',', header=true);

-- Query like SQL
SELECT * FROM clientes;

-- Show metadata
SHOW SOURCES;
SHOW TABLES FROM csv;
DESCRIBE csv.clientes;
```

**Multi-Source Management:**
```sql
-- Register multiple sources
USE './data1.csv' AS csv1 OPTIONS (delimiter=',', header=true);
USE './data2.csv' AS csv2 OPTIONS (delimiter=';', header=true);

-- Switch between sources
SHOW SOURCES;  -- See all registered sources
```

**Session Variables:**
```sql
LET myvar = 'value';
SHOW VARS;
UNSET myvar;
```

### NQL Command Output Format

All NQL commands now return SQL-style tables:

| Command | Output Columns | Type |
|---------|---------------|------|
| `SHOW SOURCES` | Alias, Tipo, Path | Table |
| `SHOW TABLES` | table | Table |
| `DESCRIBE source.table` | Campos, Tipo | Table |
| `SHOW VARS` | Variable, Valor | Table |

**Status Bar Enhancement:**
- Before: `── Fuente: csv ──`
- After: `── Fuente: csv:clientes ──`

### Technical Challenges Solved

1. **"Failed to prepare" Error**
   - **Cause:** SQL queries routed to SQLite instead of CSV source
   - **Solution:** Query routing in `execute_rql()` to check active source first

2. **Tokio Runtime Panic**
   - **Cause:** Creating runtime within existing runtime context
   - **Solution:** Spawn dedicated thread with isolated runtime for parsing

3. **OPTIONS Parsing with Commas**
   - **Cause:** Split by comma broke quoted values like `delimiter=','`
   - **Solution:** Added `split_options()` that respects quote boundaries

4. **TUI/REPL Disparity**
   - **Cause:** TUI used `execute_sql()`, REPL used `execute_rql()`
   - **Solution:** Both now use RqlProcessor and execute_rql()

### Performance & Testing

**Build:**
- Clean build: ~18s
- Incremental: ~8s
- No warnings in release mode

**Testing:**
```bash
# Manual testing performed
./target/release/noctra repl
./target/release/noctra tui

# All functionality tested:
✅ CSV loading
✅ CSV querying
✅ NQL commands (SHOW, DESCRIBE, etc.)
✅ Multi-source switching
✅ Session variables
✅ Error handling
✅ Status bar display
```

### Limitations & Known Issues

**Current CSV Backend:**
- ✅ Supports: `SELECT * FROM table`
- ❌ Not yet: `WHERE`, `JOIN`, `GROUP BY`, `ORDER BY`
- ❌ Not yet: Column-specific SELECTs
- ❌ Not yet: INSERT/UPDATE/DELETE on CSV

**Workaround:** For complex queries, load CSV into SQLite:
```sql
-- Future M4 feature (not implemented yet)
IMPORT 'data.csv' AS temp;
INSERT INTO sqlite_table SELECT * FROM temp;
```

### Lines of Code Added

| Component | Lines Added | Functionality |
|-----------|-------------|---------------|
| csv_backend.rs | ~420 | Complete CSV backend |
| datasource.rs | ~250 | Multi-source management |
| noctra_tui.rs | ~300 | NQL handlers, status bar |
| parser.rs | ~80 | OPTIONS quote handling |
| repl.rs | ~50 | Thread-spawning parser |
| **Total** | **~1100** | **Complete CSV/NQL support** |

### Documentation Updates

- [ ] Update GETTING_STARTED.md with CSV examples → **TODO**
- [ ] Create CHANGELOG.md entry → **TODO**
- [x] Update PROJECT_STATUS.md (this section) ✅
- [ ] Update ROADMAP.md to reflect M3.5 completion → **TODO**

### Impact on M4

This hotfix **accelerates M4** by implementing ~40% of section 4.10 (NQL):

**From M4.10 - Already Implemented:**
- [x] USE command
- [x] SHOW SOURCES
- [x] SHOW TABLES
- [x] DESCRIBE
- [x] LET/UNSET/SHOW VARS
- [x] CSV backend
- [x] Multi-source registry

**Still Pending for M4:**
- [ ] IMPORT/EXPORT commands
- [ ] MAP/FILTER transformations
- [ ] JSON backend
- [ ] Memory backend
- [ ] Advanced CSV queries (WHERE, JOIN)
- [ ] Pipeline transformations

### Success Metrics

✅ **6 commits** in 1 day
✅ **~1100 lines** of production code
✅ **Zero test failures**
✅ **Zero compiler warnings**
✅ **100% feature parity** between REPL and TUI for NQL
✅ **Complete CSV support** with auto-detection
✅ **Professional UX** with SQL-style tables

**Commit final M3.5:** `dbddebc - feat: Convert NQL commands to SQL-style table results`
**Fecha:** 2025-11-09

---

## 🚧 Milestone 4 - Advanced Features + NQL [EN PROGRESO - 25%]

**Fecha Inicio:** 2025-11-11
**Duración Estimada:** 3-4 semanas (dividido en fases)
**Progreso:** **Fase 1 completada (25%)** - IMPORT/EXPORT funcionales

### 🎯 Objetivos del Milestone

Implementar comandos avanzados NQL (IMPORT, EXPORT, MAP, FILTER) y mejorar el CSV backend con soporte para operaciones SQL complejas. Este milestone se divide en 2 fases principales.

#### 4.1 Editor Avanzado
- [ ] Syntax highlighting SQL/RQL
- [ ] Autocompletado inteligente
  - Nombres de tablas
  - Nombres de columnas
  - Palabras clave SQL
- [ ] Multi-line editing mejorado
- [ ] Búsqueda en editor (Ctrl+F)
- [ ] Reemplazar texto (Ctrl+H)

#### 4.2 File Operations
- [ ] **Alt+R:** Leer query desde archivo
- [ ] **Alt+W:** Guardar query en archivo
- [ ] Abrir múltiples archivos
- [ ] Buffer de archivos recientes
- [ ] Auto-save de sesión

#### 4.3 Help System
- [ ] **F1:** Sistema de ayuda contextual
- [ ] Ayuda según modo actual
- [ ] Referencia SQL/RQL
- [ ] Atajos de teclado
- [ ] Ejemplos de uso

#### 4.4 History Management
- [ ] Persistencia de historial en disco
- [ ] Búsqueda en historial (Ctrl+R)
- [ ] Favoritos de queries
- [ ] Exportar historial
- [ ] Limitar tamaño de historial

#### 4.5 Result Mode Enhancements
- [ ] Scroll horizontal/vertical
- [ ] Ordenar columnas (click en header)
- [ ] Filtrar resultados
- [ ] Seleccionar filas
- [ ] Copiar celdas/filas
- [ ] Resaltar valores NULL

#### 4.6 Data Export/Import
- [ ] Exportar resultados a CSV/JSON/XLSX
- [ ] Importar datos desde archivos
- [ ] Copiar resultados al clipboard
- [ ] Guardar queries ejecutadas

#### 4.7 Schema Management
- [ ] Comando `use <schema>` para cambiar BD
- [ ] Mostrar esquema actual en header
- [ ] Listar tablas con `show tables`
- [ ] Describir tabla con `desc <table>`
- [ ] Soporte para múltiples conexiones simultáneas

#### 4.8 Transaction Support
- [ ] Soporte completo para transacciones
- [ ] BEGIN/COMMIT/ROLLBACK
- [ ] Indicador visual de transacción activa
- [ ] Auto-rollback en errores

#### 4.9 Performance
- [ ] Streaming de resultados grandes
- [ ] Lazy loading de filas
- [ ] Caché de resultados recientes
- [ ] Ejecución async de queries
- [ ] Cancelación de queries largas (F8)

#### 4.10 NQL - Noctra Query Language (Extensión Conceptual)

**Objetivo:** Extender RQL con un dialecto unificado que permita trabajar con múltiples fuentes de datos (SQLite, CSV, archivos planos) usando la misma sintaxis.

**Visión:** El usuario debe poder consultar una base de datos SQLite, un archivo CSV o un dataset en memoria con los mismos comandos, sin distinguir el origen.

##### A. Administración de Fuentes de Datos

- [ ] **`USE <path> [AS alias];`** - Cambiar o cargar fuente de datos (BD o archivo)
  ```sql
  USE 'clientes.csv' AS csv;
  USE 'demo.db' AS demo;
  ```

- [ ] **`SHOW SOURCES;`** - Listar todas las fuentes disponibles
  ```
  +----------+-----------------+
  | Alias    | Tipo            |
  |----------|-----------------|
  | demo     | sqlite          |
  | csv      | csv (archivo)   |
  +----------+-----------------+
  ```

- [ ] **Soporte para fuentes CSV**
  - Detector automático de delimitadores (`,` `;` `\t`)
  - Inferencia de tipos de columnas
  - Manejo de headers y encoding

##### B. Inspección y Metadatos

- [ ] **`SHOW TABLES;`** - Listar tablas o datasets de la fuente actual
- [ ] **`SHOW <table>;`** - Describir columnas/campos (nombre, tipo, tamaño, nulos)
  ```sql
  SHOW demo.provin;
  SHOW csv.clientes;
  ```

- [ ] **`DESCRIBE <source>.<table>;`** - Alias para SHOW con más detalle

##### C. Operaciones de Importación/Exportación

- [ ] **`IMPORT <archivo> AS <tabla>;`** - Cargar dataset plano a fuente actual
  ```sql
  IMPORT 'ventas.csv' AS ventas;
  IMPORT 'datos.json' AS json_data;
  ```

- [ ] **`EXPORT <tabla> TO <archivo>;`** - Exportar datos a CSV/JSON
  ```sql
  EXPORT empleados TO 'export.csv';
  EXPORT resultados TO 'output.json';
  ```

- [ ] **Soporte para formatos**
  - CSV (con delimitador configurable)
  - JSON (pretty y compacto)
  - XLSX (opcional, Milestone 5)

##### D. Manipulación Declarativa y Transformación

- [ ] **`MAP <expresión>`** - Transformar datos en memoria
  ```sql
  MAP UPPER(nombre);
  MAP CONCAT(apellido, ', ', nombre);
  ```

- [ ] **`FILTER <condición>`** - Filtrar filas sin WHERE SQL
  ```sql
  FILTER edad > 30;
  FILTER pais IN ('AR', 'UY', 'CL');
  ```

- [ ] **Pipeline de transformaciones**
  ```sql
  USE 'datos.csv' AS src;
  FILTER edad > 25;
  MAP UPPER(nombre);
  SELECT * FROM src;
  ```

##### E. Sesiones, Variables y Entorno

- [ ] **`LET <variable> = <expresión>;`** - Definir variable local
  ```sql
  LET pais = 'AR';
  LET min_edad = 25;
  SELECT * FROM clientes WHERE country = $pais AND edad >= $min_edad;
  ```

- [ ] **`SHOW VARS;`** - Mostrar variables definidas
- [ ] **`UNSET <variable>;`** - Eliminar variable de sesión
- [ ] **Persistencia de variables** - Guardar/cargar variables entre sesiones

##### F. Semántica de Ejecución Unificada

**Concepto clave:** Toda fuente es ejecutable mediante un conjunto uniforme de operaciones:
- Lectura
- Filtrado
- Transformación
- Renderizado

**Ejemplo de uso unificado:**
```sql
-- Trabajar con CSV como si fuera una BD
USE 'clientes.csv' AS csv;
SELECT nombre, pais FROM csv WHERE pais = 'AR';

-- Cambiar a SQLite
USE 'demo.db' AS db;
SELECT * FROM db.empleados WHERE dept = 'IT';

-- Importar CSV a SQLite
USE 'demo.db';
IMPORT 'nuevos.csv' AS temp_import;
INSERT INTO empleados SELECT * FROM temp_import;
```

**Implementación interna:**
- Parser debe distinguir comandos NQL de SQL puro
- Executor debe tener abstracción `DataSource` trait:
  ```rust
  trait DataSource {
      fn query(&self, sql: &str) -> Result<ResultSet>;
      fn schema(&self) -> Result<Vec<Table>>;
      fn source_type(&self) -> SourceType;
  }

  enum SourceType {
      SQLite,
      CSV { delimiter: char, has_header: bool },
      JSON,
      Memory,
  }
  ```

##### G. TUI Contextual

- [ ] **Header contextual** - Mostrar fuente actual
  ```
  ──( RESULTADO ) SQL Noctra 0.1.0 ────── Fuente: csv://clientes.csv ───
  ──( COMANDO ) SQL Noctra 0.1.0 ──────── Fuente: sqlite://demo.db ─────
  ```

- [ ] **Estado de sesión visible**
  - Indicar tipo de fuente (SQL vs CSV)
  - Mostrar número de filas y columnas en resultados
  - Número de fuentes activas

- [ ] **Comandos dinámicos mejorados**
  - `Alt+R` carga SQL o CSV indistintamente
  - `Alt+W` exporta según formato seleccionado
  - `F5` ejecuta NQL o SQL según contexto

##### H. Compatibilidad y Prioridades

**Reglas de precedencia sintáctica:**

| Tipo de comando | Prioridad | Ejemplo                    | Comportamiento                       |
|-----------------|-----------|----------------------------|--------------------------------------|
| SQL puro        | Alta      | `SELECT * FROM users;`     | Ejecuta en fuente activa (SQLite)   |
| NQL puro        | Media     | `SHOW demo;`               | Describe esquema o dataset          |
| Híbrido         | Baja      | `USE file.csv; SELECT ...` | Interpreta USE → cambia contexto    |

**Compatibilidad:**
- [x] SQL estándar (100% compatible)
- [ ] NQL extensions (nuevos comandos)
- [ ] Retrocompatibilidad total con RQL actual

##### I. Casos de Uso Completos

**Caso 1: Análisis de CSV**
```sql
USE 'ventas_2024.csv' AS ventas;
SHOW ventas;  -- Ver columnas
SELECT producto, SUM(cantidad) as total
FROM ventas
GROUP BY producto
ORDER BY total DESC;
EXPORT ventas TO 'resumen.json';
```

**Caso 2: Migración de datos**
```sql
USE 'legacy.csv' AS legacy;
USE 'new.db' AS target;
IMPORT 'legacy.csv' AS staging;
INSERT INTO target.clientes
  SELECT id, nombre, UPPER(pais) FROM staging WHERE active = 1;
```

**Caso 3: Transformación y filtrado**
```sql
USE 'clientes.csv';
LET min_age = 18;
FILTER edad >= $min_age;
MAP TRIM(nombre);
SELECT * FROM clientes WHERE pais IN ('AR', 'UY');
```

##### J. Arquitectura Técnica Requerida

**Nuevos componentes:**
```
crates/
├── core/
│   └── src/
│       ├── datasource.rs      # Trait DataSource + implementaciones
│       ├── csv_backend.rs     # Backend para CSV
│       └── memory_backend.rs  # Backend en memoria
├── parser/
│   └── src/
│       ├── nql_parser.rs      # Parser NQL extensions
│       └── nql_ast.rs         # AST para comandos NQL
└── cli/
    └── src/
        └── nql_executor.rs    # Executor unificado NQL+SQL
```

**Dependencias nuevas:**
- `csv` crate - Parser CSV
- `serde_json` - Export JSON (ya incluido)
- `encoding_rs` - Detección de encoding

**Estimado M4 con NQL:** 4-6 semanas

---

## ✅ Milestone 5 - Extended Capabilities [COMPLETADO]

**Fecha Inicio:** 2025-11-11
**Fecha Fin:** 2025-11-11
**Duración:** 1 día
**Progreso:** **100% Completado**

### 🎯 Objetivos del Milestone

Implementar capacidades SQL avanzadas para el CSV Backend, incluyendo WHERE extensions, GROUP BY con agregaciones, y soporte para importación de JSON.

### Características Implementadas

#### 5.1 WHERE Extensions ✅

**Implementación:** CSV Backend ahora soporta operadores WHERE avanzados

**Operadores Nuevos:**
- `LIKE` - Pattern matching con wildcards SQL (% y _)
  ```sql
  SELECT * FROM users WHERE nombre LIKE 'Juan%';
  ```
- `IN` - Listas de valores
  ```sql
  SELECT * FROM productos WHERE categoria IN ('Electrónica', 'Ropa');
  ```
- `BETWEEN` - Rangos de valores
  ```sql
  SELECT * FROM ventas WHERE fecha BETWEEN '2024-01-01' AND '2024-12-31';
  ```
- `IS NULL` / `IS NOT NULL` - Verificación de nulos
  ```sql
  SELECT * FROM clientes WHERE email IS NOT NULL;
  ```

**Archivos Modificados:**
- `crates/core/src/csv_backend.rs` (~260 líneas añadidas)
  - Método `evaluate_where_condition()` mejorado
  - `match_like_pattern()` para wildcards SQL
  - `wildcard_match_recursive()` para matching complejo
  - Soporte para operadores compuestos con AND/OR

**Ejemplo de Uso:**
```sql
USE 'clientes.csv' AS csv;
SELECT * FROM clientes
WHERE nombre LIKE 'A%'
  AND edad BETWEEN 25 AND 50
  AND email IS NOT NULL
  AND pais IN ('AR', 'UY', 'CL');
```

#### 5.2 GROUP BY con Agregaciones ✅

**Implementación:** Soporte completo para consultas GROUP BY con funciones de agregación

**Funciones de Agregación:**
- `COUNT()` - Contar filas
- `SUM()` - Suma de valores
- `AVG()` - Promedio
- `MIN()` - Valor mínimo
- `MAX()` - Valor máximo

**Cláusulas Soportadas:**
- `GROUP BY` - Agrupación por una o más columnas
- `HAVING` - Filtrado post-agregación (implementación básica)
- `ORDER BY` - Ordenamiento de resultados agrupados
- `LIMIT` - Limitación de resultados

**Archivos Modificados:**
- `crates/core/src/csv_backend.rs` (~400 líneas añadidas)
  - Enum `ParsedQuery::GroupBy` para queries de agrupación
  - Enum `SelectColumn` para columnas mixtas (regulares + agregados)
  - Trait `Clone` para `AggregateFunction`
  - Método `parse_group_by_query()` para parseo
  - Método `execute_group_by()` con HashMap para agrupación
  - Método `calculate_aggregate_for_group()` para cálculos
  - Método `apply_order_by_on_results()` para ordenamiento

**Ejemplo de Uso:**
```sql
USE 'ventas.csv' AS ventas;

SELECT
    region,
    producto,
    COUNT(*) AS total_ventas,
    SUM(monto) AS monto_total,
    AVG(monto) AS monto_promedio
FROM ventas
GROUP BY region, producto
HAVING total_ventas > 10
ORDER BY monto_total DESC
LIMIT 20;
```

**Implementación Técnica:**
- Agrupación usando `HashMap<Vec<String>, Vec<Vec<Value>>>`
- Claves compuestas para GROUP BY múltiple
- Cálculo de agregados por grupo
- Soporte para mezclar columnas regulares y agregadas en SELECT

#### 5.3 JSON Import ✅

**Implementación:** Importación completa de archivos JSON a tablas SQLite

**Características:**
- Parseo de JSON (array de objetos)
- Inferencia automática de tipos de columnas
- Soporte para tipos: INTEGER, REAL, TEXT, BOOLEAN
- Manejo de valores NULL
- Arrays y objetos anidados como JSON strings
- Validación de estructura JSON

**Archivos Modificados:**
- `crates/cli/src/repl.rs` (~103 líneas añadidas)
- `crates/tui/src/noctra_tui.rs` (~103 líneas añadidas)

**Tipos de Datos Soportados:**
| JSON Type | SQLite Type | Conversión |
|-----------|-------------|------------|
| Number (int) | INTEGER | Directo |
| Number (float) | REAL | Directo |
| Boolean | INTEGER | 1/0 |
| String | TEXT | Directo |
| null | TEXT/NULL | NULL |
| Array | TEXT | JSON string |
| Object | TEXT | JSON string |

**Ejemplo de Uso:**
```sql
-- JSON: [{"id": 1, "nombre": "Juan", "activo": true, "edad": 30}]
IMPORT 'usuarios.json' AS usuarios;

SELECT * FROM usuarios WHERE activo = 1 AND edad > 25;
```

**Validaciones de Seguridad:**
- Tamaño máximo de archivo: 100MB
- Validación de estructura (debe ser array de objetos)
- Sandboxing de rutas de archivo
- Validación de nombres de tabla
- Escape de valores SQL

### Archivos Clave M5

```
crates/core/src/
  └── csv_backend.rs (~660 líneas añadidas)
      ├── evaluate_where_condition() - WHERE extensions
      ├── match_like_pattern() - LIKE pattern matching
      ├── parse_group_by_query() - GROUP BY parsing
      ├── execute_group_by() - GROUP BY execution
      └── calculate_aggregate_for_group() - Agregaciones

crates/cli/src/
  └── repl.rs (~103 líneas añadidas)
      └── handle_import() - JSON import en REPL

crates/tui/src/
  └── noctra_tui.rs (~103 líneas añadidas)
      └── handle_import() - JSON import en TUI
```

### Estadísticas de Desarrollo

| Métrica | Valor |
|---------|-------|
| **Duración** | 1 día |
| **Líneas Añadidas** | ~866 líneas |
| **Archivos Modificados** | 3 |
| **Nuevas Funcionalidades** | 3 principales |
| **Build Time** | 9.36s (release) |
| **Warnings** | 7 (menores, no críticos) |
| **Tests** | 29 pasando (100%) |

### Lecciones Aprendidas M5

1. **Arquitectura Modular**: La separación clara entre parsing y ejecución facilitó la implementación de GROUP BY
2. **HashMap para Agrupación**: Usar claves compuestas (`Vec<String>`) permite GROUP BY de múltiples columnas eficientemente
3. **Type Inference JSON**: La inferencia del primer objeto es suficiente para la mayoría de casos
4. **Pattern Matching**: Implementación recursiva de wildcards es más flexible que regex directo
5. **Seguridad por Defecto**: Validaciones de tamaño y sandboxing evitan vulnerabilidades
6. **Parity TUI/REPL**: Mantener idéntica la lógica de IMPORT en ambas interfaces reduce bugs

### Ejemplos de Uso Completos

#### Ejemplo 1: Análisis de Ventas con GROUP BY
```sql
USE 'ventas_2024.csv' AS ventas;

-- Ventas por región y categoría
SELECT
    region,
    categoria,
    COUNT(*) AS num_ventas,
    SUM(monto) AS total,
    AVG(monto) AS promedio,
    MIN(monto) AS min_venta,
    MAX(monto) AS max_venta
FROM ventas
WHERE fecha BETWEEN '2024-01-01' AND '2024-12-31'
  AND monto > 0
  AND estado NOT IN ('cancelado', 'reembolsado')
GROUP BY region, categoria
HAVING num_ventas > 100
ORDER BY total DESC
LIMIT 10;
```

#### Ejemplo 2: Importación y Consulta de JSON
```bash
# Archivo: usuarios.json
[
  {"id": 1, "nombre": "Ana García", "edad": 28, "activo": true, "ciudad": "Buenos Aires"},
  {"id": 2, "nombre": "Carlos López", "edad": 35, "activo": true, "ciudad": "Montevideo"},
  {"id": 3, "nombre": "María Rodríguez", "edad": 42, "activo": false, "ciudad": "Santiago"}
]
```

```sql
-- Importar JSON
IMPORT 'usuarios.json' AS usuarios;

-- Consultar con WHERE extensions
SELECT * FROM usuarios
WHERE nombre LIKE '%García%'
  AND edad BETWEEN 25 AND 40
  AND activo = 1
  AND ciudad IN ('Buenos Aires', 'Montevideo');
```

#### Ejemplo 3: Workflow Completo CSV + JSON
```sql
-- Cargar múltiples fuentes
USE 'productos.csv' AS productos;
IMPORT 'categorias.json' AS categorias;

-- Consulta combinada
SELECT
    p.nombre,
    p.precio,
    c.categoria_nombre
FROM productos p
JOIN categorias c ON p.categoria_id = c.id
WHERE p.precio BETWEEN 1000 AND 5000
  AND p.stock > 0
GROUP BY c.categoria_nombre
ORDER BY COUNT(*) DESC;
```

### Próximos Pasos

Con M5 completado, Noctra ahora tiene:
✅ CSV backend avanzado con consultas SQL completas
✅ GROUP BY y agregaciones funcionales
✅ WHERE extensions (LIKE, IN, BETWEEN, IS NULL)
✅ JSON import nativo
✅ Multi-source queries (CSV + SQLite + JSON)

**Siguiente Milestone:** M6 - Noctra 2.0 "FABRIC" (DuckDB Integration)

---

## 🎯 Milestone 6 - Noctra(🦆) "FABRIC" [EN PROGRESO]

**Fecha de Inicio:** 11 de noviembre de 2025
**Duración:** 6 semanas (11 nov — 23 dic 2025)
**Versión Target:** v0.6.0
**Estado:** 📋 Fase 1 - Fundación

### Vision Statement

> **"Transformar Noctra de 'entorno SQL interactivo' a 'entorno 4GL de análisis de datos sobre DuckDB'"**
> **"Los archivos son tablas, el staging desaparece, y el análisis es instantáneo"**

### Objetivo Estratégico

Reemplazar el backend CSV manual con DuckDB como motor universal, habilitando:
- 🦆 **Queries directos sobre archivos** sin IMPORT/staging
- ⚡ **Performance 10x superior** con zero-copy y lectura columnar
- 🔗 **JOINs cross-source** nativos (CSV + Parquet + SQLite)
- 📦 **Soporte Parquet** para datasets grandes
- 🎯 **Modo híbrido por defecto**: DuckDB para archivos, SQLite para persistencia

### Transformación Radical

| Antes (Pre-M6) | Después (M6 - FABRIC) |
|----------------|------------------------|
| `IMPORT` → staging → query | `USE 'file.csv'` → query directo |
| `csv_backend.rs` (900+ líneas) | **Eliminado** — DuckDB lo reemplaza |
| JOIN entre CSV imposible | JOIN nativo entre CSV, Parquet, SQLite |
| Máximo 100MB por archivo | Streaming ilimitado (zero-copy) |
| SQLite como motor único | **DuckDB como motor por defecto** |
| `MAP`, `FILTER` redundantes | **Deprecados** — SQL estándar es superior |

---

## Fases de Implementación (6 Semanas)

### **FASE 1: FUNDACIÓN — Integración DuckDB** (Semana 1)
**Objetivo:** Reemplazar el backend CSV manual con DuckDB como motor universal.

**Tareas Principales:**
- [ ] Crear crate `noctra-duckdb` con estructura base
- [ ] Implementar `DuckDBEngine` implementando trait `DataSource`
- [ ] `USE 'file.csv' AS alias` → `CREATE VIEW alias AS SELECT * FROM read_csv_auto('path')`
- [ ] Soporte nativo: CSV, Parquet, JSON (DuckDB functions)
- [ ] **Eliminar** `crates/core/src/csv_backend.rs` (deprecado)
- [ ] Feature flag `duckdb-engine` para compilación condicional
- [ ] Tests básicos: cargar CSV, query simple, schema introspection

**Resultado Esperado:**
```sql
USE 'ventas.csv' AS v;
SELECT * FROM v LIMIT 5;
-- DuckDB ejecuta sin staging
```

**Archivos Clave:**
```
crates/noctra-duckdb/
  ├── Cargo.toml
  └── src/
      ├── lib.rs
      ├── source.rs       # DuckDBSource impl
      ├── engine.rs       # Query execution
      └── extensions.rs   # Parquet, JSON support
```

---

### **FASE 2: MOTOR HÍBRIDO — DuckDB + SQLite** (Semana 2)
**Objetivo:** Modo híbrido por defecto: DuckDB para archivos, SQLite para persistencia.

**Tareas Principales:**
- [ ] Implementar `QueryEngine::Hybrid { duckdb, sqlite }`
- [ ] Routing inteligente: `*.csv`, `*.parquet` → DuckDB; `*.db` → SQLite
- [ ] `ATTACH` automático: `ATTACH 'db.db' AS sqlite_db (TYPE SQLITE)`
- [ ] `JOIN` cross-source: CSV + SQLite sin `IMPORT`
- [ ] Configuración `~/.config/noctra/config.toml`
  ```toml
  [engine]
  default = "hybrid"
  ```
- [ ] Tests: cross-source JOIN, routing logic, ATTACH

**Resultado Esperado:**
```sql
USE 'ventas.csv' AS v;
USE 'clientes.db' AS c;

SELECT c.nombre, v.total
FROM v JOIN c.clientes ON v.id = c.id;
-- DuckDB une ambos sin staging
```

---

### **FASE 3: RQL 4GL — Extensionalidad Nativa** (Semana 3)
**Objetivo:** Consolidar las extensiones únicas de Noctra sobre DuckDB.

**Extensiones a Mantener:**
- [x] `LET var = valor` — Variables de sesión
- [x] `#var` en SQL — Interpolación de variables
- [x] `SHOW VARS` — Tabla de variables
- [x] `SHOW SOURCES` — Catálogo unificado de fuentes
- [x] `DESCRIBE source.table` — Introspección de esquema
- [x] `EXPORT TO 'file.json'` — Sintaxis humana para COPY

**Extensiones a Deprecar:**
- [ ] `MAP`, `FILTER` → **DEPRECATE** (redundante con SQL estándar)
- [ ] `OUTPUT TO` → **DEPRECATE** (reemplazado por `EXPORT`)
- [ ] `FORM LOAD` en parser → **Mover a `noctra-tui`** (no pertenece al parser)

**Tareas:**
- [ ] Actualizar parser para deprecar `MAP`, `FILTER`, `OUTPUT TO`
- [ ] Agregar warnings de deprecación
- [ ] Documentar migración en `MIGRATION.md`
- [ ] Tests: validar que `LET`, `#var`, `SHOW VARS` funcionan con DuckDB

**Resultado Esperado:**
```sql
LET pais = 'AR';
SHOW VARS;
-- pais = 'AR'

SELECT * FROM 'ventas.csv' WHERE pais = #pais;
EXPORT TO 'ar.json' FORMAT JSON;
```

---

### **FASE 4: EXPORT & OUTPUT — Unified Output Layer** (Semana 4)
**Objetivo:** `EXPORT` como comando maestro, `OUTPUT TO` eliminado.

**Tareas Principales:**
- [ ] `EXPORT query TO 'file' FORMAT csv/json/parquet`
  - Traduce a `COPY (...) TO 'file' (FORMAT ...)`
- [ ] Soporte multi-formato: CSV, JSON, Parquet
- [ ] `EXPORT * TO 'dir/'` — Export batch (opcional)
- [ ] Deprecar completamente `OUTPUT TO`
- [ ] `PRINT "msg"` — Mantener para debug
- [ ] Tests: export CSV, JSON, Parquet; validar formato

**Resultado Esperado:**
```sql
EXPORT (SELECT * FROM 'ventas.parquet') TO 'out.csv' FORMAT CSV;
EXPORT (SELECT * FROM clientes WHERE activo = 1) TO 'activos.json' FORMAT JSON;
```

---

### **FASE 5: TUI & UX — Data Fabric Experience** (Semana 5)
**Objetivo:** Interfaz que refleje el nuevo poder de DuckDB.

**Tareas Principales:**
- [ ] Status bar dinámico con engine indicator
  ```
  Engine: 🦆 DuckDB | Source: ventas.csv (CSV) | Memory: 45MB | 8ms
  ```
- [ ] Panel `SOURCES` con tipo, tamaño, filas
  ```
  ┌──────────┬─────────┬─────────┬──────────┐
  │ Alias    │ Type    │ Size    │ Rows     │
  ├──────────┼─────────┼─────────┼──────────┤
  │ ventas   │ 🦆 CSV  │ 1.2GB   │ 1.2M     │
  │ clientes │ 📦 SQLite│ 45MB   │ 50K      │
  └──────────┴─────────┴─────────┴──────────┘
  ```
- [ ] `F5: Run` ejecuta en engine correcto (auto-detect)
- [ ] Autocomplete de tablas desde DuckDB
- [ ] `noctra 'file.csv'` — Abre con `USE` automático
- [ ] Tests: TUI rendering, status bar, autocomplete

**Resultado Esperado:**
- Status bar muestra engine activo y fuente actual
- Panel de sources lista todas las fuentes registradas
- CLI acepta archivos directamente: `noctra ventas.csv`

---

### **FASE 6: RELEASE & DOCUMENTACIÓN — v0.6.0 "FABRIC"** (Semana 6)
**Objetivo:** Lanzamiento estable, documentación completa, migración clara.

**Tareas Principales:**
- [ ] Tag `v0.6.0` oficial
- [ ] `RQL_EXTENSIONS.md` — Manual de extensiones nativas
  - Documentar `LET`, `#var`, `SHOW VARS`, `SHOW SOURCES`, `EXPORT`
  - Marcar `MAP`, `FILTER`, `OUTPUT TO` como deprecados
- [ ] `MIGRATION.md` — Guía de migración de M5 a M6
  - Cómo migrar queries que usaban `csv_backend`
  - Alternativas a `MAP`, `FILTER`
- [ ] Benchmarks: 1GB CSV: DuckDB vs SQLite
  - Cargar CSV
  - JOIN cross-source
  - GROUP BY con agregaciones
- [ ] Feature flags en `Cargo.toml`
  - `duckdb-engine` (default)
  - `sqlite-fallback`
- [ ] CHANGELOG.md con breaking changes
- [ ] Tests de regresión completos

**Documentación a Crear:**
```
docs/
  ├── RQL_EXTENSIONS.md        # ← NUEVO
  ├── MIGRATION.md             # ← NUEVO
  └── M6_IMPLEMENTATION_PLAN.md # ← NUEVO
```

---

## Depreciaciones y Eliminaciones

### **Código a Eliminar:**
- [ ] `crates/core/src/csv_backend.rs` (900+ líneas) → **DuckDB lo reemplaza**
- [ ] Tests relacionados con `csv_backend.rs`
- [ ] Dependencia `csv` crate (si no se usa en otro lugar)

### **Comandos a Deprecar:**
- [ ] `MAP expression` → **Use SQL SELECT** con expresiones
- [ ] `FILTER condition` → **Use SQL WHERE** clause
- [ ] `OUTPUT TO 'file'` → **Use EXPORT TO 'file' FORMAT ...**
- [ ] `IMPORT 'file' AS table` → **Use USE 'file' AS alias** (opcional mantener para staging)

### **Sintaxis Nueva:**
- [x] `USE 'file.csv' AS alias` — Ya implementado en M3.5, ahora usa DuckDB
- [ ] `EXPORT query TO 'file' FORMAT format` — Nueva sintaxis unificada
- [ ] `SELECT * FROM 'file.csv'` — Query directo sin registro (DuckDB native)

---

## Requerimientos Técnicos

### **Nuevo Crate: `noctra-duckdb`**
```toml
[package]
name = "noctra-duckdb"
version = "0.6.0"

[dependencies]
duckdb = { version = "1.1", features = ["bundled", "parquet", "json"] }
noctra-core = { path = "../noctra-core" }
anyhow = "1.0"
log = "0.4"
```

### **Trait `DataSource` → `DuckDBSource`**
```rust
pub struct DuckDBSource {
    conn: duckdb::Connection,
    name: String,
}

impl DataSource for DuckDBSource {
    fn query(&self, sql: &str, params: &Parameters) -> Result<ResultSet>;
    fn schema(&self) -> Result<Vec<TableInfo>>;
    fn source_type(&self) -> SourceType;
    fn name(&self) -> &str;
}
```

### **Feature Flags**
```toml
[features]
default = ["duckdb-engine"]
duckdb-engine = ["noctra-duckdb"]
sqlite-fallback = []
```

### **Configuración**
```toml
# ~/.config/noctra/config.toml
[engine]
default = "hybrid"  # duckdb, sqlite, hybrid

[duckdb]
temp_dir = "/tmp/noctra-duckdb"
memory_limit = "2GB"
threads = 4

[duckdb.extensions]
auto_install = true
enabled = ["parquet", "json"]
```

---

## Criterios de Éxito

### **Funcionales:**
- ✅ `USE 'file.csv' AS alias` carga archivo sin staging
- ✅ `SELECT * FROM 'file.csv'` funciona directamente
- ✅ JOIN entre CSV y SQLite sin IMPORT
- ✅ EXPORT a CSV, JSON, Parquet
- ✅ Modo híbrido por defecto (DuckDB + SQLite)
- ✅ `LET`, `#var`, `SHOW VARS` funcionan con DuckDB

### **Performance:**
- ✅ CSV 1GB carga en <2s (vs ~30s con csv_backend)
- ✅ JOIN 100K rows: <1s
- ✅ GROUP BY con agregaciones: <500ms
- ✅ Memoria: <200MB para 1GB CSV (streaming)

### **Calidad:**
- ✅ Test coverage: >85%
- ✅ Zero clippy warnings
- ✅ Documentación completa (RQL_EXTENSIONS.md, MIGRATION.md)
- ✅ Benchmarks publicados

---

## Comando Final del Usuario

```bash
noctra 'ventas.parquet'
```

```sql
-- Automático: USE 'ventas.parquet' AS v
LET pais = 'AR';

SELECT region, SUM(total) AS total
FROM v
WHERE pais = #pais
GROUP BY region
ORDER BY total DESC;

EXPORT TO 'resumen_ar.json' FORMAT JSON;
```

---

## 🎯 Milestone 7 - "SCRIPT" [PLANIFICADO]

**Fecha de Inicio:** 24 de diciembre de 2025 (Post-M6)
**Duración:** 6 semanas (24 dic 2025 — 3 feb 2026)
**Versión Target:** v0.7.0
**Estado:** 📋 Planificado

### Vision Statement

> **"Convertir RQL en un 4GL completo con capacidades de scripting"**
> **"De query language a programming language para datos"**

### Objetivo Estratégico

Agregar capacidades de scripting 4GL a Noctra, transformándolo de un entorno de queries a un lenguaje de programación completo para análisis de datos.

### ¿Qué es parte de M7 (NO M6)?

| Extensión | Descripción | Complejidad |
|-----------|-------------|-------------|
| `IF/THEN/ELSE` | Control de flujo condicional | Medium |
| `FOR ... IN ... DO` | Bucles sobre resultados | Medium |
| `MACRO ... AS ... END` | Definir macros reutilizables | High |
| `CALL macro(args)` | Invocar macros | Medium |
| `RUNSUM()`, `RUNAVG()` | Funciones de ventana simplificadas | Low |
| `GRAPH BAR`, `GRAPH LINE` | Visualización ASCII | Medium |
| `SAVE SESSION`, `LOAD SESSION` | Persistencia de estado | Medium |
| `PRINT "msg"` | Debug output | Low |
| `PIPE TO 'cmd'` | Canalización a shell | Low |
| `WHENEVER ERROR THEN` | Manejo de errores global | Medium |
| `IMPORT MACRO FROM 'file'` | Librerías de macros | High |

**IMPORTANTE:** Estas características NO son parte de M6. M6 se enfoca exclusivamente en DuckDB integration.

### Fases de Implementación (6 Semanas)

#### **FASE 1: SCRIPTING CORE** (Semana 1)
- [ ] `IF/THEN/ELSE` control flow
- [ ] `FOR...IN...DO` loops
- [ ] `PRINT` debug output

**Ejemplo:**
```rql
IF #pais = 'AR' THEN
  PRINT "Procesando Argentina";
  USE 'ventas_ar.csv' AS v;
ELSE
  USE 'ventas_latam.csv' AS v;
END;

FOR region IN (SELECT DISTINCT region FROM v) DO
  PRINT "Región:", region.region;
END;
```

#### **FASE 2: MACROS & REUTILIZACIÓN** (Semana 2)
- [ ] `MACRO name(params) AS ... END`
- [ ] `CALL macro(args)`
- [ ] `IMPORT MACRO FROM 'file'`

**Ejemplo:**
```rql
MACRO top_productos(n, region) AS
  SELECT producto, SUM(total) AS ventas
  FROM ventas
  WHERE region = :region
  GROUP BY producto
  ORDER BY ventas DESC
  LIMIT :n;
END;

CALL top_productos(10, 'LATAM');

IMPORT MACRO FROM 'analytics.rql';
```

#### **FASE 3: AGREGADOS & VISUALIZACIÓN** (Semana 3)
- [ ] `RUNSUM()`, `RUNAVG()`, `RUNCOUNT()`
- [ ] `GRAPH BAR FROM query`
- [ ] `GRAPH LINE FROM query`
- [ ] `GRAPH HIST FROM query`

**Ejemplo:**
```rql
SELECT
  fecha,
  ventas,
  RUNSUM(ventas) AS acumulado
FROM ventas_diarias
ORDER BY fecha;

GRAPH BAR FROM (
  SELECT region, SUM(total) FROM ventas GROUP BY region
);
```

#### **FASE 4: SESIÓN PERSISTENTE** (Semana 4)
- [ ] `SAVE SESSION 'file.toml'`
- [ ] `LOAD SESSION 'file.toml'`
- [ ] Auto-save al salir
- [ ] Prompt de restauración al iniciar

**Ejemplo:**
```rql
LET pais = 'AR';
USE 'ventas.csv' AS v;
MACRO top(n) AS SELECT * FROM v LIMIT :n; END;

SAVE SESSION 'mi_sesion.toml';

-- Nueva sesión
LOAD SESSION 'mi_sesion.toml';
CALL top(5);  -- Todo restaurado
```

#### **FASE 5: SALIDA & CANALIZACIÓN** (Semana 5)
- [ ] `query PIPE TO 'cmd'`
- [ ] `query > 'file'`, `query >> 'file'`
- [ ] `WHENEVER ERROR THEN` error handling
- [ ] Variables `ERROR_MESSAGE`, `ERROR_CODE`, `ERROR_QUERY`
- [ ] Modos: `CONTINUE`, `EXIT`, `ROLLBACK`
- [ ] Validación de seguridad

**Ejemplo:**
```rql
WHENEVER ERROR THEN
  PRINT "ERROR:", ERROR_MESSAGE;
  CONTINUE;
END;

SELECT * FROM logs
WHERE level = 'ERROR'
PIPE TO 'grep "database"';

SELECT * FROM ventas > 'reporte.txt';
```

#### **FASE 6: RELEASE v0.7.0** (Semana 6)
- [ ] Tag `v0.7.0`
- [ ] `RQL_SCRIPTING.md` manual completo
- [ ] `MIGRATION_M6_TO_M7.md`
- [ ] `demo_full_script.rql`
- [ ] Benchmarks de scripting
- [ ] CHANGELOG.md

### Ejemplo Final Completo (M7)

```rql
-- demo_full_script.rql
IMPORT MACRO FROM 'analytics.rql';

LET pais = 'AR';
USE 'ventas_2025.csv' AS v;

MACRO resumen(pais) AS
  SELECT region, SUM(total) AS total
  FROM v
  WHERE pais = :pais
  GROUP BY region
  ORDER BY total DESC;
END;

CALL resumen(#pais);
GRAPH BAR FROM resumen(#pais);

FOR region IN (SELECT DISTINCT region FROM v WHERE pais = #pais) DO
  PRINT "Procesando:", region.region;
  EXPORT (SELECT * FROM v WHERE pais = #pais AND region = region.region)
  TO CONCAT('region_', region.region, '.json')
  FORMAT JSON;
END;

SAVE SESSION 'analisis_ar_2025.toml';
```

### Criterios de Éxito

**Funcionales:**
- ✅ IF/THEN/ELSE con condiciones complejas
- ✅ FOR itera sobre resultados
- ✅ MACRO define y llama correctamente
- ✅ IMPORT MACRO carga desde archivos
- ✅ RUNSUM traduce a window functions
- ✅ GRAPH BAR renderiza ASCII
- ✅ SAVE/LOAD SESSION preserva estado
- ✅ PIPE TO envía a shell
- ✅ WHENEVER ERROR THEN captura y maneja errores

**Performance:**
- ✅ Macros expanden en <1ms
- ✅ FOR sobre 1000 filas: <100ms
- ✅ Session save/load: <500ms
- ✅ GRAPH rendering: <50ms

**Calidad:**
- ✅ Test coverage: >80%
- ✅ Zero clippy warnings
- ✅ Documentación completa

### Documentación M7

Ver [M7_IMPLEMENTATION_PLAN.md](M7_IMPLEMENTATION_PLAN.md) para detalles completos de implementación.

---

## 📊 Estadísticas del Proyecto

- **Total Commits:** 50+
- **Total Líneas de Código:** ~12,000+
- **Total Tests:** 29 (100% pasando)
- **Crates Activos:** 6
- **Dependencies:** 30+
- **Build Time (release):** ~18s
- **Binary Size:** ~6MB

---

## 🗺️ Roadmap Visual

```
2025
├── Enero - Febrero
│   ├── ✅ M0: Foundation
│   ├── ✅ M1: Core + Parser (RQL)
│   └── ✅ M2: Forms + TUI
│
├── Marzo - Abril
│   ├── ✅ M3: Backend Integration (Completado Nov 2025)
│   └── 📋 M4: Advanced Features + NQL (SIGUIENTE)
│       ├── Editor avanzado
│       ├── File operations
│       ├── Help system
│       ├── NQL - Noctra Query Language ⭐ NUEVO
│       │   ├── Soporte CSV
│       │   ├── Múltiples fuentes de datos
│       │   ├── Comandos administrativos (USE, SHOW, IMPORT, EXPORT)
│       │   └── Transformaciones (MAP, FILTER)
│       └── Performance optimizations
│
└── Mayo - Junio
    └── 📋 M5: Production Ready
        ├── PostgreSQL/MySQL backends
        ├── Packaging y distribución
        └── Documentación completa
```

---

## 📚 Arquitectura Actual

```
noctra/
├── crates/
│   ├── core/          # ✅ SQL Executor + ResultSet
│   ├── parser/        # ✅ RQL Parser
│   ├── formlib/       # ✅ Declarative Forms
│   ├── tui/           # ✅ Ratatui Widgets + NoctraTui + Backend Integration
│   ├── cli/           # ✅ Commands + REPL + TUI launcher
│   └── ffi/           # ✅ C bindings
│
├── examples/
│   ├── forms/         # ✅ Form TOML examples
│   └── scripts/       # ✅ RQL script examples
│
└── docs/              # ✅ Documentation completa
    ├── PROJECT_STATUS.md (este archivo)
    ├── DESIGN.md
    ├── ROADMAP.md
    ├── API-REFERENCE.md
    ├── RQL-EXTENSIONS.md
    ├── FDL2-SPEC.md
    ├── FORMS.md
    ├── GETTING_STARTED.md
    └── CONTRIBUTING.md
```

---

## 🔧 Stack Tecnológico

| Capa | Tecnología |
|------|-----------|
| **Language** | Rust 2021 Edition |
| **TUI** | Ratatui 0.29 + tui-textarea |
| **Terminal** | Crossterm 0.28 |
| **Database** | SQLite (rusqlite 0.32) |
| **Parsing** | sqlparser 0.40 |
| **Serialization** | serde + toml + serde_json |
| **CLI** | clap 4.x |
| **Async** | tokio 1.48 |
| **Testing** | cargo test + tempfile |

---

## 🎯 Estado Actual y Próximos Pasos

### ✅ Lo que funciona AHORA

```bash
# CLI básico
noctra --help
noctra repl                    # REPL SQL básico
noctra query "SELECT * FROM users"
noctra info

# Formularios
noctra form preview examples/forms/employee_search.toml
noctra form exec examples/forms/employee_search.toml

# TUI Completo con Backend SQL Real ✨
noctra tui                     # In-memory database
noctra tui --database demo.db  # Persistent database
noctra tui --schema demo
```

### ⚠️ Limitaciones Actuales

- Sin syntax highlighting en editor
- Sin autocompletado
- Sin persistencia de historial
- Sin split panels
- Sin export/import de datos (CSV/JSON)
- Sin F1 help system
- Sin Alt+R/W file operations
- Sin soporte para transacciones explícitas
- Sin connection pooling

---

## 🎯 NOCTRA 2.0 "FABRIC" - VISIÓN Y PLANIFICACIÓN

### Vision Statement

> **"No importes datos. Consúltalos."**
> **"Un archivo. Una tabla. Un lenguaje."**
> **"Noctra no necesita una base de datos. Tú sí."**

### Objetivos Estratégicos

Noctra 2.0 "FABRIC" transformará Noctra en un **Data Fabric Engine** mediante la integración completa de DuckDB como motor de análisis ad hoc.

**🎯 Capacidad Central:** Consultar cualquier archivo (CSV, JSON, Parquet) como tabla SQL nativa sin staging, imports ni bases de datos obligatorias.

**🚀 Innovación Clave:** Los archivos se convierten en tablas. Las consultas son instantáneas. Las bases de datos se vuelven opcionales.

### Arquitectura Propuesta

#### Nuevo Crate: `noctra-duckdb`

```
noctra/
├── crates/
│   ├── noctra-core/           # + QueryEngine::DuckDB, Hybrid
│   ├── noctra-parser/         # + NQL 2.0 extensions
│   ├── noctra-duckdb/         # ← NUEVO (2 semanas)
│   │   ├── src/
│   │   │   ├── lib.rs         # Entry point
│   │   │   ├── source.rs      # DuckDBSource impl
│   │   │   ├── engine.rs      # Query execution
│   │   │   └── extensions.rs  # Parquet, JSON support
│   │   └── Cargo.toml
│   ├── noctra-tui/            # + barra de estado dinámica
│   └── noctra-cli/            # + --engine flag
```

**QueryEngine Evolution:**
```rust
pub enum QueryEngine {
    Sqlite(Box<dyn DatabaseBackend>),
    DuckDB(DuckDBConnection),        // ← NUEVO
    Hybrid {                          // ← NUEVO (default)
        duckdb: DuckDBConnection,
        sqlite: SqliteConnection
    },
}
```

### NQL 2.0 - Extensiones Clave

| Comando | Funcionalidad |
|---------|---------------|
| `USE 'file.csv' AS t` | Registro instantáneo de archivo como tabla |
| `SELECT * FROM 'file.csv'` | Consulta directa sin pre-registro |
| `EXPORT ... TO 'file.parquet'` | Export multi-formato (CSV, JSON, Parquet) |
| `MAP col = expr` | Transformaciones declarativas |
| `FILTER condition` | Filtrado sin WHERE SQL |
| JOINs cross-source | CSV ⟷ SQLite ⟷ JSON sin ETL |

**Ejemplo Completo:**
```sql
USE 'sales_*.csv' AS sales;    -- Multi-file glob
USE 'warehouse.db' AS db;       -- SQLite database

SELECT s.product, p.name, SUM(s.total)
FROM sales s
JOIN db.products p ON s.product_id = p.id
WHERE s.date >= '2024-01-01'
GROUP BY s.product, p.name;

EXPORT (SELECT * FROM sales WHERE region = 'LATAM')
TO 'latam.parquet' FORMAT PARQUET;
```

### Modos de Operación

```bash
# Ad Hoc: Solo DuckDB, sin base de datos
noctra --engine duckdb --use 'data.csv'

# Híbrido: SQLite + DuckDB (default)
noctra --engine hybrid --db warehouse.db --use 'recent.csv'

# Tradicional: Solo SQLite (retrocompatibilidad)
noctra --engine sqlite --db database.db
```

### TUI Enhancements

**Barra de Estado Dinámica:**
```
──( RESULT ) Noctra 2.0 ─── Engine: DuckDB ─── Source: 'ventas.csv' ─── 12ms
3 filas | Memory: 45MB | F5:Run | Ctrl+E:Export
```

**Indicadores de Fuente:**
```
┌─────────────────────────────────────────────────┐
│ 📊 ACTIVE SOURCES                               │
├──────────┬─────────┬──────────────────────────┤
│ ventas   │ 🦆 CSV  │ ./data/ventas_2024.csv    │
│ clientes │ 🦆 JSON │ ./data/clientes.json      │
│ main     │ 📦 SQLite│ ./database.db           │
└──────────┴─────────┴──────────────────────────┘
```

### Roadmap de Implementación

**Duration:** 2 semanas
**Target:** 2026-03-01
**Version:** v2.0.0

| Semana | Fase | Tareas Clave |
|--------|------|--------------|
| **1** | Core DuckDB | - Crate `noctra-duckdb`<br>- `DataSource` implementation<br>- `USE 'file.csv'` → CREATE VIEW<br>- Parser NQL 2.0 extensions |
| **2** | Integration | - EXPORT multi-formato<br>- TUI status bar dinámico<br>- CLI `--engine` flag<br>- Configuration system<br>- Modo ad hoc |

### Criterios de Éxito

**Funcionales:**
- ✅ Cargar CSV/JSON/Parquet con `USE`
- ✅ Consultas directas sobre archivos
- ✅ JOIN cross-source (CSV + SQLite)
- ✅ EXPORT a múltiples formatos
- ✅ Modo ad hoc sin base de datos

**Performance:**
- ✅ CSV 10MB en <500ms
- ✅ Agregación 100K filas en <1s
- ✅ Parquet 10x más rápido que CSV
- ✅ Memoria <100MB (workloads típicos)

**Calidad:**
- ✅ Coverage >90%
- ✅ Zero clippy warnings
- ✅ Documentación completa
- ✅ Migration guide de v1.0

### Impacto Esperado

**Casos de Uso Desbloqueados:**
1. **Análisis ad hoc** sin base de datos
2. **Pipelines ligeros** sin ETL complejo
3. **Exploración rápida** de datasets
4. **Prototipado** de queries sobre archivos
5. **Cross-source analytics** sin staging

**Diferenciación:**
- ❌ **Antes:** Import CSV → SQLite → Query (lento, staging requerido)
- ✅ **Después:** Query CSV directamente (instantáneo, zero-copy)

**Valor para Usuarios:**
- Reducción de 80% en tiempo de setup para análisis
- Eliminación de staging manual
- Soporte nativo de formatos modernos (Parquet)
- Análisis multi-fuente sin herramientas externas

---

### 🎯 Siguiente Acción Recomendada

**Comenzar M4** - Agregar características avanzadas del TUI y **NQL (Noctra Query Language)** para soporte multi-fuente.

**Prioridades M4:**
1. **NQL - Soporte CSV y múltiples fuentes** ⭐ NUEVO
   - Comandos administrativos (USE, SHOW SOURCES, IMPORT, EXPORT)
   - Backend CSV con detección automática
   - Transformaciones (MAP, FILTER)
   - Semántica unificada de ejecución
2. File operations (Alt+R/W)
3. Help system (F1)
4. History management persistente
5. Data export/import mejorado
6. Schema management
7. Transaction support
8. TUI contextual (mostrar fuente actual)

---

## 📝 Documentación del Proyecto

### Documentos Principales
- [PROJECT_STATUS.md](./PROJECT_STATUS.md) - Este archivo (estado consolidado)
- [DESIGN.md](./DESIGN.md) - Arquitectura técnica completa
- [ROADMAP.md](./ROADMAP.md) - Timeline de desarrollo
- [API-REFERENCE.md](./API-REFERENCE.md) - Referencia de API
- [RQL-EXTENSIONS.md](./RQL-EXTENSIONS.md) - Extensiones RQL
- [FDL2-SPEC.md](./FDL2-SPEC.md) - Especificación de formularios
- [FORMS.md](./FORMS.md) - Documentación de formularios
- [GETTING_STARTED.md](./GETTING_STARTED.md) - Guía de inicio rápido
- [CONTRIBUTING.md](./CONTRIBUTING.md) - Guía para contribuidores

### Documentos Históricos (Archivados)
- [archive/M3_IMPLEMENTATION_PLAN.md](./archive/M3_IMPLEMENTATION_PLAN.md) - Plan M3 (ejecutado)
- [archive/REPOSITORY_ANALYSIS.md](./archive/REPOSITORY_ANALYSIS.md) - Análisis inicial
- [archive/TESTING_REPORT.md](./archive/TESTING_REPORT.md) - Reporte de testing

---

## 📞 Referencias

- **GitHub**: https://github.com/wirednil/noctra
- **Issues**: https://github.com/wirednil/noctra/issues
- **Milestones**: Ver ROADMAP.md para detalles completos

---

**Noctra 0.1.0** - Entorno SQL interactivo moderno para la era Rust 🚀

**Última actualización de este documento:** 2025-11-09
