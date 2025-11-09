# Sistema de Formularios Noctra (FDL2)

> **Versión:** 1.0
> **Última actualización:** 2025-11-08
> **Estado:** Milestone 2 - En Desarrollo

## Tabla de Contenidos

1. [Introducción](#introducción)
2. [Arquitectura](#arquitectura)
3. [FDL2 - Form Definition Language](#fdl2---form-definition-language)
4. [FormGraph - Navegación Jerárquica](#formgraph---navegación-jerárquica)
5. [NWM - Noctra Window Manager](#nwm---noctra-window-manager)
6. [Ejemplos](#ejemplos)
7. [API Reference](#api-reference)

---

## Introducción

El **Sistema de Formularios de Noctra** permite definir interfaces de usuario y flujos de trabajo de manera declarativa usando archivos TOML. El sistema se compone de tres capas principales:

1. **FormLib** - Definición, carga y validación de formularios
2. **FormGraph** - Navegación jerárquica entre formularios y menús
3. **NWM** - Gestor de ventanas para renderizado en terminal

### Características Principales

✅ Formularios declarativos en TOML/JSON
✅ Validación de campos con tipos y reglas
✅ Navegación jerárquica con breadcrumbs
✅ Sistema de modos TUI (Command, Result, Form, Dialog)
✅ Integración con SQL (SELECT, INSERT, UPDATE, DELETE)
✅ Arquitectura modular y testeable

---

## Arquitectura

### Capas del Sistema

```
┌─────────────────────────────────────────────────┐
│           Capa CLI (noctra-cli)                 │
│   Comandos: form load, form exec, form preview │
└─────────────────┬───────────────────────────────┘
                  │
┌─────────────────▼───────────────────────────────┐
│        Capa TUI (noctra-tui)                    │
│    NWM + Renderer + Widgets + Modos             │
└─────────────────┬───────────────────────────────┘
                  │
┌─────────────────▼───────────────────────────────┐
│       Capa FormLib (noctra-formlib)             │
│   Parser + Validator + Runtime + FormGraph     │
└─────────────────┬───────────────────────────────┘
                  │
┌─────────────────▼───────────────────────────────┐
│        Capa Core (noctra-core)                  │
│      Executor + Backend + ResultSet             │
└─────────────────────────────────────────────────┘
```

### Flujo de Datos

```
app.toml ──> FormGraph ──> GraphNavigator
   │                           │
   ▼                           ▼
form.toml ──> FormLoader ──> FormInstance
   │                           │
   ▼                           ▼
FormValidator ──> Executor ──> ResultSet
   │                           │
   ▼                           ▼
NWM ──> TuiRenderer ──> Terminal Output
```

---

## FDL2 - Form Definition Language

### Estructura Básica

```toml
# Metadata del formulario
title = "Employee Search"
description = "Search and filter employees"
schema = "hr_database"

# Campos del formulario
[fields.field_name]
label = "Display Label"
type = "text"        # text, int, float, bool, date, datetime, email, password
required = true
width = 30

# Validaciones del campo
[fields.field_name.validations]
min_length = 3
max_length = 100
pattern = "^[A-Za-z\\s]+$"
allowed_values = ["option1", "option2"]

# Acciones disponibles
[actions.action_name]
action_type = "query"     # query, insert, update, delete, script
param_type = "named"       # named (:param) or positional ($1)
sql = "SELECT * FROM table WHERE field = :field_name"

# Configuración de UI
[ui_config]
layout = "single"          # single, double, flexible
width = 80
height = 24
theme = "default"
buttons = ["search", "clear"]

# Configuración de paginación
[pagination]
page_size = 20
order_by = ["created_at DESC"]
```

### Tipos de Campos

| Tipo | Descripción | Ejemplo |
|------|-------------|---------|
| `text` | Texto simple | "John Doe" |
| `int` | Número entero | 42 |
| `float` | Número decimal | 3.14 |
| `bool` | Booleano | true/false |
| `date` | Fecha | "2025-11-08" |
| `datetime` | Fecha y hora | "2025-11-08 14:30:00" |
| `email` | Email | "user@example.com" |
| `password` | Contraseña (enmascarada) | "••••••" |
| `textarea` | Texto multilínea | "Line 1\nLine 2" |

### Validaciones Disponibles

```toml
[fields.my_field.validations]
# Rango numérico
min = "0"
max = "100"

# Longitud de texto
min_length = 3
max_length = 50

# Patrón regex
pattern = "^[A-Z][a-z]+$"

# Valores permitidos
allowed_values = ["SALES", "HR", "IT"]
```

### Tipos de Acciones

- **query** - Consulta SELECT
- **insert** - Inserción de datos
- **update** - Actualización de registros
- **delete** - Eliminación de registros
- **script** - Script personalizado
- **apicall** - Llamada a API externa

---

## FormGraph - Navegación Jerárquica

### Archivo de Aplicación

El archivo `app.toml` define la estructura de navegación:

```toml
version = "1.0"
title = "My Application"
base_path = "forms"

[config]
default_database = "mydb.db"
show_breadcrumbs = true
enable_history = true

[root]
id = "main_menu"
title = "Main Menu"
type = "menu"
icon = "🏠"

[[root.children]]
id = "employees"
title = "Employees"
type = "menu"
icon = "👥"

[[root.children.children]]
id = "employee_search"
title = "Search"
type = "form"
path = "forms/employee_search.toml"
icon = "🔍"
```

### Tipos de Nodos

| Tipo | Descripción | Requiere |
|------|-------------|----------|
| `menu` | Menú de navegación | children |
| `form` | Formulario FDL2 | path |
| `query` | Consulta SQL directa | action (SQL) |
| `link` | Enlace externo | action (URL) |

### API del GraphNavigator

```rust
use noctra_formlib::{FormGraph, GraphNavigator};

// Cargar grafo
let graph = FormGraph::load_from_file("app.toml")?;
let mut navigator = GraphNavigator::new(graph);

// Navegar
navigator.navigate_to("employee_search")?;

// Historial
navigator.go_back()?;
navigator.go_forward()?;
navigator.go_home()?;

// Información
let current = navigator.current_node()?;
let children = navigator.get_current_children()?;
let breadcrumb = navigator.get_breadcrumb()?;

// Cargar formulario del nodo actual
let form = navigator.load_current_form()?;
```

---

## NWM - Noctra Window Manager

### Modos de UI

El NWM soporta 4 modos de interfaz:

| Modo | Descripción | Icono | Uso |
|------|-------------|-------|-----|
| `Command` | REPL interactivo | `>_` | Comandos SQL |
| `Result` | Visualización de datos | `📊` | Tablas de resultados |
| `Form` | Entrada de datos | `📝` | Formularios |
| `Dialog` | Mensajes/Confirmaciones | `💬` | Alertas/Diálogos |

### Stack de Ventanas

El NWM gestiona ventanas en un stack LIFO:

```rust
use noctra_tui::{NoctraWindowManager, NwmWindow, UiMode};

let mut nwm = NoctraWindowManager::default();

// Crear ventanas
let window1 = NwmWindow::command("cmd".into(), "Command".into());
let window2 = NwmWindow::form("form1".into(), "Employee Form".into(), form);

// Gestionar stack
nwm.push_window(window1);
nwm.push_window(window2);

let current = nwm.current_window()?;
nwm.pop_window()?;
nwm.close_current_window()?;
```

### Layout del Terminal

```
╔═══════════════════════════════════════════════════╗
║  Main Menu > Employees > Search                   ║ ← Breadcrumb
╠───────────────────────────────────────────────────╣
║  📝 Employee Search - Form Mode                   ║ ← Header
║                                                    ║
║  Employee ID: [     ]                             ║
║  Name:        [                    ]              ║ ← Main Area
║  Department:  [SALES      ▼]                      ║   (Contenido)
║                                                    ║
║  [ Search ]  [ Clear ]  [ Cancel ]                ║
║                                                    ║
╠───────────────────────────────────────────────────╣
║ Windows: 2 | Mode: Form    F1=Help | ESC=Back    ║ ← Footer
╚═══════════════════════════════════════════════════╝
```

### Configuración

```rust
use noctra_tui::NwmConfig;

let config = NwmConfig {
    show_breadcrumbs: true,
    show_status_bar: true,
    header_height: 3,
    footer_height: 2,
    theme: "default".into(),
    min_window_size: (80, 24),
};

let nwm = NoctraWindowManager::new(config);
```

---

## Ejemplos

### Ejemplo 1: Formulario de Búsqueda

**Archivo:** `forms/employee_search.toml`

```toml
title = "Employee Search"
schema = "hr_database"

[fields.name]
label = "Employee Name"
type = "text"
required = false

[fields.department]
label = "Department"
type = "text"
required = false

[actions.search]
action_type = "query"
param_type = "named"
sql = """
SELECT id, name, department, email
FROM employees
WHERE (:name IS NULL OR name LIKE '%' || :name || '%')
  AND (:department IS NULL OR department = :department)
ORDER BY name
"""
```

### Ejemplo 2: Formulario de Alta

**Archivo:** `forms/employee_add.toml`

```toml
title = "Add Employee"
schema = "hr_database"

[fields.name]
label = "Full Name"
type = "text"
required = true

[fields.name.validations]
min_length = 3
max_length = 100

[fields.email]
label = "Email"
type = "email"
required = true

[fields.salary]
label = "Annual Salary"
type = "float"
required = true

[fields.salary.validations]
min = "30000"
max = "500000"

[actions.save]
action_type = "insert"
param_type = "named"
sql = """
INSERT INTO employees (name, email, salary, created_at)
VALUES (:name, :email, :salary, datetime('now'))
"""
```

### Ejemplo 3: Aplicación Completa

**Archivo:** `app.toml`

```toml
version = "1.0"
title = "HR System"
base_path = "examples"

[root]
id = "main"
title = "Main Menu"
type = "menu"

[[root.children]]
id = "search"
title = "Search Employees"
type = "form"
path = "forms/employee_search.toml"

[[root.children]]
id = "add"
title = "Add Employee"
type = "form"
path = "forms/employee_add.toml"
```

---

## API Reference

### FormLib

#### Cargar Formulario

```rust
use noctra_formlib::{load_form_from_path, Form};
use std::path::Path;

let form: Form = load_form_from_path(Path::new("form.toml"))?;
println!("Form: {}", form.title);
```

#### Validar Formulario

```rust
use noctra_formlib::{FormValidator, Form};
use std::collections::HashMap;

let validator = FormValidator::new();
let values = HashMap::from([
    ("name".to_string(), "John Doe".to_string()),
    ("email".to_string(), "john@example.com".to_string()),
]);

match validator.validate_form(&form, &values) {
    Ok(()) => println!("Valid!"),
    Err(errors) => {
        for error in errors {
            eprintln!("Error: {}", error);
        }
    }
}
```

### FormGraph

#### Cargar y Navegar

```rust
use noctra_formlib::FormGraph;
use std::path::Path;

let graph = FormGraph::load_from_file(Path::new("app.toml"))?;
graph.validate()?;

let node = graph.find_node("employee_search")?;
println!("Node: {} - {}", node.id, node.title);

let form = graph.load_form_from_node("employee_search")?;
```

### NWM

#### Crear y Gestionar Ventanas

```rust
use noctra_tui::{NoctraWindowManager, NwmWindow, WindowContent};
use noctra_core::ResultSet;

let mut nwm = NoctraWindowManager::default();

// Ventana de comando
let cmd_window = NwmWindow::command("cmd1".into(), "Command".into());
nwm.push_window(cmd_window);

// Ventana de resultado
let result_window = NwmWindow::result(
    "result1".into(),
    "Query Results".into(),
    result_set
);
nwm.push_window(result_window);

// Renderizar
let output = nwm.render_layout((80, 24))?;
println!("{}", output);
```

---

## Roadmap

### Completado ✅

- [x] Parser TOML para formularios
- [x] Sistema de validación de campos
- [x] FormGraph con navegación jerárquica
- [x] NWM con modos (Command, Result, Form, Dialog)
- [x] Stack de ventanas con historial
- [x] Ejemplos de formularios

### En Progreso 🚧

- [ ] Renderer de formularios en TUI
- [ ] Integración con CLI (comandos `form load/exec/preview`)
- [ ] Tests de integración completos
- [ ] Documentación API completa

### Futuro 🔮

- [ ] Formularios con validación en tiempo real
- [ ] Widgets avanzados (date picker, autocomplete)
- [ ] Temas visuales personalizables
- [ ] Export de formularios a JSON/YAML
- [ ] Generador de formularios desde esquema DB

---

## Contribuir

Para contribuir al sistema de formularios:

1. Lee la documentación de arquitectura en `DESIGN.md`
2. Revisa los ejemplos en `examples/forms/`
3. Ejecuta los tests: `cargo test --package noctra-formlib`
4. Sigue las convenciones de código del proyecto

## Licencia

MIT OR Apache-2.0
