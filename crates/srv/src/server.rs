//! Servidor principal de Noctra
//! 
//! Servidor HTTP/TCP que expone APIs REST para consultas SQL,
//! formularios FDL2 y gestión de sesiones.

use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};

use axum::{
    extract::{State, ConnectInfo},
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use tracing::{info, warn, error};
use tokio::signal;

use noctra_core::{Session, Executor};
use noctra_parser::RqlParser;

use crate::routes::{create_router, NoctraRouter};
use crate::handlers::{QueryHandler, FormHandler, SessionHandler};
use crate::types::{QueryRequest, QueryResponse, FormRequest, FormResponse, ServerStatus, ServerError};
use crate::performance::{PerformanceMiddleware, PerformanceConfig, SerializedMetrics};

/// Configuración extendida del servidor
#[derive(Debug, Clone)]
pub struct ServerConfig {
    /// Dirección IP y puerto
    pub bind_address: SocketAddr,
    
    /// URL de base de datos
    pub database_url: String,
    
    /// Timeout de requests (segundos)
    pub request_timeout: Duration,
    
    /// Máximo de conexiones concurrentes
    pub max_connections: usize,
    
    /// Secret para autenticación
    pub auth_secret: Option<String>,
    
    /// Habilitar CORS
    pub cors_enabled: bool,
    
    /// Habilitar WebSockets
    pub websocket_enabled: bool,
    
    /// Modo desarrollo
    pub dev_mode: bool,
    
    /// Habilitar métricas
    pub metrics_enabled: bool,
    
    /// Configuración adicional para rutas
    pub database_path: Option<std::path::PathBuf>,
    pub forms_directory: Option<std::path::PathBuf>,
    pub token_file: Option<std::path::PathBuf>,
    
    /// Configuraciones de performance
    pub rate_limiting_enabled: bool,
    pub query_timeout: Duration,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            bind_address: "127.0.0.1:8080".parse().unwrap(),
            database_url: "sqlite:noctra.db".to_string(),
            request_timeout: Duration::from_secs(30),
            max_connections: 100,
            auth_secret: None,
            cors_enabled: true,
            websocket_enabled: true,
            dev_mode: false,
            metrics_enabled: false,
            database_path: None,
            forms_directory: None,
            token_file: None,
            rate_limiting_enabled: true,
            query_timeout: Duration::from_secs(30),
        }
    }
}

/// Estado compartido del servidor
#[derive(Clone)]
pub struct ServerState {
    /// Executor para consultas
    pub executor: Arc<tokio::sync::RwLock<Option<Executor>>>,
    
    /// Parser RQL
    pub parser: Arc<tokio::sync::RwLock<Option<RqlParser>>>,
    
    /// Sesiones activas
    pub sessions: Arc<tokio::sync::RwLock<Vec<Session>>>,
    
    /// Configuración del servidor
    pub config: Arc<tokio::sync::RwLock<ServerConfig>>,
    
    /// Middleware de performance
    pub performance: Arc<PerformanceMiddleware>,
    
    /// Inicio del servidor
    pub start_time: std::time::Instant,
}

impl ServerState {
    /// Crear nuevo estado del servidor
    pub async fn new(config: ServerConfig) -> Result<Self, Box<dyn std::error::Error>> {
        let performance = Arc::new(PerformanceMiddleware::new(&config));
        
        // Inicializar tasks de background
        performance.start_background_tasks();
        
        // Crear executor si hay database path
        let executor = if let Some(db_path) = &config.database_path {
            if db_path.exists() {
                let executor = Executor::new_sqlite(db_path).await?;
                Some(executor)
            } else {
                warn!("Database file not found: {:?}", db_path);
                None
            }
        } else {
            Some(Executor::new(config.database_url.clone()))
        };
        
        // Crear parser
        let parser = RqlParser::new();
        
        let state = Self {
            executor: Arc::new(tokio::sync::RwLock::new(executor)),
            parser: Arc::new(tokio::sync::RwLock::new(Some(parser))),
            sessions: Arc::new(tokio::sync::RwLock::new(Vec::new())),
            config: Arc::new(tokio::sync::RwLock::new(config.clone())),
            performance: performance.clone(),
            start_time: std::time::Instant::now(),
        };
        
        info!("Estado del servidor inicializado");
        
        Ok(state)
    }
    
