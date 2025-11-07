# Noctra - Entorno SQL Interactivo en Rust

> **Entorno SQL interactivo y framework TUI para formularios** moderno: SQL real, formulación declarativa de formularios, runtime ligero y ejecución batch.

## 🎯 Descripción

Noctra es un **entorno de consulta interactivo** moderno implementado en **Rust** que ofrece una experiencia de consulta SQL fluida con filosofía 4GL, agregando las ventajas de la tecnología contemporánea:

- **Seguridad**: Memory safety, no segfaults como en C original
- **Performance**: Compile-time optimizations y async/await nativo
- **Portabilidad**: Single binary deployment sin dependencias externas
- **Herencia visual**: Noctra Window Manager (NWM) basado en ncurses

## 🏗️ Arquitectura

### Crates del Workspace

- **`noctra-core`** - Runtime principal, executor, tipos base
- **`noctra-parser`** - Parser RQL con extensiones sqlparser
- **`noctra-cli`** - CLI interactivo y REPL
- **`noctra-tui`** - TUI components y Window Manager (NWM)
- **`noctra-srv`** - Daemon server (noctrad)
- **`noctra-formlib`** - Formularios FDL2 en TOML
- **`noctra-ffi`** - Bindings C para integraciones externas

### Características Principales

#### 🎨 Noctra Window Manager (NWM)
Interfaz TUI moderna inspirada en interfaces clásicas:
- Tres modos operativos: Comando, Resultado, Diálogo
- Layout persistente con header/footer siempre visibles
- Navegación por teclas función: F5 (procesar), F1 (ayuda), End (salir)
- Paleta retro: Fondo negro, texto verde fósforo, resaltado cyan/amarillo

#### 🗣️ RQL (SQL Extendido)
- Parámetros posicionados: `$1`, `$2`, ...
- Parámetros nombrados: `:name`
- Comandos extendidos: `USE`, `LET`, `FORM LOAD`
- Template processing automático

#### 📋 FDL2 (Form Definition Language)
- Formularios declarativos en TOML
- Validaciones automáticas
- Template SQL condicional
- Actions predefinidas: query, insert, update, delete

## 🚀 Inicio Rápido

### Prerrequisitos
- Rust 1.70+
- Terminal compatible con ncurses
- SQLite (para MVP)

### Instalación

```bash
# Clonar repository
git clone https://github.com/noctra/noctra.git
cd noctra

# Build workspace
cargo build --workspace

# Ejecutar CLI interactivo
cargo run --bin noctra
```

### Ejemplo de Uso

```bash
$ noctra
Noctra 0.1.0 - Interactive Query Language
noctra> use demo;
Schema changed to: demo

noctra> select * from employees where dept = :dept;
:param dept => SALES
+-----------+------------------+
| nroleg    | nombre           |
+-----------+------------------+
| 1001      | Juan Pérez       |
| 1002      | María González   |
+-----------+------------------+
(2 rows)

noctra> form empleados.toml
[Form: Consulta de Empleados]
Nro. Legajo: [          ]
Nombre:     [                          ]
Departamento: [SALES▼]
[Consultar] [Cancelar]
```

## 📋 Roadmap MVP

### Milestone 0 ✅ (Completado)
- Workspace Cargo configurado
- Todos los crates creados
- CI básico configurado

### Milestone 1 🔄 (En progreso - 83% completado)

**Estado de Compilación:**
- ✅ **noctra-core** - Runtime, executor, tipos (0 errores)
- ✅ **noctra-parser** - Parser RQL/SQL con templates (0 errores)
- ✅ **noctra-tui** - Terminal UI, layout, widgets (0 errores)
- ✅ **noctra-formlib** - Parser FDL2 formularios (0 errores)
- ✅ **noctra-ffi** - Bindings C (0 errores)
- ⚠️ **noctra-cli** - REPL interactivo (39 errores pendientes)
- 🚫 **noctra-srv** - Temporalmente deshabilitado (Milestone 4)

**Progreso:**
- `core::Executor` funcional ✅
- `SqliteBackend` con rusqlite ✅
- Parser RQL completo ✅
- CLI REPL básico con rustyline 🔄 (en corrección)
- Ejecución simple de SELECT ⏳ (pendiente de CLI)

### Milestones Siguientes
- **Milestone 2**: Form loader & TUI renderer
- **Milestone 3**: Parser RQL + batch mode
- **Milestone 4**: Daemon noctrad (opcional)
- **Milestone 5**: Testing y documentación

## 🧪 Testing

```bash
# Ejecutar todos los tests
cargo test --workspace

# Tests unitarios
cargo test --lib

# Tests de integración
cargo test --test integration

# Coverage
cargo install cargo-tarpaulin
cargo tarpaulin --workspace --out html
```

## 📚 Documentación

- **[DESIGN.md](../DESIGN.md)** - Especificación técnica completa
- **[FDL2-SPEC.md](docs/FDL2-SPEC.md)** - Especificación de formularios
- **[RQL-EXTENSIONS.md](docs/RQL-EXTENSIONS.md)** - Extensiones SQL
- **[API-REFERENCE.md](docs/API-REFERENCE.md)** - API reference

## 🔧 Development

### Estructura del Proyecto

```
noctra/
├── Cargo.toml                 # Workspace root
├── README.md                  # Este archivo
├── DESIGN.md                  # Especificación completa
├── .github/
│   └── workflows/
│       └── ci.yml             # CI pipeline
├── crates/
│   ├── core/                  # Runtime principal
│   ├── parser/                # Parser RQL
│   ├── cli/                   # CLI/REPL
│   ├── tui/                   # TUI + NWM
│   ├── srv/                   # Daemon
│   ├── formlib/               # Formularios
│   └── ffi/                   # C bindings
├── docs/                      # Documentación
├── examples/                  # Ejemplos
└── tests/                     # Test suite
```

### Contribución

1. Fork del repository
2. Crear feature branch: `git checkout -b feature/nueva-feature`
3. Commit cambios: `git commit -am 'Agregar nueva feature'`
4. Push al branch: `git push origin feature/nueva-feature`
5. Crear Pull Request

### Convenciones de Código

- **Format**: `cargo fmt --all`
- **Lint**: `cargo clippy --all-targets --all-features -- -D warnings`
- **Commits**: Conventional Commits (`feat:`, `fix:`, `docs:`, etc.)

## 🔧 Integraciones y Extensiones

Noctra proporciona herramientas para integración con sistemas existentes:

```bash
# Cargar scripts externos
$ noctra --batch scripts/consultas.rql
Processing: consultas.rql
✓ Executed successfully

# Usar formularios personalizados
$ noctra form examples/empleados.toml
[Form: Consulta de Empleados]
Nro. Legajo: [          ]
```

## 📄 Licencia

Dual licensed under either:

- MIT License ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)
- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)

## 🙏 Créditos

- **Inspiración**: Filosofía 4GL clásica y experiencia de usuario intuitiva
- **Implementación**: Noctra Project Team
- **Tecnologías**: Rust, sqlparser, ratatui, tokio
- **Filosofía**: Simplicidad y productividad en consultas SQL interactivas

## 📞 Contacto

- **GitHub**: https://github.com/noctra/noctra
- **Issues**: https://github.com/noctra/noctra/issues
- **Discussions**: https://github.com/noctra/noctra/discussions

---

**Noctra 0.1.0** - Entorno SQL interactivo moderno para la era Rust 🚀