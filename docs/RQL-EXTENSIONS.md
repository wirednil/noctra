# RQL (Extended SQL) - Especificación Completa

## Introducción

RQL (Extended SQL) es el lenguaje de consultas de Noctra que extiende SQL estándar con características específicas para parámetros, variables de sesión y comandos especiales. Combina la potencia de sqlparser con extensiones modernas para consultas interactivas.

## SQL Base

RQL soporta SQL estándar basado en el dialecto SQLite con extensiones:

### Consultas Básicas

```sql
-- SELECT con filtros
SELECT id, nombre, salario 
FROM employees 
WHERE dept = 'IT' AND salario > 50000;

-- JOINs
SELECT e.nombre, d.nombre as dept_nombre
FROM employees e
JOIN departments d ON e.dept_id = d.id;

-- Subconsultas
SELECT nombre, salario
FROM employees
WHERE salario > (SELECT AVG(salario) FROM employees);
```

### Operaciones DML

```sql
-- INSERT
INSERT INTO employees (nombre, dept, salario)
VALUES ('Juan Pérez', 'IT', 75000);

-- UPDATE
UPDATE employees 
SET salario = salario * 1.10 
WHERE dept = 'IT';

-- DELETE
DELETE FROM employees 
WHERE fecha_egreso < '2020-01-01';
```

## Extensiones RQL

### Parámetros Posicionados

Los parámetros posicionados utilizan la sintaxis `$1`, `$2`, `$3`, etc.:

```sql
-- Parámetros posicionados
SELECT * FROM employees 
WHERE dept = $1 AND salario > $2;

-- Múltiples parámetros
INSERT INTO employees (nombre, dept, salario)
VALUES ($1, $2, $3);
```

### Parámetros Nombrados

Los parámetros nombrados utilizan la sintaxis `:nombre`:

```sql
-- Parámetros nombrados
SELECT * FROM employees 
WHERE dept = :dept AND salario > :salario_minimo;

-- Con valor por defecto
SELECT * FROM employees 
WHERE (:filtro IS NULL OR nombre LIKE '%' || :filtro || '%');
```

### Detección Automática

El parser RQL extrae automáticamente todos los parámetros:

```sql
-- Input
SELECT * FROM employees WHERE dept = $1 AND nombre = :nombre

-- Parámetros extraídos
["$1", ":nombre"]
```

## Comandos Extendidos

### USE - Cambio de Esquema

```sql
-- Cambiar esquema por defecto
USE payroll;

-- Los siguientes queries usarán el esquema 'payroll'
SELECT * FROM employees;
```

### LET - Variables de Sesión

```sql
-- Asignar variables simples
LET dept = 'SALES';
LET salario_base = 50000;

-- Asignar expresiones
LET fecha_desde = '2023-01-01';
LET condicion = dept || '_';

-- Uso en queries
SELECT * FROM employees 
WHERE dept = :dept AND fecha_ingreso >= :fecha_desde;
```

### FORM - Formularios

```sql
-- Cargar formulario
FORM LOAD 'empleados.toml';

-- Ejecutar formulario
EXECFORM 'empleados.toml';

-- Con parámetros
EXECFORM 'consulta_avanzada.toml' WITH params = (dept='IT', activo=true);
```

### OUTPUT - Redirección de Salida

```sql
-- Redirigir a archivo CSV
OUTPUT TO 'reporte_empleados.csv' FORMAT csv;

-- Redirigir a JSON
OUTPUT TO 'datos.json' FORMAT json;

-- Redirigir a printer (sistema)
OUTPUT TO PRINTER;

-- Cancelar redirección
OUTPUT TO STDOUT;
```

## Template Processing

RQL incluye procesamiento de templates similar a FDL2 para generar SQL dinámico:

### Condicionales

```sql
{{#if dept}} 
    AND dept = :dept 
{{/if}}

{{#unless activo}} 
    AND activo = 0 
{{/unless}}

{{#if_eq tipo "gerente"}} 
    AND nivel >= 5 
{{else}} 
    AND nivel < 5 
{{/if_eq}}
```

### Funciones de Filtro

```sql
{{#if_like nombre "%test%"}} 
    AND nombre LIKE :nombre 
{{/if_like}}

{{#if_in dept ["VENTAS", "MARKETING"]}} 
    AND dept IN (:dept) 
{{/if_in}}

{{#if_between salario 50000 100000}} 
    AND salario BETWEEN :salario_min AND :salario_max 
{{/if_between}}
```

