# Milestone 7 — "SCRIPT" Implementation Plan

**Noctra(🦆): 4GL Scripting Environment**
**Fecha de Inicio:** 24 de diciembre de 2025
**Duración:** 6 semanas (24 dic 2025 — 3 feb 2026)
**Versión Target:** v0.7.0
**Branch:** `claude/scripting-*`

---

## 🎯 OBJETIVO ESTRATÉGICO

> **Convertir RQL en un 4GL completo con capacidades de scripting**
> **De "query language" a "programming language para datos"**

### Pre-requisitos

- ✅ M6 "FABRIC" completado (DuckDB integrado)
- ✅ `USE 'file.csv'` funcional
- ✅ `LET`, `#var` funcionando
- ✅ `EXPORT` multi-formato

---

## 📋 PANORAMA GENERAL DEL MILESTONE

### Qué NO es parte de M7

| Característica | Milestone |
|----------------|-----------|
| DuckDB integration | ✅ M6 |
| `USE 'file.csv'` | ✅ M6 |
| `EXPORT TO 'file'` | ✅ M6 |
| `SHOW SOURCES` | ✅ M6 |
| `LET`, `#var` | ✅ M6 |
| Modo híbrido | ✅ M6 |

### Qué SÍ es parte de M7

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
| `WHENEVER ERROR THEN` | Manejo de errores | Medium |
| `IMPORT MACRO FROM 'file'` | Librerías de macros | High |

---

## 🗓️ TIMELINE — 6 Semanas → 6 Fases

```
Diciembre 2025           Enero 2026              Febrero 2026
24  25  26  27  28  29   30  31  01  02  03  04  05
│   │   │   │   │   │    │   │   │   │   │   │   │
├───┼───┼───┼───┼───┼────┼───┼───┼───┼───┼───┼───┤
│ F1: SCRIPTING CORE   │ F2: MACROS           │
├───────────────────────┼──────────────────────┤
│                       │ F3: AGREGADOS        │
├───────────────────────┼──────────────────────┤
│                       │ F4: SESIÓN           │
├───────────────────────┼──────────────────────┤
│                       │ F5: SALIDA           │
├───────────────────────┼──────────────────────┤
│                       │ F6: RELEASE          │
└───────────────────────┴──────────────────────┘
```

---

## 📦 FASE 1: SCRIPTING CORE (Semana 1)

**Fecha:** 24-30 dic 2025
**Objetivo:** Control de flujo básico (IF, FOR, PRINT)

### 1.1 Comando `IF/THEN/ELSE`

**Sintaxis:**
```rql
IF condition THEN
  statements
ELSE
  statements
END;
```

**Ejemplo:**
```rql
LET pais = 'AR';

IF #pais = 'AR' THEN
  PRINT "Procesando Argentina";
  USE 'ventas_ar.csv' AS v;
ELSE
  PRINT "Procesando otros países";
  USE 'ventas_latam.csv' AS v;
END;

SELECT * FROM v LIMIT 10;
```

**Parser:** `crates/parser/src/scripting.rs` (NUEVO)

```rust
pub enum ScriptStatement {
    If {
        condition: Expr,
        then_block: Vec<RqlStatement>,
        else_block: Option<Vec<RqlStatement>>,
    },
    For {
        variable: String,
        query: Box<RqlStatement>,
        body: Vec<RqlStatement>,
    },
    Print {
        values: Vec<PrintValue>,
    },
}

pub enum PrintValue {
    Literal(String),
    Variable(String),
    Expression(Expr),
}
```

**Executor:** `crates/core/src/script_executor.rs` (NUEVO)

```rust
pub struct ScriptExecutor {
    engine: QueryEngine,
    session: Session,
}

impl ScriptExecutor {
    pub fn execute_if(&mut self, stmt: &IfStatement) -> Result<()> {
        let condition = self.evaluate_condition(&stmt.condition)?;

        if condition {
            for s in &stmt.then_block {
                self.execute(s)?;
            }
        } else if let Some(else_block) = &stmt.else_block {
            for s in else_block {
                self.execute(s)?;
            }
        }

        Ok(())
    }
}
```

