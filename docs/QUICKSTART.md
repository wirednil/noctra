# Guía de Inicio Rápido - Noctra CLI

Esta guía muestra cómo usar el CLI de Noctra con las nuevas características de M6 Phase 2.

## 🚀 Inicio Rápido

### 1. Configurar Variables de Entorno

```bash
# DuckDB precompilado
export DUCKDB_LIB_DIR=/opt/duckdb
export DUCKDB_INCLUDE_DIR=/opt/duckdb
export LD_LIBRARY_PATH=/opt/duckdb:$LD_LIBRARY_PATH
```

O usar direnv con `.envrc`:

```bash
# En la raíz del proyecto
direnv allow
```

### 2. Compilar el Proyecto

```bash
cargo build --release
```

### 3. Ejecutar el REPL

```bash
./target/release/noctra repl
```

O con logging para desarrollo:

```bash
RUST_LOG=debug ./target/debug/noctra repl
```

## 📊 Comandos Disponibles

### Cargar Archivos de Datos

#### CSV
```sql
-- Cargar archivo CSV
use 'examples/clientes.csv' as clientes;

-- Verificar que se cargó
show sources;

-- Consultar datos
select * from clientes;
```

#### JSON
```sql
-- Cargar archivo JSON
use 'data/events.json' as eventos;

-- Consultar con filtros
select * from eventos where status = 'active';
```

#### Parquet
```sql
-- Cargar archivo Parquet
use 'data/sales.parquet' as ventas;

-- Agregaciones
select category, sum(amount) as total
from ventas
group by category;
```

### Consultas Multi-Fuente

```sql
-- Cargar múltiples archivos
use 'data/customers.csv' as customers;
use 'data/orders.json' as orders;

-- JOIN entre fuentes
select
    c.name,
    count(o.order_id) as order_count,
    sum(o.amount) as total_spent
from customers c
join orders o on c.customer_id = o.customer_id
group by c.name
order by total_spent desc;
```

### Comandos de Gestión

```sql
-- Ver todas las fuentes activas
show sources;

-- Ver tablas de una fuente específica
show tables from customers;

-- Describir estructura de tabla
describe customers.clientes;

-- Ayuda
help;

-- Salir
quit;
```

## 🎯 Ejemplo Completo

### Preparar Datos de Prueba

```bash
# Crear directorio de datos
mkdir -p examples/data

# Crear CSV de productos
cat > examples/data/products.csv <<EOF
product_id,name,category,price
1,Laptop,Electronics,1299.99
2,Mouse,Electronics,29.99
3,Desk,Furniture,499.99
4,Chair,Furniture,399.99
EOF

# Crear JSON de ventas
cat > examples/data/sales.json <<EOF
[
  {"sale_id": 1, "product_id": 1, "quantity": 2, "date": "2024-01-15"},
  {"sale_id": 2, "product_id": 2, "quantity": 5, "date": "2024-01-16"},
  {"sale_id": 3, "product_id": 3, "quantity": 1, "date": "2024-01-17"},
  {"sale_id": 4, "product_id": 1, "quantity": 1, "date": "2024-01-18"}
]
EOF
```

### Sesión REPL de Ejemplo

```sql
noctra> use 'examples/data/products.csv' as products;
✅ Fuente 'examples/data/products.csv' cargada como 'products' (DuckDB)

noctra> use 'examples/data/sales.json' as sales;
✅ Fuente 'examples/data/sales.json' cargada como 'sales' (DuckDB)

noctra> show sources;
📊 Fuentes disponibles:
  • products (memory) - (in-memory)
  • sales (memory) - (in-memory)

noctra> -- Análisis de ventas por categoría
select
    p.category,
    sum(s.quantity) as total_units,
    sum(s.quantity * p.price) as revenue
from products p
join sales s on p.product_id = s.product_id
group by p.category
order by revenue desc;

category    | total_units | revenue
------------+-------------+----------
Electronics | 8           | 3049.91
Furniture   | 1           | 499.99

(2 filas)

noctra> -- Producto más vendido
select
    p.name,
    p.price,
    sum(s.quantity) as units_sold,
    sum(s.quantity * p.price) as total_revenue
from products p
join sales s on p.product_id = s.product_id
group by p.product_id, p.name, p.price
order by units_sold desc
limit 3;

name   | price   | units_sold | total_revenue
-------+---------+------------+--------------
Laptop | 1299.99 | 3          | 3899.97
Mouse  | 29.99   | 5          | 149.95
Desk   | 499.99  | 1          | 499.99

(3 filas)

noctra> quit
👋 ¡Hasta luego!
```

