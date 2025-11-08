//! Form Graph - Navegación jerárquica de formularios
//!
//! Sistema para describir y navegar árboles de formularios, menús
//! y consultas de manera declarativa.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use thiserror::Error;

use crate::forms::Form;
use crate::loader::{load_form_from_path, LoadError};

/// Error de FormGraph
#[derive(Error, Debug)]
pub enum GraphError {
    /// Nodo no encontrado
    #[error("Nodo '{0}' no encontrado en el grafo")]
    NodeNotFound(String),

    /// Ciclo detectado
    #[error("Ciclo detectado en navegación: {0}")]
    CycleDetected(String),

    /// Error de carga de formulario
    #[error("Error cargando formulario: {0}")]
    LoadError(#[from] LoadError),

    /// Path inválido
    #[error("Path inválido: {0}")]
    InvalidPath(String),

    /// Configuración inválida
    #[error("Configuración inválida: {0}")]
    InvalidConfig(String),
}

/// Resultado de operaciones con FormGraph
pub type GraphResult<T> = Result<T, GraphError>;

/// Tipo de nodo en el grafo
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum NodeType {
    /// Nodo que carga un formulario
    Form,

    /// Nodo que representa un menú de navegación
    Menu,

    /// Nodo que representa una consulta SQL directa
    Query,

    /// Nodo que representa un enlace externo
    Link,
}

/// Definición de un nodo en el grafo
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeDefinition {
    /// ID único del nodo
    pub id: String,

    /// Título visible del nodo
    pub title: String,

    /// Tipo de nodo
    #[serde(rename = "type")]
    pub node_type: NodeType,

    /// Path al recurso (formulario, archivo, etc.)
    pub path: Option<String>,

    /// Descripción opcional
    pub description: Option<String>,

    /// Nodos hijos
    #[serde(default)]
    pub children: Vec<NodeDefinition>,

    /// Metadata adicional
    #[serde(default)]
    pub metadata: HashMap<String, String>,

    /// Acción personalizada
    pub action: Option<String>,

    /// Icono o símbolo para mostrar en menús
    pub icon: Option<String>,
}

/// Grafo de formularios completo
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormGraph {
    /// Versión del esquema del grafo
    pub version: String,

    /// Título de la aplicación
    pub title: String,

    /// Directorio base para paths relativos
    pub base_path: Option<String>,

    /// Nodo raíz del grafo
    pub root: NodeDefinition,

    /// Configuración global
    #[serde(default)]
    pub config: GraphConfig,
}

/// Configuración global del grafo
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GraphConfig {
    /// Base de datos por defecto
    pub default_database: Option<String>,

    /// Tema visual por defecto
    pub default_theme: Option<String>,

    /// Tamaño de página por defecto
    pub default_page_size: Option<usize>,

    /// Habilitar navegación con breadcrumbs
    #[serde(default = "default_true")]
    pub show_breadcrumbs: bool,

    /// Habilitar historial de navegación
    #[serde(default = "default_true")]
    pub enable_history: bool,
}

fn default_true() -> bool {
    true
}

impl FormGraph {
    /// Cargar grafo desde archivo TOML
    pub fn load_from_file(path: &Path) -> GraphResult<Self> {
        if !path.exists() {
            return Err(GraphError::InvalidPath(
                path.to_string_lossy().to_string(),
            ));
        }

        let content = std::fs::read_to_string(path).map_err(|e| {
            GraphError::InvalidPath(format!("{}: {}", path.display(), e))
        })?;

        let graph: FormGraph = toml::from_str(&content).map_err(|e| {
            GraphError::InvalidConfig(format!("Error parseando TOML: {}", e))
        })?;

        // Validar el grafo
        graph.validate()?;

        Ok(graph)
    }

    /// Validar estructura del grafo
    pub fn validate(&self) -> GraphResult<()> {
        // Validar que no haya ciclos
        self.check_cycles(&self.root, &mut Vec::new())?;

        // Validar que todos los paths existan si están especificados
        if self.config.enable_history {
            self.validate_paths(&self.root)?;
        }

        Ok(())
    }