### 1.2 Comando `FOR ... IN ... DO`

**Sintaxis:**
```rql
FOR variable IN (query) DO
  statements
END;
```

**Ejemplo:**
```rql
USE 'regiones.csv' AS r;

FOR region IN (SELECT DISTINCT region FROM r) DO
  PRINT "Procesando región:", region.region;

  EXPORT (SELECT * FROM r WHERE region = region.region)
  TO CONCAT('region_', region.region, '.csv')
  FORMAT CSV;
END;
```

**Implementación:**
```rust
impl ScriptExecutor {
    pub fn execute_for(&mut self, stmt: &ForStatement) -> Result<()> {
        // Execute query
        let result = self.engine.execute(&stmt.query)?;

        // Iterate over rows
        for row in result.rows {
            // Bind variable
            self.session.set_variable(&stmt.variable, Value::Row(row.clone()));

            // Execute body
            for s in &stmt.body {
                self.execute(s)?;
            }
        }

        Ok(())
    }
}
```

### 1.3 Comando `PRINT`

**Sintaxis:**
```rql
PRINT value1, value2, ...;
```

**Ejemplo:**
```rql
LET pais = 'AR';
LET total = 12345;

PRINT "País:", #pais;
PRINT "Total ventas:", #total;
PRINT "---";
```

**Implementación:**
```rust
impl ScriptExecutor {
    pub fn execute_print(&mut self, stmt: &PrintStatement) -> Result<()> {
        let mut output = String::new();

        for value in &stmt.values {
            match value {
                PrintValue::Literal(s) => output.push_str(s),
                PrintValue::Variable(name) => {
                    let var = self.session.get_variable(name)?;
                    output.push_str(&format!("{}", var));
                },
                PrintValue::Expression(expr) => {
                    let result = self.evaluate_expression(expr)?;
                    output.push_str(&format!("{}", result));
                },
            }
            output.push(' ');
        }

        println!("{}", output.trim());
        Ok(())
    }
}
```

### Entregables Fase 1

- [ ] Parser para IF/THEN/ELSE
- [ ] Parser para FOR...IN...DO
- [ ] Parser para PRINT
- [ ] ScriptExecutor básico
- [ ] Tests: IF con condiciones simples
- [ ] Tests: FOR iterando sobre resultados
- [ ] Tests: PRINT con variables y literales

**Criterio de Éxito:**
```rql
FOR x IN (SELECT 1 AS num UNION SELECT 2 UNION SELECT 3) DO
  PRINT "Número:", x.num;
END;

-- Output:
-- Número: 1
-- Número: 2
-- Número: 3
```

---

## 🔧 FASE 2: MACROS & REUTILIZACIÓN (Semana 2)

**Fecha:** 31 dic 2025 - 6 ene 2026
**Objetivo:** Sistema de macros para reutilización de código

### 2.1 Definición de Macros

**Sintaxis:**
```rql
MACRO name(param1, param2, ...) AS
  statements
END;
```

**Ejemplo:**
```rql
-- Definir macro
MACRO top_productos(n, region) AS
  SELECT producto, SUM(total) AS ventas
  FROM ventas
  WHERE region = :region
  GROUP BY producto
  ORDER BY ventas DESC
  LIMIT :n;
END;
```

**AST:**
```rust
pub struct MacroDefinition {
    pub name: String,
    pub parameters: Vec<String>,
    pub body: Vec<RqlStatement>,
}

pub struct MacroRegistry {
    macros: HashMap<String, MacroDefinition>,
}

impl MacroRegistry {
    pub fn register(&mut self, macro_def: MacroDefinition) {
        self.macros.insert(macro_def.name.clone(), macro_def);
    }

    pub fn expand(&self, name: &str, args: Vec<Value>) -> Result<Vec<RqlStatement>> {
        let macro_def = self.macros.get(name)
            .ok_or_else(|| anyhow!("Macro '{}' not found", name))?;

        // Bind parameters
        let mut context = HashMap::new();
        for (param, arg) in macro_def.parameters.iter().zip(args.iter()) {
            context.insert(param.clone(), arg.clone());
        }

        // Expand body with substitutions
        Ok(self.substitute_parameters(&macro_def.body, &context)?)
    }
}
```