    /// Obtener executor (se crea si no existe)
    pub async fn get_executor(&self) -> Result<Arc<Executor>, String> {
        let mut executor_opt = self.executor.write().await;
        
        if executor_opt.is_none() {
            let config = self.config.read().await.clone();
            
            // Crear nuevo executor
            if let Some(db_path) = config.database_path {
                if db_path.exists() {
                    let new_executor = Executor::new_sqlite(&db_path).await.map_err(|e| e.to_string())?;
                    *executor_opt = Some(new_executor);
                } else {
                    return Err(format!("Database file not found: {:?}", db_path));
                }
            } else {
                let config = self.config.read().await;
                let new_executor = Executor::new(config.database_url.clone());
                *executor_opt = Some(new_executor);
            }
        }
        
        Ok(Arc::new(executor_opt.as_ref().unwrap().clone()))
    }
    
    /// Obtener parser
    pub async fn get_parser(&self) -> Arc<RqlParser> {
        let mut parser_opt = self.parser.write().await;
        
        if parser_opt.is_none() {
            *parser_opt = Some(RqlParser::new());
        }
        
        Arc::new(parser_opt.as_ref().unwrap().clone())
    }
    
    /// Actualizar configuración
    pub async fn update_config(&self, new_config: ServerConfig) {
        let mut config = self.config.write().await;
        *config = new_config;
    }
    
    /// Obtener métricas de performance
    pub async fn get_performance_metrics(&self) -> SerializedMetrics {
        self.performance.metrics.get_metrics().await
    }
}

/// Servidor HTTP principal
pub struct Server {
    state: ServerState,
    router: Router,
}

impl Server {
    /// Crear nuevo servidor con estado
    pub fn new(state: ServerState) -> Self {
        let router = Self::build_router(state.clone());
        
        Self { state, router }
    }
    
    /// Construir router con todas las rutas
    fn build_router(state: ServerState) -> Router {
        let mut router = Router::new()
            // Rutas principales
            .route("/", get(root_handler))
            .route("/health", get(health_handler))
            .route("/status", get(status_handler))
            
            // Rutas de consultas SQL/RQL
            .route("/api/v1/query/execute", post(query_execute_handler))
            .route("/api/v1/query/validate", post(query_validate_handler))
            .route("/api/v1/query/batch", post(batch_query_handler))
            
            // Rutas de formularios
            .route("/api/v1/form/:name", post(form_execute_handler))
            .route("/api/v1/form/:name/validate", post(form_validate_handler))
            .route("/api/v1/forms", get(forms_list_handler))
            
            // Rutas de sesiones
            .route("/api/v1/session", post(session_create_handler))
            .route("/api/v1/session/:id", get(session_get_handler))
            .route("/api/v1/session/:id", delete(session_delete_handler))
            .route("/api/v1/sessions", get(sessions_list_handler))
            
            // Rutas de configuración
            .route("/api/v1/config", get(config_handler))
            
            // Rutas de métricas
            .route("/api/v1/metrics", get(metrics_handler));
        
        // Agregar CORS si está habilitado
        {
            let config = state.config.blocking_read();
            if config.cors_enabled {
                router = router.layer(
                    CorsLayer::new()
                        .allow_origin(tower_http::cors::Any)
                        .allow_methods(tower_http::cors::Any)
                        .allow_headers(tower_http::cors::Any),
                );
            }
        }
        
        // Agregar tracing y manejo de errores
        router = router
            .layer(TraceLayer::new_for_http())
            .with_state(state)
    }
    
    /// Iniciar servidor
    pub async fn run(self) -> Result<(), Box<dyn std::error::Error>> {
        let addr = {
            let config = self.state.config.blocking_read();
            config.bind_address
        };
        
        info!("🚀 Iniciando servidor Noctra en {}", addr);
        info!("📊 Configuración:");
        
        let config = self.state.config.blocking_read();
        info!("   🗄️ Base de datos: {}", config.database_url);
        info!("   ⏱️ Timeout: {:?}", config.request_timeout);
        info!("   🔗 Conexiones máximas: {}", config.max_connections);
        info!("   🌐 CORS: {}", if config.cors_enabled { "Habilitado" } else { "Deshabilitado" });
        info!("   🔌 WebSockets: {}", if config.websocket_enabled { "Habilitado" } else { "Deshabilitado" });
        info!("   🛠️ Modo desarrollo: {}", if config.dev_mode { "Habilitado" } else { "Deshabilitado" });
        info!("   📊 Métricas: {}", if config.metrics_enabled { "Habilitado" } else { "Deshabilitado" });
        
        // Configurar graceful shutdown
        let server_handle = tokio::spawn(async move {
            axum::Server::bind(&addr)
                .serve(self.router)
                .with_graceful_shutdown(shutdown_signal())
                .await
                .expect("Error iniciando servidor")
        });
        
        // Esperar a que termine
        server_handle.await?;
        
        info!("👋 Servidor Noctra finalizado");
        
        Ok(())
    }
    
