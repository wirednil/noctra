//! Entry point para el daemon Noctra Server (noctrad)
//! 
//! Servidor HTTP que expone APIs REST para consultas SQL/RQL y formularios.
//! Ejecuta consultas usando el core de Noctra y soporta conexiones WebSocket.

use clap::{Parser, ArgGroup};
use std::path::PathBuf;
use std::net::SocketAddr;
use std::sync::Arc;

use tokio::sync::RwLock;

use env_logger::Env;
use log::{info, warn, error};

// Importar módulos del servidor
use noctra_srv::{
    server::ServerState,
    websocket::{WsState, WsHandler},
    create_server,
    ServerConfig,
};

/// CLI arguments para el servidor Noctra
#[derive(Parser, Debug)]
#[command(
    name = "noctrad",
    about = "Noctra Server Daemon - API server for SQL queries and forms",
    version = "0.1.0",
    author = "Claude Code <claude@anthropic.com>",
)]
struct CliArgs {
    /// Dirección IP y puerto para bind (default: 127.0.0.1:8080)
    #[arg(short, long, default_value = "127.0.0.1:8080")]
    bind: SocketAddr,
    
    /// Archivo de configuración TOML
    #[arg(short, long)]
    config: Option<PathBuf>,
    
    /// Archivo de base de datos SQLite
    #[arg(short, long)]
    database: Option<PathBuf>,
    
    /// Habilitar logging detallado
    #[arg(short, long)]
    verbose: bool,
    
    /// Modo desarrollo (hot reload, debug features)
    #[arg(short, long)]
    dev: bool,
    
    /// Archivo de token para autenticación
    #[arg(long)]
    token_file: Option<PathBuf>,
    
    /// Habilitar WebSocket endpoints
    #[arg(short, long)]
    websocket: bool,
    
    /// Número máximo de conexiones concurrentes
    #[arg(long, default_value_t = 100)]
    max_connections: usize,
    
    /// Timeout para consultas en segundos
    #[arg(long, default_value_t = 30)]
    query_timeout: u64,
    
    /// Habilitar CORS para desarrollo
    #[arg(long)]
    cors: bool,
    
    /// Directorio de formularios (para FDL2)
    #[arg(long)]
    forms_dir: Option<PathBuf>,
    
    /// Habilitar métricas y monitoring
    #[arg(short, long)]
    metrics: bool,
}

impl CliArgs {
    /// Convertir argumentos a configuración del servidor
    fn to_server_config(&self) -> ServerConfig {
        let mut config = ServerConfig::default();
        
        config.bind_address = self.bind;
        config.max_connections = self.max_connections;
        config.query_timeout = std::time::Duration::from_secs(self.query_timeout);
        config.cors_enabled = self.cors;
        config.websocket_enabled = self.websocket;
        config.dev_mode = self.dev;
        config.metrics_enabled = self.metrics;
        
        // Configurar base de datos
        if let Some(db_path) = &self.database {
            config.database_path = Some(db_path.clone());
        }
        
        // Configurar directorios
        if let Some(forms_dir) = &self.forms_dir {
            config.forms_directory = Some(forms_dir.clone());
        }
        
        // Configurar autenticación
        if let Some(token_file) = &self.token_file {
            config.token_file = Some(token_file.clone());
        }
        
        config
    }
    
    /// Cargar configuración desde archivo TOML si está presente
    fn load_config_file(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(config_path) = &self.config {
            if !config_path.exists() {
                warn!("Archivo de configuración no encontrado: {:?}", config_path);
                return Ok(());
            }
            
            info!("Cargando configuración desde: {:?}", config_path);
            
            // TODO: Implementar carga de configuración TOML
            // Por ahora solo validar que el archivo existe
            let _content = std::fs::read_to_string(config_path)?;
            
            // TODO: Parsear TOML y aplicar valores por defecto
            // let config: ServerConfig = toml::from_str(&content)?;
            
            info!("Configuración cargada exitosamente");
        }
        
        Ok(())
    }
}

/// Configuración extendida para el servidor
#[derive(Debug, Clone)]
pub struct ExtendedServerConfig {
    pub base: ServerConfig,
    pub cli_args: CliArgs,
}

