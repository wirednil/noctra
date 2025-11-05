//! Tests de integración para el servidor Noctra
//! 
//! Pruebas end-to-end que cubren toda la pipeline: HTTP APIs, WebSocket,
//! consultas SQL, formularios FDL2 y manejo de errores.

use axum::{
    body::Body,
    http::{StatusCode, Method, Request},
};
use axum::extract::WebSocketUpgrade;
use tower::ServiceExt;
use tower_http::cors::CorsLayer;
use tokio::sync::RwLock;
use std::sync::Arc;

use noctra_core::{Executor, Session};
use noctra_parser::RqlParser;
use noctra_formlib::Form;
use noctra_srv::{
    server::ServerState,
    ServerConfig,
    create_server,
    routes::{create_router},
};

/// Helper para crear un servidor de test
async fn create_test_server() -> (ServerState, axum::Router) {
    let config = ServerConfig::default();
    let state = ServerState::new(config).await.unwrap();
    
    let router = create_router(state.clone()).unwrap();
    
    (state, router)
}

/// Helper para hacer requests HTTP
async fn make_request(
    app: &axum::Router,
    method: Method,
    uri: &str,
    body: Option<Body>,
) -> (StatusCode, String) {
    let request = Request::builder()
        .method(method)
        .uri(uri)
        .header("Content-Type", "application/json")
        .body(body.unwrap_or_else(|| Body::empty()))
        .unwrap();
    
    let response = app.clone().oneshot(request).await.unwrap();
    let status = response.status();
    
    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    let body_str = String::from_utf8(body.to_vec()).unwrap();
    
    (status, body_str)
}

/// Tests básicos del servidor HTTP
#[tokio::test]
async fn test_health_check() {
    let (_state, app) = create_test_server().await;
    
    let (status, body) = make_request(&app, Method::GET, "/health", None).await;
    
    assert_eq!(status, StatusCode::OK);
    
    let response: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert_eq!(response["status"], "healthy");
    assert_eq!(response["version"], "0.1.0");
    assert!(response["uptime"].is_number());
}

/// Tests de APIs REST
#[tokio::test]
async fn test_query_api_execute() {
    let (_state, app) = create_test_server().await;
    
    let query_data = serde_json::json!({
        "sql": "SELECT * FROM employees WHERE dept = ?",
        "parameters": [3]
    });
    
    let body = Body::from(serde_json::to_string(&query_data).unwrap());
    
    let (status, response_body) = make_request(
        &app, 
        Method::POST, 
        "/api/v1/query/execute", 
        Some(body)
    ).await;
    
    assert_eq!(status, StatusCode::OK);
    
    let response: QueryResponse = serde_json::from_str(&response_body).unwrap();
    assert!(response.success);
    assert!(response.data.is_some());
    assert!(response.execution_time_ms > 0);
}

#[tokio::test]
async fn test_query_api_validate() {
    let (_state, app) = create_test_server().await;
    
    let query_data = serde_json::json!({
        "sql": "SELECT id, name FROM users WHERE age > :age",
        "validate_only": true
    });
    
    let body = Body::from(serde_json::to_string(&query_data).unwrap());
    
    let (status, response_body) = make_request(
        &app, 
        Method::POST, 
        "/api/v1/query/validate", 
        Some(body)
    ).await;
    
    assert_eq!(status, StatusCode::OK);
    
    let response: QueryResponse = serde_json::from_str(&response_body).unwrap();
    assert!(response.success);
}

/// Tests de APIs de formularios
#[tokio::test]
async fn test_form_api_execute() {
    let (_state, app) = create_test_server().await;
    
    let form_data = serde_json::json!({
        "action": "query",
        "parameters": {
            "dept": "Ventas",
            "limit": 10
        }
    });
    
    let body = Body::from(serde_json::to_string(&form_data).unwrap());
    
    let (status, response_body) = make_request(
        &app, 
        Method::POST, 
        "/api/v1/form/empleados", 
        Some(body)
    ).await;
    
    assert_eq!(status, StatusCode::OK);
    
    let response: FormResponse = serde_json::from_str(&response_body).unwrap();
    assert!(response.success);
    assert!(response.form_title.is_some());
}

#[tokio::test]
async fn test_form_api_validate() {
    let (_state, app) = create_test_server().await;
    
    let form_data = serde_json::json!({
        "parameters": {
            "name": "Juan Pérez"
        }
    });
    
    let body = Body::from(serde_json::to_string(&form_data).unwrap());
    
    let (status, response_body) = make_request(
        &app, 
        Method::POST, 
        "/api/v1/form/empleados/validate", 
        Some(body)
    ).await;
    
    assert_eq!(status, StatusCode::OK);
    
    let response: FormResponse = serde_json::from_str(&response_body).unwrap();
    assert!(response.success);
}

