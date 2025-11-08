//! Optimizaciones de performance para el servidor Noctra
//! 
//! Implementa connection pooling, caching de consultas y optimizations
//! para mejorar throughput y latencia.

use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{RwLock, Mutex};
use std::collections::{HashMap, BTreeMap};
use futures::future::BoxFuture;
use serde::{Serialize, Deserialize};

use crate::server::ServerConfig;
use crate::error::Result;

/// Cache de consultas preparadas
#[derive(Debug)]
#[derive(Clone)]
pub struct QueryCache {
    cache: Arc<RwLock<LruCache<String, CachedQuery>>>,
    max_size: usize,
    ttl: Duration,
}

#[derive(Debug, Clone)]
struct CachedQuery {
    sql_hash: String,
    sql: String,
    plan: String, // Plan de ejecución serializado
    created_at: std::time::Instant,
    access_count: u32,
}

/// Cache LRU básico
#[derive(Debug, Clone)]
struct LruCache<K, V> {
    map: BTreeMap<(std::time::Instant, K), V>,
    max_size: usize,
}

impl<K: Clone + Ord, V> LruCache<K, V> {
    fn new(max_size: usize) -> Self {
        Self {
            map: BTreeMap::new(),
            max_size,
        }
    }
    
    fn get(&self, key: &K) -> Option<&V> {
        let now = std::time::Instant::now();
        self.map.iter()
            .find_map(|((_time, k), v)| {
                if k == key {
                    Some(v)
                } else {
                    None
                }
            })
    }
    
    fn insert(&mut self, key: K, value: V) -> Option<V> {
        let now = std::time::Instant::now();
        
        // Remover si existe
        let existing = self.map.remove(&(now, key.clone()));
        
        // Insertar nueva entrada
        self.map.insert((now, key), value);
        
        // Evitar overflow de tamaño
        while self.map.len() > self.max_size {
            let first_key = self.map.keys().next().cloned();
            if let Some(k) = first_key {
                self.map.remove(&k);
            }
        }
        
        existing
    }
    
    fn remove(&mut self, key: &K) -> Option<V> {
        let now = std::time::Instant::now();
        self.map.remove(&(now, key.clone()))
    }
    
    fn len(&self) -> usize {
        self.map.len()
    }
    
    fn is_empty(&self) -> bool {
        self.map.is_empty()
    }
}

impl QueryCache {
    pub fn new(max_size: usize, ttl: Duration) -> Self {
        Self {
            cache: Arc::new(RwLock::new(LruCache::new(max_size))),
            max_size,
            ttl,
        }
    }
    
    /// Obtener consulta del cache
    pub async fn get(&self, sql: &str) -> Option<CachedQuery> {
        let cache = self.cache.read().await;
        let query = cache.get(sql)?;
        
        // Verificar TTL
        if query.created_at.elapsed() < self.ttl {
            // TODO: Incrementar access_count (requiere mutabilidad)
            Some(query.clone())
        } else {
            None
        }
    }
    
    /// Insertar consulta en cache
    pub async fn insert(&self, sql: String, plan: String) {
        let query = CachedQuery {
            sql_hash: self.hash_sql(&sql),
            sql: sql.clone(),
            plan,
            created_at: std::time::Instant::now(),
            access_count: 1,
        };
        
        let mut cache = self.cache.write().await;
        cache.insert(sql, query);
    }
    
    /// Remover consulta del cache
    pub async fn remove(&self, sql: &str) {
        let mut cache = self.cache.write().await;
        cache.remove(sql);
    }
    
    /// Limpiar entradas expiradas
    pub async fn cleanup_expired(&self) {
        let mut cache = self.cache.write().await;
        let now = std::time::Instant::now();
        
        // TODO: Implementar cleanup real de entradas expiradas
        // Por simplicidad, solo contar cache hits
    }
    
    /// Obtener estadísticas del cache
    pub async fn stats(&self) -> CacheStats {
        let cache = self.cache.read().await;
        
        CacheStats {
            size: cache.len(),
            max_size: self.max_size,
            ttl_seconds: self.ttl.as_secs(),
        }
    }
    
    /// Hash simple para SQL
    fn hash_sql(&self, sql: &str) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        sql.hash(&mut hasher);
        format!("{:x}", hasher.finish())
    }
}

/// Pool de conexiones a la base de datos
#[derive(Debug)]
pub struct ConnectionPool {
    connections: Arc<Mutex<Vec<Arc<rusqlite::Connection>>>>,
    max_size: usize,
    min_size: usize,
    current_size: Arc<RwLock<usize>>,
    waiting_queue: Arc<Mutex<Vec<tokio::sync::oneshot::Sender<Arc<rusqlite::Connection>>>>>,
}