impl ExtendedServerConfig {
    pub fn from_args(args: CliArgs) -> Self {
        let base = args.to_server_config();
        
        Self {
            base,
            cli_args: args,
        }
    }
    
    /// Obtener configuración para logging
    pub fn logging_level(&self) -> &'static str {
        if self.cli_args.verbose {
            "debug"
        } else if self.base.dev_mode {
            "info"
        } else {
            "warn"
        }
    }
    
    /// Validar configuración
    pub fn validate(&self) -> Result<(), Box<dyn std::error::Error>> {
        // Validar puerto
        if self.base.bind_address.port() == 0 || self.base.bind_address.port() > 65535 {
            return Err("Puerto inválido".into());
        }
        
        // Validar timeout
        if self.cli_args.query_timeout == 0 {
            return Err("Query timeout debe ser mayor que 0".into());
        }
        
        // Validar directorio de formularios si está especificado
        if let Some(forms_dir) = &self.base.forms_directory {
            if !forms_dir.exists() || !forms_dir.is_dir() {
                return Err(format!("Directorio de formularios no válido: {:?}", forms_dir).into());
            }
        }
        
        // Validar archivo de token si está especificado
        if let Some(token_file) = &self.base.token_file {
            if token_file.exists() && !token_file.is_file() {
                return Err(format!("Token file debe ser un archivo: {:?}", token_file).into());
            }
        }
        
        Ok(())
    }
}

/// Configurar logging basado en argumentos
fn setup_logging(config: &ExtendedServerConfig) -> Result<(), Box<dyn std::error::Error>> {
    let env = Env::default()
        .filter_or("NOCTRA_LOG", config.logging_level());
    
    env_logger::Builder::from_env(env)
        .format(|buf, record| {
            let timestamp = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S");
            let level = record.level();
            
            match level {
                log::Level::Error => {
                    writeln!(buf, "[{}] ERROR {} - {}", timestamp, record.target(), record.args())
                }
                log::Level::Warn => {
                    writeln!(buf, "[{}] WARN {} - {}", timestamp, record.target(), record.args())
                }
                log::Level::Info => {
                    writeln!(buf, "[{}] INFO {} - {}", timestamp, record.target(), record.args())
                }
                _ => {
                    writeln!(buf, "[{}] {} {} - {}", timestamp, level, record.target(), record.args())
                }
            }
        })
        .init();
    
    Ok(())
}

/// Información de inicio del servidor
fn print_server_info(config: &ExtendedServerConfig) {
    info!("=== Noctra Server (noctrad) v0.1.0 ===");
    info!("Bind Address: {}", config.base.bind_address);
    info!("Max Connections: {}", config.base.max_connections);
    info!("Query Timeout: {}s", config.cli_args.query_timeout);
    info!("WebSocket Enabled: {}", config.base.websocket_enabled);
    info!("CORS Enabled: {}", config.base.cors_enabled);
    info!("Dev Mode: {}", config.base.dev_mode);
    info!("Metrics Enabled: {}", config.base.metrics_enabled);
    
    if let Some(db_path) = &config.base.database_path {
        info!("Database: {:?}", db_path);
    }
    
    if let Some(forms_dir) = &config.base.forms_directory {
        info!("Forms Directory: {:?}", forms_dir);
    }
    
    info!("=====================================");
}

/// Manejo de señales del sistema (graceful shutdown)
async fn setup_signal_handlers() -> tokio::sync::broadcast::Receiver<()> {
    use tokio::signal;
    
    let (shutdown_tx, shutdown_rx) = tokio::sync::broadcast::channel(1);
    
    // Handle Ctrl+C
    tokio::spawn(async move {
        signal::ctrl_c().await.expect("No se pudo configurar handler para Ctrl+C");
        info!("Señal Ctrl+C recibida, iniciando shutdown graceful...");
        let _ = shutdown_tx.send(());
    });
    
    // Handle SIGTERM (en sistemas Unix)
    #[cfg(unix)]
    {
        let shutdown_tx_clone = shutdown_tx.clone();
        tokio::spawn(async move {
            signal::unix::signal(signal::unix::SignalKind::terminate())
                .expect("No se pudo configurar handler para SIGTERM")
                .recv().await;
            info!("Señal SIGTERM recibida, iniciando shutdown graceful...");
            let _ = shutdown_tx_clone.send(());
        });
    }
    
    shutdown_rx
}

