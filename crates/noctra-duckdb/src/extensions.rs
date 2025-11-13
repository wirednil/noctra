//! DuckDB extensions support
//!
//! This module handles loading and managing DuckDB extensions
//! for additional file format support.

use crate::error::Result;
use duckdb::Connection;

/// DuckDB extensions manager
pub struct ExtensionsManager {
    conn: Connection,
    loaded_extensions: Vec<String>,
}

impl ExtensionsManager {
    /// Create a new extensions manager
    pub fn new(conn: Connection) -> Self {
        Self {
            conn,
            loaded_extensions: Vec::new(),
        }
    }

    /// Load a DuckDB extension
    pub fn load_extension(&mut self, extension_name: &str) -> Result<()> {
        log::info!("Loading DuckDB extension: {}", extension_name);

        // Enable auto-install for extensions
        self.conn.execute("SET autoinstall_known_extensions = true", [])?;
        self.conn.execute("SET autoload_known_extensions = true", [])?;

        let sql = format!("LOAD {}", extension_name);
        self.conn.execute(&sql, [])?;

        self.loaded_extensions.push(extension_name.to_string());
        log::info!("Successfully loaded extension: {}", extension_name);

        Ok(())
    }

    /// Check if an extension is loaded
    pub fn is_loaded(&self, extension_name: &str) -> bool {
        self.loaded_extensions.contains(&extension_name.to_string())
    }

    /// Get list of loaded extensions
    pub fn loaded_extensions(&self) -> &[String] {
        &self.loaded_extensions
    }

    /// Load common extensions for file formats
    pub fn load_common_extensions(&mut self) -> Result<()> {
        let extensions = vec![
            "parquet",
            "json",
            // Add more extensions as needed
        ];

        for ext in extensions {
            if !self.is_loaded(ext) {
                if let Err(e) = self.load_extension(ext) {
                    log::warn!("Failed to load extension {}: {}", ext, e);
                    // Continue with other extensions
                }
            }
        }

        Ok(())
    }
}