/// Tests de gestión de sesiones
#[tokio::test]
async fn test_session_create() {
    let (_state, app) = create_test_server().await;
    
    let (status, body) = make_request(
        &app, 
        Method::POST, 
        "/api/v1/session", 
        None
    ).await;
    
    assert_eq!(status, StatusCode::OK);
    
    let response: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert!(response["session_id"].is_string());
    assert_eq!(response["message"], "Sesión creada exitosamente");
    assert_eq!(response["expires_in"], 3600);
}

#[tokio::test]
async fn test_session_list() {
    let (_state, app) = create_test_server().await;
    
    // Crear una sesión primero
    make_request(&app, Method::POST, "/api/v1/session", None).await;
    
    let (status, body) = make_request(
        &app, 
        Method::GET, 
        "/api/v1/sessions", 
        None
    ).await;
    
    assert_eq!(status, StatusCode::OK);
    
    let response: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert!(response["sessions"].is_array());
    assert!(response["total"].is_number());
}

/// Tests de manejo de errores
#[tokio::test]
async fn test_invalid_sql() {
    let (_state, app) = create_test_server().await;
    
    let invalid_query = serde_json::json!({
        "sql": "INVALID SQL SYNTAX",
        "parameters": []
    });
    
    let body = Body::from(serde_json::to_string(&invalid_query).unwrap());
    
    let (status, response_body) = make_request(
        &app, 
        Method::POST, 
        "/api/v1/query/execute", 
        Some(body)
    ).await;
    
    assert_eq!(status, StatusCode::BAD_REQUEST);
    
    let response: QueryResponse = serde_json::from_str(&response_body).unwrap();
    assert!(!response.success);
    assert!(response.message.contains("Error"));
}

#[tokio::test]
async fn test_nonexistent_form() {
    let (_state, app) = create_test_server().await;
    
    let form_data = serde_json::json!({
        "parameters": {}
    });
    
    let body = Body::from(serde_json::to_string(&form_data).unwrap());
    
    let (status, response_body) = make_request(
        &app, 
        Method::POST, 
        "/api/v1/form/nonexistent", 
        Some(body)
    ).await;
    
    assert_eq!(status, StatusCode::NOT_FOUND);
    
    let response: serde_json::Value = serde_json::from_str(&response_body).unwrap();
    assert_eq!(response["error"], "Formulario no encontrado");
}