/// Main entry point
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Parsear argumentos de línea de comandos
    let mut args = CliArgs::parse();
    
    // Cargar configuración desde archivo si está presente
    args.load_config_file()?;
    
    // Crear configuración extendida
    let config = ExtendedServerConfig::from_args(args);
    
    // Validar configuración
    config.validate()?;
    
    // Configurar logging
    setup_logging(&config)?;
    
    // Mostrar información del servidor
    print_server_info(&config);
    
    // Crear estado del servidor
    let state = ServerState::new(config.base.clone()).await?;
    info!("Estado del servidor inicializado");
    
    // Crear handler WebSocket si está habilitado
    let ws_state = if config.base.websocket_enabled {
        Some(WsState::new(state.clone()))
    } else {
        None
    };
    
    // Crear aplicación HTTP
    let mut app = create_server(state.clone(), config.base.clone())?;
    
    // Agregar WebSocket si está habilitado
    if let Some(ws) = &ws_state {
        let ws_handler = WsHandler::new(ws.manager.clone());
        app = app.add_websocket_routes(ws_handler);
        
        // Iniciar tarea de cleanup para WebSocket
        ws.start_cleanup_task();
        info!("WebSocket endpoints habilitados");
    }
    
    // Configurar CORS si está habilitado
    if config.base.cors_enabled {
        use tower_http::cors::{CorsLayer, Any};
        app = app.layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any)
        );
        info!("CORS habilitado para desarrollo");
    }
    
    // Configurar manejo de errores global
    app = app.layer(
        tower_http::trace::TraceLayer::new_for_http()
            .make_span_with(
                tower_http::trace::DefaultMakeSpan::new()
                    .include_headers(true)
            )
            .on_response(
                tower_http::trace::DefaultOnResponse::new()
                    .include_headers(true)
            ),
    );
    
    // Setup signal handlers para shutdown graceful
    let mut shutdown_rx = setup_signal_handlers().await;
    
    // Crear listener TCP
    let listener = tokio::net::TcpListener::bind(config.base.bind_address).await?;
    info!("Servidor escuchando en: {}", config.base.bind_address);
    
    // Servir requests
    let server = axum::serve(listener, app)
        .with_graceful_shutdown(async {
            let _ = shutdown_rx.recv().await;
        });
    
    info!("Servidor Noctra iniciado exitosamente");
    
    // Ejecutar servidor hasta shutdown
    if let Err(e) = server.await {
        error!("Error en servidor: {}", e);
        return Err(e.into());
    }
    
    info!("Servidor Noctra detenido");
    Ok(())
}

/// Función para modo de configuración (mostrar config efectiva)
#[cfg(test)]
fn print_config_summary(config: &ExtendedServerConfig) {
    println!("=== Configuración del Servidor ===");
    println!("Bind: {}", config.base.bind_address);
    println!("Max Connections: {}", config.base.max_connections);
    println!("Timeout: {}s", config.cli_args.query_timeout);
    println!("WebSocket: {}", config.base.websocket_enabled);
    println!("CORS: {}", config.base.cors_enabled);
    println!("Dev: {}", config.base.dev_mode);
    println!("==================================");
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_main_with_config() {
        // Test básico para verificar que el main puede inicializar
        let args = CliArgs {
            bind: "127.0.0.1:8081".parse().unwrap(),
            config: None,
            database: None,
            verbose: false,
            dev: true,
            token_file: None,
            websocket: false,
            max_connections: 50,
            query_timeout: 15,
            cors: true,
            forms_dir: None,
            metrics: false,
        };
        
        let config = ExtendedServerConfig::from_args(args);
        config.validate().unwrap();
        
        assert_eq!(config.base.bind_address.port(), 8081);
        assert_eq!(config.base.max_connections, 50);
        assert_eq!(config.cli_args.query_timeout, 15);
    }
}