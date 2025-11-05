//! Noctra Server - API server library
//! 
//! Librería principal del servidor HTTP para Noctra que expone APIs REST
//! para consultas SQL/RQL, formularios FDL2 y gestión de sesiones.

pub mod server;
pub mod routes;
pub mod handlers;
pub mod websocket;
pub mod types;
pub mod performance;

pub use server::{ServerState, ServerConfig, create_server, run_server, run_server_cli};
pub use routes::{NoctraRouter, create_router};
pub use handlers::{QueryHandler, FormHandler, SessionHandler, ServerHandler};
pub use websocket::{WsManager, WsHandler, WsState};
pub use types::{QueryRequest, QueryResponse, FormRequest, FormResponse, ServerStatus, ServerError};

use std::net::SocketAddr;
use std::time::Duration;

/// Versión del servidor
pub const VERSION: &str = "0.1.0";

/// Puerto por defecto
pub const DEFAULT_PORT: u16 = 8080;

/// Host por defecto
pub const DEFAULT_HOST: &str = "127.0.0.1";

/// Crear configuración rápida
pub fn quick_config() -> ServerConfig {
    ServerConfig {
        bind_address: format!("{}:{}", DEFAULT_HOST, DEFAULT_PORT).parse().unwrap(),
        database_url: "sqlite:noctra.db".to_string(),
        request_timeout: Duration::from_secs(30),
        max_connections: 100,
        auth_secret: None,
        cors_enabled: true,
        websocket_enabled: true,
        dev_mode: false,
        metrics_enabled: true,
        database_path: None,
        forms_directory: None,
        token_file: None,
        rate_limiting_enabled: true,
        query_timeout: Duration::from_secs(30),
    }
}

/// Crear configuración de desarrollo
pub fn dev_config() -> ServerConfig {
    let mut config = quick_config();
    config.dev_mode = true;
    config.cors_enabled = true;
    config.metrics_enabled = true;
    config.bind_address = "127.0.0.1:8081".parse().unwrap();
    config
}

/// Crear configuración de producción
pub fn prod_config() -> ServerConfig {
    let mut config = quick_config();
    config.dev_mode = false;
    config.cors_enabled = false;
    config.metrics_enabled = false;
    config.bind_address = "0.0.0.0:8080".parse().unwrap();
    config
}

/// CLI helpers para el servidor
pub mod cli {
    use super::*;
    use clap::{Parser, ArgGroup};
    use std::path::PathBuf;
    
    /// Argumentos CLI simplificados
    #[derive(Parser, Debug)]
    #[command(name = "noctrad")]
    pub struct SimpleArgs {
        /// Puerto para bind
        #[arg(short, long, default_value_t = 8080)]
        pub port: u16,
        
        /// Archivo de base de datos
        #[arg(short, long)]
        pub database: Option<PathBuf>,
        
        /// Directorio de formularios
        #[arg(short, long)]
        pub forms: Option<PathBuf>,
        
        /// Modo desarrollo
        #[arg(short, long)]
        pub dev: bool,
        
        /// Habilitar métricas
        #[arg(short, long)]
        pub metrics: bool,
        
        /// Configuración personalizada
        #[arg(short, long)]
        pub config: Option<PathBuf>,
    }
    
    /// Configuración desde argumentos simplificados
    pub fn config_from_args(args: SimpleArgs) -> ServerConfig {
        let mut config = if args.dev { dev_config() } else { quick_config() };
        
        config.bind_address = format!("{}:{}", DEFAULT_HOST, args.port).parse().unwrap();
        config.database_path = args.database;
        config.forms_directory = args.forms;
        config.metrics_enabled = args.metrics;
        config.dev_mode = args.dev;
        
        config
    }
    
    /// Ejecutar servidor con argumentos simples
    pub async fn run_simple() -> Result<(), Box<dyn std::error::Error>> {
        let args = SimpleArgs::parse();
        let config = config_from_args(args);
        run_server(config).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_quick_config() {
        let config = quick_config();
        assert_eq!(config.bind_address.port(), DEFAULT_PORT);
        assert_eq!(config.bind_address.ip().to_string(), DEFAULT_HOST);
        assert!(config.cors_enabled);
        assert!(!config.dev_mode);
    }
    
    #[test]
    fn test_dev_config() {
        let config = dev_config();
        assert!(config.dev_mode);
        assert_eq!(config.bind_address.port(), 8081);
        assert!(config.cors_enabled);
        assert!(config.metrics_enabled);
    }
    
    #[test]
    fn test_prod_config() {
        let config = prod_config();
        assert!(!config.dev_mode);
        assert_eq!(config.bind_address.ip().to_string(), "0.0.0.0");
        assert!(!config.cors_enabled);
        assert!(!config.metrics_enabled);
    }
}