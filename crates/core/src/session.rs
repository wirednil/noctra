//! Gestión de sesiones para Noctra

use crate::error::{NoctraError, Result};
use crate::types::{Parameters, SessionVariables, Value};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Una sesión de trabajo de Noctra
#[derive(Debug, Clone)]
pub struct Session {
    /// Variables de sesión
    variables: SessionVariables,

    /// Parámetros de la consulta actual
    parameters: Parameters,

    /// Esquema por defecto
    default_schema: String,

    /// Estado de la sesión
    state: SessionState,

    /// ID único de la sesión
    id: String,
}

impl Session {
    /// Crear nueva sesión
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
            parameters: HashMap::new(),
            default_schema: "main".to_string(),
            state: SessionState::Active,
            id: uuid::Uuid::new_v4().to_string(),
        }
    }

    /// Crear sesión con configuración específica
    pub fn with_schema<T: Into<String>>(schema: T) -> Self {
        Self {
            variables: HashMap::new(),
            parameters: HashMap::new(),
            default_schema: schema.into(),
            state: SessionState::Active,
            id: uuid::Uuid::new_v4().to_string(),
        }
    }

    /// Obtener ID de la sesión
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Obtener esquema por defecto
    pub fn default_schema(&self) -> &str {
        &self.default_schema
    }

    /// Establecer esquema por defecto
    pub fn set_default_schema<T: Into<String>>(&mut self, schema: T) {
        self.default_schema = schema.into();
    }

    /// Estado de la sesión
    pub fn state(&self) -> SessionState {
        self.state.clone()
    }

    /// Cambiar estado
    pub fn set_state(&mut self, state: SessionState) {
        self.state = state;
    }

    /// Verificar si la sesión está activa
    pub fn is_active(&self) -> bool {
        matches!(self.state, SessionState::Active)
    }

    // === GESTIÓN DE VARIABLES ===

    /// Establecer variable de sesión
    pub fn set_variable<T: Into<String>, V: Into<Value>>(&mut self, name: T, value: V) {
        self.variables.insert(name.into(), value.into());
    }

    /// Obtener variable de sesión
    pub fn get_variable(&self, name: &str) -> Option<&Value> {
        self.variables.get(name)
    }

    /// Obtener variable de sesión con tipo específico
    pub fn get_variable_as<T>(&self, name: &str) -> Result<Option<T>>
    where
        T: std::str::FromStr,
        T::Err: std::fmt::Display,
    {
        Ok(self.get_variable(name).and_then(|v| match v {
            Value::Text(text) => T::from_str(text).ok(),
            Value::Integer(int) => T::from_str(&int.to_string()).ok(),
            Value::Float(float) => T::from_str(&float.to_string()).ok(),
            Value::Boolean(bool) => T::from_str(&bool.to_string()).ok(),
            _ => None,
        }))
    }

    /// Remover variable de sesión
    pub fn remove_variable(&mut self, name: &str) -> Option<Value> {
        self.variables.remove(name)
    }

    /// Listar todas las variables
    pub fn list_variables(&self) -> &SessionVariables {
        &self.variables
    }

    // === GESTIÓN DE PARÁMETROS ===

    /// Establecer parámetro
    pub fn set_parameter<T: Into<String>, V: Into<Value>>(&mut self, name: T, value: V) {
        self.parameters.insert(name.into(), value.into());
    }

    /// Obtener parámetro
    pub fn get_parameter(&self, name: &str) -> Option<&Value> {
        self.parameters.get(name)
    }

    /// Obtener parámetro posicionado ($1, $2, etc.)
    pub fn get_positional_parameter(&self, index: usize) -> Option<&Value> {
        let key = format!("${}", index + 1);
        self.parameters.get(&key)
    }

    /// Obtener parámetro nombrado (:name)
    pub fn get_named_parameter(&self, name: &str) -> Option<&Value> {
        let key = format!(":{}", name);
        self.parameters.get(&key)
    }

    /// Establecer parámetro posicionado
    pub fn set_positional_parameter(&mut self, index: usize, value: impl Into<Value>) {
        let key = format!("${}", index + 1);
        self.parameters.insert(key, value.into());
    }

    /// Establecer parámetro nombrado
    pub fn set_named_parameter<T: Into<String>>(&mut self, name: T, value: impl Into<Value>) {
        let key = format!(":{}", name.into());
        self.parameters.insert(key, value.into());
    }

    /// Limpiar parámetros
    pub fn clear_parameters(&mut self) {
        self.parameters.clear();
    }

    /// Obtener todos los parámetros
    pub fn list_parameters(&self) -> &Parameters {
        &self.parameters
    }

    // === UTILIDADES ===

    /// Clonar sesión para operaciones seguras
    pub fn clone_for_operation(&self) -> Self {
        Session {
            variables: self.variables.clone(),
            parameters: self.parameters.clone(),
            default_schema: self.default_schema.clone(),
            state: self.state.clone(),
            id: self.id.clone(),
        }
    }

    /// Resetear sesión (mantener ID)
    pub fn reset(&mut self) {
        self.variables.clear();
        self.parameters.clear();
        self.default_schema = "main".to_string();
        self.state = SessionState::Active;
    }

    /// Obtener información de debug
    pub fn debug_info(&self) -> SessionDebugInfo {
        SessionDebugInfo {
            id: self.id.clone(),
            schema: self.default_schema.clone(),
            state: self.state.clone(),
            variables_count: self.variables.len(),
            parameters_count: self.parameters.len(),
        }
    }
}