### 2.2 Invocación de Macros

**Sintaxis:**
```rql
CALL macro_name(arg1, arg2, ...);
```

**Ejemplo:**
```rql
CALL top_productos(10, 'LATAM');

-- Expande a:
-- SELECT producto, SUM(total) AS ventas
-- FROM ventas
-- WHERE region = 'LATAM'
-- GROUP BY producto
-- ORDER BY ventas DESC
-- LIMIT 10;
```

### 2.3 Macros desde Archivos

**Sintaxis:**
```rql
IMPORT MACRO FROM 'analytics.rql';
```

**Ejemplo: analytics.rql**
```rql
MACRO resumen_ventas(periodo) AS
  SELECT
    DATE_TRUNC(:periodo, fecha) AS periodo,
    COUNT(*) AS num_ventas,
    SUM(total) AS total,
    AVG(total) AS promedio
  FROM ventas
  GROUP BY periodo
  ORDER BY periodo;
END;

MACRO top_clientes(n) AS
  SELECT
    cliente_id,
    COUNT(*) AS compras,
    SUM(total) AS total_gastado
  FROM ventas
  GROUP BY cliente_id
  ORDER BY total_gastado DESC
  LIMIT :n;
END;
```

**Usage:**
```rql
IMPORT MACRO FROM 'analytics.rql';

CALL resumen_ventas('month');
CALL top_clientes(20);
```

### 2.4 Macros con Estado

**Ejemplo:**
```rql
MACRO inicializar_session(pais) AS
  LET pais_activo = :pais;
  USE CONCAT('ventas_', :pais, '.csv') AS v;
  PRINT "Sesión iniciada para:", :pais;
END;

CALL inicializar_session('AR');
```

### Entregables Fase 2

- [ ] MacroRegistry implementado
- [ ] Parser para MACRO...AS...END
- [ ] Parser para CALL macro()
- [ ] Parser para IMPORT MACRO FROM
- [ ] Sustitución de parámetros
- [ ] Tests: definir y llamar macros
- [ ] Tests: macros con múltiples parámetros
- [ ] Tests: IMPORT MACRO

**Criterio de Éxito:**
```rql
MACRO greet(name) AS
  PRINT "Hello,", :name;
END;

CALL greet('Noctra');
-- Output: Hello, Noctra
```

---

## 📊 FASE 3: AGREGADOS & VISUALIZACIÓN (Semana 3)

**Fecha:** 7-13 ene 2026
**Objetivo:** Funciones de ventana simplificadas y gráficos ASCII

### 3.1 Funciones de Ventana Simplificadas

**Funciones:**
```rql
RUNSUM(column)   → SUM(column) OVER (ORDER BY ...)
RUNCOUNT(*)      → COUNT(*) OVER (ORDER BY ...)
RUNAVG(column)   → AVG(column) OVER (ORDER BY ...)
RUNMIN(column)   → MIN(column) OVER (ORDER BY ...)
RUNMAX(column)   → MAX(column) OVER (ORDER BY ...)
```

**Ejemplo:**
```rql
SELECT
  fecha,
  ventas,
  RUNSUM(ventas) AS acumulado,
  RUNAVG(ventas) AS promedio_movil
FROM ventas_diarias
ORDER BY fecha;
```

**Traducción a SQL:**
```sql
SELECT
  fecha,
  ventas,
  SUM(ventas) OVER (ORDER BY fecha ROWS UNBOUNDED PRECEDING) AS acumulado,
  AVG(ventas) OVER (ORDER BY fecha ROWS UNBOUNDED PRECEDING) AS promedio_movil
FROM ventas_diarias
ORDER BY fecha;
```

**Parser:**
```rust
pub enum WindowFunction {
    RunSum(Expr),
    RunCount,
    RunAvg(Expr),
    RunMin(Expr),
    RunMax(Expr),
}

impl WindowFunction {
    pub fn to_sql(&self, order_by: &[OrderByExpr]) -> String {
        let order_clause = order_by.iter()
            .map(|o| format!("{} {}", o.expr, o.direction))
            .collect::<Vec<_>>()
            .join(", ");

        match self {
            Self::RunSum(expr) => {
                format!("SUM({}) OVER (ORDER BY {} ROWS UNBOUNDED PRECEDING)",
                    expr, order_clause)
            },
            // ... similar for others
        }
    }
}
```

