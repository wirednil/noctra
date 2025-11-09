# Noctra - Entorno SQL Interactivo en Rust

> **Entorno SQL interactivo y framework TUI para formularios** moderno: SQL real, formulación declarativa de formularios, runtime ligero y ejecución batch.

## 📚 Documentación

- **[Project Status](docs/PROJECT_STATUS.md)** - Estado actual del proyecto y progreso de milestones
- **[Getting Started](docs/GETTING_STARTED.md)** - Guía de inicio rápido y tutorial
- **[Design Document](docs/DESIGN.md)** - Arquitectura técnica completa
- **[Roadmap](docs/ROADMAP.md)** - Timeline de desarrollo y milestones
- **[RQL Extensions](docs/RQL-EXTENSIONS.md)** - Referencia del lenguaje SQL extendido
- **[FDL2 Specification](docs/FDL2-SPEC.md)** - Especificación de formularios
- **[API Reference](docs/API-REFERENCE.md)** - API de programación
- **[Contributing](docs/CONTRIBUTING.md)** - Guía para contribuidores

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

## 📋 Estado del Proyecto

**Versión Actual:** 0.1.0
**Progreso:** M1 ✅ | M2 ✅ | M3 ✅ | M4 📋 | M5 📋

| Milestone | Estado | Progreso |
|-----------|--------|----------|
| **M0: Foundation** | ✅ Completado | 100% |
| **M1: Core + Parser** | ✅ Completado | 100% |
| **M2: Forms + TUI** | ✅ Completado | 100% |
| **M3: Backend Integration** | ✅ Completado | 100% |
| **M4: Advanced Features** | 📋 Planificado | 0% |
| **M5: Production Ready** | 📋 Planificado | 0% |

### ✅ Funcionalidad Actual

- **Core Runtime**: Executor SQL con SQLite (in-memory y file-based)
- **RQL Parser**: SQL extendido con parámetros y templates
- **TUI Completo**: Interfaz Ratatui con 3 modos (Command/Result/Dialog)
- **Formularios FDL2**: Sistema declarativo de formularios en TOML
- **Backend Integration**: TUI ejecuta SQL real (no simulado)
- **CLI**: REPL interactivo y comandos batch

### 🎯 Próximos Pasos

Ver [docs/PROJECT_STATUS.md](docs/PROJECT_STATUS.md) para detalles completos del roadmap y próximos milestones.

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


## 🔧 Development

### Estructura del Proyecto

```
noctra/
├── Cargo.toml                 # Workspace root
├── README.md                  # Este archivo
├── .github/
│   └── workflows/
│       └── ci.yml             # CI pipeline
├── crates/
│   ├── core/                  # ✅ Runtime principal (Executor, Session, Backend)
│   ├── parser/                # ✅ Parser RQL/SQL
│   ├── cli/                   # ✅ CLI/REPL + TUI launcher
│   ├── tui/                   # ✅ TUI + NWM con backend integration
│   ├── srv/                   # 📋 Daemon (Milestone 5)
│   ├── formlib/               # ✅ Formularios FDL2
│   └── ffi/                   # ✅ C bindings
├── docs/                      # 📚 Documentación completa
│   ├── PROJECT_STATUS.md      # Estado actual y progreso
│   ├── DESIGN.md              # Arquitectura técnica
│   ├── ROADMAP.md             # Timeline de desarrollo
│   ├── GETTING_STARTED.md     # Guía de inicio
│   ├── RQL-EXTENSIONS.md      # Referencia RQL
│   ├── FDL2-SPEC.md           # Especificación de formularios
│   ├── FORMS.md               # Documentación de formularios
│   ├── API-REFERENCE.md       # API reference
│   ├── CONTRIBUTING.md        # Guía para contribuir
│   └── archive/               # Documentos históricos
├── examples/                  # Ejemplos de uso
│   ├── forms/                 # Formularios TOML
│   └── scripts/               # Scripts RQL
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