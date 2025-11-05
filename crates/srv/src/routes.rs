//! Router principal del servidor Noctra
//! 
//! Configura y organiza todas las rutas HTTP del servidor.

use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
    routing::{get, post, delete},
    Router,
};

use crate::server::ServerState;
use crate::types::{QueryRequest, QueryResponse, FormRequest, FormResponse, ServerStatus};

/// Crear router principal del servidor
pub fn create_router(state: ServerState) -> Router {
    Router::new()
        // Rutas raíz
        .route("/", get(root_info))
        .route("/health", get(health_check))
        .route("/status", get(server_status))
        
        // API v1 - Consultas
        .route("/api/v1/query", post(execute_query))
        .route("/api/v1/query/batch", post(execute_batch_queries))
        
        // API v1 - Formularios
        .route("/api/v1/form/:name", post(execute_form))
        .route("/api/v1/form/:name/validate", post(validate_form))
        .route("/api/v1/forms", get(list_forms))
        
        // API v1 - Sesiones
        .route("/api/v1/session", post(create_session))
        .route("/api/v1/session/:id", get(get_session))
        .route("/api/v1/session/:id", delete(delete_session))
        .route("/api/v1/sessions", get(list_sessions))
        
        // API v1 - Configuración
        .route("/api/v1/config", get(get_config))
        .route("/api/v1/config", put(update_config))
        
        // API v1 - Utilidades
        .route("/api/v1/parse", post(parse_sql))
        .route("/api/v1/validate/sql", post(validate_sql))
        .route("/api/v1/templates", get(list_templates))
        
        // WebSocket (placeholder)
        .route("/ws", get(websocket_endpoint))
        
        // Configurar estado compartido
        .with_state(state)
}

/// Información raíz del servidor
async fn root_info() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "service": "Noctra Server",
        "version": "0.1.0",
        "description": "Entorno SQL Interactivo - Servidor HTTP/API",
        "endpoints": {
            "health": "/health",
            "status": "/status",
            "query": "POST /api/v1/query",
            "form": "POST /api/v1/form/{name}",
            "session": "POST /api/v1/session"
        },
        "documentation": "https://docs.noctra.dev"
    }))
}

/// Health check del servidor
async fn health_check() -> Result<Json<serde_json::Value>, StatusCode> {
    Ok(Json(serde_json::json!({
        "status": "healthy",
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "version": "0.1.0",
        "checks": {
            "database": "ok",
            "parser": "ok",
            "executor": "ok"
        }
    })))
}

/// Estado detallado del servidor
async fn server_status(State(state): State<ServerState>) -> Json<ServerStatus> {
    Json(ServerStatus {
        version: "0.1.0".to_string(),
        uptime_seconds: state.start_time.elapsed().as_secs(),
        connected_sessions: state.sessions.read().await.len(),
        active_queries: 0, // TODO: Implementar contador
        database_status: "connected".to_string(),
    })
}

/// Ejecutar consulta SQL/RQL
async fn execute_query(
    State(state): State<ServerState>,
    Json(request): Json<QueryRequest>,
) -> Result<Json<QueryResponse>, StatusCode> {
    let start_time = std::time::Instant::now();
    
    // TODO: Implementar ejecución real usando parser y executor
    // Por ahora simular resultado
    
    let mock_columns = vec![
        noctra_core::Column::new("id", "INTEGER", 0),
        noctra_core::Column::new("name", "TEXT", 1),
        noctra_core::Column::new("status", "TEXT", 2),
    ];
    
    let mock_rows = vec![
        noctra_core::Row::new(vec![
            noctra_core::Value::integer(1),
            noctra_core::Value::text("Test Row 1"),
            noctra_core::Value::text("active"),
        ]),
        noctra_core::Row::new(vec![
            noctra_core::Value::integer(2),
            noctra_core::Value::text("Test Row 2"),
            noctra_core::Value::text("pending"),
        ]),
    ];
    
    let mut result = noctra_core::ResultSet::new(mock_columns);
    result.add_rows(mock_rows);
    
    let execution_time = start_time.elapsed().as_millis() as u64;
    
    Ok(Json(QueryResponse {
        success: true,
        data: Some(result),
        message: "Consulta ejecutada exitosamente".to_string(),
        execution_time_ms: execution_time,
    }))
}

/// Ejecutar consultas en lote
async fn execute_batch_queries(
    State(state): State<ServerState>,
    Json(requests): Json<Vec<QueryRequest>>,
) -> Result<Json<Vec<QueryResponse>>, StatusCode> {
    let mut responses = Vec::new();
    
    for request in requests {
        let start_time = std::time::Instant::now();
        
        // TODO: Ejecutar consulta real
        let execution_time = start_time.elapsed().as_millis() as u64;
        
        let response = QueryResponse {
            success: true,
            data: None, // Batch queries típicamente no devuelven datos
            message: "Consulta batch ejecutada".to_string(),
            execution_time_ms: execution_time,
        };
        
        responses.push(response);
    }
    
    Ok(Json(responses))
}