### 3.2 Gráficos ASCII

**Sintaxis:**
```rql
GRAPH BAR FROM query;
GRAPH LINE FROM query;
GRAPH HIST FROM query;
```

**Ejemplo:**
```rql
GRAPH BAR FROM (
  SELECT region, SUM(total) AS total
  FROM ventas
  GROUP BY region
  ORDER BY total DESC
  LIMIT 10
);
```

**Output:**
```
┌────────────────────────────────────────────────┐
│ Ventas por Región                              │
├────────────────────────────────────────────────┤
│ Norte    ████████████████████████████ 142,345  │
│ Sur      ████████████████████ 98,234           │
│ Este     ███████████████ 76,123                │
│ Oeste    ████████████ 54,892                   │
│ Centro   ████████ 43,567                       │
└────────────────────────────────────────────────┘
```

**Implementación:**
```rust
pub struct AsciiBarChart {
    data: Vec<(String, f64)>,
    width: usize,
}

impl AsciiBarChart {
    pub fn render(&self) -> String {
        let max_value = self.data.iter()
            .map(|(_, v)| *v)
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap_or(1.0);

        let mut output = String::new();

        // Header
        output.push_str("┌────────────────────────────────────────────────┐\n");
        output.push_str("│ Gráfico de Barras                              │\n");
        output.push_str("├────────────────────────────────────────────────┤\n");

        // Bars
        for (label, value) in &self.data {
            let bar_length = ((value / max_value) * (self.width as f64)) as usize;
            let bar = "█".repeat(bar_length);
            output.push_str(&format!("│ {:10} {} {:>10}\n",
                label,
                bar,
                format_number(*value)
            ));
        }

        // Footer
        output.push_str("└────────────────────────────────────────────────┘\n");

        output
    }
}
```

### Entregables Fase 3

- [ ] Parser para RUNSUM, RUNAVG, etc.
- [ ] Traducción a window functions
- [ ] GRAPH BAR implementado
- [ ] GRAPH LINE implementado
- [ ] GRAPH HIST implementado
- [ ] Tests: funciones de ventana
- [ ] Tests: rendering de gráficos

**Criterio de Éxito:**
```rql
SELECT fecha, ventas, RUNSUM(ventas) AS acumulado
FROM ventas
ORDER BY fecha
LIMIT 5;

GRAPH BAR FROM (SELECT region, COUNT(*) FROM ventas GROUP BY region);
```

---

## 💾 FASE 4: SESIÓN PERSISTENTE (Semana 4)

**Fecha:** 14-20 ene 2026
**Objetivo:** Guardar y cargar estado de sesión

### 4.1 Guardar Sesión

**Sintaxis:**
```rql
SAVE SESSION 'nombre.toml';
```

**Qué se guarda:**
- Variables (`LET`)
- Fuentes activas (`USE`)
- Macros definidas (`MACRO`)
- Configuración

**Formato TOML:**
```toml
[session]
created_at = "2025-12-25T10:30:00Z"
version = "0.7.0"

[variables]
pais = "AR"
min_total = 1000
fecha_desde = "2025-01-01"

[[sources]]
alias = "ventas"
path = "./ventas_2025.csv"
type = "csv"

[[sources]]
alias = "clientes"
path = "./clientes.db"
type = "sqlite"

[[macros]]
name = "top_productos"
parameters = ["n", "region"]
body = """
SELECT producto, SUM(total) AS ventas
FROM ventas
WHERE region = :region
GROUP BY producto
ORDER BY ventas DESC
LIMIT :n;
"""
```

