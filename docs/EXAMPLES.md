# Ejemplos Completos de Uso - Noctra

Guía completa con ejemplos de uso para todos los modos de Noctra.

## 📚 Tabla de Contenidos

1. [Modo REPL](#modo-repl---interactive-sql)
2. [Modo TUI](#modo-tui---terminal-ui-retro-4gl)
3. [Modo Batch](#modo-batch---procesamiento-por-lotes)
4. [Modo Form](#modo-form---formularios-interactivos)
5. [Modo Query](#modo-query---queries-directos)

---

## Modo REPL - Interactive SQL

El REPL (Read-Eval-Print Loop) es el modo interactivo principal para consultas SQL.

### Inicio del REPL

```bash
# REPL básico
./target/debug/noctra repl

# Con base de datos específica
./target/debug/noctra --database my_data.db repl

# En memoria (temporal)
./target/debug/noctra --memory repl

# Con logging para desarrollo
RUST_LOG=debug ./target/debug/noctra repl
```

### Ejemplo 1: Análisis de Ventas Multi-Formato

```sql
-- Preparar datos de prueba primero
-- Ver sección "Datos de Prueba" al final

-- Sesión REPL
noctra> use 'data/products.csv' as products;
✅ Fuente 'data/products.csv' cargada como 'products' (DuckDB)

noctra> use 'data/sales.json' as sales;
✅ Fuente 'data/sales.json' cargada como 'sales' (DuckDB)

noctra> use 'data/customers.parquet' as customers;
✅ Fuente 'data/customers.parquet' cargada como 'customers' (DuckDB)

noctra> show sources;
📊 Fuentes disponibles:
  • products (memory) - (in-memory)
  • sales (memory) - (in-memory)
  • customers (memory) - (in-memory)

noctra> -- Análisis de ventas por categoría
select
    p.category,
    count(distinct c.customer_id) as unique_customers,
    sum(s.quantity) as total_units,
    round(sum(s.quantity * p.price), 2) as revenue
from products p
join sales s on p.product_id = s.product_id
join customers c on s.customer_id = c.customer_id
group by p.category
order by revenue desc;

category    | unique_customers | total_units | revenue
------------+------------------+-------------+----------
Electronics | 3                | 8           | 3049.91
Furniture   | 1                | 1           | 499.99

(2 filas)

noctra> -- Top 3 clientes por gasto
select
    c.name,
    c.email,
    count(s.sale_id) as orders,
    sum(s.quantity * p.price) as total_spent
from customers c
join sales s on c.customer_id = s.customer_id
join products p on s.product_id = p.product_id
group by c.customer_id, c.name, c.email
order by total_spent desc
limit 3;

name        | email               | orders | total_spent
------------+---------------------+--------+-------------
Alice Brown | alice@example.com   | 2      | 2749.97
Bob Smith   | bob@example.com     | 1      | 149.95
Carol White | carol@example.com   | 1      | 499.99

(3 filas)
```

### Ejemplo 2: Análisis Temporal con Window Functions

```sql
noctra> use 'data/daily_sales.csv' as daily_sales;
✅ Fuente cargada

noctra> -- Tendencia de ventas con promedio móvil
select
    date,
    revenue,
    round(avg(revenue) over (
        order by date
        rows between 6 preceding and current row
    ), 2) as ma_7_days,
    round(revenue - lag(revenue, 1) over (order by date), 2) as daily_change,
    round(100.0 * (revenue - lag(revenue, 1) over (order by date)) /
        nullif(lag(revenue, 1) over (order by date), 0), 2) as pct_change
from daily_sales
order by date desc
limit 10;

date       | revenue | ma_7_days | daily_change | pct_change
-----------+---------+-----------+--------------+------------
2024-11-14 | 1250.50 | 1180.30   | 75.25        | 6.40
2024-11-13 | 1175.25 | 1155.75   | -35.80       | -2.95
...
```

### Ejemplo 3: Agregaciones Complejas

```sql
noctra> -- Matriz de productos por región
select
    p.category,
    sum(case when c.region = 'North' then s.quantity else 0 end) as north,
    sum(case when c.region = 'South' then s.quantity else 0 end) as south,
    sum(case when c.region = 'East' then s.quantity else 0 end) as east,
    sum(case when c.region = 'West' then s.quantity else 0 end) as west,
    sum(s.quantity) as total
from products p
join sales s on p.product_id = s.product_id
join customers c on s.customer_id = c.customer_id
group by p.category
order by total desc;

category    | north | south | east | west | total
------------+-------+-------+------+------+------
Electronics | 3     | 2     | 2    | 1    | 8
Furniture   | 0     | 1     | 0    | 0    | 1
```

### Comandos de Utilidad REPL

```sql
-- Ver historial de comandos
noctra> :history

-- Mostrar configuración actual
noctra> :config

-- Estadísticas de sesión
noctra> :stats
📊 Estado del REPL:
  Líneas procesadas: 15
  Comandos en historial: 12
  Estado: Ready

-- Variables de sesión
noctra> let region = 'North';
✅ Variable 'region' = 'North'

noctra> show vars;
🔧 Variables de sesión:
  region = North

-- Limpiar pantalla
noctra> clear

-- Salir
noctra> quit
👋 ¡Hasta luego!
```

---

## Modo TUI - Terminal UI Retro (4GL)

El modo TUI proporciona una interfaz completa estilo años 80/90 con navegación por teclado.

### Inicio del TUI

```bash
# TUI básico
./target/debug/noctra tui

# Con base de datos específica
./target/debug/noctra tui --database analytics.db

# Con schema inicial
./target/debug/noctra tui --schema public

# Con script SQL pre-cargado
./target/debug/noctra tui --load setup.sql
```

### Layout del TUI

```
┌────────────────────────────────────────────────────────────────────┐
│ 🐍 Noctra TUI v0.1.0                           [F1: Help] [ESC: Quit]│
├────────────────────────────────────────────────────────────────────┤
│ Query Editor                                           [Ctrl+E: Execute]│
│                                                                    │
│ SELECT * FROM products WHERE price > 100;█                        │
│                                                                    │
│                                                                    │
├────────────────────────────────────────────────────────────────────┤
│ Results                                               [5 rows]     │
│┌──────────────┬────────────┬──────────┬────────┐                  │
││ product_id   │ name       │ category │ price  │                  │
│├──────────────┼────────────┼──────────┼────────┤                  │
││ 1            │ Laptop     │ Electro  │ 1299.99│                  │
││ 3            │ Desk       │ Furnitur │ 499.99 │                  │
││ 4            │ Chair      │ Furnitur │ 399.99 │                  │
││ 6            │ Monitor    │ Electro  │ 349.99 │                  │
││ 9            │ Keyboard   │ Electro  │ 149.99 │                  │
│└──────────────┴────────────┴──────────┴────────┘                  │
│                                                                    │
├────────────────────────────────────────────────────────────────────┤
│ Schema Browser              │ Commands                            │
│ ├─ products (5 cols)        │ F1  - Help                           │
│ ├─ sales (4 cols)           │ F2  - Load File                      │
│ ├─ customers (3 cols)       │ F3  - Export Results                 │
│ └─ daily_sales (2 cols)     │ F5  - Execute Query                  │
│                             │ F9  - Schema Browser                 │
│                             │ F10 - Options                        │
│                             │ ESC - Quit                           │
├────────────────────────────────────────────────────────────────────┤
│ Status: Ready | DB: analytics.db | Row: 1/5 | Col: 1/4           │
└────────────────────────────────────────────────────────────────────┘
```

### Navegación por Teclado

```
┌─────────────────────┬────────────────────────────────────┐
│ Tecla               │ Acción                             │
├─────────────────────┼────────────────────────────────────┤
│ F1                  │ Ayuda                              │
│ F2                  │ Cargar archivo SQL                 │
│ F3                  │ Exportar resultados                │
│ F5 / Ctrl+E         │ Ejecutar query                     │
│ F9                  │ Alternar schema browser            │
│ F10                 │ Opciones                           │
│ ESC                 │ Salir / Cancelar                   │
│ Tab                 │ Cambiar entre paneles              │
│ Flechas             │ Navegar (editor o resultados)      │
│ Page Up/Down        │ Scroll en resultados               │
│ Home/End            │ Primera/Última fila                │
│ Ctrl+A              │ Seleccionar todo (editor)          │
│ Ctrl+C              │ Copiar (resultados)                │
│ Ctrl+S              │ Guardar query                      │
│ Ctrl+O              │ Abrir query                        │
│ Ctrl+N              │ Nueva query                        │
└─────────────────────┴────────────────────────────────────┘
```

### Ejemplo de Sesión TUI

```bash
# Iniciar TUI
$ ./target/debug/noctra tui --database analytics.db
🖥️  Noctra TUI v0.1.0 - Modo Terminal Interactivo
📂 Base de datos: analytics.db

# El TUI se abre en modo completo...

# 1. Usuario presiona F2 para cargar archivo
# 2. Navega a setup.sql
# 3. El query se carga en el editor:

USE 'data/products.csv' AS products;
USE 'data/sales.json' AS sales;

SELECT
    p.name,
    SUM(s.quantity) as total_sold
FROM products p
JOIN sales s ON p.product_id = s.product_id
GROUP BY p.product_id, p.name
ORDER BY total_sold DESC;

# 4. Usuario presiona F5 para ejecutar
# 5. Los resultados aparecen en el panel de resultados
# 6. Usuario puede navegar con flechas
# 7. F3 para exportar a CSV/JSON
# 8. ESC para salir
```

### Características Retro 4GL

- **Colores VGA Clásicos**: Verde sobre negro, cian sobre azul
- **Bordes ASCII Art**: Estilo DOS/dBASE
- **Navegación por Teclado**: Sin mouse necesario
- **Hotkeys F1-F12**: Estilo Turbo Pascal/FoxPro
- **Status Line**: Información en tiempo real
- **Multi-Panel**: Editor, resultados, schema, comandos

---

## Modo Batch - Procesamiento por Lotes

Ejecutar scripts RQL no interactivos.

### Sintaxis

```bash
noctra batch <script.rql> [opciones]
```

### Ejemplo 1: Script de ETL Diario

**Archivo: `daily_etl.rql`**

```sql
-- Daily ETL Script
-- Cargar datos de múltiples fuentes

USE 'exports/sales_2024-11-14.csv' AS today_sales;
USE 'exports/inventory.json' AS inventory;
USE 'exports/customers.parquet' AS customers;

-- Crear reporte de ventas
CREATE TABLE daily_report AS
SELECT
    s.sale_id,
    s.product_id,
    s.quantity,
    s.sale_date,
    c.name as customer_name,
    c.region,
    i.stock_level
FROM today_sales s
JOIN customers c ON s.customer_id = c.customer_id
LEFT JOIN inventory i ON s.product_id = i.product_id
WHERE s.sale_date = CURRENT_DATE;

-- Exportar reporte
EXPORT daily_report TO 'reports/daily_sales_2024-11-14.csv' FORMAT CSV;

-- Estadísticas
SELECT
    'Total Sales' as metric,
    COUNT(*) as value
FROM daily_report
UNION ALL
SELECT
    'Total Revenue',
    SUM(quantity * price)
FROM daily_report;
```

### Ejecutar Script Batch

```bash
# Ejecución básica
./target/debug/noctra batch daily_etl.rql

📜 Ejecutando script: daily_etl.rql
✅ Fuente 'today_sales' cargada
✅ Fuente 'inventory' cargada
✅ Fuente 'customers' cargada
✅ Tabla 'daily_report' creada (245 filas)
✅ Exportadas 245 filas a 'reports/daily_sales_2024-11-14.csv'

metric         | value
---------------+--------
Total Sales    | 245
Total Revenue  | 45678.90

✅ Script completado exitosamente

# Con parámetros
./target/debug/noctra batch daily_etl.rql \
    --param date=2024-11-14 \
    --param region=North

# Con output a archivo
./target/debug/noctra batch daily_etl.rql \
    --output results.json \
    --format json

# Modo silencioso (solo errores)
./target/debug/noctra batch daily_etl.rql --quiet

# Continuar en caso de error
./target/debug/noctra batch daily_etl.rql --continue-on-error
```

### Ejemplo 2: Script de Validación

**Archivo: `validate_data.rql`**

```sql
-- Data Validation Script
USE 'imports/new_products.csv' AS new_products;

-- Validar datos obligatorios
SELECT 'Missing Product Names' as issue, COUNT(*) as count
FROM new_products
WHERE name IS NULL OR name = ''
UNION ALL
SELECT 'Invalid Prices', COUNT(*)
FROM new_products
WHERE price <= 0 OR price IS NULL
UNION ALL
SELECT 'Missing Categories', COUNT(*)
FROM new_products
WHERE category IS NULL OR category = '';

-- Si hay errores, el script puede salir con código de error
-- para integración con CI/CD
```

```bash
# Ejecutar validación
./target/debug/noctra batch validate_data.rql --quiet

# En CI/CD pipeline
if ! ./target/debug/noctra batch validate_data.rql; then
    echo "Data validation failed!"
    exit 1
fi
```

---

## Modo Form - Formularios Interactivos

Ejecutar formularios definidos en FDL2 (Form Definition Language 2).

### Ejemplo 1: Formulario de Captura de Cliente

**Archivo: `customer_form.toml`**

```toml
title = "Registro de Nuevo Cliente"
description = "Formulario para captura de datos de cliente"
schema = "crm"

[ui_config]
layout = "vertical"
width = 60
height = 20

[fields.customer_id]
label = "ID de Cliente"
field_type = "int"
required = true
default = "auto"

[fields.name]
label = "Nombre Completo"
field_type = "text"
required = true
min_length = 3
max_length = 100
placeholder = "Ej: Juan Pérez"

[fields.email]
label = "Correo Electrónico"
field_type = "email"
required = true
validation_pattern = "^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\\.[a-zA-Z]{2,}$"

[fields.phone]
label = "Teléfono"
field_type = "text"
required = false
validation_pattern = "^\\+?[0-9]{10,15}$"
placeholder = "+52 555 123 4567"

[fields.region]
label = "Región"
field_type = "select"
required = true
options = ["North", "South", "East", "West"]
default = "North"

[fields.customer_type]
label = "Tipo de Cliente"
field_type = "select"
required = true
options = ["Individual", "Business", "Government"]

[fields.credit_limit]
label = "Límite de Crédito"
field_type = "float"
required = false
default = "0.0"
min_value = 0.0
max_value = 100000.0

[fields.notes]
label = "Notas"
field_type = "text"
required = false
multiline = true
placeholder = "Información adicional del cliente..."

[actions.save]
action_type = "save_to_table"
table = "customers"
on_success = "Cliente guardado exitosamente"
on_error = "Error al guardar cliente"

[actions.validate]
action_type = "validate"
validation_query = "SELECT COUNT(*) FROM customers WHERE email = :email"
```

### Usar el Formulario

```bash
# 1. Cargar y validar
./target/debug/noctra form load customer_form.toml

📋 Cargando formulario: customer_form.toml

📝 Formulario: Registro de Nuevo Cliente
   Descripción: Formulario para captura de datos de cliente
   Schema: crm

🔢 Campos (8):
   - customer_id*: ID de Cliente (Int)
   - name*: Nombre Completo (Text)
   - email*: Correo Electrónico (Email)
   - phone: Teléfono (Text)
   - region*: Región (Select)
   - customer_type*: Tipo de Cliente (Select)
   - credit_limit: Límite de Crédito (Float)
   - notes: Notas (Text)

⚡ Acciones (2):
   - save: SaveToTable
   - validate: Validate

✅ Formulario cargado correctamente

# 2. Ejecutar interactivamente
./target/debug/noctra form exec customer_form.toml

🚀 Ejecutando formulario: customer_form.toml

🎯 Modo interactivo
   TAB/Shift+TAB: Navegar entre campos
   Escribir: Editar valor del campo
   Backspace: Borrar carácter
   Enter: Validar y continuar
   ESC: Cancelar

┌──────────────────────────────────────────────────────────────┐
│ 📋 Registro de Nuevo Cliente                                 │
├──────────────────────────────────────────────────────────────┤
│                                                              │
│ ID de Cliente *: [auto________________]                     │
│                                                              │
│ Nombre Completo *: [Juan Pérez________]  ✓                  │
│                                                              │
│ Correo Electrónico *: [juan@example.com]  ✓                 │
│                                                              │
│ Teléfono: [+52 555 123 4567_____]  ✓                        │
│                                                              │
│ Región *: [North ▼]                                          │
│   ├ North                                                    │
│   ├ South                                                    │
│   ├ East                                                     │
│   └ West                                                     │
│                                                              │
│ Tipo de Cliente *: [Individual ▼]                           │
│                                                              │
│ Límite de Crédito: [5000.00_______]                         │
│                                                              │
│ Notas: [Cliente corporativo VIP_____]                       │
│        [Requiere atención especial__]                       │
│                                                              │
├──────────────────────────────────────────────────────────────┤
│ [Guardar]  [Cancelar]                       * Campo requerido│
└──────────────────────────────────────────────────────────────┘

# Después de llenar y guardar...

✅ Formulario completado

📊 Valores:
   customer_id: 1523
   name: Juan Pérez
   email: juan@example.com
   phone: +52 555 123 4567
   region: North
   customer_type: Individual
   credit_limit: 5000.00
   notes: Cliente corporativo VIP\nRequiere atención especial

💾 Guardado en tabla: customers
✅ Cliente guardado exitosamente

# 3. Modo no interactivo (batch)
./target/debug/noctra form exec customer_form.toml \
    --param name="María García" \
    --param email="maria@example.com" \
    --param region="South" \
    --param customer_type="Business" \
    --non-interactive \
    --output customer_1524.json

✅ Formulario validado correctamente

📊 Valores:
   name: María García
   email: maria@example.com
   region: South
   customer_type: Business

💾 Guardado en: customer_1524.json

# 4. Preview del formulario
./target/debug/noctra form preview customer_form.toml --with-examples

👁️  Preview de formulario: customer_form.toml

┌──────────────────────────────────────────────────────────────┐
│ 📋 Registro de Nuevo Cliente                                 │
├──────────────────────────────────────────────────────────────┤
│ ID de Cliente: [42]                                          │
│ Nombre Completo: [Example Text]                             │
│ Correo Electrónico: [user@example.com]                      │
│ Teléfono: [Example Text]                                    │
│ Región: [North]                                             │
│ Tipo de Cliente: [Individual]                               │
│ Límite de Crédito: [0.00]                                   │
│ Notas: []                                                    │
└──────────────────────────────────────────────────────────────┘

✨ Este es un preview del formulario.
   Usa 'noctra form exec customer_form.toml' para ejecutarlo.
```

---

## Modo Query - Queries Directos

Ejecutar queries SQL directamente desde la línea de comandos.

### Sintaxis

```bash
noctra query "<SQL>" [opciones]
```

### Ejemplo 1: Query Simple

```bash
# Query básico
./target/debug/noctra query "SELECT * FROM products WHERE price > 100"

# Con formato específico
./target/debug/noctra query "SELECT * FROM products" --format json

# Con output a archivo
./target/debug/noctra query "SELECT * FROM customers" \
    --output customers.csv \
    --format csv

# Dry run (solo mostrar SQL)
./target/debug/noctra query "SELECT * FROM sales WHERE date = CURRENT_DATE" \
    --dry-run

📝 SQL generado:
SELECT * FROM sales WHERE date = CURRENT_DATE
```

### Ejemplo 2: Query con Parámetros

```bash
# Parámetros posicionales
./target/debug/noctra query \
    "SELECT * FROM customers WHERE region = :region AND active = :active" \
    --param region=North \
    --param active=true

# Output JSON
./target/debug/noctra query \
    "SELECT name, email, total_orders FROM customer_summary" \
    --format json \
    --output report.json

# Formato markdown para documentación
./target/debug/noctra query \
    "SELECT * FROM products LIMIT 5" \
    --format markdown > products.md
```

### Ejemplo 3: Queries Complejos

```bash
# Query con CTE
./target/debug/noctra query "
WITH top_customers AS (
    SELECT customer_id, SUM(amount) as total
    FROM orders
    GROUP BY customer_id
    ORDER BY total DESC
    LIMIT 10
)
SELECT c.name, c.email, tc.total
FROM customers c
JOIN top_customers tc ON c.customer_id = tc.customer_id
" --format table

# Query multi-línea desde archivo
cat > complex_query.sql <<'EOF'
SELECT
    p.category,
    COUNT(*) as product_count,
    AVG(p.price) as avg_price,
    SUM(s.quantity) as total_sold
FROM products p
LEFT JOIN sales s ON p.product_id = s.product_id
GROUP BY p.category
ORDER BY total_sold DESC NULLS LAST;
EOF

./target/debug/noctra query "$(cat complex_query.sql)" --format table
```

---

## 📦 Datos de Prueba

### Crear Datos de Ejemplo

```bash
# Crear directorio
mkdir -p data

# Products CSV
cat > data/products.csv <<EOF
product_id,name,category,price
1,Laptop,Electronics,1299.99
2,Mouse,Electronics,29.99
3,Desk,Furniture,499.99
4,Chair,Furniture,399.99
5,Headphones,Electronics,89.99
6,Monitor 27",Electronics,349.99
7,Lamp,Furniture,79.99
8,USB Hub,Electronics,59.99
9,Keyboard,Electronics,149.99
10,Webcam,Electronics,129.99
EOF

# Sales JSON
cat > data/sales.json <<EOF
[
  {"sale_id": 1, "product_id": 1, "customer_id": 1, "quantity": 2, "sale_date": "2024-01-15"},
  {"sale_id": 2, "product_id": 2, "customer_id": 2, "quantity": 5, "sale_date": "2024-01-16"},
  {"sale_id": 3, "product_id": 3, "customer_id": 3, "quantity": 1, "sale_date": "2024-01-17"},
  {"sale_id": 4, "product_id": 1, "customer_id": 1, "quantity": 1, "sale_date": "2024-01-18"},
  {"sale_id": 5, "product_id": 5, "customer_id": 2, "quantity": 3, "sale_date": "2024-01-19"},
  {"sale_id": 6, "product_id": 6, "customer_id": 3, "quantity": 2, "sale_date": "2024-01-20"}
]
EOF

# Customers Parquet (usar Python o DuckDB para crear)
# Si tienes Python con pandas y pyarrow:
python3 << 'PYTHON'
import pandas as pd
customers = pd.DataFrame({
    'customer_id': [1, 2, 3],
    'name': ['Alice Brown', 'Bob Smith', 'Carol White'],
    'email': ['alice@example.com', 'bob@example.com', 'carol@example.com'],
    'region': ['North', 'South', 'West']
})
customers.to_parquet('data/customers.parquet', index=False)
print("✅ customers.parquet created")
PYTHON
```

### O Crear con DuckDB CLI

```bash
# Si no tienes Python, usa DuckDB directamente
duckdb << 'SQL'
COPY (
    SELECT 1 as customer_id, 'Alice Brown' as name, 'alice@example.com' as email, 'North' as region
    UNION ALL
    SELECT 2, 'Bob Smith', 'bob@example.com', 'South'
    UNION ALL
    SELECT 3, 'Carol White', 'carol@example.com', 'West'
) TO 'data/customers.parquet' (FORMAT PARQUET);
SQL
```

---

## 🎯 Combinando Modos

### Flujo de Trabajo Completo

```bash
# 1. Desarrollo inicial en REPL
./target/debug/noctra repl
noctra> use 'data/products.csv' as products;
noctra> select * from products limit 5;
noctra> # Iterar hasta que el query esté correcto
noctra> quit

# 2. Guardar query a archivo
cat > analysis.sql <<'EOF'
USE 'data/products.csv' AS products;
USE 'data/sales.json' AS sales;

SELECT
    p.category,
    SUM(s.quantity * p.price) as revenue
FROM products p
JOIN sales s ON p.product_id = s.product_id
GROUP BY p.category;
EOF

# 3. Ejecutar en batch para automatización
./target/debug/noctra batch analysis.sql --output report.csv

# 4. Visualizar resultados en TUI
./target/debug/noctra tui --load analysis.sql

# 5. Ejecutar query directo cuando se necesite
./target/debug/noctra query "SELECT * FROM products" --format json > products.json
```

---

## 🔧 Tips y Trucos

### REPL

- Usa `Ctrl+R` para buscar en historial
- Queries multi-línea terminan con `;`
- Variables de sesión persisten en la sesión
- `clear` para limpiar pantalla sin perder sesión

### TUI

- `F1` siempre muestra ayuda contextual
- Schema browser autocompleta nombres de tablas
- Results panel soporta búsqueda con `/`
- Hotkeys son configurables en `config.toml`

### Batch

- Scripts pueden incluir comentarios con `--`
- Parámetros se acceden con `:param_name`
- `--continue-on-error` útil para ETL
- Exit code != 0 si hay errores (útil para CI/CD)

### Forms

- Validación ocurre en tiempo real
- Campos requeridos marcados con `*`
- Preview mode útil para documentación
- JSON output compatible con APIs

---

**Versión**: v0.6.0-alpha2
**Milestone**: M6 Phase 2
**Fecha**: 14 de noviembre de 2025
