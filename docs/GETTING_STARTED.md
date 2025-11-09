# Getting Started - Guía de Inicio Rápido

## 📚 Documentación Relacionada

- **[README](../README.md)** - Visión general del proyecto
- **[Design Document](DESIGN.md)** - Arquitectura técnica detallada
- **[Roadmap](ROADMAP.md)** - Milestones y timeline de desarrollo
- **[RQL Extensions](RQL-EXTENSIONS.md)** - Referencia completa del lenguaje
- **[NQL Specification](NQL-SPEC.md)** - 🆕 Noctra Query Language (multi-fuente)
- **[FDL2 Specification](FDL2-SPEC.md)** - Lenguaje de definición de formularios
- **[API Reference](API-REFERENCE.md)** - API de programación
- **[Contributing](../CONTRIBUTING.md)** - Cómo contribuir al proyecto

## Introducción

Bienvenido a **Noctra**, el entorno SQL interactivo moderno implementado en Rust. Esta guía te ayudará a instalar, configurar y usar Noctra por primera vez.

## Instalación

### Prerrequisitos

- **Rust**: Versión 1.70 o superior
- **Terminal**: Compatible con ncurses (Linux, macOS, Windows con WSL)
- **SQLite**: Para el backend por defecto
- **Git**: Para clonar el repository

### Verificar Prerrequisitos

```bash
# Verificar Rust
rustc --version
# Debe mostrar: rustc 1.70.0 (ec8a8a0ca 2023-01-25) o superior

# Verificar Cargo
cargo --version

# Verificar SQLite (opcional)
sqlite3 --version
```

### Instalación desde Código Fuente

```bash
# Clonar repository
git clone https://github.com/noctra/noctra.git
cd noctra

# Build del workspace completo
cargo build --workspace

# Ejecutar tests
cargo test --workspace

# Instalar binarios (opcional)
cargo install --bin noctra --bin noctrad --path .
```

### Verificar Instalación

```bash
# Verificar CLI
./target/debug/noctra --help

# Verificar versión
./target/debug/noctra --version
# Debe mostrar: noctra 0.1.0
```

## Primer Uso

### Ejecutar Modo Interactivo

```bash
./target/debug/noctra
```

Deberías ver la interfaz TUI (Noctra Window Manager):

```
+--------------------------------------------------------------------------------+
|                                                                                |
|──( INSERTAR ) SQL Noctra 0.1.0──────────────────────────────────── Cmd: 1────|
|                                                                                |
|                                                                                |
|                                                                                |
|────────────────────────────────────────────────────────────────────────────────|
| F5          :Procesar el comando       End         :Terminar sesión de Noctra  |
| F1          :Ayuda comandos editor     F8          :Interrumpir procesamiento  |
| Prox. pantal:Comando siguiente         Pantall. pre:Comando anterior           |
| Insert      :Insertar espacio          Delete      :Borrar un caracter         |
| Alt      r  :Leer de un Archivo        Alt      w  :Grabar en un archivo       |
|                                                                                |
+--------------------------------------------------------------------------------+
```

### Primer Comando

1. **Tipo tu primer comando** en el área de comandos (debajo del header)
2. **Presiona F5** para ejecutar
3. **Usa las teclas de función** para navegar

```sql
-- Ejemplo: Conectar a una base de datos
use demo;
```

### Comandos Básicos

```sql
-- Ver tablas disponibles
show tables;

-- Ver estructura de una tabla
show demo.employees;

-- Consulta simple
select * from employees limit 5;

-- Usar parámetros
select * from employees where dept = :dept;
-- El sistema te pedirá el valor de :dept
```

## Tutorial Paso a Paso

### Paso 1: Crear Base de Datos de Ejemplo

```bash
# Crear base de datos SQLite
sqlite3 ejemplo.db << 'EOF'
CREATE TABLE employees (
    id INTEGER PRIMARY KEY,
    nombre TEXT NOT NULL,
    dept TEXT NOT NULL,
    salario REAL,
    fecha_ingreso DATE
);

INSERT INTO employees VALUES 
    (1, 'Juan Pérez', 'IT', 75000, '2020-01-15'),
    (2, 'María González', 'VENTAS', 65000, '2021-03-20'),
    (3, 'Carlos Rodríguez', 'IT', 80000, '2019-08-10'),
    (4, 'Ana Martínez', 'RRHH', 55000, '2022-02-01'),
    (5, 'Luis García', 'VENTAS', 70000, '2020-11-05');
EOF
```

### Paso 2: Conectar a la Base de Datos