impl ConnectionPool {
    pub fn new(max_size: usize, min_size: usize) -> Self {
        Self {
            connections: Arc::new(Mutex::new(Vec::new())),
            max_size,
            min_size,
            current_size: Arc::new(RwLock::new(0)),
            waiting_queue: Arc::new(Mutex::new(Vec::new())),
        }
    }
    
    /// Obtener conexión del pool
    pub async fn get_connection(&self, db_path: &str) -> Result<Arc<rusqlite::Connection>> {
        // Intentar reutilizar conexión existente
        {
            let mut connections = self.connections.lock().await;
            if let Some(conn) = connections.pop() {
                *self.current_size.write().await += 1;
                return Ok(conn);
            }
        }
        
        // Crear nueva conexión si no excedemos el límite
        let current = *self.current_size.read().await;
        if current < self.max_size {
            *self.current_size.write().await += 1;
            
            let connection = self.create_connection(db_path).await?;
            Ok(Arc::new(connection))
        } else {
            // TODO: Implementar cola de espera para conexiones
            // Por ahora, crear nueva conexión anyway
            self.create_connection(db_path).await.map(Arc::new)
        }
    }
    
    /// Devolver conexión al pool
    pub async fn return_connection(&self, connection: Arc<rusqlite::Connection>) {
        let mut connections = self.connections.lock().await;
        
        if connections.len() < self.min_size {
            // Mantener conexión en el pool
            connections.push(connection);
        } else {
            // Cerrar conexión si excedemos el tamaño mínimo
            *self.current_size.write().await -= 1;
        }
    }
    
    /// Crear nueva conexión
    async fn create_connection(&self, db_path: &str) -> Result<rusqlite::Connection> {
        let mut connection = rusqlite::Connection::open(db_path)?;
        
        // Configurar para mejor performance
        connection.pragma_check_integrity(false)?;
        connection.pragma_journal_mode(rusqlite::JournalMode::WAL)?;
        connection.pragma_synchronous(rusqlite::Synchronous::Normal)?;
        connection.pragma_cache_size(10000)?;
        connection.pragma_temp_store(rusqlite::TempStore::Memory)?;
        
        Ok(connection)
    }
    
    /// Obtener estadísticas del pool
    pub async fn stats(&self) -> PoolStats {
        let connections = self.connections.lock().await;
        let current = *self.current_size.read().await;
        
        PoolStats {
            available_connections: connections.len(),
            total_connections: current,
            max_size: self.max_size,
            min_size: self.min_size,
            utilization: current as f64 / self.max_size as f64,
        }
    }
}

/// Rate limiter para endpoints API
#[derive(Debug)]
#[derive(Clone)]
pub struct RateLimiter {
    tokens: Arc<Mutex<BTreeMap<String, usize>>>,
    max_tokens: usize,
    refill_rate: usize,
    refill_interval: Duration,
}

impl RateLimiter {
    pub fn new(max_tokens: usize, refill_rate: usize, refill_interval: Duration) -> Self {
        Self {
            tokens: Arc::new(Mutex::new(BTreeMap::new())),
            max_tokens,
            refill_rate,
            refill_interval,
        }
    }
    
    /// Verificar si se permite la request
    pub async fn check_limit(&self, client_id: &str) -> bool {
        let mut tokens = self.tokens.lock().await;
        
        let current_tokens = tokens.entry(client_id.to_string())
            .or_insert(self.max_tokens);
        
        if *current_tokens > 0 {
            *current_tokens -= 1;
            true
        } else {
            false
        }
    }
    
    /// Refill tokens periódicamente (llamado por tarea background)
    pub async fn refill_tokens(&self) {
        let mut tokens = self.tokens.lock().await;
        
        for tokens_available in tokens.values_mut() {
            *tokens_available = (*tokens_available + self.refill_rate).min(self.max_tokens);
        }
    }
    
    /// Obtener tokens disponibles para un cliente
    pub async fn get_remaining_tokens(&self, client_id: &str) -> usize {
        let tokens = self.tokens.lock().await;
        *tokens.entry(client_id.to_string()).or_insert(self.max_tokens)
    }
}