    /// Verificar ciclos en el grafo
    fn check_cycles(
        &self,
        node: &NodeDefinition,
        visited: &mut Vec<String>,
    ) -> GraphResult<()> {
        if visited.contains(&node.id) {
            return Err(GraphError::CycleDetected(format!(
                "Ciclo en nodo '{}': {:?}",
                node.id, visited
            )));
        }

        visited.push(node.id.clone());

        for child in &node.children {
            self.check_cycles(child, visited)?;
        }

        visited.pop();

        Ok(())
    }

    /// Validar que los paths de formularios existan
    fn validate_paths(&self, node: &NodeDefinition) -> GraphResult<()> {
        if matches!(node.node_type, NodeType::Form) {
            if let Some(path) = &node.path {
                let full_path = self.resolve_path(path);
                if !full_path.exists() {
                    return Err(GraphError::InvalidPath(format!(
                        "Formulario no encontrado: {}",
                        full_path.display()
                    )));
                }
            }
        }

        for child in &node.children {
            self.validate_paths(child)?;
        }

        Ok(())
    }

    /// Resolver path relativo basado en base_path
    fn resolve_path(&self, path: &str) -> PathBuf {
        if let Some(base) = &self.base_path {
            PathBuf::from(base).join(path)
        } else {
            PathBuf::from(path)
        }
    }

    /// Buscar nodo por ID
    pub fn find_node(&self, node_id: &str) -> GraphResult<&NodeDefinition> {
        self.find_node_recursive(&self.root, node_id)
            .ok_or_else(|| GraphError::NodeNotFound(node_id.to_string()))
    }

    /// Buscar nodo recursivamente
    fn find_node_recursive<'a>(
        &'a self,
        node: &'a NodeDefinition,
        node_id: &str,
    ) -> Option<&'a NodeDefinition> {
        if node.id == node_id {
            return Some(node);
        }

        for child in &node.children {
            if let Some(found) = self.find_node_recursive(child, node_id) {
                return Some(found);
            }
        }

        None
    }

    /// Cargar formulario desde un nodo
    pub fn load_form_from_node(&self, node_id: &str) -> GraphResult<Form> {
        let node = self.find_node(node_id)?;

        if !matches!(node.node_type, NodeType::Form) {
            return Err(GraphError::InvalidConfig(format!(
                "Nodo '{}' no es de tipo Form",
                node_id
            )));
        }

        let path = node
            .path
            .as_ref()
            .ok_or_else(|| GraphError::InvalidConfig(format!(
                "Nodo '{}' no tiene path definido",
                node_id
            )))?;

        let full_path = self.resolve_path(path);
        load_form_from_path(&full_path).map_err(GraphError::LoadError)
    }

    /// Obtener hijos de un nodo
    pub fn get_children(&self, node_id: &str) -> GraphResult<&[NodeDefinition]> {
        let node = self.find_node(node_id)?;
        Ok(&node.children)
    }

    /// Obtener path completo de navegación (breadcrumb)
    pub fn get_breadcrumb(&self, node_id: &str) -> GraphResult<Vec<String>> {
        let mut path = Vec::new();
        self.build_breadcrumb(&self.root, node_id, &mut path)?;
        Ok(path)
    }

    /// Construir breadcrumb recursivamente
    fn build_breadcrumb(
        &self,
        node: &NodeDefinition,
        target_id: &str,
        path: &mut Vec<String>,
    ) -> Result<bool, GraphError> {
        path.push(node.id.clone());

        if node.id == target_id {
            return Ok(true);
        }

        for child in &node.children {
            if self.build_breadcrumb(child, target_id, path)? {
                return Ok(true);
            }
        }

        path.pop();
        Ok(false)
    }
}

/// Navegador de grafo con estado
pub struct GraphNavigator {
    /// Grafo subyacente
    graph: FormGraph,

    /// Nodo actual
    current_node: String,

    /// Historial de navegación
    history: Vec<String>,

