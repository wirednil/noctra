# DuckDB Setup Guide

Este documento explica cómo configurar DuckDB precompilado para desarrollar Noctra sin recompilar DuckDB en cada build.

## 🎯 Objetivo

Usar la biblioteca DuckDB precompilada (`libduckdb.so`) en lugar de la feature `bundled` de Rust, reduciendo el tiempo de compilación de ~60s a ~20s.

## 📦 Instalación de DuckDB Precompilado

### 1. Descargar DuckDB v1.1.0

```bash
cd /tmp
wget https://github.com/duckdb/duckdb/releases/download/v1.1.0/libduckdb-linux-amd64.zip
unzip libduckdb-linux-amd64.zip
```

Esto descarga:
- `libduckdb.so` - Biblioteca compartida (55MB)
- `duckdb.h` - Header C (149KB)
- `duckdb.hpp` - Header C++ (1.3MB)

### 2. Instalar en /opt/duckdb

```bash
sudo mkdir -p /opt/duckdb
sudo cp libduckdb.so duckdb.h duckdb.hpp /opt/duckdb/
sudo chmod 644 /opt/duckdb/*
```

**Verificación:**
```bash
ls -lh /opt/duckdb/
# Debe mostrar:
# -rw-r--r-- duckdb.h      (149K)
# -rw-r--r-- duckdb.hpp    (1.3M)
# -rwxr-xr-x libduckdb.so  (55M)
```

### 3. Configurar Variables de Entorno

#### Opción A: Source manual (temporal)

```bash
source duckdb.env
```

Esto configura las variables para la sesión actual.

#### Opción B: Agregar a ~/.bashrc (permanente)

```bash
cat >> ~/.bashrc << 'EOF'

# DuckDB precompilado (Rust)
export DUCKDB_LIB_DIR=/opt/duckdb
export DUCKDB_INCLUDE_DIR=/opt/duckdb
export LD_LIBRARY_PATH=/opt/duckdb:$LD_LIBRARY_PATH
EOF

source ~/.bashrc
```

#### Opción C: Usar direnv (recomendado para proyectos)

Si usas [direnv](https://direnv.net/):

```bash
# Instalar direnv (si no lo tienes)
sudo apt install direnv  # Debian/Ubuntu
brew install direnv      # macOS

# El proyecto ya tiene .envrc configurado
direnv allow
```

## 🔧 Configuración de Cargo.toml

El `crates/noctra-duckdb/Cargo.toml` ya está configurado para usar DuckDB dinámico:

```toml
[dependencies]
duckdb = { version = "1.1", default-features = false }
```

**NO usar:**
```toml
# ❌ Esto recompila DuckDB desde fuente (lento)
duckdb = { version = "1.1", features = ["bundled"] }
```

## 🏗️ Compilar y Testear

### Compilar noctra-duckdb

```bash
# Asegúrate de que las variables estén configuradas
source duckdb.env

# Compilar
cargo build -p noctra-duckdb

# Debería tomar ~20s en lugar de ~60s
```

### Ejecutar Tests

```bash
source duckdb.env
cargo test -p noctra-duckdb

# Todos los tests deben pasar:
# ✓ 8 unit tests
# ✓ 1 doctest
```

## 📊 Comparación de Performance

| Método | Primera Build | Rebuild | Ventajas |
|--------|---------------|---------|----------|
| **Bundled** | ~60s | ~60s | ✓ Sin configuración externa |
| **Precompiled** | ~20s | ~8s | ✓ 3x más rápido<br>✓ No recompila DuckDB |

## 🔍 Troubleshooting

### Error: "unable to find library -lduckdb"

**Causa:** Variables de entorno no configuradas.

**Solución:**
```bash
source duckdb.env
cargo clean -p noctra-duckdb
cargo build -p noctra-duckdb
```

### Error: "cannot open shared object file"

**Causa:** `LD_LIBRARY_PATH` no incluye `/opt/duckdb`.

**Solución:**
```bash
export LD_LIBRARY_PATH=/opt/duckdb:$LD_LIBRARY_PATH
ldd target/debug/deps/libnoctra_duckdb-*.so | grep duckdb
# Debe mostrar: libduckdb.so => /opt/duckdb/libduckdb.so
```

### Warning: "Mutex poisoned"

**Causa:** Test concurrente accediendo al mismo recurso.

**Solución:** Ejecutar tests con `--test-threads=1`:
```bash
cargo test -p noctra-duckdb -- --test-threads=1
```

## 📚 Referencias

- [DuckDB Releases](https://github.com/duckdb/duckdb/releases)
- [duckdb-rs Documentation](https://docs.rs/duckdb/latest/duckdb/)
- [Noctra M6 Implementation Plan](M6_IMPLEMENTATION_PLAN_v2.md)

## 🎯 Siguiente Paso

Una vez configurado DuckDB, puedes continuar con el desarrollo de Noctra:

```bash
# Compilar todo el workspace
cargo build --workspace

# Ejecutar Noctra TUI con DuckDB backend
cargo run -- tui
```

---

**Última actualización:** 2025-11-13
**Versión DuckDB:** 1.1.0
**Milestone:** M6 Phase 1