```sql
-- En Noctra
use demo;
-- Error: Base de datos no encontrada

-- Crear alias para nuestra base
use /ruta/completa/a/ejemplo.db as demo;
-- o configurar en el archivo de configuración
```

### Paso 3: Explorar Datos

```sql
-- Ver todas las tablas
show;

-- Ver estructura de employees
show employees;

-- Ver datos
select * from employees;
```

### Paso 4: Usar Parámetros

```sql
-- Consulta con parámetro
select * from employees where dept = :dept;

-- Cuando ejecutes, el sistema te pedirá:
-- :dept => IT
-- :dept => VENTAS
-- :dept => RRHH
```

### Paso 5: Crear Formulario FDL2

Crea el archivo `consulta_empleados.toml`:

```toml
title = "Consulta de Empleados"

[fields.dept]
label = "Departamento"
field_type = "enum"
options = ["IT", "VENTAS", "RRHH"]
required = false

[fields.salario_min]
label = "Salario Mínimo"
field_type = "integer"
required = false

[actions.consultar]
sql = """
SELECT id, nombre, dept, salario, fecha_ingreso
FROM employees 
WHERE 1=1
{{#if dept}} AND dept = :dept {{/if}}
{{#if salario_min}} AND salario >= :salario_min {{/if}}
ORDER BY nombre;
"""
params = ["dept", "salario_min"]

[views.resultados]
type = "table"
title = "Empleados"
pager = true
```

### Paso 6: Ejecutar Formulario

```sql
-- Cargar formulario
form load 'consulta_empleados.toml';

-- La interfaz TUI mostrará el formulario
-- Llena los campos y presiona F5 para ejecutar
```

## Configuración

### Archivo de Configuración

Crea `~/.config/noctra/config.toml`:

```toml
# Configuración general
[general]
theme = "classic"
timeout = 30
max_rows = 1000

# Configuración de base de datos por defecto
[database.default]
type = "sqlite"
path = "./demo.db"

# Configuración de formularios
[forms]
search_paths = ["./forms", "~/forms"]
auto_save = true

# Configuración de salida
[output]
default_format = "table"
csv_delimiter = ";"
decimal_separator = "."
```

### Variables de Entorno

```bash
# Configuración de tema
export NOCTRA_THEME=classic

# Base de datos por defecto
export NOCTRA_DB_PATH=./demo.db

# Directorio de formularios
export NOCTRA_FORMS_PATH=./forms

# Modo debug
export NOCTRA_DEBUG=1
```

## Casos de Uso Comunes

### 1. Consulta Rápida

```sql
-- Consultar empleados de IT con salario > 70000
use demo;
LET dept_filter = 'IT';
LET salario_min = 70000;

select id, nombre, salario 
from employees 
where dept = :dept_filter and salario > :salario_min;
```

### 2. Análisis de Datos

```sql
-- Agrupar por departamento
select dept, 
       count(*) as cantidad,
       avg(salario) as salario_promedio,
       min(salario) as salario_min,
       max(salario) as salario_max
from employees 
group by dept
order by salario_promedio desc;

-- Mostrar en formato CSV
output to 'reporte_dept.csv' format csv;
```

### 3. Formulario de Alta

```toml
title = "Alta de Empleado"

[fields.nombre]
label = "Nombre Completo"
field_type = "text"
required = true
validations = [{type = "min_length", value = 3}]

[fields.dept]
label = "Departamento"
field_type = "enum"
options = ["IT", "VENTAS", "RRHH", "FINANZAS"]
required = true

[fields.salario]
label = "Salario"
field_type = "float"
required = true
validations = [{type = "min", value = 0}]

[actions.guardar]
type = "insert"
table = "employees"
mapping = { 
    nombre = "nombre",
    dept = "dept", 
    salario = "salario"
}
on_success = "mostrar_confirmacion"

[views.mostrar_confirmacion]
type = "message"
title = "Éxito"
message = "Empleado registrado exitosamente"
```

### 4. Batch Processing

Crea el archivo `proceso_mensual.rql`:

```sql
-- Proceso mensual de empleados
use demo;

-- Configurar parámetros
LET mes_proceso = '2023-11';
LET incremento = 1.05;

-- Mostrar estadísticas actuales
select 'Estadísticas antes del proceso:' as info;
select dept, count(*) as cantidad, avg(salario) as promedio
from employees 
group by dept;

-- Aplicar incrementos
update employees 
set salario = salario * :incremento
where activo = 1;

-- Verificar resultados
select 'Estadísticas después del proceso:' as info;
select dept, count(*) as cantidad, avg(salario) as promedio
from employees 
group by dept;

-- Generar reporte
output to 'reporte_mensual.csv' format csv;
select * from employees order by dept, nombre;
```