    /// Índice en el historial
    history_index: usize,
}

impl GraphNavigator {
    /// Crear nuevo navegador
    pub fn new(graph: FormGraph) -> Self {
        let root_id = graph.root.id.clone();

        Self {
            graph,
            current_node: root_id.clone(),
            history: vec![root_id],
            history_index: 0,
        }
    }

    /// Obtener nodo actual
    pub fn current_node(&self) -> GraphResult<&NodeDefinition> {
        self.graph.find_node(&self.current_node)
    }

    /// Navegar a un nodo
    pub fn navigate_to(&mut self, node_id: &str) -> GraphResult<()> {
        // Verificar que el nodo exista
        self.graph.find_node(node_id)?;

        // Agregar al historial
        self.history.truncate(self.history_index + 1);
        self.history.push(node_id.to_string());
        self.history_index = self.history.len() - 1;

        // Actualizar nodo actual
        self.current_node = node_id.to_string();

        Ok(())
    }

    /// Navegar hacia atrás en el historial
    pub fn go_back(&mut self) -> GraphResult<bool> {
        if self.history_index > 0 {
            self.history_index -= 1;
            self.current_node = self.history[self.history_index].clone();
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Navegar hacia adelante en el historial
    pub fn go_forward(&mut self) -> GraphResult<bool> {
        if self.history_index < self.history.len() - 1 {
            self.history_index += 1;
            self.current_node = self.history[self.history_index].clone();
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Navegar al nodo raíz
    pub fn go_home(&mut self) -> GraphResult<()> {
        let root_id = self.graph.root.id.clone();
        self.navigate_to(&root_id)
    }

    /// Obtener breadcrumb actual
    pub fn get_breadcrumb(&self) -> GraphResult<Vec<String>> {
        self.graph.get_breadcrumb(&self.current_node)
    }

    /// Obtener hijos del nodo actual
    pub fn get_current_children(&self) -> GraphResult<&[NodeDefinition]> {
        self.graph.get_children(&self.current_node)
    }

    /// Cargar formulario del nodo actual
    pub fn load_current_form(&self) -> GraphResult<Form> {
        self.graph.load_form_from_node(&self.current_node)
    }

    /// Obtener referencia al grafo
    pub fn graph(&self) -> &FormGraph {
        &self.graph
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node_definition() {
        let node = NodeDefinition {
            id: "test".to_string(),
            title: "Test Node".to_string(),
            node_type: NodeType::Menu,
            path: None,
            description: Some("Test description".to_string()),
            children: vec![],
            metadata: HashMap::new(),
            action: None,
            icon: Some("📋".to_string()),
        };

        assert_eq!(node.id, "test");
        assert_eq!(node.title, "Test Node");
    }

    #[test]
    fn test_graph_cycle_detection() {
        // Este test requeriría un grafo con ciclo para verificar la detección
        // Por ahora solo verificamos la estructura básica
        let graph = FormGraph {
            version: "1.0".to_string(),
            title: "Test App".to_string(),
            base_path: None,
            root: NodeDefinition {
                id: "root".to_string(),
                title: "Root".to_string(),
                node_type: NodeType::Menu,
                path: None,
                description: None,
                children: vec![],
                metadata: HashMap::new(),
                action: None,
                icon: None,
            },
            config: GraphConfig::default(),
        };

        assert!(graph.validate().is_ok());
    }

    #[test]
    fn test_navigator_creation() {
        let graph = FormGraph {
            version: "1.0".to_string(),
            title: "Test App".to_string(),
            base_path: None,
            root: NodeDefinition {
                id: "root".to_string(),
                title: "Root".to_string(),
                node_type: NodeType::Menu,
                path: None,
                description: None,
                children: vec![],
                metadata: HashMap::new(),
                action: None,
                icon: None,
            },
            config: GraphConfig::default(),
        };

        let navigator = GraphNavigator::new(graph);
        assert_eq!(navigator.current_node, "root");
        assert_eq!(navigator.history.len(), 1);
    }
}