### Variables de Sesión en Templates

```sql
-- Se expande automáticamente con variables de sesión
SELECT * FROM employees 
WHERE dept = :dept 
  AND fecha_ingreso >= :fecha_desde
ORDER BY :ordenamiento;
```

## Funciones Incorporadas

### Funciones de Texto

```sql
-- Manipulación de strings
UPPER(nombre)              -- Convertir a mayúsculas
LOWER(email)               -- Convertir a minúsculas
TRIM(espacios)             -- Eliminar espacios
SUBSTR(texto, 1, 10)       -- Substring
LENGTH(texto)              -- Longitud
REPLACE(texto, 'a', 'b')   -- Reemplazar
```

### Funciones de Fecha

```sql
-- Manejo de fechas
CURRENT_DATE               -- Fecha actual
CURRENT_TIMESTAMP          -- Timestamp actual
DATE_ADD(fecha, INTERVAL 1 DAY)  -- Sumar días
DATEDIFF(fecha1, fecha2)   -- Diferencia en días
FORMAT_DATE(fecha, 'YYYY-MM-DD')  -- Formatear fecha
```

### Funciones Numéricas

```sql
-- Matemáticas
ROUND(numero, decimales)    -- Redondear
CEIL(numero)               -- Techo
FLOOR(numero)              -- Piso
ABS(numero)                -- Valor absoluto
POWER(base, exponente)     -- Potencia

-- Agregaciones personalizadas
RUNSUM(salario)            -- Suma acumulada en ventana
RUNCOUNT(*)                -- Conteo acumulado en ventana
```

## Manejo de Transacciones

```sql
-- Iniciar transacción
BEGIN TRANSACTION;

-- Operaciones múltiples
INSERT INTO log (accion) VALUES ('inicio_proceso');
UPDATE employees SET activo = 1 WHERE dept = 'IT';
INSERT INTO movimientos (tipo, usuario) VALUES ('actualizacion', :usuario);

-- Confirmar o cancelar
COMMIT;
-- o
ROLLBACK;
```

## Control de Flujo (Futuro)

RQL planea incluir estructuras de control modernas:

```sql
-- IF condicional (futuro)
IF :dept = 'IT' THEN
    SELECT * FROM employees WHERE dept = 'IT';
ELSE
    SELECT * FROM employees WHERE dept != 'IT';
END IF;

-- WHILE loop (futuro)
WHILE :contador < 10 DO
    INSERT INTO logs (mensaje) VALUES ('Iteración ' || :contador);
    SET :contador = :contador + 1;
END WHILE;
```

## Precedencia y Asociatividad

### Precedencia de Operadores

1. `()`, `[]` - Agrupación
2. `NOT`, `!` - Negación lógica
3. `*`, `/`, `%` - Multiplicación, división, módulo
4. `+`, `-` - Suma, resta (binarios)
5. `||` - Concatenación
6. `=`, `!=`, `<>`, `<`, `<=`, `>`, `>=` - Comparación
7. `IN`, `NOT IN` - Pertenencia
8. `LIKE`, `NOT LIKE` - Coincidencia de patrones
9. `AND` - Conjunción lógica
10. `OR` - Disyunción lógica

### Asociatividad

```sql
-- Izquierda a derecha (estándar SQL)
1 + 2 + 3  -- 6
10 / 2 / 5 -- 1

-- Derecha a izquierda
NOT NOT true  -- true
```

## Errores y Diagnósticos

### Mensajes de Error Comunes

```
ERROR: Parameter $1 not bound
→ El parámetro $1 fue usado en el query pero no se proporcionó valor

ERROR: Variable 'dept' not defined
→ La variable :dept se usa pero no existe en la sesión

ERROR: Invalid template syntax
→ Error de sintaxis en template processing

ERROR: SQL compilation failed
→ El SQL generado no es válido
```

### Información de Debug

```sql
-- Mostrar plan de ejecución
EXPLAIN SELECT * FROM employees WHERE dept = :dept;

-- Mostrar variables de sesión
SHOW VARIABLES;

-- Mostrar parámetros de query
SHOW PARAMETERS;
```

## Ejemplos Completos

### Consulta con Múltiples Filtros