    /// Obtener estado del servidor
    pub fn get_status(&self) -> ServerStatus {
        let uptime = self.state.start_time.elapsed();
        let sessions = self.state.sessions.blocking_read();
        
        ServerStatus {
            version: "0.1.0".to_string(),
            uptime_seconds: uptime.as_secs(),
            connected_sessions: sessions.len(),
            active_queries: 0, // TODO: Implementar contador real
            database_status: "connected".to_string(),
        }
    }
}

/// Función para crear servidor y router
pub fn create_server(
    state: ServerState, 
    config: ServerConfig
) -> Result<Router, Box<dyn std::error::Error>> {
    let mut server = Server::new(state);
    Ok(server.router)
}

/// Función para manejar graceful shutdown
async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("Error configurando Ctrl+C handler");
    };
    
    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("Error configurando terminate handler")
            .recv()
            .await;
    };
    
    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();
    
    tokio::select! {
        _ = ctrl_c => {
            info!("📴 Recibida señal Ctrl+C");
        },
        _ = terminate => {
            info!("📴 Recibida señal terminate");
        },
    }
    
    info!("🛑 Cerrando servidor...");
}

// =================== HANDLERS ===================

/// Handler raíz - información del servidor
async fn root_handler() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "name": "Noctra Server",
        "version": "0.1.0",
        "description": "Entorno SQL Interactivo - Servidor HTTP",
        "endpoints": {
            "health": "/health",
            "status": "/status", 
            "query": "/api/v1/query/execute",
            "form": "/api/v1/form/{name}",
            "session": "/api/v1/session",
            "metrics": "/api/v1/metrics"
        }
    }))
}

/// Handler de health check
async fn health_handler(State(state): State<ServerState>) -> Result<Json<serde_json::Value>, StatusCode> {
    // Verificar que el executor esté disponible
    if let Err(_) = state.get_executor().await {
        return Err(StatusCode::SERVICE_UNAVAILABLE);
    }
    
    Ok(Json(serde_json::json!({
        "status": "healthy",
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "version": "0.1.0",
        "uptime_seconds": state.start_time.elapsed().as_secs()
    })))
}

/// Handler de estado del servidor
async fn status_handler(State(state): State<ServerState>) -> Json<ServerStatus> {
    let status = ServerStatus {
        version: "0.1.0".to_string(),
        uptime_seconds: state.start_time.elapsed().as_secs(),
        connected_sessions: state.sessions.read().await.len(),
        active_queries: 0, // TODO: Implementar
        database_status: "connected".to_string(),
    };
    
    Json(status)
}

/// Handler para ejecutar consulta SQL/RQL
async fn query_execute_handler(
    State(state): State<ServerState>,
    Json(request): Json<QueryRequest>,
) -> Result<Json<QueryResponse>, StatusCode> {
    let start_time = std::time::Instant::now();
    
    // TODO: Usar performance middleware para cache y rate limiting
    let executor = state.get_executor().await.map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
    let parser = state.get_parser().await;
    
    // TODO: Ejecutar consulta real usando executor
    // Por ahora simular resultado
    let mock_data = noctra_core::ResultSet::empty();
    
    let execution_time = start_time.elapsed().as_millis() as u64;
    
    let response = QueryResponse {
        success: true,
        data: Some(mock_data),
        message: "Consulta ejecutada (simulada)".to_string(),
        execution_time_ms: execution_time,
    };
    
    // Registrar métricas de performance
    state.performance.metrics.record_success(start_time.elapsed()).await;
    
    Ok(Json(response))
}