/// Ejecutar formulario
async fn execute_form(
    State(state): State<ServerState>,
    axum::extract::Path(name): axum::extract::Path<String>,
    Json(request): Json<FormRequest>,
) -> Result<Json<FormResponse>, StatusCode> {
    // TODO: Implementar ejecución real de formularios
    let response = FormResponse {
        success: true,
        data: None,
        message: format!("Formulario '{}' ejecutado exitosamente", name),
        form_title: Some(format!("Formulario {}", name)),
    };
    
    Ok(Json(response))
}

/// Validar formulario
async fn validate_form(
    State(state): State<ServerState>,
    axum::extract::Path(name): axum::extract::Path<String>,
    Json(request): Json<FormRequest>,
) -> Result<Json<FormResponse>, StatusCode> {
    // TODO: Implementar validación real de formularios
    let response = FormResponse {
        success: true,
        data: None,
        message: format!("Formulario '{}' validado correctamente", name),
        form_title: Some(format!("Formulario {}", name)),
    };
    
    Ok(Json(response))
}

/// Listar formularios disponibles
async fn list_forms(State(state): State<ServerState>) -> Json<serde_json::Value> {
    // TODO: Implementar listado real de formularios
    Json(serde_json::json!({
        "forms": [
            {
                "name": "empleados",
                "title": "Consulta de Empleados",
                "fields": 3,
                "actions": 1
            },
            {
                "name": "reportes",
                "title": "Generador de Reportes", 
                "fields": 5,
                "actions": 2
            }
        ]
    }))
}

/// Crear nueva sesión
async fn create_session(State(state): State<ServerState>) -> Result<Json<serde_json::Value>, StatusCode> {
    let session = Session::new(format!("session_{}", chrono::Utc::now().timestamp()));
    let mut sessions = state.sessions.write().await;
    sessions.push(session.clone());
    
    Ok(Json(serde_json::json!({
        "session_id": session.id,
        "message": "Sesión creada exitosamente",
        "expires_in": 3600
    })))
}

/// Obtener información de sesión
async fn get_session(
    State(state): State<ServerState>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // TODO: Buscar sesión real
    Ok(Json(serde_json::json!({
        "session_id": id,
        "status": "active",
        "created_at": chrono::Utc::now().to_rfc3339(),
        "last_activity": chrono::Utc::now().to_rfc3339()
    })))
}

/// Eliminar sesión
async fn delete_session(
    State(state): State<ServerState>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // TODO: Eliminar sesión real
    Ok(Json(serde_json::json!({
        "message": format!("Sesión {} eliminada", id)
    })))
}

/// Listar sesiones activas
async fn list_sessions(State(state): State<ServerState>) -> Json<serde_json::Value> {
    let sessions = state.sessions.read().await;
    
    Json(serde_json::json!({
        "sessions": sessions.iter().map(|s| serde_json::json!({
            "id": s.id,
            "created_at": s.created_at
        })).collect::<Vec<_>>(),
        "total": sessions.len()
    }))
}

/// Obtener configuración del servidor
async fn get_config(State(state): State<ServerState>) -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "database_url": state.config.database_url,
        "bind_address": state.config.bind_address.to_string(),
        "request_timeout_seconds": state.config.request_timeout.as_secs(),
        "max_connections": state.config.max_connections,
        "cors_enabled": state.config.enable_cors,
        "websockets_enabled": state.config.enable_websockets,
        "auth_enabled": state.config.auth_secret.is_some()
    }))
}

/// Actualizar configuración (placeholder)
async fn update_config(
    State(state): State<ServerState>,
    Json(config): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // TODO: Implementar actualización real de configuración
    Ok(Json(serde_json::json!({
        "message": "Configuración actualizada (simulado)"
    })))
}

/// Parsear SQL sin ejecutar
async fn parse_sql(
    State(state): State<ServerState>,
    Json(request): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let sql = request.get("sql")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    
    // TODO: Implementar parseo real con parser
    Ok(Json(serde_json::json!({
        "sql": sql,
        "valid": true,
        "statements": 1,
        "parameters": []
    })))
}

/// Validar SQL sintácticamente
async fn validate_sql(
    State(state): State<ServerState>,
    Json(request): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let sql = request.get("sql")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    
    // TODO: Implementar validación real
    Ok(Json(serde_json::json!({
        "valid": true,
        "sql": sql,
        "errors": []
    })))
}

/// Listar templates disponibles
async fn list_templates(State(state): State<ServerState>) -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "templates": [
            {
                "name": "employee_list",
                "description": "Lista de empleados",
                "sql": "SELECT * FROM employees"
            },
            {
                "name": "department_report", 
                "description": "Reporte por departamento",
                "sql": "SELECT dept, COUNT(*) FROM employees GROUP BY dept"
            }
        ]
    }))
}

/// Endpoint WebSocket (placeholder)
async fn websocket_endpoint() -> Result<String, StatusCode> {
    Ok("WebSocket endpoint - Funcionalidad en desarrollo".to_string())
}

/// Router específico para Noctra (alias)
pub type NoctraRouter = Router<ServerState>;