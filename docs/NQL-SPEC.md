# NQL - Noctra Query Language Specification

**Version:** 1.0 (Draft)
**Date:** 2025-11-09
**Status:** Propuesta / M4 Planificado

---

## 📋 Tabla de Contenidos

1. [Introducción](#introducción)
2. [Filosofía y Objetivos](#filosofía-y-objetivos)
3. [Sintaxis General](#sintaxis-general)
4. [Comandos Administrativos](#comandos-administrativos)
5. [Operaciones de Datos](#operaciones-de-datos)
6. [Transformaciones Declarativas](#transformaciones-declarativas)
7. [Variables y Sesiones](#variables-y-sesiones)
8. [Compatibilidad con SQL](#compatibilidad-con-sql)
9. [Ejemplos Completos](#ejemplos-completos)
10. [Implementación Técnica](#implementación-técnica)

---

## Introducción

**NQL (Noctra Query Language)** es una extensión del lenguaje SQL estándar que permite trabajar con múltiples fuentes de datos (bases de datos relacionales, archivos CSV, JSON, datasets en memoria) mediante una sintaxis unificada.

### Características Clave

- ✅ **100% compatible con SQL estándar**
- 📄 **Soporte nativo para CSV y archivos planos**
- 🔄 **Import/Export entre formatos**
- 🎯 **Sintaxis unificada** para todas las fuentes
- 🛠️ **Transformaciones declarativas**
- 📊 **Administración multi-fuente**

### Relación con RQL

NQL extiende **RQL (RQL Extensions)** agregando:
- Soporte para fuentes no-SQL (CSV, JSON)
- Comandos administrativos de fuentes
- Transformaciones funcionales (MAP, FILTER)
- Semántica unificada de ejecución

---

## Filosofía y Objetivos

### Objetivo Central

> **Permitir al usuario trabajar con cualquier fuente de datos usando la misma sintaxis SQL familiar, sin distinguir el origen.**

### Principios de Diseño

1. **Uniformidad**: Todos los comandos funcionan igual en SQLite, CSV, o memoria
2. **Simplicidad**: Sintaxis intuitiva basada en SQL estándar
3. **Compatibilidad**: No romper ningún código SQL existente
4. **Extensibilidad**: Fácil agregar nuevos tipos de fuentes
5. **Performance**: Optimizaciones específicas por tipo de fuente

---

## Sintaxis General

### Categorías de Comandos

| Categoría | Comandos | Propósito |
|-----------|----------|-----------|
| SQL Estándar | SELECT, INSERT, UPDATE, DELETE | Manipulación de datos |
| Administrativos | USE, SHOW, DESCRIBE | Gestión de fuentes |
| Import/Export | IMPORT, EXPORT | Transferencia de datos |
| Transformación | MAP, FILTER | Procesamiento declarativo |
| Sesión | LET, UNSET, SHOW VARS | Variables y estado |

### Precedencia Sintáctica

```
1. SQL Estándar (SELECT, INSERT, etc.)    ← Prioridad ALTA
2. Comandos NQL (USE, SHOW, etc.)         ← Prioridad MEDIA
3. Transformaciones (MAP, FILTER)         ← Prioridad BAJA
```

---

## Comandos Administrativos

### USE - Cambiar Fuente de Datos

**Sintaxis:**
```sql
USE <path> [AS <alias>] [OPTIONS];
```

**Ejemplos:**
```sql
-- Cargar base de datos SQLite
USE 'demo.db' AS demo;

-- Cargar archivo CSV
USE 'clientes.csv' AS csv;

-- Cambiar a fuente ya cargada
USE demo;

-- Con opciones específicas
USE 'data.csv' AS csv OPTIONS (delimiter=';', header=true);
```

**Opciones por tipo de fuente:**

#### SQLite
```sql
USE 'mydb.db' OPTIONS (
    readonly = true,
    timeout = 5000
);
```

#### CSV
```sql
USE 'data.csv' OPTIONS (
    delimiter = ';',      -- Delimitador (default: auto-detect)
    header = true,        -- Primera fila como headers (default: true)
    encoding = 'utf-8',   -- Encoding (default: auto-detect)
    quote = '"'           -- Carácter de quote (default: ")
);
```

### SHOW SOURCES - Listar Fuentes

**Sintaxis:**
```sql
SHOW SOURCES;
```

**Output:**
```
+----------+-----------------+-------------------+
| Alias    | Tipo            | Path              |
|----------|-----------------|-------------------|
| demo     | sqlite          | demo.db           |
| csv      | csv             | clientes.csv      |
| memory   | memory          | (in-memory)       |
+----------+-----------------+-------------------+
```

### SHOW TABLES - Listar Tablas/Datasets

**Sintaxis:**
```sql
SHOW TABLES [FROM <source>];
```

**Ejemplos:**
```sql
-- Mostrar tablas de la fuente actual
SHOW TABLES;

-- Mostrar tablas de fuente específica
SHOW TABLES FROM demo;
```

### SHOW / DESCRIBE - Describir Esquema

**Sintaxis:**
```sql
SHOW <table>;
DESCRIBE <source>.<table>;
```

**Ejemplos:**
```sql
-- Describir tabla de fuente actual
SHOW empleados;

-- Describir tabla de fuente específica
DESCRIBE demo.empleados;
SHOW csv.clientes;
```

**Output:**
```
+------------+---------+------+--------+
| Columna    | Tipo    | Null | Default|
|------------|---------|------|--------|
| id         | INTEGER | NO   | NULL   |
| nombre     | TEXT    | NO   | NULL   |
| edad       | INTEGER | YES  | NULL   |
| activo     | BOOLEAN | NO   | true   |
+------------+---------+------+--------+
```

---

## Operaciones de Datos

### IMPORT - Importar Datos

**Status:** ✅ Implementado en M4 Fase 1 (2025-11-11)

**Sintaxis:**
```sql
IMPORT '<archivo>' AS <tabla> [OPTIONS (key=value, ...)];
```

**Parámetros:**
- `<archivo>`: Ruta al archivo (con comillas simples)
- `<tabla>`: Nombre de la tabla destino en SQLite
- `OPTIONS`:
  - `delimiter`: Delimitador de campos (`,`, `;`, `\t`, `|`) - default: `,`
  - `header`: Si tiene encabezados (`true`/`false`) - default: `true`

**Ejemplos:**
```sql
-- Importar CSV básico (comma-delimited, con headers)
IMPORT 'ventas.csv' AS ventas;

-- Importar TSV (tab-delimited)
IMPORT 'datos.tsv' AS datos OPTIONS (delimiter='\t', header=true);

-- Importar CSV sin headers
IMPORT 'numeros.csv' AS numeros OPTIONS (header=false);

-- Importar con pipe delimiter
IMPORT 'legacy.txt' AS legacy OPTIONS (delimiter='|', header=true);
```

**Comportamiento:**
- ✅ **Crea tabla SQLite** con columnas detectadas del header
- ✅ **Auto-detección de tipos**: Por ahora todas las columnas son TEXT (inferencia de tipos en M4 Fase 2)
- ✅ **Quote-aware parsing**: Respeta comillas en valores CSV
- ✅ **Disponible en TUI y REPL**
- ✅ **SQL injection prevention**: Usa valores literales escapados

**Formatos Soportados:**
- ✅ CSV (`.csv`) - completamente funcional
- ❌ JSON (`.json`) - no implementado en M4 Fase 1 (planeado para M5)

**Limitaciones Actuales:**
- No soporta archivos >1GB (sin streaming)
- No infiere tipos automáticamente (todas columnas TEXT)
- No soporta skip_rows (planned for M4 Fase 2)
- Parsing CSV simplificado (no RFC 4180 completo)

### EXPORT - Exportar Datos

**Status:** ✅ Implementado en M4 Fase 1 (2025-11-11)

**Sintaxis:**
```sql
EXPORT <tabla|query> TO '<archivo>' FORMAT <formato> [OPTIONS (key=value, ...)];
```

**Parámetros:**
- `<tabla|query>`: Nombre de tabla o query SELECT completa
- `<archivo>`: Ruta del archivo destino (con comillas simples)
- `FORMAT`: Formato de exportación requerido (CSV, JSON, XLSX)
- `OPTIONS`:
  - **Para CSV:**
    - `delimiter`: Delimitador (`,`, `;`, `\t`, `|`) - default: `,`
    - `header`: Incluir encabezados (`true`/`false`) - default: `true`
  - **Para JSON:** (todas aplicadas automáticamente)
    - Pretty-printing automático
    - Conversión de tipos automática

**Ejemplos:**
```sql
-- Exportar tabla completa a CSV
EXPORT empleados TO 'empleados.csv' FORMAT CSV;

-- Exportar con delimitador personalizado
EXPORT ventas TO 'ventas.tsv' FORMAT CSV OPTIONS (delimiter='\t', header=true);

-- Exportar resultado de query a CSV
EXPORT (SELECT * FROM empleados WHERE activo = true)
TO 'empleados_activos.csv' FORMAT CSV;

-- Exportar a JSON (pretty-printed automático)
EXPORT empleados TO 'empleados.json' FORMAT JSON;

-- Exportar query compleja a JSON
EXPORT (SELECT nombre, email, COUNT(pedidos.id) AS total_pedidos
        FROM usuarios
        LEFT JOIN pedidos ON usuarios.id = pedidos.usuario_id
        GROUP BY usuarios.id)
TO 'reporte_usuarios.json' FORMAT JSON;

-- CSV sin headers
EXPORT datos TO 'datos_raw.csv' FORMAT CSV OPTIONS (header=false);
```

**Formatos Soportados:**
- ✅ **CSV** (`.csv`) - completamente funcional
  - Escaping automático de comillas, newlines, delimiters
  - Soporte para custom delimiters
  - Headers opcionales
- ✅ **JSON** (`.json`) - completamente funcional
  - Pretty-printed automático
  - Conversión automática de tipos (INTEGER, FLOAT, BOOLEAN, NULL, TEXT)
  - Arrays de objetos estándar
- ❌ **XLSX** (`.xlsx`) - no implementado en M4 Fase 1 (planeado para M5)

**Comportamiento:**
- ✅ **Soporta queries complejas**: SELECT, JOINs, GROUP BY, etc.
- ✅ **Soporta nombres de tabla**: Convierte automáticamente a `SELECT * FROM tabla`
- ✅ **Disponible en TUI y REPL**
- ✅ **Proper CSV escaping**: Maneja comillas, newlines y delimiters en valores
- ✅ **Type-aware JSON**: Convierte tipos SQL a tipos JSON correctos

**Limitaciones Actuales:**
- No soporta exportación parcial (column selection) - debe hacerse en query
- No hay progreso de exportación para archivos grandes
- JSON siempre es array de objetos (no soporta otros formatos)
- XLSX no implementado (planned for M5)

---

## Transformaciones Declarativas

### MAP - Transformar Valores

**Sintaxis:**
```sql
MAP <expresión>;
```

**Ejemplos:**
```sql
-- Convertir nombres a mayúsculas
MAP UPPER(nombre);

-- Concatenar campos
MAP CONCAT(apellido, ', ', nombre) AS nombre_completo;

-- Cálculos
MAP precio * 1.21 AS precio_con_iva;

-- Múltiples transformaciones
MAP UPPER(nombre), TRIM(apellido), edad + 1 AS siguiente_edad;
```

**Comportamiento:**
- Aplica transformación a todos los registros
- Crea columnas virtuales temporales
- No modifica los datos originales
- Se puede combinar con SELECT

### FILTER - Filtrar Registros

**Sintaxis:**
```sql
FILTER <condición>;
```

**Ejemplos:**
```sql
-- Filtro simple
FILTER edad > 18;

-- Filtro compuesto
FILTER pais = 'AR' AND activo = true;

-- Con IN
FILTER departamento IN ('IT', 'Ventas', 'RRHH');

-- Con LIKE
FILTER nombre LIKE 'Juan%';
```

**Comportamiento:**
- Filtra registros antes de procesar
- Similar a WHERE pero más declarativo
- Se puede combinar con MAP y SELECT

### Pipeline de Transformaciones

**Sintaxis:**
```sql
USE <source>;
FILTER <condición>;
MAP <expresión>;
SELECT ...;
```

**Ejemplo completo:**
```sql
USE 'empleados.csv' AS emp;
FILTER activo = true AND edad >= 25;
MAP UPPER(nombre) AS nombre_upper, salario * 12 AS salario_anual;
SELECT nombre_upper, departamento, salario_anual
FROM emp
ORDER BY salario_anual DESC
LIMIT 10;
```

---

## Variables y Sesiones

### LET - Definir Variables

**Sintaxis:**
```sql
LET <variable> = <valor|expresión>;
```

**Ejemplos:**
```sql
-- Variable simple
LET pais = 'AR';

-- Variable numérica
LET min_edad = 18;

-- Variable calculada
LET tasa_iva = 1.21;

-- Usar variables en queries
SELECT * FROM clientes
WHERE pais = $pais AND edad >= $min_edad;
```

**Tipos de variables:**
- String: `LET nombre = 'texto';`
- Number: `LET edad = 25;`
- Boolean: `LET activo = true;`
- Null: `LET vacio = NULL;`

### SHOW VARS - Mostrar Variables

**Sintaxis:**
```sql
SHOW VARS;
```

**Output:**
```
+------------+---------+-------+
| Variable   | Tipo    | Valor |
|------------|---------|-------|
| pais       | TEXT    | AR    |
| min_edad   | INTEGER | 18    |
| tasa_iva   | REAL    | 1.21  |
+------------+---------+-------+
```

### UNSET - Eliminar Variables

**Sintaxis:**
```sql
UNSET <variable>;
```

**Ejemplos:**
```sql
-- Eliminar una variable
UNSET pais;

-- Eliminar múltiples
UNSET min_edad, tasa_iva;
```

---

## Compatibilidad con SQL

### SQL Estándar

NQL es **100% compatible** con SQL estándar. Todo query SQL válido es un query NQL válido.

**Ejemplos:**
```sql
-- SQL puro
SELECT * FROM empleados WHERE dept = 'IT';
INSERT INTO empleados (id, nombre) VALUES (1, 'Juan');
UPDATE empleados SET salario = 50000 WHERE id = 1;
DELETE FROM empleados WHERE id = 1;

-- DDL
CREATE TABLE productos (id INTEGER, nombre TEXT);
DROP TABLE temporal;
```

### Extensiones RQL

NQL incluye todas las extensiones RQL:

```sql
-- Parámetros nombrados
SELECT * FROM users WHERE id = :user_id;

-- Parámetros posicionados
SELECT * FROM users WHERE dept = $1 AND active = $2;

-- Templates condicionales
SELECT * FROM employees
WHERE 1=1
{{#if dept}} AND dept = :dept {{/if}}
{{#if min_salary}} AND salary >= :min_salary {{/if}};
```

### Resolución de Ambigüedades

**Orden de interpretación:**

1. **Comandos NQL específicos** (USE, IMPORT, etc.)
2. **SQL estándar** (SELECT, INSERT, etc.)
3. **Transformaciones** (MAP, FILTER)

**Ejemplo:**
```sql
-- Esto es interpretado como:
-- 1. USE (comando NQL)
-- 2. SELECT (SQL estándar)
USE 'data.csv';
SELECT * FROM data;
```

---

## Ejemplos Completos

### Caso 1: Análisis Básico de CSV

```sql
-- Cargar archivo CSV
USE 'ventas_2024.csv' AS ventas;

-- Ver estructura
SHOW ventas;

-- Consultar datos
SELECT producto, SUM(cantidad) as total_vendido
FROM ventas
GROUP BY producto
ORDER BY total_vendido DESC
LIMIT 10;

-- Exportar resultados
EXPORT (
    SELECT producto, SUM(cantidad) as total
    FROM ventas
    GROUP BY producto
) TO 'resumen_productos.csv';
```

### Caso 2: Migración entre Formatos

```sql
-- Cargar datos legacy de CSV
USE 'legacy_data.csv' AS legacy;

-- Ver qué hay
SHOW TABLES FROM legacy;
SHOW legacy;

-- Conectar a base de datos destino
USE 'production.db' AS prod;

-- Importar CSV como tabla temporal
IMPORT 'legacy_data.csv' AS staging;

-- Transformar y migrar
INSERT INTO prod.clientes
SELECT
    id,
    UPPER(TRIM(nombre)) as nombre,
    LOWER(TRIM(email)) as email,
    pais
FROM staging
WHERE activo = 1 AND email IS NOT NULL;

-- Verificar
SELECT COUNT(*) FROM prod.clientes;
```

### Caso 3: Transformaciones Complejas

```sql
-- Cargar fuente
USE 'empleados.csv' AS emp;

-- Definir variables
LET pais_filtro = 'AR';
LET edad_minima = 25;
LET salario_base = 50000;

-- Pipeline de transformaciones
FILTER pais = $pais_filtro AND edad >= $edad_minima;
MAP
    UPPER(TRIM(nombre)) AS nombre_limpio,
    salario * 1.21 AS salario_bruto,
    CASE
        WHEN salario >= $salario_base THEN 'Senior'
        ELSE 'Junior'
    END AS nivel;

-- Query final
SELECT
    nombre_limpio,
    departamento,
    salario_bruto,
    nivel
FROM emp
WHERE nivel = 'Senior'
ORDER BY salario_bruto DESC;

-- Exportar
EXPORT (
    SELECT * FROM emp WHERE nivel = 'Senior'
) TO 'empleados_senior.json' FORMAT json OPTIONS (pretty=true);
```

### Caso 4: Multi-Fuente

```sql
-- Trabajar con múltiples fuentes simultáneamente
USE 'clientes.csv' AS csv_clientes;
USE 'orders.db' AS db_orders;

-- Ver fuentes activas
SHOW SOURCES;

-- Join entre fuentes (requiere IMPORT a una fuente común)
USE db_orders;
IMPORT 'clientes.csv' AS temp_clientes;

SELECT
    c.nombre,
    c.pais,
    COUNT(o.id) as total_pedidos,
    SUM(o.monto) as total_gastado
FROM temp_clientes c
LEFT JOIN orders o ON c.id = o.cliente_id
GROUP BY c.id, c.nombre, c.pais
ORDER BY total_gastado DESC;
```

---

## Implementación Técnica

### Arquitectura de Componentes

```
┌─────────────────────────────────────────┐
│          NQL Parser                     │
│  ┌────────────┬──────────────────────┐ │
│  │ SQL Parser │ NQL Extensions Parser│ │
│  └────────────┴──────────────────────┘ │
└─────────────────┬───────────────────────┘
                  │
                  ▼
┌─────────────────────────────────────────┐
│       NQL Executor                      │
│  ┌─────────────────────────────────┐   │
│  │  DataSource Trait               │   │
│  │  ┌──────┬──────┬──────┬──────┐  │   │
│  │  │SQLite│ CSV  │ JSON │Memory│  │   │
│  │  └──────┴──────┴──────┴──────┘  │   │
│  └─────────────────────────────────┘   │
└─────────────────────────────────────────┘
```

### DataSource Trait

```rust
pub trait DataSource: Send + Sync {
    /// Ejecutar query contra la fuente
    fn query(&self, sql: &str) -> Result<ResultSet>;

    /// Obtener esquema de la fuente
    fn schema(&self) -> Result<Vec<Table>>;

    /// Tipo de fuente
    fn source_type(&self) -> SourceType;

    /// Nombre/identificador de la fuente
    fn name(&self) -> &str;

    /// Cerrar la fuente
    fn close(&mut self) -> Result<()>;
}

pub enum SourceType {
    SQLite { path: String },
    CSV {
        path: String,
        delimiter: char,
        has_header: bool,
        encoding: String
    },
    JSON { path: String },
    Memory { capacity: usize },
}
```

### CSV Query Capabilities

**Status:** ✅ Implementado en M4 Fase 2 (2025-11-11)

El backend CSV de Noctra soporta consultas SQL avanzadas directamente sobre archivos CSV sin necesidad de importarlos a SQLite.

#### Operaciones Soportadas

**SELECT con WHERE:**
```sql
-- Filtrado básico
SELECT * FROM employees WHERE dept = 'IT';

-- Operadores de comparación
SELECT * FROM products WHERE price > 100 AND stock <= 50;

-- Operadores lógicos
SELECT * FROM users WHERE (age >= 18 AND age <= 65) OR vip = true;
```

**Operadores WHERE:**
- `=` - Igualdad
- `!=` - Desigualdad
- `<` - Menor que
- `>` - Mayor que
- `<=` - Menor o igual
- `>=` - Mayor o igual
- `AND` - Conjunción lógica
- `OR` - Disyunción lógica

**ORDER BY:**
```sql
-- Ordenamiento simple
SELECT * FROM products ORDER BY price DESC;

-- Ordenamiento múltiple
SELECT * FROM employees ORDER BY dept ASC, salary DESC;

-- Con WHERE
SELECT * FROM products WHERE category = 'Electronics' ORDER BY price ASC;
```

**LIMIT y OFFSET (Paginación):**
```sql
-- Primeros 10 registros
SELECT * FROM users LIMIT 10;

-- Registros 11-20 (página 2)
SELECT * FROM users LIMIT 10 OFFSET 10;

-- Top 5 productos más caros
SELECT * FROM products ORDER BY price DESC LIMIT 5;
```

**Funciones de Agregación:**
```sql
-- Contar registros
SELECT COUNT(*) FROM sales;
SELECT COUNT(customer_id) FROM orders;

-- Sumar valores numéricos
SELECT SUM(amount) FROM transactions;
SELECT SUM(quantity * price) FROM orders WHERE status = 'completed';

-- Promedio
SELECT AVG(salary) FROM employees WHERE dept = 'Engineering';

-- Mínimo y máximo
SELECT MIN(age) FROM users;
SELECT MAX(price) FROM products WHERE category = 'Laptops';

-- Agregaciones con WHERE
SELECT COUNT(*) FROM sales WHERE date >= '2024-01-01';
SELECT AVG(rating) FROM reviews WHERE product_id = 123;
```

#### Inferencia de Tipos

El backend CSV detecta automáticamente los tipos de datos:
- **INTEGER**: Valores numéricos enteros (ej: `123`, `-456`)
- **REAL**: Valores numéricos decimales (ej: `19.99`, `-3.14`)
- **BOOLEAN**: Valores booleanos (`true`, `false`, `t`, `f`, `1`, `0`, `yes`, `no`)
- **TEXT**: Cualquier otro valor

Los tipos se infieren analizando las primeras 100 filas del CSV.

#### Comparaciones Type-Aware

Las comparaciones respetan los tipos detectados:
```sql
-- Comparación numérica (no lexicográfica)
SELECT * FROM data WHERE age > 25;  -- 30 > 25 ✅, "3" > "25" ❌

-- Comparación de texto
SELECT * FROM users WHERE name >= 'M';  -- Ordenamiento alfabético

-- Comparación booleana
SELECT * FROM products WHERE active = true;
```

#### Seguridad y Límites

**File Path Sandboxing:**
- ❌ Bloqueado: Directorios del sistema (`/etc`, `/sys`, `/proc`, `/dev`, `/root`, `/boot`)
- ❌ Bloqueado: Path traversal (`..` patterns)
- ❌ Bloqueado: Dispositivos/sockets (solo archivos regulares)
- ✅ Permitido: Archivos locales y rutas relativas

**SQL Injection Prevention:**
- Validación de nombres de tabla (solo alfanuméricos, `_`, `-`)
- Escapado de valores en IMPORT (`'` → `''`)
- Sanitización de nombres de columna

**Resource Limits:**
- Tamaño máximo de archivo: 100MB
- Filas máximas por CSV: 1,000,000
- Timeout de query: No implementado (los límites de tamaño/filas proveen protección)

#### Limitaciones Actuales

- ❌ No soporta JOIN entre archivos CSV (planeado para M5)
- ❌ No soporta GROUP BY con agregaciones (solo una agregación por query)
- ❌ WHERE no soporta LIKE, IN, BETWEEN, IS NULL (planeado para M5)
- ❌ No soporta subconsultas
- ❌ Las agregaciones convierten todos los numéricos a REAL (f64)
- ❌ No soporta expresiones complejas en SELECT (ej: `SELECT price * 1.1 AS price_with_tax`)

#### Ejemplos Completos

**Análisis de Ventas:**
```sql
-- Cargar archivo CSV
USE 'sales_2024.csv' AS sales;

-- Total de ventas
SELECT COUNT(*) FROM sales;

-- Ventas superiores a $1000
SELECT * FROM sales WHERE amount > 1000 ORDER BY amount DESC;

-- Suma de ventas por trimestre (manual)
SELECT SUM(amount) FROM sales WHERE month IN (1, 2, 3);  -- Q1

-- Top 10 ventas
SELECT * FROM sales ORDER BY amount DESC LIMIT 10;

-- Promedio de ventas en región
SELECT AVG(amount) FROM sales WHERE region = 'West';
```

**Análisis de Empleados:**
```sql
-- Cargar empleados
USE 'employees.csv' AS emp;

-- Empleados IT con salario alto
SELECT * FROM employees WHERE dept = 'IT' AND salary > 80000 ORDER BY salary DESC;

-- Conteo por departamento (manual para cada dept)
SELECT COUNT(*) FROM employees WHERE dept = 'Engineering';

-- Rango salarial
SELECT MIN(salary) FROM employees;
SELECT MAX(salary) FROM employees;
SELECT AVG(salary) FROM employees;
```

### CSV Backend Implementation

```rust
pub struct CsvDataSource {
    path: PathBuf,
    delimiter: char,
    has_header: bool,
    encoding: String,
    schema: Schema,
    data: Vec<Row>,
}

impl CsvDataSource {
    pub fn new(path: PathBuf, options: CsvOptions) -> Result<Self> {
        // Auto-detect delimiter if not specified
        let delimiter = options.delimiter.unwrap_or_else(|| {
            Self::detect_delimiter(&path)
        });

        // Auto-detect encoding
        let encoding = options.encoding.unwrap_or_else(|| {
            Self::detect_encoding(&path)
        });

        // Load and parse CSV
        let (schema, data) = Self::parse_csv(&path, delimiter, options.has_header, &encoding)?;

        Ok(Self {
            path,
            delimiter,
            has_header: options.has_header,
            encoding,
            schema,
            data,
        })
    }

    fn detect_delimiter(path: &Path) -> char {
        // Implementation: sample first rows and detect most common delimiter
    }

    fn detect_encoding(path: &Path) -> String {
        // Implementation: use encoding_rs to detect
    }

    fn parse_csv(
        path: &Path,
        delimiter: char,
        has_header: bool,
        encoding: &str
    ) -> Result<(Schema, Vec<Row>)> {
        // Implementation: parse CSV and infer types
    }
}

impl DataSource for CsvDataSource {
    fn query(&self, sql: &str) -> Result<ResultSet> {
        // Convert CSV data to temporary SQLite table
        // Execute query against SQLite
        // Return results
    }

    fn schema(&self) -> Result<Vec<Table>> {
        Ok(vec![Table {
            name: self.path.file_stem().unwrap().to_string_lossy().to_string(),
            columns: self.schema.columns.clone(),
        }])
    }

    fn source_type(&self) -> SourceType {
        SourceType::CSV {
            path: self.path.to_string_lossy().to_string(),
            delimiter: self.delimiter,
            has_header: self.has_header,
            encoding: self.encoding.clone(),
        }
    }

    fn name(&self) -> &str {
        self.path.file_stem().unwrap().to_str().unwrap()
    }
}
```

### NQL Parser Extensions

```rust
pub enum NqlCommand {
    // Administrativos
    Use { path: String, alias: Option<String>, options: HashMap<String, Value> },
    ShowSources,
    ShowTables { source: Option<String> },
    Describe { source: Option<String>, table: String },

    // Import/Export
    Import { file: String, table: String, options: ImportOptions },
    Export { query: String, file: String, format: ExportFormat, options: ExportOptions },

    // Transformaciones
    Map { expressions: Vec<MapExpression> },
    Filter { condition: SqlExpression },

    // Variables
    Let { name: String, value: Value },
    Unset { names: Vec<String> },
    ShowVars,

    // SQL estándar (pass-through)
    Sql { statement: SqlStatement },
}

impl NqlParser {
    pub fn parse(&self, input: &str) -> Result<NqlCommand> {
        let tokens = self.tokenize(input)?;

        match tokens.first() {
            Some(Token::Keyword(kw)) => match kw.to_uppercase().as_str() {
                "USE" => self.parse_use(&tokens),
                "SHOW" => self.parse_show(&tokens),
                "DESCRIBE" | "DESC" => self.parse_describe(&tokens),
                "IMPORT" => self.parse_import(&tokens),
                "EXPORT" => self.parse_export(&tokens),
                "MAP" => self.parse_map(&tokens),
                "FILTER" => self.parse_filter(&tokens),
                "LET" => self.parse_let(&tokens),
                "UNSET" => self.parse_unset(&tokens),
                _ => self.parse_sql(&tokens),
            },
            _ => self.parse_sql(&tokens),
        }
    }
}
```

---

## Roadmap de Implementación

### Fase 1: Foundation (Semana 1-2)
- [ ] Definir trait `DataSource`
- [ ] Implementar `CsvDataSource` básico
- [ ] Parser NQL básico (USE, SHOW)
- [ ] Tests unitarios

### Fase 2: Core Features (Semana 3-4)
- [ ] IMPORT/EXPORT
- [ ] MAP/FILTER
- [ ] Variables (LET, UNSET)
- [ ] Integration tests

### Fase 3: Advanced (Semana 5-6)
- [ ] Auto-detección CSV
- [ ] JSON support
- [ ] TUI contextual
- [ ] Optimizaciones

### Fase 4: Polish (Semana 7-8)
- [ ] Documentación completa
- [ ] Ejemplos de uso
- [ ] Performance tuning
- [ ] E2E tests

---

## Referencias

- [RQL Extensions](./RQL-EXTENSIONS.md) - Extensiones SQL base
- [Design Document](./DESIGN.md) - Arquitectura general
- [Project Status](./PROJECT_STATUS.md) - Estado actual y roadmap

---

**Última actualización:** 2025-11-09
**Estado:** Draft - Propuesta para M4
**Autores:** Noctra Development Team