Ejecuta en batch:

```bash
noctra -b proceso_mensual.rql
```

## 📄 Trabajar con Archivos CSV (NQL)

**Nuevo en v0.1.0 (M3.5)** - Noctra ahora soporta archivos CSV como fuentes de datos mediante NQL (Noctra Query Language).

### 5. Cargar y Consultar CSV

```sql
-- Cargar archivo CSV
USE './datos/clientes.csv' AS csv OPTIONS (delimiter=',', header=true);

-- Ver fuentes disponibles
SHOW SOURCES;

-- Resultado:
-- Alias | Tipo  | Path
-- ------|-------|----------------------
-- csv   | csv   | ./datos/clientes.csv

-- Ver tablas del CSV
SHOW TABLES FROM csv;

-- Ver estructura
DESCRIBE csv.clientes;

-- Consultar datos
SELECT * FROM clientes;
```

### 6. Opciones de Carga CSV

```sql
-- CSV con punto y coma como delimitador
USE './datos/ventas.csv' AS ventas OPTIONS (delimiter=';', header=true);

-- CSV sin headers
USE './datos/numeros.csv' AS nums OPTIONS (delimiter=',', header=false);

-- El sistema generará columnas: col1, col2, col3, etc.

-- CSV con tabulador
USE './datos/export.tsv' AS tsv OPTIONS (delimiter='\t', header=true);
```

### 7. Múltiples Fuentes de Datos

```sql
-- Registrar varias fuentes
USE './datos/clientes.csv' AS clientes OPTIONS (delimiter=',', header=true);
USE './datos/productos.csv' AS productos OPTIONS (delimiter=',', header=true);
USE './basedatos.db' AS db;

-- Listar todas las fuentes
SHOW SOURCES;

-- Consultar diferentes fuentes
SELECT * FROM clientes;  -- CSV
SELECT * FROM productos; -- CSV
USE db;
SELECT * FROM empleados; -- SQLite
```

### 8. Variables de Sesión

```sql
-- Definir variables
LET pais = 'Argentina';
LET año = '2024';

-- Ver variables
SHOW VARS;

-- Resultado:
-- Variable | Valor
-- ---------|----------
-- pais     | Argentina
-- año      | 2024

-- Usar variables en queries
SELECT * FROM clientes WHERE pais = $pais;

-- Eliminar variables
UNSET pais, año;
```

### 9. Ejemplo Completo: Análisis de CSV

**Archivo: ventas_2024.csv**
```csv
fecha,producto,cantidad,precio,vendedor
2024-01-15,Laptop,2,1200.50,Juan Pérez
2024-01-16,Mouse,5,25.00,María González
2024-01-17,Teclado,3,80.00,Juan Pérez
2024-01-18,Monitor,1,350.00,Carlos López
```

**Análisis en Noctra:**
```sql
-- Cargar CSV
USE './ventas_2024.csv' AS ventas OPTIONS (delimiter=',', header=true);

-- Inspeccionar datos
DESCRIBE ventas.ventas_2024;

-- Resultado:
-- Campos    | Tipo
-- ----------|--------
-- fecha     | TEXT
-- producto  | TEXT
-- cantidad  | INTEGER
-- precio    | REAL
-- vendedor  | TEXT

-- Ver todos los datos
SELECT * FROM ventas_2024;

-- Análisis por vendedor (requiere SQLite para GROUP BY)
-- Por ahora solo soporta SELECT * FROM,
-- para análisis avanzado usar IMPORT a SQLite
```

### 10. Limitaciones Actuales de CSV

**Soportado:**
- ✅ `SELECT * FROM table` - Consulta completa
- ✅ Detección automática de delimitadores
- ✅ Inferencia de tipos de datos
- ✅ Headers automáticos o generados

**No soportado aún (próximo M4):**
- ❌ `WHERE`, `JOIN`, `GROUP BY`, `ORDER BY`
- ❌ Selección de columnas específicas
- ❌ `INSERT`, `UPDATE`, `DELETE` en CSV
- ❌ `IMPORT`/`EXPORT` entre fuentes

**Workaround para análisis avanzado:**
```sql
-- Opción 1: Usar herramientas externas
-- sqlite3 basedatos.db
-- .mode csv
-- .import ventas_2024.csv temp_ventas
-- Luego usar Noctra con esa base

-- Opción 2: Esperar M4 que implementará:
-- IMPORT 'ventas.csv' AS temp;
-- INSERT INTO sqlite_table SELECT * FROM temp;
```

### Tipo de Datos CSV

Noctra infiere automáticamente los tipos:

