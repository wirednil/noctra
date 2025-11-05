//! Soporte WebSocket para el servidor Noctra
//! 
//! Permite streaming de consultas y actualizaciones en tiempo real.

use axum::{
    extract::{
        State,
        WebSocketUpgrade,
        ConnectInfo,
        Host,
    },
    response::IntoResponse,
};
use axum::extract::ws::{Message, WebSocket};
use tokio::sync::broadcast;
use serde::{Deserialize, Serialize};

use crate::server::ServerState;
use crate::types::{QueryRequest, QueryResponse, WsMessage};

/// Cliente WebSocket conectado
#[derive(Debug, Clone)]
pub struct WsClient {
    pub id: String,
    pub host: String,
    pub connected_at: chrono::DateTime<chrono::Utc>,
    pub sender: broadcast::Sender<WsMessage>,
}

/// Manager para clientes WebSocket conectados
#[derive(Debug, Clone)]
pub struct WsManager {
    clients: Arc<tokio::sync::RwLock<Vec<WsClient>>>,
    state: ServerState,
}

impl WsManager {
    pub fn new(state: ServerState) -> Self {
        Self {
            clients: Arc::new(tokio::sync::RwLock::new(Vec::new())),
            state,
        }
    }
    
    /// Agregar nuevo cliente
    pub async fn add_client(&self, client: WsClient) {
        let mut clients = self.clients.write().await;
        clients.push(client);
    }
    
    /// Remover cliente por ID
    pub async fn remove_client(&self, client_id: &str) {
        let mut clients = self.clients.write().await;
        clients.retain(|c| c.id != client_id);
    }
    
    /// Broadcast mensaje a todos los clientes
    pub async fn broadcast(&self, message: WsMessage) {
        let clients = self.clients.read().await;
        
        for client in clients.iter() {
            if let Err(_) = client.sender.send(message.clone()) {
                // Cliente desconectado, será removido en cleanup
                continue;
            }
        }
    }
    
    /// Obtener estadísticas de clientes
    pub async fn get_stats(&self) -> serde_json::Value {
        let clients = self.clients.read().await;
        
        serde_json::json!({
            "connected_clients": clients.len(),
            "clients": clients.iter().map(|c| serde_json::json!({
                "id": c.id,
                "host": c.host,
                "connected_at": c.connected_at,
                "active": c.sender.receiver_count() > 0
            })).collect::<Vec<_>>()
        })
    }
    
    /// Cleanup de clientes desconectados
    pub async fn cleanup(&self) {
        let mut clients = self.clients.write().await;
        clients.retain(|c| c.sender.receiver_count() > 0);
    }
}

/// Handler para conexión WebSocket principal
pub struct WsHandler {
    manager: WsManager,
}

impl WsHandler {
    pub fn new(manager: WsManager) -> Self {
        Self { manager }
    }
    
    /// Endpoint WebSocket principal
    pub async fn handle_websocket(
        WebSocketUpgrade { 
            protocol, 
            state, 
            response: resp 
        }: WebSocketUpgrade,
        ConnectInfo(addr): ConnectInfo<axum::extract::connect_info::Client>,
        Host(host): Host,
    ) -> impl IntoResponse {
        resp.on_upgrade(move |socket| self.handle_socket(socket, addr, host))
    }
    
    /// Manejar socket WebSocket individual
    async fn handle_socket(
        &self,
        mut socket: WebSocket,
        addr: std::net::SocketAddr,
        host: String,
    ) {
        let client_id = format!("ws_{}_{}", addr, chrono::Utc::now().timestamp());
        let (tx, rx) = broadcast::channel(100);
        
        // Crear cliente
        let client = WsClient {
            id: client_id.clone(),
            host,
            connected_at: chrono::Utc::now(),
            sender: tx,
        };
        
        // Registrar cliente
        self.manager.add_client(client).await;
        
        // Enviar mensaje de bienvenida
        if let Err(_) = socket.send(Message::Text(
            serde_json::json!({
                "type": "welcome",
                "client_id": client_id,
                "timestamp": chrono::Utc::now().to_rfc3339(),
                "message": "Conexión WebSocket establecida con Noctra Server"
            }).to_string()
        )).await {
            self.manager.remove_client(&client_id).await;
            return;
        }
        
        // Spawn tarea para recibir mensajes
        let manager_clone = self.manager.clone();
        let client_id_clone = client_id.clone();
        
        tokio::spawn(async move {
            while let Some(msg) = socket.recv().await {
                match msg {
                    Ok(Message::Text(text)) => {
                        if let Err(e) = Self::handle_client_message(
                            &manager_clone,
                            &client_id_clone,
                            &text,
                        ).await {
                            // Enviar error al cliente
                            let _ = socket.send(Message::Text(
                                serde_json::json!({
                                    "type": "error",
                                    "error": e.to_string(),
                                    "timestamp": chrono::Utc::now().to_rfc3339()
                                }).to_string()
                            )).await;
                        }
                    }
                    Ok(Message::Binary(data)) => {
                        // Manejar datos binarios si es necesario
                        let _ = socket.send(Message::Text(
                            format!("Mensaje binario recibido: {} bytes", data.len())
                        )).await;
                    }
                    Ok(Message::Close(_)) => break,
                    Ok(Message::Ping(_)) => {
                        let _ = socket.send(Message::Pong(())).await;
                    }
                    Ok(Message::Pong(_)) => {}
                    Err(_) => break,
                }
            }
            
            // Cliente desconectado
            manager_clone.remove_client(&client_id_clone).await;
        });
        
        // Broadcast de nueva conexión
        self.manager.broadcast(WsMessage {
            message_type: "connection".to_string(),
            data: serde_json::json!({
                "event": "client_connected",
                "client_id": client_id,
                "host": host,
                "address": addr.to_string(),
                "timestamp": chrono::Utc::now().to_rfc3339()
            }),
            timestamp: chrono::Utc::now(),
        }).await;
        
        // Spawn tarea para enviar mensajes broadcast
        let mut rx = rx.subscribe();
        let client_id_clone = client_id.clone();
        let mut socket_for_broadcast = socket.split();
        
        tokio::spawn(async move {
            while let Ok(message) = rx.recv().await {
                // No re-broadcast del mensaje a sí mismo
                if let Some(client_data) = message.data.get("client_id") {
                    if client_data == &client_id_clone {
                        continue;
                    }
                }
                
                if let Err(_) = socket_for_broadcast
                    .send(Message::Text(serde_json::to_string(&message).unwrap()))
                    .await {
                    break;
                }
            }
        });
    }
    
