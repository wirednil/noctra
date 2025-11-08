# Getting Started with Noctra

Noctra es un entorno SQL interactivo moderno escrito en Rust. Este documento te ayudará a comenzar a usar Noctra.

## Instalación

### Compilar desde el código fuente

```bash
# Clonar el repositorio
git clone https://github.com/wirednil/noctra.git
cd noctra

# Compilar en modo release
cargo build --release

# El binario estará en target/release/noctra
./target/release/noctra --version
```

## Uso Básico

### Modo REPL Interactivo

El modo REPL (Read-Eval-Print Loop) es la forma principal de interactuar con Noctra:

```bash
# Iniciar el REPL con una base de datos en memoria
./target/release/noctra

# O especificar una base de datos SQLite
./target/release/noctra --database mi_base.db
```

### Ejemplos de Queries

Una vez en el REPL, puedes ejecutar queries SQL estándar:

```sql
-- Crear una tabla
CREATE TABLE empleados (
    id INTEGER PRIMARY KEY,
    nombre TEXT NOT NULL,
    departamento TEXT,
    salario INTEGER
);

-- Insertar datos
INSERT INTO empleados (id, nombre, departamento, salario)
VALUES
    (1, 'Ana García', 'IT', 75000),
    (2, 'Carlos López', 'Ventas', 65000),
    (3, 'María Rodríguez', 'IT', 80000);

-- Consultar datos
SELECT * FROM empleados WHERE departamento = 'IT';

-- Consultas con agregación
SELECT departamento, COUNT(*) as total, AVG(salario) as promedio
FROM empleados
GROUP BY departamento;

-- Actualizar registros
UPDATE empleados SET salario = 85000 WHERE id = 3;

-- Eliminar registros
DELETE FROM empleados WHERE id = 2;
```

### Comandos Especiales del REPL

El REPL de Noctra incluye comandos especiales para facilitar el trabajo:

```
help          - Mostrar ayuda de comandos
quit          - Salir del REPL (también: exit, q)
clear         - Limpiar la pantalla (también: cls)
:version      - Mostrar versión de Noctra
:config       - Mostrar configuración actual
:status       - Mostrar estado del REPL
:set KEY=VAL  - Configurar una variable
```

### Ejemplo de Sesión Completa

```bash
$ ./target/release/noctra
🐍 Noctra v0.1.0 - Entorno SQL Interactivo
🎯 Noctra REPL iniciado - Escribe 'help' para ayuda
noctra> CREATE TABLE productos (id INTEGER, nombre TEXT, precio REAL);
✅ Query ejecutado

noctra> INSERT INTO productos VALUES (1, 'Laptop', 999.99), (2, 'Mouse', 29.99);
✅ 2 filas afectadas

noctra> SELECT * FROM productos;
┌────┬────────┬────────┐
│ id │ nombre │ precio │
├────┼────────┼────────┤
│ 1  │ Laptop │ 999.99 │
│ 2  │ Mouse  │ 29.99  │
└────┴────────┴────────┘

(2 filas)

noctra> SELECT nombre, precio * 1.16 AS precio_con_iva FROM productos;
┌────────┬────────────────┐
│ nombre │ precio_con_iva │
├────────┼────────────────┤
│ Laptop │ 1159.9884      │
│ Mouse  │ 34.7884        │
└────────┴────────────────┘

(2 filas)

noctra> quit
👋 ¡Hasta luego!
```

## Opciones de Línea de Comandos

```bash
# Ver todas las opciones disponibles
./target/release/noctra --help

# Especificar base de datos SQLite
./target/release/noctra --database mi_base.db

# Modo debug (más información de errores)
./target/release/noctra --debug

# Archivo de configuración personalizado
./target/release/noctra --config mi_config.toml
```

## Estructura del Proyecto

Noctra está organizado en múltiples crates especializados:

- **noctra-core**: Runtime, executor y tipos fundamentales
- **noctra-parser**: Parser RQL/SQL
- **noctra-cli**: Interfaz de línea de comandos y REPL
- **noctra-formlib**: Sistema de formularios FDL2
- **noctra-tui**: Interfaz de usuario terminal
- **noctra-ffi**: Interfaz C para integración con otros lenguajes

## Siguientes Pasos

1. **Experimentar con Queries**: Prueba diferentes tipos de queries SQL en el REPL
2. **Explorar los Tests**: Ve `crates/cli/tests/integration_test.rs` para ejemplos de uso programático
3. **Revisar la Documentación**: Ejecuta `cargo doc --open` para ver la documentación completa
4. **Ver STATUS.md**: Revisa el estado actual del proyecto y roadmap

## Troubleshooting

### Error: "Database locked"
Si ves este error, asegúrate de que no hay otra instancia de Noctra accediendo a la misma base de datos.

### El REPL no muestra colores
Verifica que tu terminal soporte colores ANSI. Puedes usar la variable de entorno:
```bash
export NOCTRA_COLOR_MODE=always
```

### Errores de compilación
Asegúrate de tener la versión correcta de Rust:
```bash
rustc --version  # Debe ser 1.70 o superior
```

## Contribuir

¿Encontraste un bug o tienes una sugerencia? Abre un issue en:
https://github.com/wirednil/noctra/issues

## Licencia

Noctra está licenciado bajo MIT License.