**Implementación:**
```rust
pub struct SessionState {
    pub variables: HashMap<String, Value>,
    pub sources: Vec<SourceInfo>,
    pub macros: Vec<MacroDefinition>,
}

impl SessionState {
    pub fn save(&self, path: &Path) -> Result<()> {
        let toml = self.to_toml()?;
        std::fs::write(path, toml)?;
        Ok(())
    }

    pub fn load(path: &Path) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let session: SessionState = toml::from_str(&content)?;
        Ok(session)
    }
}
```

### 4.2 Cargar Sesión

**Sintaxis:**
```rql
LOAD SESSION 'nombre.toml';
```

**Comportamiento:**
- Restaura todas las variables
- Re-registra todas las fuentes
- Re-carga todas las macros
- Valida que los archivos existen

**Ejemplo:**
```rql
-- Sesión inicial
LET pais = 'AR';
USE 'ventas.csv' AS v;
MACRO top(n) AS SELECT * FROM v LIMIT :n; END;

SAVE SESSION 'mi_sesion.toml';

-- Nueva sesión
LOAD SESSION 'mi_sesion.toml';

-- Todo está restaurado
PRINT #pais;  -- Output: AR
CALL top(5);  -- Funciona
```

### 4.3 Auto-save

**Configuración:**
```toml
# ~/.config/noctra/config.toml
[session]
auto_save = true
auto_save_path = "~/.noctra/last_session.toml"
```

**Comportamiento:**
- Al salir de REPL/TUI: guarda automáticamente
- Al iniciar: pregunta si quiere restaurar

### Entregables Fase 4

- [ ] SessionState serialization
- [ ] SAVE SESSION comando
- [ ] LOAD SESSION comando
- [ ] Auto-save al salir
- [ ] Prompt de restauración al iniciar
- [ ] Tests: save/load roundtrip

**Criterio de Éxito:**
```bash
$ noctra
> LET test = 123
> SAVE SESSION 'test.toml'
> exit

$ noctra
> LOAD SESSION 'test.toml'
> PRINT #test
123
```

---

## 🔌 FASE 5: SALIDA & CANALIZACIÓN (Semana 5)

**Fecha:** 21-27 ene 2026
**Objetivo:** Salida flexible y canalización a comandos shell

### 5.1 Comando PIPE TO

**Sintaxis:**
```rql
query PIPE TO 'shell_command';
```

**Ejemplo:**
```rql
-- Filtrar resultados con grep
SELECT * FROM logs
WHERE level = 'ERROR'
PIPE TO 'grep "database"';

-- Formatear con column
SELECT * FROM ventas
LIMIT 10
PIPE TO 'column -t';

-- Enviar a less para paginación
SELECT * FROM huge_table
PIPE TO 'less';
```

**Implementación:**
```rust
impl ScriptExecutor {
    pub fn execute_pipe(&mut self, query: &RqlStatement, cmd: &str) -> Result<()> {
        // Execute query
        let result = self.engine.execute(query)?;

        // Format as CSV
        let csv = self.format_as_csv(&result)?;

        // Pipe to command
        let mut child = std::process::Command::new("sh")
            .arg("-c")
            .arg(cmd)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::inherit())
            .spawn()?;

        if let Some(mut stdin) = child.stdin.take() {
            use std::io::Write;
            stdin.write_all(csv.as_bytes())?;
        }

        child.wait()?;
        Ok(())
    }
}
```

### 5.2 Output Redirection

**Sintaxis:**
```rql
query > 'file.txt';
query >> 'file.txt';  -- append
```

**Ejemplo:**
```rql
SELECT * FROM ventas
WHERE fecha >= '2025-01-01'
> 'ventas_2025.txt';

PRINT "Procesado", CURRENT_TIMESTAMP
>> 'log.txt';
```

### 5.3 Manejo de Errores Global (`WHENEVER ERROR THEN`)

**Sintaxis:**
```rql
WHENEVER ERROR THEN
  statements
END;
```

**Variables disponibles:**
- `ERROR_MESSAGE` → Texto descriptivo del error
- `ERROR_CODE` → Código numérico del error
- `ERROR_QUERY` → Query que causó el fallo

**Acciones posibles:**
- `CONTINUE` → Continúa la ejecución del script
- `EXIT` → Sale del script inmediatamente
- `ROLLBACK` → Revierte transacciones activas