    /// Manejar mensaje del cliente
    async fn handle_client_message(
        manager: &WsManager,
        client_id: &str,
        text: &str,
    ) -> Result<(), String> {
        // Parsear mensaje
        let message: serde_json::Value = serde_json::from_str(text)
            .map_err(|e| format!("Error parseando mensaje JSON: {}", e))?;
        
        // Determinar tipo de mensaje
        match message.get("type")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown") {
            
            "ping" => {
                let response = WsMessage {
                    message_type: "pong".to_string(),
                    data: serde_json::json!({
                        "timestamp": chrono::Utc::now().to_rfc3339()
                    }),
                    timestamp: chrono::Utc::now(),
                };
                manager.broadcast(response).await;
            }
            
            "query" => {
                // Manejar consulta en tiempo real
                let query = message.get("query")
                    .and_then(|v| v.as_str())
                    .ok_or("Query no especificada")?;
                
                // TODO: Ejecutar consulta usando el executor del servidor
                // Por ahora simular respuesta
                let response = WsMessage {
                    message_type: "query_result".to_string(),
                    data: serde_json::json!({
                        "query": query,
                        "status": "completed",
                        "rows": 5,
                        "execution_time_ms": 42
                    }),
                    timestamp: chrono::Utc::now(),
                };
                
                manager.broadcast(response).await;
            }
            
            "subscribe" => {
                // Suscribirse a eventos específicos
                let event_type = message.get("event")
                    .and_then(|v| v.as_str())
                    .unwrap_or("general");
                
                let response = WsMessage {
                    message_type: "subscription".to_string(),
                    data: serde_json::json!({
                        "event": event_type,
                        "status": "subscribed",
                        "timestamp": chrono::Utc::now().to_rfc3339()
                    }),
                    timestamp: chrono::Utc::now(),
                };
                
                manager.broadcast(response).await;
            }
            
            "stats" => {
                // Enviar estadísticas del servidor
                let stats = manager.get_stats().await;
                let response = WsMessage {
                    message_type: "stats".to_string(),
                    data: stats,
                    timestamp: chrono::Utc::now(),
                };
                
                // Solo al cliente que pidió stats
                let client_response = serde_json::json!({
                    "type": "stats_response",
                    "stats": stats,
                    "timestamp": chrono::Utc::now().to_rfc3339()
                }).to_string();
                
                // Nota: Esto requiere acceso directo al socket del cliente
                // Por simplicidad, lo broadcast a todos
                manager.broadcast(response).await;
            }
            
            _ => {
                return Err(format!("Tipo de mensaje desconocido: {}", 
                    message.get("type").unwrap_or(&serde_json::Value::String("unknown".to_string()))
                ));
            }
        }
        
        Ok(())
    }
}

/// Configuración para WebSocket
#[derive(Debug, Clone)]
pub struct WsConfig {
    pub max_clients: usize,
    pub ping_interval: std::time::Duration,
    pub message_buffer: usize,
}

impl Default for WsConfig {
    fn default() -> Self {
        Self {
            max_clients: 100,
            ping_interval: std::time::Duration::from_secs(30),
            message_buffer: 1000,
        }
    }
}

/// Extensión para agregar WebSocket a la aplicación
pub trait WsAppExt {
    fn add_websocket_routes(self, ws_handler: &WsHandler) -> Self;
}

impl WsAppExt for axum::Router {
    fn add_websocket_routes(self, ws_handler: &WsHandler) -> Self {
        self.route(
            "/ws",
            axum::routing::get(|ws: WebSocketUpgrade, state: State<ServerState>, host: Host, addr: ConnectInfo<std::net::SocketAddr>| {
                ws_handler.handle_websocket(ws, addr, host.0)
            })
        )
    }
}

/// Estado compartido para WebSocket
#[derive(Debug, Clone)]
pub struct WsState {
    pub manager: WsManager,
    pub config: WsConfig,
}

impl WsState {
    pub fn new(state: ServerState) -> Self {
        Self {
            manager: WsManager::new(state),
            config: WsConfig::default(),
        }
    }
    
    /// Inicializar limpieza periódica de clientes
    pub fn start_cleanup_task(&self) {
        let manager = self.manager.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(
                std::time::Duration::from_secs(60)
            );
            
            loop {
                interval.tick().await;
                manager.cleanup().await;
            }
        });
    }
}