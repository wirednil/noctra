//! Handlers HTTP para el servidor Noctra
//! 
//! Handlers específicos para consultas SQL, formularios y sesiones.

use axum::{
    extract::{State, Path},
    http::StatusCode,
    response::Json,
};
use std::sync::Arc;
use std::time::Instant;

use noctra_core::{Executor, Session, ResultSet, Value, Row, Column};
use noctra_parser::RqlParser;
use noctra_formlib::{load_form_from_path, FormExecutionContext};

use crate::server::ServerState;
use crate::types::{QueryRequest, QueryResponse, FormRequest, FormResponse};

/// Handler para consultas SQL/RQL
pub struct QueryHandler {
    executor: Arc<Executor>,
    parser: Arc<RqlParser>,
}

impl QueryHandler {
    pub fn new(executor: Arc<Executor>, parser: Arc<RqlParser>) -> Self {
        Self { executor, parser }
    }
    
    /// Ejecutar consulta individual
    pub async fn execute_query(&self, request: QueryRequest) -> QueryResponse {
        let start_time = Instant::now();
        
        // Parsear SQL con RQL parser
        let parse_result = self.parser.parse_rql(&request.sql).await;
        
        match parse_result {
            Ok(ast) => {
                // TODO: Ejecutar AST usando executor
                // Por ahora crear resultado mock
                let result = self.create_mock_result(&request.sql);
                
                let execution_time = start_time.elapsed().as_millis() as u64;
                
                QueryResponse {
                    success: true,
                    data: Some(result),
                    message: format!("Consulta ejecutada: {} filas", result.row_count()),
                    execution_time_ms: execution_time,
                }
            }
            Err(e) => QueryResponse {
                success: false,
                data: None,
                message: format!("Error parseando SQL: {}", e),
                execution_time_ms: start_time.elapsed().as_millis() as u64,
            }
        }
    }
    
    /// Ejecutar consultas en lote
    pub async fn execute_batch(&self, requests: Vec<QueryRequest>) -> Vec<QueryResponse> {
        let mut responses = Vec::new();
        
        for request in requests {
            let response = self.execute_query(request).await;
            responses.push(response);
        }
        
        responses
    }
    
    /// Crear resultado mock para demo
    fn create_mock_result(&self, sql: &str) -> ResultSet {
        let columns = vec![
            Column::new("id", "INTEGER", 0),
            Column::new("name", "TEXT", 1),
            Column::new("description", "TEXT", 2),
            Column::new("created_at", "TEXT", 3),
        ];
        
        let mut result = ResultSet::new(columns);
        
        // Generar datos mock basados en el tipo de consulta
        let is_select = sql.to_uppercase().contains("SELECT");
        let has_where = sql.to_uppercase().contains("WHERE");
        
        if is_select {
            let num_rows = if has_where { 2 } else { 5 };
            
            for i in 1..=num_rows {
                let row = Row::new(vec![
                    Value::integer(i as i64),
                    Value::text(format!("Record {}", i)),
                    Value::text(format!("Descripción del registro {}", i)),
                    Value::text(format!("2024-01-{:02}", i)),
                ]);
                result.add_row(row);
            }
        }
        
        result
    }
    
    /// Validar SQL sin ejecutar
    pub async fn validate_sql(&self, sql: &str) -> Result<bool, String> {
        match self.parser.parse_rql(sql).await {
            Ok(_) => Ok(true),
            Err(e) => Err(format!("Error de validación: {}", e)),
        }
    }
}

/// Handler para formularios FDL2
pub struct FormHandler {
    executor: Arc<Executor>,
    parser: Arc<RqlParser>,
}

impl FormHandler {
    pub fn new(executor: Arc<Executor>, parser: Arc<RqlParser>) -> Self {
        Self { executor, parser }
    }
    
    /// Ejecutar formulario
    pub async fn execute_form(&self, name: String, request: FormRequest) -> FormResponse {
        // TODO: Implementar carga y ejecución real de formularios
        // Por ahora simular ejecución
        
        FormResponse {
            success: true,
            data: None,
            message: format!("Formulario '{}' ejecutado exitosamente", name),
            form_title: Some(format!("Formulario {}", name)),
        }
    }
    
