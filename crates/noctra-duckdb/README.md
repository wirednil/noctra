# Noctra DuckDB Backend

DuckDB-powered data source implementation for Noctra, providing native file-native queries for CSV, JSON, and Parquet files.

## Features

- **File-Native Queries**: Query CSV, JSON, and Parquet files directly without importing
- **Auto-Detection**: Automatic format detection and type inference
- **Full SQL Support**: Leverage DuckDB's complete SQL implementation
- **API Compatibility**: Drop-in replacement for existing Noctra data sources
- **Performance**: Optimized for analytical workloads

## Usage

```rust
use noctra_duckdb::DuckDBSource;

// Create in-memory DuckDB source
let mut source = DuckDBSource::new_in_memory()?;

// Register files as virtual tables
source.register_file("sales.csv", "sales")?;
source.register_file("customers.json", "customers")?;

// Query across multiple sources
let result = source.query(
    "SELECT c.name, SUM(s.amount) as total
     FROM customers c
     JOIN sales s ON c.id = s.customer_id
     GROUP BY c.name
     ORDER BY total DESC",
    &noctra_core::types::Parameters::new()
)?;
```

## Supported File Formats

- **CSV**: Comma-separated values with auto-detection of delimiters and headers
- **JSON**: Newline-delimited JSON or JSON arrays
- **Parquet**: Columnar format with full schema preservation

## Architecture

The `DuckDBSource` implements Noctra's `DataSource` trait, providing:

- `query()`: Execute SQL queries with parameter binding
- `schema()`: Introspect table schemas and metadata
- `source_type()`: Identify the data source type
- `name()`: Get the source identifier

## Migration from CSV Backend

This crate replaces the legacy `csv_backend.rs` with a more powerful DuckDB-based implementation:

```rust
// Old way (deprecated)
let csv_source = CsvDataSource::new("data.csv", "mydata", CsvOptions::default())?;

// New way (recommended)
let mut duckdb_source = DuckDBSource::new_in_memory()?;
duckdb_source.register_file("data.csv", "mydata")?;
```

## Performance Characteristics

- **CSV Loading**: <500ms for typical files (<10MB)
- **Query Execution**: Sub-millisecond for simple queries
- **Memory Usage**: ~2-3x file size for in-memory operations
- **Concurrent Access**: Thread-safe for read operations

## Dependencies

- `duckdb`: Core database engine with bundled extensions
- `noctra-core`: Noctra's core types and traits
- `anyhow`, `thiserror`: Error handling
- `log`: Structured logging

## Testing

Run the test suite:

```bash
cargo test --package noctra-duckdb
```

The test suite includes:
- File registration and format detection
- Query execution with various SQL constructs
- Schema introspection
- Error handling for unsupported formats
- Type conversion validation

## License

MIT OR Apache-2.0