**Ejemplo básico:**
```rql
WHENEVER ERROR THEN
  PRINT "ERROR:", ERROR_MESSAGE;
  PRINT "QUERY:", ERROR_QUERY;
  EXPORT ERROR TO 'error_log.json';
  CONTINUE;
END;

-- Si falla cualquier query después...
SELECT * FROM tabla_inexistente;
-- → Se ejecuta el bloque de error
```

**Ejemplo con ROLLBACK:**
```rql
WHENEVER ERROR THEN
  PRINT "Fallo en transacción:", ERROR_MESSAGE;
  ROLLBACK;
  EXIT;
END;

BEGIN TRANSACTION;
  INSERT INTO ventas VALUES (...);
  INSERT INTO invalid_table VALUES (...);  -- Falla aquí
COMMIT;
```

**Ejemplo con logging:**
```rql
WHENEVER ERROR THEN
  PRINT "Error:", ERROR_MESSAGE;
  PIPE TO 'echo "ERROR: ' + ERROR_MESSAGE + '" >> /tmp/noctra_error.log';
  CONTINUE;
END;
```

**Implementación:**
```rust
pub struct ErrorHandler {
    pub block: Vec<RqlStatement>,
    pub mode: ErrorMode,
}

pub enum ErrorMode {
    Continue,
    Exit,
    Rollback,
}

impl ScriptExecutor {
    pub fn execute_with_error_handler(&mut self, stmt: &RqlStatement) -> Result<()> {
        match self.execute(stmt) {
            Ok(result) => Ok(result),
            Err(e) => {
                if let Some(handler) = &self.error_handler {
                    // Set error variables
                    self.session.set_variable("ERROR_MESSAGE", Value::String(e.to_string()));
                    self.session.set_variable("ERROR_CODE", Value::Integer(e.code()));
                    self.session.set_variable("ERROR_QUERY", Value::String(stmt.to_string()));

                    // Execute handler block
                    for s in &handler.block {
                        self.execute(s)?;
                    }

                    // Apply error mode
                    match handler.mode {
                        ErrorMode::Continue => Ok(()),
                        ErrorMode::Exit => std::process::exit(1),
                        ErrorMode::Rollback => {
                            self.engine.rollback()?;
                            Err(e)
                        },
                    }
                } else {
                    Err(e)
                }
            }
        }
    }
}
```

### Entregables Fase 5

- [ ] PIPE TO implementado
- [ ] Redirección > y >>
- [ ] WHENEVER ERROR THEN parser
- [ ] ErrorHandler en ScriptExecutor
- [ ] Variables ERROR_MESSAGE, ERROR_CODE, ERROR_QUERY
- [ ] Modos: CONTINUE, EXIT, ROLLBACK
- [ ] Validación de comandos (security)
- [ ] Tests: PIPE TO grep
- [ ] Tests: redirección
- [ ] Tests: error handling con CONTINUE
- [ ] Tests: error handling con EXIT

**Criterio de Éxito:**
```rql
WHENEVER ERROR THEN
  PRINT "Error capturado:", ERROR_MESSAGE;
  CONTINUE;
END;

SELECT * FROM tabla_inexistente;  -- Error capturado, continúa
PRINT "Script sigue ejecutándose";
```

---

## 🚀 FASE 6: RELEASE & DOCUMENTACIÓN (Semana 6)

**Fecha:** 28 ene - 3 feb 2026
**Objetivo:** Release v0.7.0, documentación, demos

### 6.1 Tag v0.7.0

```bash
git tag -a v0.7.0 -m "Release v0.7.0 - SCRIPT (4GL Scripting)"
git push origin v0.7.0
```

### 6.2 Documentación

**Crear `docs/RQL_SCRIPTING.md`:**

```markdown
# RQL Scripting Manual

## Control de Flujo

### IF/THEN/ELSE
Control condicional:
```rql
IF condition THEN
  statements
ELSE
  statements
END;
```

### FOR...IN...DO
Bucles sobre resultados:
```rql
FOR variable IN (query) DO
  statements
END;
```

## Macros

### MACRO Definition
```rql
MACRO name(param1, param2) AS
  statements