## 🔧 Características M6 Phase 2

### 1. Auto-Detección de Formato

El sistema detecta automáticamente el formato por extensión:

```sql
-- .csv → CSV con delimitadores auto-detectados
use 'data.csv' as mydata;

-- .json/.jsonl/.ndjson → JSON arrays o newline-delimited
use 'events.json' as events;

-- .parquet → Parquet columnar
use 'analytics.parquet' as analytics;
```

### 2. Prepared Statement Cache

Automático para todas las queries repetidas (10-30% más rápido):

```sql
-- Primera ejecución: parsea y compila
select * from products where price > 100;

-- Ejecuciones siguientes: usa cache
select * from products where price > 100;
```

### 3. Configuración Optimizada

El sistema usa configuración optimizada por defecto:
- **Threads**: CPU cores (ej: 16 en sistema con 16 cores)
- **Memory Limit**: Usa default de DuckDB (~80% RAM disponible)
- **Catalog Errors**: Limitado a 10 schemas para mensajes rápidos

### 4. Cross-Source JOINs

Combina archivos CSV, JSON y Parquet en una sola query:

```sql
-- CSV + JSON
select * from customers_csv c
join orders_json o on c.id = o.customer_id;

-- CSV + Parquet
select * from products_csv p
join analytics_parquet a on p.product_id = a.product_id;
```

## 📝 Sintaxis RQL Extendida

### USE - Registrar Archivo

```sql
-- Sintaxis básica
use 'ruta/archivo.ext' as alias;

-- Ejemplos
use 'data.csv' as datos;
use './sales/2024.json' as sales_2024;
use '/tmp/analytics.parquet' as analytics;
```

### SHOW SOURCES - Listar Fuentes

```sql
show sources;
```

Salida:
```
📊 Fuentes disponibles:
  • datos (memory) - (in-memory)
  • sales_2024 (memory) - (in-memory)
  • analytics (memory) - (in-memory)
```

### SHOW TABLES - Listar Tablas

```sql
-- Todas las tablas
show tables;

-- Tablas de una fuente específica
show tables from datos;
```

### DESCRIBE - Estructura de Tabla

```sql
describe datos.clientes;
```

Salida:
```
📊 Estructura de datos.clientes:
  Columnas:
    • id (INTEGER)
    • nombre (TEXT)
    • email (TEXT)
  Filas: 150
```

## 🐛 Solución de Problemas

### Error: "unable to find library -lduckdb"

**Causa**: Variables de entorno de DuckDB no configuradas.

**Solución**:
```bash
export DUCKDB_LIB_DIR=/opt/duckdb
export LD_LIBRARY_PATH=/opt/duckdb:$LD_LIBRARY_PATH
cargo build
```

### Error: "syntax error at or near '/'"

**Causa**: Versión antigua con bug de parser (antes de fix 9a82397).

**Solución**: Actualizar a la última versión:
```bash
git pull
cargo build
```

### Error: "File not found"

**Causa**: Ruta incorrecta al archivo.

**Solución**: Usar rutas absolutas o relativas correctas:
```sql
-- ❌ Incorrecto
use 'archivo.csv' as data;

-- ✅ Correcto (relativa a donde se ejecuta noctra)
use './data/archivo.csv' as data;

-- ✅ Correcto (absoluta)
use '/home/user/data/archivo.csv' as data;
```

### Warning: "SQLite extension not available"

**Causa**: DuckDB no puede descargar la extensión SQLite (requiere internet).

**Impacto**: No se pueden usar comandos ATTACH para SQLite.

**Solución**:
- Para archivos: No se necesita, use `USE` directamente
- Para SQLite: Requiere conexión a internet para primera instalación

## 📚 Recursos Adicionales

- **Demo Programático**: `cargo run --example hybrid_demo -p noctra-duckdb`
- **Tests de Integración**: `cargo test -p noctra-duckdb --test integration_hybrid`
- **Documentación Completa**: `docs/M6_PHASE2_STATUS.md`
- **README noctra-duckdb**: `crates/noctra-duckdb/README.md`

## 🎉 Siguientes Pasos

1. **Explorar Formatos**: Prueba CSV, JSON y Parquet
2. **Combinar Fuentes**: Experimenta con JOINs multi-fuente
3. **Analizar Datos**: Usa agregaciones, GROUP BY, window functions
4. **Ver Documentación**: Revisa `docs/` para características avanzadas

---

**Versión**: v0.6.0-alpha2
**Milestone**: M6 Phase 2 - Hybrid Query Engine
**Fecha**: 14 de noviembre de 2025
