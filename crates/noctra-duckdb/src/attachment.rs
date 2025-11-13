//! Attachment Registry for persistent cross-database connections
//!
//! Manages ATTACH statements which are non-persistent in DuckDB.
//! Must be restored on each connection initialization.
//!
//! ## Problem
//! DuckDB's ATTACH statements are session-only and lost upon restart.
//! Views defined over attached databases will fail after reconnection.
//!
//! ## Solution
//! This registry tracks all attachments and provides methods to:
//! - Register attachments when they're created
//! - Generate SQL to restore all attachments
//! - Serialize/deserialize for persistence (optional)

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Registry of database attachments
///
/// Tracks all active database attachments for restoration after connection restart
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AttachmentRegistry {
    attachments: HashMap<String, AttachmentConfig>,
}

/// Configuration for a single database attachment
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AttachmentConfig {
    /// Database type (e.g., "sqlite", "postgres")
    pub db_type: String,

    /// Path or connection string
    pub path: String,

    /// Alias name in DuckDB
    pub alias: String,

    /// Read-only mode
    pub read_only: bool,
}

impl AttachmentRegistry {
    /// Create new empty registry
    pub fn new() -> Self {
        Self {
            attachments: HashMap::new(),
        }
    }

    /// Register a new attachment
    ///
    /// If an attachment with the same alias already exists, it will be replaced
    pub fn register(&mut self, config: AttachmentConfig) {
        self.attachments.insert(config.alias.clone(), config);
    }

    /// Remove an attachment by alias
    ///
    /// Returns the removed config if it existed
    pub fn unregister(&mut self, alias: &str) -> Option<AttachmentConfig> {
        self.attachments.remove(alias)
    }

    /// Get all attachments as a vector
    pub fn list(&self) -> Vec<&AttachmentConfig> {
        self.attachments.values().collect()
    }

    /// Get specific attachment by alias
    pub fn get(&self, alias: &str) -> Option<&AttachmentConfig> {
        self.attachments.get(alias)
    }

    /// Check if an attachment exists
    pub fn contains(&self, alias: &str) -> bool {
        self.attachments.contains_key(alias)
    }

    /// Get number of registered attachments
    pub fn len(&self) -> usize {
        self.attachments.len()
    }

    /// Check if registry is empty
    pub fn is_empty(&self) -> bool {
        self.attachments.is_empty()
    }

    /// Clear all attachments
    pub fn clear(&mut self) {
        self.attachments.clear();
    }

    /// Generate SQL statements to restore all attachments
    ///
    /// Returns a vector of SQL ATTACH commands that can be executed
    /// sequentially to restore all registered attachments
    pub fn to_sql_commands(&self) -> Vec<String> {
        self.attachments
            .values()
            .map(|config| {
                let read_only = if config.read_only { " READ_ONLY" } else { "" };
                format!(
                    "ATTACH '{}' AS {} (TYPE {}{});",
                    config.path, config.alias, config.db_type, read_only
                )
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_registry() {
        let registry = AttachmentRegistry::new();
        assert!(registry.is_empty());
        assert_eq!(registry.len(), 0);
    }

    #[test]
    fn test_register_attachment() {
        let mut registry = AttachmentRegistry::new();

        let config = AttachmentConfig {
            db_type: "sqlite".to_string(),
            path: "/path/to/db.sqlite".to_string(),
            alias: "test_db".to_string(),
            read_only: false,
        };

        registry.register(config.clone());

        assert_eq!(registry.len(), 1);
        assert!(registry.contains("test_db"));
        assert_eq!(registry.get("test_db"), Some(&config));
    }

    #[test]
    fn test_unregister_attachment() {
        let mut registry = AttachmentRegistry::new();

        let config = AttachmentConfig {
            db_type: "sqlite".to_string(),
            path: "/path/to/db.sqlite".to_string(),
            alias: "test_db".to_string(),
            read_only: false,
        };

        registry.register(config.clone());
        let removed = registry.unregister("test_db");

        assert_eq!(removed, Some(config));
        assert!(registry.is_empty());
    }

    #[test]
    fn test_list_attachments() {
        let mut registry = AttachmentRegistry::new();

        registry.register(AttachmentConfig {
            db_type: "sqlite".to_string(),
            path: "/db1.sqlite".to_string(),
            alias: "db1".to_string(),
            read_only: false,
        });

        registry.register(AttachmentConfig {
            db_type: "sqlite".to_string(),
            path: "/db2.sqlite".to_string(),
            alias: "db2".to_string(),
            read_only: true,
        });

        let list = registry.list();
        assert_eq!(list.len(), 2);
    }

    #[test]
    fn test_to_sql_commands() {
        let mut registry = AttachmentRegistry::new();

        registry.register(AttachmentConfig {
            db_type: "sqlite".to_string(),
            path: "/path/to/db.sqlite".to_string(),
            alias: "test_db".to_string(),
            read_only: false,
        });

        registry.register(AttachmentConfig {
            db_type: "sqlite".to_string(),
            path: "/readonly.sqlite".to_string(),
            alias: "readonly_db".to_string(),
            read_only: true,
        });

        let commands = registry.to_sql_commands();
        assert_eq!(commands.len(), 2);

        // Check that commands contain expected patterns
        assert!(commands.iter().any(|cmd| cmd.contains("test_db")));
        assert!(commands.iter().any(|cmd| cmd.contains("readonly_db")));
        assert!(commands.iter().any(|cmd| cmd.contains("READ_ONLY")));
    }

    #[test]
    fn test_clear() {
        let mut registry = AttachmentRegistry::new();

        registry.register(AttachmentConfig {
            db_type: "sqlite".to_string(),
            path: "/db1.sqlite".to_string(),
            alias: "db1".to_string(),
            read_only: false,
        });

        assert!(!registry.is_empty());
        registry.clear();
        assert!(registry.is_empty());
    }

    #[test]
    fn test_replace_existing() {
        let mut registry = AttachmentRegistry::new();

        let config1 = AttachmentConfig {
            db_type: "sqlite".to_string(),
            path: "/old.sqlite".to_string(),
            alias: "mydb".to_string(),
            read_only: false,
        };

        let config2 = AttachmentConfig {
            db_type: "sqlite".to_string(),
            path: "/new.sqlite".to_string(),
            alias: "mydb".to_string(), // Same alias
            read_only: true,
        };

        registry.register(config1);
        registry.register(config2.clone());

        // Should only have one attachment
        assert_eq!(registry.len(), 1);
        // Should be the new config
        assert_eq!(registry.get("mydb"), Some(&config2));
    }
}