END;
```

### CALL
```rql
CALL name(arg1, arg2);
```

### IMPORT MACRO
```rql
IMPORT MACRO FROM 'file.rql';
```

## Funciones de Ventana

- `RUNSUM(col)` - Running sum
- `RUNAVG(col)` - Running average
- `RUNCOUNT(*)` - Running count
- `RUNMIN(col)` - Running minimum
- `RUNMAX(col)` - Running maximum

## Visualización

- `GRAPH BAR FROM query` - Bar chart
- `GRAPH LINE FROM query` - Line chart
- `GRAPH HIST FROM query` - Histogram

## Sesión

- `SAVE SESSION 'file.toml'` - Save session state
- `LOAD SESSION 'file.toml'` - Load session state

## Salida

- `PRINT value1, value2, ...` - Print to console
- `query PIPE TO 'cmd'` - Pipe to shell command
- `query > 'file'` - Redirect to file
```

### 6.3 Demo Completo

**Archivo: `examples/demo_full_script.rql`**

```rql
-- ============================================
-- Demo Completo: Noctra v0.7.0 "SCRIPT"
-- ============================================

PRINT "=== Inicializando Sesión ===";

-- Variables
LET pais = 'AR';
LET min_ventas = 1000;
LET fecha_desde = '2025-01-01';

-- Fuentes
USE 'ventas_2025.csv' AS v;
USE 'clientes.db' AS c;

PRINT "País:", #pais;
PRINT "Filtro mínimo:", #min_ventas;

-- Macro de resumen
MACRO resumen(pais, min) AS
  SELECT
    region,
    COUNT(*) AS num_ventas,
    SUM(total) AS total,
    AVG(total) AS promedio
  FROM v
  WHERE pais = :pais
    AND total >= :min
    AND fecha >= '2025-01-01'
  GROUP BY region
  ORDER BY total DESC;
END;

-- Ejecutar análisis
PRINT "=== Análisis por Región ===";
CALL resumen(#pais, #min_ventas);

-- Visualización
PRINT "=== Gráfico de Barras ===";
GRAPH BAR FROM resumen(#pais, #min_ventas);

-- Análisis temporal
PRINT "=== Tendencia Temporal ===";
SELECT
  DATE_TRUNC('month', fecha) AS mes,
  SUM(total) AS ventas,
  RUNSUM(SUM(total)) AS acumulado
FROM v
WHERE pais = #pais
GROUP BY mes
ORDER BY mes;

-- Export por región
PRINT "=== Exportando por Región ===";
FOR region IN (SELECT DISTINCT region FROM v WHERE pais = #pais) DO
  PRINT "Exportando región:", region.region;

  EXPORT (SELECT * FROM v WHERE pais = #pais AND region = region.region)
  TO CONCAT('export_', #pais, '_', region.region, '.json')
  FORMAT JSON;
END;

-- Guardar sesión
SAVE SESSION 'analisis_ar_2025.toml';

PRINT "=== Análisis Completado ===";
```

### 6.4 CHANGELOG

```markdown
# Changelog

## [0.7.0] - 2026-02-03 - "SCRIPT"

### Added
- 🎯 **4GL Scripting** capabilities
- `IF/THEN/ELSE` control flow
- `FOR...IN...DO` loops
- `MACRO ... AS ... END` macro system
- `CALL macro()` macro invocation
- `IMPORT MACRO FROM 'file'` macro libraries
- `RUNSUM()`, `RUNAVG()`, `RUNCOUNT()` window functions
- `GRAPH BAR`, `GRAPH LINE` ASCII visualization
- `SAVE SESSION`, `LOAD SESSION` session persistence
- `PRINT` debug output
- `PIPE TO 'cmd'` shell piping
- Auto-save on exit

### Changed
- RQL is now a full 4GL scripting language
- Sessions can be saved and restored

### Migration Guide
See `docs/MIGRATION_M6_TO_M7.md`
```

### Entregables Fase 6

- [ ] Tag v0.7.0
- [ ] RQL_SCRIPTING.md completo
- [ ] MIGRATION_M6_TO_M7.md
- [ ] demo_full_script.rql
- [ ] Benchmarks de scripting
- [ ] CHANGELOG.md actualizado