/// Estados posibles de una sesión
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SessionState {
    /// Sesión activa
    Active,

    /// Sesión en espera
    Waiting,

    /// Sesión finalizada
    Finished,

    /// Sesión con error
    Error(String),

    /// Sesión suspendida
    Suspended,
}

impl Default for SessionState {
    fn default() -> Self {
        Self::Active
    }
}

/// Información de debug de una sesión
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionDebugInfo {
    pub id: String,
    pub schema: String,
    pub state: SessionState,
    pub variables_count: usize,
    pub parameters_count: usize,
}

/// Gestor de sesiones múltiples
#[derive(Debug)]
pub struct SessionManager {
    /// Sesiones activas
    sessions: HashMap<String, Session>,

    /// Configuración global
    config: SessionConfig,
}

impl SessionManager {
    /// Crear nuevo gestor de sesiones
    pub fn new(config: SessionConfig) -> Self {
        Self {
            sessions: HashMap::new(),
            config,
        }
    }

    /// Crear nueva sesión
    pub fn create_session(&mut self) -> Result<Session> {
        let session = Session::new();
        let id = session.id().to_string();

        if self.sessions.len() >= self.config.max_sessions {
            return Err(NoctraError::Configuration(format!(
                "Máximo de sesiones alcanzado: {}",
                self.config.max_sessions
            )));
        }

        self.sessions.insert(id, session.clone());
        Ok(session)
    }

    /// Obtener sesión por ID
    pub fn get_session(&self, id: &str) -> Option<&Session> {
        self.sessions.get(id)
    }

    /// Obtener sesión mutable por ID
    pub fn get_session_mut(&mut self, id: &str) -> Option<&mut Session> {
        self.sessions.get_mut(id)
    }

    /// Remover sesión
    pub fn remove_session(&mut self, id: &str) -> Option<Session> {
        self.sessions.remove(id)
    }

    /// Limpiar sesiones finalizadas
    pub fn cleanup_finished_sessions(&mut self) {
        let finished: Vec<String> = self
            .sessions
            .iter()
            .filter(|(_, session)| matches!(session.state(), SessionState::Finished))
            .map(|(id, _)| id.clone())
            .collect();

        for id in finished {
            self.sessions.remove(&id);
        }
    }

    /// Número de sesiones activas
    pub fn active_sessions_count(&self) -> usize {
        self.sessions.len()
    }

    /// Configuración del gestor
    pub fn config(&self) -> &SessionConfig {
        &self.config
    }
}

/// Configuración del gestor de sesiones
#[derive(Debug, Clone)]
pub struct SessionConfig {
    /// Máximo número de sesiones concurrentes
    pub max_sessions: usize,

    /// Timeout de sesión en segundos
    pub session_timeout: u64,

    /// Auto-cleanup de sesiones finalizadas
    pub auto_cleanup: bool,
}

impl Default for SessionConfig {
    fn default() -> Self {
        Self {
            max_sessions: 100,
            session_timeout: 3600, // 1 hora
            auto_cleanup: true,
        }
    }
}

impl Default for Session {
    fn default() -> Self {
        Self::new()
    }
}