/// Handler para validar consulta
async fn query_validate_handler(
    State(state): State<ServerState>,
    Json(request): Json<QueryRequest>,
) -> Result<Json<QueryResponse>, StatusCode> {
    // TODO: Usar parser para validar SQL
    let response = QueryResponse {
        success: true,
        data: None,
        message: "Consulta válida (validación simulada)".to_string(),
        execution_time_ms: 0,
    };
    
    Ok(Json(response))
}

/// Handler para consultas batch
async fn batch_query_handler(
    State(state): State<ServerState>,
    Json(requests): Json<Vec<QueryRequest>>,
) -> Result<Json<Vec<QueryResponse>>, StatusCode> {
    let mut responses = Vec::new();
    
    for request in requests {
        let start_time = std::time::Instant::now();
        
        // TODO: Ejecutar consulta real
        let mock_data = noctra_core::ResultSet::empty();
        let execution_time = start_time.elapsed().as_millis() as u64;
        
        let response = QueryResponse {
            success: true,
            data: Some(mock_data),
            message: "Consulta batch ejecutada (simulada)".to_string(),
            execution_time_ms: execution_time,
        };
        
        responses.push(response);
    }
    
    Ok(Json(responses))
}

/// Handler para ejecutar formulario
async fn form_execute_handler(
    State(state): State<ServerState>,
    axum::extract::Path(name): axum::extract::Path<String>,
    Json(request): Json<FormRequest>,
) -> Result<Json<FormResponse>, StatusCode> {
    // TODO: Cargar y ejecutar formulario real
    let response = FormResponse {
        success: true,
        data: None,
        message: format!("Formulario '{}' ejecutado (simulado)", name),
        form_title: Some(name),
    };
    
    Ok(Json(response))
}

/// Handler para validar formulario
async fn form_validate_handler(
    State(state): State<ServerState>,
    axum::extract::Path(name): axum::extract::Path<String>,
    Json(request): Json<FormRequest>,
) -> Result<Json<FormResponse>, StatusCode> {
    // TODO: Validar formulario real
    let response = FormResponse {
        success: true,
        data: None,
        message: format!("Formulario '{}' validado (simulado)", name),
        form_title: Some(name),
    };
    
    Ok(Json(response))
}

/// Handler para listar formularios
async fn forms_list_handler(State(_state): State<ServerState>) -> Result<Json<serde_json::Value>, StatusCode> {
    // TODO: Listar formularios desde forms directory
    Ok(Json(serde_json::json!({
        "forms": [
            {
                "name": "empleados",
                "title": "Consulta Empleados",
                "description": "Formulario para consultar empleados"
            }
        ],
        "total": 1
    })))
}

/// Handler para crear sesión
async fn session_create_handler(State(state): State<ServerState>) -> Result<Json<serde_json::Value>, StatusCode> {
    let session_id = format!("session_{}", chrono::Utc::now().timestamp());
    let session = Session::new(session_id.clone());
    let mut sessions = state.sessions.write().await;
    sessions.push(session);
    
    Ok(Json(serde_json::json!({
        "session_id": session_id,
        "message": "Sesión creada exitosamente",
        "expires_in": 3600,
        "created_at": chrono::Utc::now().to_rfc3339()
    })))
}

/// Handler para obtener sesión
async fn session_get_handler(
    State(state): State<ServerState>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let sessions = state.sessions.read().await;
    
    // Buscar sesión
    for session in sessions.iter() {
        if session.id == id {
            return Ok(Json(serde_json::json!({
                "session_id": session.id,
                "status": "active",
                "created_at": session.created_at,
                "variables": session.variables
            })));
        }
    }
    
    Err(StatusCode::NOT_FOUND)
}