/// Caching de metadatos de base de datos
#[derive(Debug)]
#[derive(Clone)]
pub struct DatabaseMetadataCache {
    schemas: Arc<RwLock<HashMap<String, SchemaInfo>>>,
    tables: Arc<RwLock<HashMap<String, Vec<TableInfo>>>>,
    last_updated: Arc<RwLock<std::time::Instant>>,
    ttl: Duration,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaInfo {
    pub name: String,
    pub tables: Vec<String>,
    pub views: Vec<String>,
    pub functions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableInfo {
    pub name: String,
    pub schema: String,
    pub columns: Vec<ColumnInfo>,
    pub row_count: Option<usize>,
    pub last_analyzed: std::time::Instant,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnInfo {
    pub name: String,
    pub data_type: String,
    pub is_nullable: bool,
    pub default_value: Option<String>,
}

impl DatabaseMetadataCache {
    pub fn new(ttl: Duration) -> Self {
        Self {
            schemas: Arc::new(RwLock::new(HashMap::new())),
            tables: Arc::new(RwLock::new(HashMap::new())),
            last_updated: Arc::new(RwLock::new(std::time::Instant::now())),
            ttl,
        }
    }
    
    /// Obtener información de esquema
    pub async fn get_schema(&self, schema_name: &str) -> Option<SchemaInfo> {
        let schemas = self.schemas.read().await;
        let last_update = *self.last_updated.read().await;
        
        if last_update.elapsed() > self.ttl {
            None
        } else {
            schemas.get(schema_name).cloned()
        }
    }
    
    /// Cachear información de esquema
    pub async fn cache_schema(&self, schema: SchemaInfo) {
        let mut schemas = self.schemas.write().await;
        schemas.insert(schema.name.clone(), schema.clone());
        
        *self.last_updated.write().await = std::time::Instant::now();
    }
    
    /// Limpiar cache expirado
    pub async fn cleanup(&self) {
        let last_update = *self.last_updated.read().await;
        
        if last_update.elapsed() > self.ttl {
            let mut schemas = self.schemas.write().await;
            let mut tables = self.tables.write().await;
            let mut last_updated = self.last_updated.write().await;
            
            schemas.clear();
            tables.clear();
            *last_updated = std::time::Instant::now();
        }
    }
}

/// Métricas de performance
#[derive(Debug, Clone)]
pub struct PerformanceMetrics {
    pub start_time: std::time::Instant,
    pub requests_total: Arc<RwLock<u64>>,
    pub requests_success: Arc<RwLock<u64>>,
    pub requests_error: Arc<RwLock<u64>>,
    pub avg_response_time: Arc<RwLock<f64>>,
    pub memory_usage: Arc<RwLock<MemoryUsage>>,
}

#[derive(Debug, Clone)]
pub struct MemoryUsage {
    pub heap_size: usize,
    pub cache_size: usize,
    pub connection_pool_size: usize,
}

impl PerformanceMetrics {
    pub fn new() -> Self {
        Self {
            start_time: std::time::Instant::now(),
            requests_total: Arc::new(RwLock::new(0)),
            requests_success: Arc::new(RwLock::new(0)),
            requests_error: Arc::new(RwLock::new(0)),
            avg_response_time: Arc::new(RwLock::new(0.0)),
            memory_usage: Arc::new(RwLock::new(MemoryUsage {
                heap_size: 0,
                cache_size: 0,
                connection_pool_size: 0,
            })),
        }
    }
    
    /// Registrar request exitosa
    pub async fn record_success(&self, response_time: Duration) {
        let mut total = self.requests_total.write().await;
        let mut success = self.requests_success.write().await;
        let mut avg_time = self.avg_response_time.write().await;
        
        *total += 1;
        *success += 1;
        
        // Actualizar promedio móvil de response time
        let current_avg = *avg_time;
        let new_value = ((current_avg * (*total - 1) as f64) + response_time.as_secs_f64()) / *total as f64;
        *avg_time = new_value;
    }
    
    /// Registrar request con error
    pub async fn record_error(&self, response_time: Duration) {
        let mut total = self.requests_total.write().await;
        let mut error = self.requests_error.write().await;
        let mut avg_time = self.avg_response_time.write().await;
        
        *total += 1;
        *error += 1;
        
        let current_avg = *avg_time;
        let new_value = ((current_avg * (*total - 1) as f64) + response_time.as_secs_f64()) / *total as f64;
        *avg_time = new_value;
    }
    
    /// Obtener métricas actuales
    pub async fn get_metrics(&self) -> SerializedMetrics {
        let total = *self.requests_total.read().await;
        let success = *self.requests_success.read().await;
        let error = *self.requests_error.read().await;
        let avg_time = *self.avg_response_time.read().await;
        let uptime = self.start_time.elapsed().as_secs();
        
        let success_rate = if total > 0 {
            success as f64 / total as f64
        } else {
            0.0
        };
        
        SerializedMetrics {
            uptime_seconds: uptime,
            requests_total: total,
            requests_success: success,
            requests_error: error,
            success_rate,
            avg_response_time_ms: avg_time * 1000.0,
            requests_per_second: if uptime > 0 { total as f64 / uptime as f64 } else { 0.0 },
        }
    }
    
    /// Actualizar uso de memoria
    pub async fn update_memory_usage(&self, memory: MemoryUsage) {
        let mut mem = self.memory_usage.write().await;
        *mem = memory;
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializedMetrics {
    pub uptime_seconds: u64,
    pub requests_total: u64,
    pub requests_success: u64,
    pub requests_error: u64,
    pub success_rate: f64,
    pub avg_response_time_ms: f64,
    pub requests_per_second: f64,
}

/// Configuración de performance
#[derive(Debug, Clone)]
pub struct PerformanceConfig {
    pub enable_query_cache: bool,
    pub query_cache_size: usize,
    pub query_cache_ttl: Duration,
    pub enable_connection_pool: bool,
    pub connection_pool_min: usize,
    pub connection_pool_max: usize,
    pub enable_rate_limiting: bool,
    pub rate_limit_tokens: usize,
    pub rate_limit_refill: usize,
    pub rate_limit_interval: Duration,
    pub enable_metadata_cache: bool,
    pub metadata_cache_ttl: Duration,
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            enable_query_cache: true,
            query_cache_size: 1000,
            query_cache_ttl: Duration::from_secs(300),
            enable_connection_pool: true,
            connection_pool_min: 2,
            connection_pool_max: 20,
            enable_rate_limiting: true,
            rate_limit_tokens: 100,
            rate_limit_refill: 10,
            rate_limit_interval: Duration::from_secs(60),
            enable_metadata_cache: true,
            metadata_cache_ttl: Duration::from_secs(1800),
        }
    }
}

/// Estadísticas de cache
#[derive(Debug, Clone)]
pub struct CacheStats {
    pub size: usize,
    pub max_size: usize,
    pub ttl_seconds: u64,
}

/// Estadísticas de pool de conexiones
#[derive(Debug, Clone)]
pub struct PoolStats {
    pub available_connections: usize,
    pub total_connections: usize,
    pub max_size: usize,
    pub min_size: usize,
    pub utilization: f64,
}

/// Middleware de performance
pub struct PerformanceMiddleware {
    pub metrics: PerformanceMetrics,
    pub rate_limiter: Option<RateLimiter>,
    pub query_cache: Option<QueryCache>,
    pub connection_pool: Option<ConnectionPool>,
    pub metadata_cache: Option<DatabaseMetadataCache>,
}

impl PerformanceMiddleware {
    pub fn new(config: &ServerConfig) -> Self {
        let perf_config = PerformanceConfig::default();
        
        let rate_limiter = if config.rate_limiting_enabled {
            Some(RateLimiter::new(
                perf_config.rate_limit_tokens,
                perf_config.rate_limit_refill,
                perf_config.rate_limit_interval,
            ))
        } else {
            None
        };
        
        let query_cache = if perf_config.enable_query_cache {
            Some(QueryCache::new(
                perf_config.query_cache_size,
                perf_config.query_cache_ttl,
            ))
        } else {
            None
        };
        
        let connection_pool = if perf_config.enable_connection_pool {
            Some(ConnectionPool::new(
                perf_config.connection_pool_max,
                perf_config.connection_pool_min,
            ))
        } else {
            None
        };
        
        let metadata_cache = if perf_config.enable_metadata_cache {
            Some(DatabaseMetadataCache::new(perf_config.metadata_cache_ttl))
        } else {
            None
        };
        
        Self {
            metrics: PerformanceMetrics::new(),
            rate_limiter,
            query_cache,
            connection_pool,
            metadata_cache,
        }
    }
    
    /// Inicializar tareas background para mantenimiento
    pub fn start_background_tasks(&self) {
        // Tarea para refill de tokens rate limiter
        if let Some(rate_limiter) = &self.rate_limiter {
            let limiter = rate_limiter.clone();
            tokio::spawn(async move {
                let mut interval = tokio::time::interval(Duration::from_secs(10));
                loop {
                    interval.tick().await;
                    limiter.refill_tokens().await;
                }
            });
        }
        
        // Tarea para cleanup de cache
        if let Some(query_cache) = &self.query_cache {
            let cache = query_cache.clone();
            tokio::spawn(async move {
                let mut interval = tokio::time::interval(Duration::from_secs(60));
                loop {
                    interval.tick().await;
                    cache.cleanup_expired().await;
                }
            });
        }
        
        // Tarea para cleanup de metadata cache
        if let Some(metadata_cache) = &self.metadata_cache {
            let cache = metadata_cache.clone();
            tokio::spawn(async move {
                let mut interval = tokio::time::interval(Duration::from_secs(300));
                loop {
                    interval.tick().await;
                    cache.cleanup().await;
                }
            });
        }
    }
}