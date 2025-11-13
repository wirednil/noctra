//! DuckDB Configuration Management
//!
//! Provides production-ready configuration for DuckDB connections.
//! Enables fine-tuning of memory limits, thread counts, and other performance parameters.

use serde::{Deserialize, Serialize};

/// Configuration for DuckDB connection and performance tuning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DuckDBConfig {
    /// Memory limit (e.g., "16GB", "512MB")
    ///
    /// Default: None (uses DuckDB default of ~80% system RAM)
    /// Note: Percentage values like "80%" are not supported in DuckDB 1.1
    pub memory_limit: Option<String>,

    /// Number of threads for query execution
    ///
    /// Default: CPU core count
    /// For remote I/O (S3, HTTP): 2-5x cores recommended to mask network latency
    pub threads: Option<usize>,

    /// Maximum schemas to search during catalog errors
    ///
    /// Lower values = faster error messages in systems with many attached databases
    /// Default: 10
    pub catalog_error_max_schemas: Option<usize>,

    /// Enable query profiling via EXPLAIN ANALYZE
    ///
    /// When true, queries can be profiled for performance debugging
    /// Default: false
    pub enable_profiling: bool,
}

impl Default for DuckDBConfig {
    fn default() -> Self {
        Self {
            // None = use DuckDB's default (80% of available RAM)
            memory_limit: None,
            threads: Some(num_cpus::get()),
            catalog_error_max_schemas: Some(10),
            enable_profiling: false,
        }
    }
}

impl DuckDBConfig {
    /// Create config optimized for local file I/O
    ///
    /// Uses CPU core count for threads (optimal for local disk access)
    pub fn local() -> Self {
        Self {
            threads: Some(num_cpus::get()),
            ..Default::default()
        }
    }

    /// Create config optimized for remote I/O (S3, HTTP)
    ///
    /// Uses 3x CPU cores for threads to mask network latency
    /// DuckDB uses synchronous I/O for remote files, so more threads = more parallel requests
    pub fn remote() -> Self {
        Self {
            threads: Some(num_cpus::get() * 3),
            ..Default::default()
        }
    }

    /// Create minimal config (low memory, single thread)
    ///
    /// Useful for embedded scenarios or testing
    pub fn minimal() -> Self {
        Self {
            memory_limit: Some("512MB".to_string()),
            threads: Some(1),
            catalog_error_max_schemas: Some(5),
            enable_profiling: false,
        }
    }

    /// Generate SQL SET statements from config
    ///
    /// Returns a vec of SQL commands to be executed after connection init
    pub fn to_sql_commands(&self) -> Vec<String> {
        let mut commands = Vec::new();

        if let Some(ref mem) = self.memory_limit {
            commands.push(format!("SET memory_limit = '{}'", mem));
        }

        if let Some(threads) = self.threads {
            commands.push(format!("SET threads = {}", threads));
        }

        if let Some(max_schemas) = self.catalog_error_max_schemas {
            commands.push(format!("SET catalog_error_max_schemas = {}", max_schemas));
        }

        commands
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = DuckDBConfig::default();
        // Default uses None for memory_limit (DuckDB handles it automatically)
        assert!(config.memory_limit.is_none());
        assert!(config.threads.is_some());
        assert_eq!(config.enable_profiling, false);
    }

    #[test]
    fn test_local_config() {
        let config = DuckDBConfig::local();
        assert_eq!(config.threads, Some(num_cpus::get()));
    }

    #[test]
    fn test_remote_config() {
        let config = DuckDBConfig::remote();
        assert_eq!(config.threads, Some(num_cpus::get() * 3));
    }

    #[test]
    fn test_minimal_config() {
        let config = DuckDBConfig::minimal();
        assert_eq!(config.memory_limit, Some("512MB".to_string()));
        assert_eq!(config.threads, Some(1));
    }

    #[test]
    fn test_to_sql_commands() {
        let config = DuckDBConfig {
            memory_limit: Some("8GB".to_string()),
            threads: Some(4),
            catalog_error_max_schemas: Some(5),
            enable_profiling: false,
        };

        let commands = config.to_sql_commands();
        assert_eq!(commands.len(), 3);
        assert!(commands[0].contains("memory_limit"));
        assert!(commands[1].contains("threads"));
        assert!(commands[2].contains("catalog_error_max_schemas"));
    }
}