/// Handler para eliminar sesión
async fn session_delete_handler(
    State(state): State<ServerState>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let mut sessions = state.sessions.write().await;
    let original_len = sessions.len();
    
    sessions.retain(|s| s.id != id);
    
    if sessions.len() < original_len {
        Ok(Json(serde_json::json!({
            "message": format!("Sesión {} eliminada", id)
        })))
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

/// Handler para listar sesiones
async fn sessions_list_handler(State(state): State<ServerState>) -> Result<Json<serde_json::Value>, StatusCode> {
    let sessions = state.sessions.read().await;
    
    Ok(Json(serde_json::json!({
        "sessions": sessions.iter().map(|s| serde_json::json!({
            "id": s.id,
            "created_at": s.created_at,
            "status": "active"
        })).collect::<Vec<_>>(),
        "total": sessions.len()
    })))
}

/// Handler para configuración
async fn config_handler(State(state): State<ServerState>) -> Json<serde_json::Value> {
    let config = state.config.read().await;
    
    Json(serde_json::json!({
        "database_url": config.database_url,
        "request_timeout": config.request_timeout.as_secs(),
        "max_connections": config.max_connections,
        "cors_enabled": config.cors_enabled,
        "websocket_enabled": config.websocket_enabled,
        "dev_mode": config.dev_mode,
        "metrics_enabled": config.metrics_enabled
    }))
}

/// Handler para métricas
async fn metrics_handler(State(state): State<ServerState>) -> Result<Json<serde_json::Value>, StatusCode> {
    let metrics = state.get_performance_metrics().await;
    
    Ok(Json(serde_json::json!({
        "server": {
            "version": "0.1.0",
            "uptime": metrics.uptime_seconds,
        },
        "api": {
            "total_requests": metrics.requests_total,
            "success_requests": metrics.requests_success,
            "error_requests": metrics.requests_error,
            "success_rate": metrics.success_rate,
            "avg_response_time_ms": metrics.avg_response_time_ms,
            "requests_per_second": metrics.requests_per_second
        },
        "sessions": {
            "active": state.sessions.read().await.len()
        }
    })))
}

/// Función helper para crear server y ejecutarlo
pub async fn run_server(config: ServerConfig) -> Result<(), Box<dyn std::error::Error>> {
    let state = ServerState::new(config).await?;
    let server = Server::new(state);
    server.run().await
}

/// Ejecutar servidor con argumentos CLI
pub async fn run_server_cli() -> Result<(), Box<dyn std::error::Error>> {
    let args = CliArgs::parse();
    
    let mut config = ServerConfig::default();
    config.bind_address = args.bind_address.parse()?;
    config.database_url = args.database_url;
    config.database_path = args.database_path;
    config.forms_directory = args.forms_dir;
    config.token_file = args.token_file;
    config.cors_enabled = !args.no_cors;
    config.websocket_enabled = !args.no_websockets;
    config.dev_mode = args.dev;
    config.metrics_enabled = args.metrics;
    
    if let Some(secret) = args.auth_secret {
        config.auth_secret = Some(secret);
    }
    
    run_server(config).await
}

/// Argumentos CLI para el servidor
#[derive(clap::Parser, Debug)]
#[command(name = "noctrad")]
struct CliArgs {
    /// Dirección para vincular (formato: host:puerto)
    #[arg(short, long, default_value = "127.0.0.1:8080")]
    bind_address: String,
    
    /// URL de base de datos
    #[arg(short, long, default_value = "sqlite:noctra.db")]
    database_url: String,
    
    /// Archivo de base de datos SQLite
    #[arg(short, long)]
    database_path: Option<std::path::PathBuf>,
    
    /// Secret de autenticación
    #[arg(short, long)]
    auth_secret: Option<String>,
    
    /// Deshabilitar CORS
    #[arg(long)]
    no_cors: bool,
    
    /// Deshabilitar WebSockets
    #[arg(long)]
    no_websockets: bool,
    
    /// Modo desarrollo
    #[arg(short, long)]
    dev: bool,
    
    /// Habilitar métricas
    #[arg(short, long)]
    metrics: bool,
    
    /// Archivo de token para autenticación
    #[arg(long)]
    token_file: Option<std::path::PathBuf>,
    
    /// Directorio de formularios
    #[arg(long)]
    forms_dir: Option<std::path::PathBuf>,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_server_state_creation() {
        let config = ServerConfig::default();
        let state = ServerState::new(config).await.unwrap();
        
        assert!(state.executor.blocking_read().is_some());
        assert!(state.parser.blocking_read().is_some());
        assert_eq!(state.sessions.blocking_read().len(), 0);
    }
    
    #[tokio::test]
    async fn test_server_get_executor() {
        let config = ServerConfig::default();
        let state = ServerState::new(config).await.unwrap();
        
        let executor = state.get_executor().await.unwrap();
        assert!(executor.is_some());
    }
}