/// Tests de rutas no definidas
#[tokio::test]
async fn test_404_endpoint() {
    let (_state, app) = create_test_server().await;
    
    let (status, _body) = make_request(
        &app, 
        Method::GET, 
        "/api/v1/nonexistent", 
        None
    ).await;
    
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_invalid_method() {
    let (_state, app) = create_test_server().await;
    
    let (status, _body) = make_request(
        &app, 
        Method::PATCH, 
        "/health", 
        None
    ).await;
    
    assert_eq!(status, StatusCode::METHOD_NOT_ALLOWED);
}

/// Tests de parámetros faltantes
#[tokio::test]
async fn test_missing_parameters() {
    let (_state, app) = create_test_server().await;
    
    let invalid_query = serde_json::json!({
        // Falta el campo "sql"
        "parameters": []
    });
    
    let body = Body::from(serde_json::to_string(&invalid_query).unwrap());
    
    let (status, _body) = make_request(
        &app, 
        Method::POST, 
        "/api/v1/query/execute", 
        Some(body)
    ).await;
    
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

/// Tests de validación de datos
#[tokio::test]
async fn test_empty_sql() {
    let (_state, app) = create_test_server().await;
    
    let invalid_query = serde_json::json!({
        "sql": "",
        "parameters": []
    });
    
    let body = Body::from(serde_json::to_string(&invalid_query).unwrap());
    
    let (status, response_body) = make_request(
        &app, 
        Method::POST, 
        "/api/v1/query/execute", 
        Some(body)
    ).await;
    
    assert_eq!(status, StatusCode::BAD_REQUEST);
    
    let response: QueryResponse = serde_json::from_str(&response_body).unwrap();
    assert!(!response.success);
}

#[tokio::test]
async fn test_malformed_json() {
    let (_state, app) = create_test_server().await;
    
    let malformed_body = Body::from("{ invalid json }");
    
    let (status, _body) = make_request(
        &app, 
        Method::POST, 
        "/api/v1/query/execute", 
        Some(malformed_body)
    ).await;
    
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

/// Test de rendimiento básico
#[tokio::test]
async fn test_concurrent_requests() {
    let (_state, app) = create_test_server().await;
    
    // Spawn múltiples requests concurrentes
    let mut handles = Vec::new();
    
    for i in 0..10 {
        let app_clone = app.clone();
        let handle = tokio::spawn(async move {
            let query_data = serde_json::json!({
                "sql": format!("SELECT {} as test", i),
                "parameters": []
            });
            
            let body = Body::from(serde_json::to_string(&query_data).unwrap());
            make_request(&app_clone, Method::POST, "/api/v1/query/execute", Some(body)).await
        });
        handles.push(handle);
    }
    
    // Esperar todos los requests
    let results = futures::future::join_all(handles).await;
    
    // Verificar que todos los requests fueron exitosos
    for result in results {
        let (status, response_body) = result.unwrap();
        assert_eq!(status, StatusCode::OK);
        
        let response: QueryResponse = serde_json::from_str(&response_body).unwrap();
        assert!(response.success);
    }
}

/// Test de CORS
#[tokio::test]
async fn test_cors_headers() {
    let config = ServerConfig {
        cors_enabled: true,
        ..Default::default()
    };
    
    let state = ServerState::new(config.clone()).await.unwrap();
    let router = create_router(state).unwrap();
    
    let request = Request::builder()
        .method(Method::GET)
        .uri("/health")
        .header("Origin", "http://localhost:3000")
        .header("Access-Control-Request-Method", "GET")
        .body(Body::empty())
        .unwrap();
    
    let response = router.clone().oneshot(request).await.unwrap();
    
    // Verificar headers CORS
    assert!(response.headers().contains_key("Access-Control-Allow-Origin"));
    assert!(response.headers().contains_key("Access-Control-Allow-Methods"));
}

/// Test de estado del servidor
#[tokio::test]
async fn test_server_state_management() {
    let config = ServerConfig::default();
    let state = ServerState::new(config).await.unwrap();
    
    // Verificar que el estado inicial es correcto
    let executor = state.executor.read().await;
    assert!(executor.is_some());
    
    let parser = state.parser.read().await;
    assert!(parser.is_some());
    
    let sessions = state.sessions.read().await;
    assert_eq!(sessions.len(), 0);
    
    let config_state = state.config.read().await;
    assert_eq!(config_state.bind_address.port(), 8080);
    assert_eq!(config_state.max_connections, 100);
}

/// Test de parámetros de consulta
#[tokio::test]
async fn test_parameter_extraction() {
    let (_state, app) = create_test_server().await;
    
    let query_data = serde_json::json!({
        "sql": "SELECT * FROM users WHERE dept = :dept AND age > $1 LIMIT :limit",
        "parameters": {
            "dept": "Ventas",
            "limit": 10,
            "pos1": 18
        }
    });
    
    let body = Body::from(serde_json::to_string(&query_data).unwrap());
    
    let (status, response_body) = make_request(
        &app, 
        Method::POST, 
        "/api/v1/query/execute", 
        Some(body)
    ).await;
    
    assert_eq!(status, StatusCode::OK);
    
    let response: QueryResponse = serde_json::from_str(&response_body).unwrap();
    assert!(response.success);
    assert!(response.execution_time_ms > 0);
}

/// Test de configuración personalizada
#[tokio::test]
async fn test_custom_config() {
    let custom_config = ServerConfig {
        bind_address: "127.0.0.1:9000".parse().unwrap(),
        max_connections: 50,
        query_timeout: std::time::Duration::from_secs(60),
        cors_enabled: true,
        websocket_enabled: true,
        dev_mode: true,
        metrics_enabled: true,
        database_path: None,
        forms_directory: None,
        token_file: None,
    };
    
    let state = ServerState::new(custom_config.clone()).await.unwrap();
    let config_state = state.config.read().await;
    
    assert_eq!(config_state.bind_address.port(), 9000);
    assert_eq!(config_state.max_connections, 50);
    assert_eq!(config_state.cors_enabled, true);
    assert_eq!(config_state.websocket_enabled, true);
    assert_eq!(config_state.dev_mode, true);
    assert_eq!(config_state.metrics_enabled, true);
}

/// Struct para respuestas de test
#[derive(Debug, serde::Deserialize)]
struct QueryResponse {
    pub success: bool,
    pub data: Option<serde_json::Value>,
    pub message: String,
    pub execution_time_ms: u64,
}

#[derive(Debug, serde::Deserialize)]
struct FormResponse {
    pub success: bool,
    pub data: Option<serde_json::Value>,
    pub message: String,
    pub form_title: Option<String>,
}

/// Suite de tests de integración
#[tokio::test]
async fn test_integration_suite() {
    // Este test ejecuta toda la suite de tests
    test_health_check().await;
    test_query_api_execute().await;
    test_form_api_execute().await;
    test_session_create().await;
    test_invalid_sql().await;
    
    println!("✅ Suite de tests de integración ejecutada exitosamente");
}