```sql
-- Variables de sesión
LET dept_default = 'IT';
LET fecha_corte = '2023-06-30';

-- Query con parámetros y templates
SELECT 
    e.id,
    e.nombre,
    e.salario,
    d.nombre as dept_nombre,
    e.fecha_ingreso
FROM employees e
JOIN departments d ON e.dept_id = d.id
WHERE 1=1
{{#if dept}} AND e.dept = :dept {{/if}}
{{#if dept_default}} AND e.dept = :dept_default {{/if}}
{{#if fecha_corte}} AND e.fecha_ingreso <= :fecha_corte {{/if}}
{{#if_like nombre "%test%"}} AND e.nombre LIKE :nombre {{/if_like}}
ORDER BY e.salario DESC
LIMIT :limite;
```

### Batch Processing

```sql
-- Script de procesamiento batch
-- archivo: procesar_empleados.rql

USE payroll;

-- Configurar parámetros
LET fecha_proceso = CURRENT_DATE;
LET dept_objetivo = 'SALES';
LET incremento = 1.05;

-- Mostrar configuración
SELECT 'Procesando empleados de' || :dept_objetivo || 'al' || :fecha_proceso as info;

-- Actualizar salarios
UPDATE employees 
SET salario = salario * :incremento
WHERE dept = :dept_objetivo
  AND activo = 1;

-- Generar reporte
OUTPUT TO 'reporte_salarios_' || :fecha_proceso || '.csv' FORMAT csv;
SELECT dept, COUNT(*) as cantidad, AVG(salario) as salario_promedio
FROM employees 
WHERE activo = 1
GROUP BY dept
ORDER BY salario_promedio DESC;
```

### Formulario Dinámico

```sql
-- Cargar formulario con lógica condicional
FORM LOAD 'consulta_empleados.toml';

-- Ejecutar con parámetros específicos
EXECFORM 'consulta_empleados.toml' WITH 
    dept = 'IT',
    incluir_inactivos = false,
    ordenamiento = 'salario DESC';

-- Redirigir salida
OUTPUT TO 'empleados_it.csv' FORMAT csv;
```

## API de Programación

### Parsing RQL

```rust
use noctra_parser::{RqlParser, RqlAst};

let parser = RqlParser::new();
let input = "SELECT * FROM employees WHERE dept = :dept";
let ast = parser.parse_rql(input)?;

println!("Parsed statements: {}", ast.statements.len());
println!("Extracted parameters: {:?}", ast.parameters);
```

### Template Processing

```rust
use noctra_parser::{TemplateProcessor, Session};

let mut session = Session::new();
session.set_variable("dept", Value::Text("IT".to_string()));

let template = "SELECT * FROM employees WHERE dept = :dept{{#if activo}} AND activo = 1{{/if}}";
let sql = TemplateProcessor::process(&template, &session)?;
```

### Execution

```rust
use noctra_core::{Executor, Session};

let mut executor = Executor::new_sqlite("payroll.db")?;
let mut session = Session::new();

// Set parameters
session.set_parameter("dept", Value::Text("IT".to_string()));

// Execute RQL
let result = executor.exec_rql("SELECT * FROM employees WHERE dept = :dept", &session)?;
```

## Comandos RQL

### Mapeo de Comandos

| RQL Command | Description | Ejemplo |
|-------------|-------------|---------|
| `USE schema` | Cambiar esquema por defecto | `USE payroll;` |
| `LET var = expr` | Asignar variables de sesión | `LET dept = 'IT';` |
| `SELECT ...` | Consulta SQL estándar | `SELECT * FROM employees` |
| `FORM LOAD file` | Cargar formulario TOML | `FORM LOAD 'empleados.toml'` |
| `EXECFORM file` | Ejecutar formulario | `EXECFORM 'empleados.toml'` |
| `OUTPUT TO file` | Redirigir salida | `OUTPUT TO 'data.csv' FORMAT csv` |

### Parámetros en RQL

| RQL Syntax | Description | Ejemplo |
|------------|-------------|---------|
| `$1`, `$2` | Parámetros posicionados | `WHERE dept = $1` |
| `:nombre` | Parámetros nombrados | `WHERE nombre = :nombre` |
| `{{#var}}` | Variables en templates | SQL dinámico con variables |

---

**RQL v1.0** - Extended SQL para Noctra  
**Última actualización:** 2025-11-04