    /// Validar formulario
    pub async fn validate_form(&self, name: String, request: FormRequest) -> FormResponse {
        // TODO: Implementar validación real
        FormResponse {
            success: true,
            data: None,
            message: format!("Formulario '{}' validado correctamente", name),
            form_title: Some(format!("Formulario {}", name)),
        }
    }
    
    /// Cargar formulario desde archivo
    pub fn load_form(&self, form_path: &str) -> Result<noctra_formlib::Form, String> {
        let path = std::path::Path::new(form_path);
        match noctra_formlib::load_form_from_path(path) {
            Ok(form) => Ok(form),
            Err(e) => Err(format!("Error cargando formulario: {}", e)),
        }
    }
}

/// Handler para gestión de sesiones
pub struct SessionHandler {
    sessions: Arc<tokio::sync::RwLock<Vec<Session>>>,
}

impl SessionHandler {
    pub fn new(sessions: Arc<tokio::sync::RwLock<Vec<Session>>>) -> Self {
        Self { sessions }
    }
    
    /// Crear nueva sesión
    pub async fn create_session(&self) -> Result<serde_json::Value, String> {
        let session_id = format!("session_{}", chrono::Utc::now().timestamp());
        let session = Session::new(session_id.clone());
        
        let mut sessions = self.sessions.write().await;
        sessions.push(session.clone());
        
        Ok(serde_json::json!({
            "session_id": session_id,
            "message": "Sesión creada exitosamente",
            "expires_in": 3600
        }))
    }
    
    /// Obtener sesión por ID
    pub async fn get_session(&self, session_id: &str) -> Result<serde_json::Value, String> {
        let sessions = self.sessions.read().await;
        
        // Buscar sesión (implementación simplificada)
        for session in sessions.iter() {
            if session.id == session_id {
                return Ok(serde_json::json!({
                    "session_id": session.id,
                    "status": "active",
                    "created_at": session.created_at,
                    "last_activity": chrono::Utc::now().to_rfc3339()
                }));
            }
        }
        
        Err("Sesión no encontrada".to_string())
    }
    
    /// Eliminar sesión
    pub async fn delete_session(&self, session_id: &str) -> Result<serde_json::Value, String> {
        let mut sessions = self.sessions.write().await;
        
        // Remover sesión
        let original_len = sessions.len();
        sessions.retain(|s| s.id != session_id);
        
        if sessions.len() < original_len {
            Ok(serde_json::json!({
                "message": format!("Sesión {} eliminada", session_id)
            }))
        } else {
            Err("Sesión no encontrada".to_string())
        }
    }
    
    /// Listar todas las sesiones
    pub async fn list_sessions(&self) -> serde_json::Value {
        let sessions = self.sessions.read().await;
        
        serde_json::json!({
            "sessions": sessions.iter().map(|s| serde_json::json!({
                "id": s.id,
                "created_at": s.created_at,
                "status": "active"
            })).collect::<Vec<_>>(),
            "total": sessions.len()
        })
    }
}

/// Handler combinado con acceso al estado del servidor
pub struct ServerHandler {
    query_handler: QueryHandler,
    form_handler: FormHandler,
    session_handler: SessionHandler,
}

impl ServerHandler {
    pub fn new(state: ServerState) -> Self {
        Self {
            query_handler: QueryHandler::new(state.executor.clone(), state.parser.clone()),
            form_handler: FormHandler::new(state.executor.clone(), state.parser.clone()),
            session_handler: SessionHandler::new(state.sessions.clone()),
        }
    }
    
    pub fn query_handler(&self) -> &QueryHandler {
        &self.query_handler
    }
    
    pub fn form_handler(&self) -> &FormHandler {
        &self.form_handler
    }
    
    pub fn session_handler(&self) -> &SessionHandler {
        &self.session_handler
    }
}

/// Extensiones para extraer handlers del estado
pub trait ServerHandlerExt {
    fn get_handlers(&self) -> ServerHandler;
}

impl ServerHandlerExt for ServerState {
    fn get_handlers(&self) -> ServerHandler {
        ServerHandler::new(self.clone())
    }
}