---

## ✅ CRITERIOS DE ÉXITO GLOBALES

### Funcionales

- ✅ IF/THEN/ELSE funciona con condiciones complejas
- ✅ FOR itera sobre resultados correctamente
- ✅ MACRO se puede definir, llamar y reutilizar
- ✅ IMPORT MACRO carga desde archivos
- ✅ RUNSUM/RUNAVG traducen a window functions
- ✅ GRAPH BAR renderiza gráficos ASCII
- ✅ SAVE/LOAD SESSION preserva estado completo
- ✅ PRINT output funciona en REPL y TUI
- ✅ PIPE TO envía a comandos shell

### Performance

- ✅ Macros se expanden en <1ms
- ✅ FOR sobre 1000 filas: <100ms
- ✅ Session save/load: <500ms
- ✅ GRAPH rendering: <50ms

### Calidad

- ✅ Test coverage: >80%
- ✅ Zero clippy warnings
- ✅ RQL_SCRIPTING.md completo
- ✅ demo_full_script.rql funcional

---

## 🚧 DEPENDENCIAS Y RIESGOS

### Dependencias

| Dependencia | Criticidad | Notas |
|-------------|------------|-------|
| M6 "FABRIC" | **CRITICAL** | Debe estar 100% completo |
| DuckDB window functions | High | Para RUNSUM, etc. |
| TOML serialization | Medium | Para SAVE/LOAD SESSION |

### Riesgos

| Riesgo | Probabilidad | Impacto | Mitigación |
|--------|--------------|---------|------------|
| Complejidad de macros | Medium | High | Tests exhaustivos |
| Performance de FOR loops | Low | Medium | Optimización lazy |
| Security en PIPE TO | High | High | Whitelist de comandos |

---

## 📞 RECURSOS Y REFERENCIAS

### Inspiración

- **4GL Languages**: Informix 4GL, Progress 4GL
- **Scripting**: AWK, sed
- **Macros**: C preprocessor, Rust macros

### Noctra Docs

- [M6_IMPLEMENTATION_PLAN.md](M6_IMPLEMENTATION_PLAN.md) - Pre-requisito
- [PROJECT_STATUS.md](PROJECT_STATUS.md)
- [DESIGN.md](DESIGN.md)

---

## 🎯 EJEMPLO FINAL COMPLETO

```rql
-- analisis_completo.rql
IMPORT MACRO FROM 'analytics_lib.rql';

LET pais = 'AR';
LET periodo = 'month';

USE 'ventas_2025.csv' AS v;
USE 'clientes.db' AS c;

-- Resumen general
PRINT "=== Análisis de Ventas", #pais, "===";

CALL resumen_periodo(#periodo);

-- Top productos
MACRO top_n(tabla, columna, n) AS
  SELECT :columna, COUNT(*) AS cnt
  FROM :tabla
  GROUP BY :columna
  ORDER BY cnt DESC
  LIMIT :n;
END;

CALL top_n('v', 'producto', 10);

GRAPH BAR FROM top_n('v', 'producto', 10);

-- Análisis por región
FOR region IN (SELECT DISTINCT region FROM v WHERE pais = #pais) DO
  PRINT "Región:", region.region;

  SELECT
    fecha,
    SUM(total) AS ventas,
    RUNSUM(SUM(total)) AS acumulado
  FROM v
  WHERE pais = #pais AND region = region.region
  GROUP BY fecha
  ORDER BY fecha
  LIMIT 30
  PIPE TO 'column -t | head -10';

  EXPORT (SELECT * FROM v WHERE pais = #pais AND region = region.region)
  TO CONCAT('region_', region.region, '.parquet')
  FORMAT PARQUET;
END;

SAVE SESSION 'analisis_completo.toml';

PRINT "=== Completado ===";
```

---

**Noctra(🦆) — 4GL Scripting Environment**
**"De query language a programming language para datos"**

---

**Última actualización:** 2025-11-11
**Autor:** Claude (Anthropic) + wirednil
**Versión del Plan:** 1.0