| Valores en CSV | Tipo Inferido | Ejemplo |
|----------------|---------------|---------|
| 1, 2, 100 | INTEGER | id, cantidad |
| 1.5, 3.14, 99.99 | REAL | precio, porcentaje |
| true, false, yes, no | BOOLEAN | activo, disponible |
| Texto mezclado | TEXT | nombre, dirección |

### Ejemplo TUI: CSV en Acción

```
──( RESULTADO ) SQL Noctra 0.1.0 ── Fuente: csv:clientes ────Cmd: 3───

┌──────────────────────────────────────────────────────────────┐
│id    │nombre           │email                  │pais          │
│1     │Juan Pérez       │juan@example.com       │Argentina     │
│2     │María González   │maria@example.com      │Chile         │
│3     │Pedro Rodríguez  │pedro@example.com      │Uruguay       │
│                                                                │
│                                                                │
3 fila(s) retornada(s) - Comando: SELECT * FROM clientes;       │
└──────────────────────────────────────────────────────────────┘
```

Nota el indicador `Fuente: csv:clientes` en la barra de estado.

## Modo Batch

### Ejecutar Script

```bash
# Ejecutar script RQL
noctra -b script.rql

# Ejecutar con parámetros
noctra -b script.rql --param dept=IT --param salario_min=60000

# Ejecutar comando directo
noctra -c "select * from employees where dept = 'IT'"
```

### Crear Script RQL

```sql
-- archivo: consulta_datos.rql
use demo;

-- Configurar output
output to 'resultados.csv' format csv;

-- Query principal
select 
    id,
    nombre,
    dept,
    salario,
    fecha_ingreso,
    case 
        when salario > 70000 then 'Alto'
        when salario > 55000 then 'Medio'
        else 'Bajo'
    end as categoria
from employees 
where 1=1
{{#if dept}} and dept = :dept {{/if}}
{{#if fecha_desde}} and fecha_ingreso >= :fecha_desde {{/if}}
order by salario desc;
```

## Troubleshooting

### Problemas Comunes

**Error: "Database not found"**
```sql
-- Verificar ruta de base de datos
show databases;

-- Conectar explícitamente
use /ruta/completa/a/base.db;
```

**Error: "Table not found"**
```sql
-- Verificar esquema actual
show;

-- Cambiar esquema
use nombre_esquema;
```

**Error: "Parameter not bound"**
```sql
-- El parámetro :param no tiene valor
-- El sistema te pedirá el valor automáticamente
-- O puedes usar:
LET param = 'valor';
```

**Problemas de Display TUI**
```bash
# Verificar soporte de terminal
echo $TERM

# Si hay problemas, usar modo texto
noctra --text-mode

# Configurar variables de terminal
export TERM=xterm-256color
```

### Logs y Debug

```bash
# Ejecutar con logs detallados
NOCTRA_DEBUG=1 noctra

# Ver logs en archivo
noctra 2> noctra.log

# Modo verbose
noctra --verbose
```

### Performance

```sql
-- Ver plan de ejecución
explain select * from employees where dept = :dept;

-- Limitar resultados para testing
select * from employees limit 10;

-- Usar índices
create index idx_employees_dept on employees(dept);
```

## Recursos Adicionales

### Documentación
- **[DESIGN.md](../DESIGN.md)** - Especificación técnica completa
- **[FDL2-SPEC.md](FDL2-SPEC.md)** - Especificación de formularios
- **[RQL-EXTENSIONS.md](RQL-EXTENSIONS.md)** - Extensiones SQL

### Ejemplos
- **[examples/](../examples/)** - Formularios y scripts de ejemplo
- **[tests/](../tests/)** - Casos de prueba

### Comunidad
- **GitHub Issues**: Reportar bugs y solicitar features
- **Discussions**: Preguntas y discusiones
- **Wiki**: Documentación adicional

### Scripts y Automatización
```bash
# Ejecutar scripts SQL batch
noctra --batch scripts/procesar_datos.rql

# Ejecutar scripts con parámetros
noctra --batch scripts/reporte.rql --param dept=IT --param fecha=2023-01-01

# Formularios personalizados
noctra form examples/empleados.toml --param filtro=SALES
```

---

## Próximos Pasos

1. **Explora los ejemplos** en el directorio `examples/`
2. **Crea tus propios formularios** FDL2
3. **Experimenta con RQL** y sus extensiones
4. **Únete a la comunidad** en GitHub
5. **Contribuye** al proyecto

**¡Bienvenido a Noctra! 🚀**

---

**Getting Started v1.1** - Guía para nuevos usuarios
**Última actualización:** 2025-11-09 (Agregado soporte CSV/NQL)