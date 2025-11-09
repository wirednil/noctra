//! Configuración del CLI de Noctra

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Configuración global del CLI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalConfig {
    /// Base de datos por defecto
    pub default_database: Option<PathBuf>,

    /// Modo verbose
    pub verbose: bool,

    /// Modo debug
    pub debug: bool,

    /// Archivo de configuración personalizado
    pub config_file: Option<PathBuf>,

    /// Directorio de trabajo
    pub working_dir: PathBuf,

    /// Archivo de historial del REPL
    pub history_file: PathBuf,

    /// Timeout por defecto
    pub default_timeout: u64,

    /// Límite de filas por defecto
    pub default_row_limit: Option<usize>,

    /// Formato de salida por defecto
    pub default_output_format: OutputFormat,

    /// Color de la terminal
    pub color_mode: ColorMode,

    /// Tema del CLI
    pub theme: CliTheme,
}

/// Configuración del CLI específica
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CliConfig {
    /// Configuración global
    pub global: GlobalConfig,

    /// Configuración del REPL
    pub repl: ReplConfig,

    /// Configuración de batch processing
    pub batch: BatchConfig,

    /// Configuración de la base de datos
    pub database: DatabaseConfig,
}

/// Configuración del REPL
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplConfig {
    /// Activar REPL
    pub enabled: bool,

    /// Prompt personalizado
    pub prompt: String,

    /// Prompt multi-línea
    pub multiline_prompt: String,

    /// Habilitar auto-completado
    pub auto_completion: bool,

    /// Habilitar syntax highlighting
    pub syntax_highlighting: bool,

    /// Número de líneas de historial
    pub history_size: usize,

    /// Editor externo para queries complejas
    pub external_editor: Option<String>,

    /// Configuración de key bindings
    pub key_bindings: KeyBindings,
}

/// Configuración de batch processing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchConfig {
    /// Archivo de script
    pub script_file: Option<PathBuf>,

    /// Query inline
    pub inline_query: Option<String>,

    /// Parámetros del script
    pub parameters: Vec<(String, String)>,

    /// Output file
    pub output_file: Option<PathBuf>,

    /// Formato de output
    pub output_format: OutputFormat,

    /// Modo silencioso
    pub quiet: bool,

    /// Continuar en caso de error
    pub continue_on_error: bool,
}

/// Configuración de base de datos
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    /// Tipo de backend
    pub backend_type: BackendType,

    /// Connection string o path
    pub connection_string: String,

    /// Timeout de conexión
    pub connection_timeout: u64,

    /// Pool de conexiones
    pub pool_size: u32,

    /// Configuración SSL (para PostgreSQL)
    pub ssl_mode: Option<SslMode>,

    /// Configuración de autenticación
    pub auth_config: Option<AuthConfig>,
}

/// Formatos de salida soportados
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum OutputFormat {
    /// Tabla ASCII
    Table,

    /// CSV
    Csv,

    /// JSON
    Json,

    /// XML
    Xml,

    /// Markdown
    Markdown,

    /// Formato personalizado
    Custom(String),
}

/// Modos de color
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ColorMode {
    /// Auto-detectar
    Auto,

    /// Siempre activar
    Always,

    /// Nunca activar
    Never,
}

/// Temas del CLI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CliTheme {
    /// Tema clásico (verde sobre negro)
    Classic,

    /// Tema moderno
    Modern,

    /// Tema minimalista
    Minimal,

    /// Tema oscuro
    Dark,

    /// Tema claro
    Light,
}

/// Key bindings del REPL
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyBindings {
    /// Ejecutar query
    pub execute: String,

    /// Limpiar pantalla
    pub clear: String,

    /// Salir
    pub exit: String,

    /// Ayuda
    pub help: String,

    /// Historial
    pub history: String,

    /// Editor externo
    pub editor: String,
}

/// Tipos de backend soportados
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BackendType {
    /// SQLite (en memoria o archivo)
    Sqlite,

    /// PostgreSQL
    Postgres,

    /// DuckDB
    Duckdb,
}

/// Modos SSL
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SslMode {
    /// Requerir SSL
    Require,

    /// Preferir SSL
    Prefer,

    /// Deshabilitar SSL
    Disable,
}

/// Configuración de autenticación
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthConfig {
    /// Usuario
    pub username: Option<String>,

    /// Contraseña
    pub password: Option<String>,

    /// Archivo de credenciales
    pub credential_file: Option<PathBuf>,
}

impl Default for GlobalConfig {
    fn default() -> Self {
        let home_dir = std::env::var("HOME")
            .or_else(|_| std::env::var("USERPROFILE"))
            .unwrap_or_else(|_| ".".to_string());

        Self {
            default_database: None,
            verbose: false,
            debug: false,
            config_file: None,
            working_dir: PathBuf::from("."),
            history_file: PathBuf::from(format!("{}/.noctra_history", home_dir)),
            default_timeout: 30,
            default_row_limit: Some(1000),
            default_output_format: OutputFormat::Table,
            color_mode: ColorMode::Auto,
            theme: CliTheme::Classic,
        }
    }
}

impl Default for ReplConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            prompt: "noctra> ".to_string(),
            multiline_prompt: "   ".to_string(),
            auto_completion: true,
            syntax_highlighting: true,
            history_size: 1000,
            external_editor: None,
            key_bindings: KeyBindings::default(),
        }
    }
}

impl Default for BatchConfig {
    fn default() -> Self {
        Self {
            script_file: None,
            inline_query: None,
            parameters: Vec::new(),
            output_file: None,
            output_format: OutputFormat::Table,
            quiet: false,
            continue_on_error: false,
        }
    }
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            backend_type: BackendType::Sqlite,
            connection_string: ":memory:".to_string(),
            connection_timeout: 30,
            pool_size: 10,
            ssl_mode: None,
            auth_config: None,
        }
    }
}

impl Default for KeyBindings {
    fn default() -> Self {
        Self {
            execute: "Enter".to_string(),
            clear: "Ctrl+L".to_string(),
            exit: "Ctrl+D".to_string(),
            help: "F1".to_string(),
            history: "Up/Down".to_string(),
            editor: "Ctrl+E".to_string(),
        }
    }
}

impl CliConfig {
    /// Cargar configuración desde archivo
    pub fn load_from_file(path: &PathBuf) -> Result<Self, Box<dyn std::error::Error>> {
        let content = std::fs::read_to_string(path)?;
        let config: CliConfig = toml::from_str(&content)?;
        Ok(config)
    }

    /// Guardar configuración a archivo
    pub fn save_to_file(&self, path: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
        let content = toml::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }

    /// Obtener archivo de configuración por defecto
    pub fn default_config_path() -> Result<PathBuf, Box<dyn std::error::Error>> {
        let home_dir = std::env::var("HOME")
            .or_else(|_| std::env::var("USERPROFILE"))
            .map_err(|e| format!("Cannot get home directory: {}", e))?;

        Ok(PathBuf::from(format!("{}/.noctra/config.toml", home_dir)))
    }

    /// Validar configuración
    pub fn validate(&self) -> Result<(), Box<dyn std::error::Error>> {
        // Validar configuraciones críticas
        if self.repl.history_size == 0 {
            return Err("REPL history size must be greater than 0".into());
        }

        if self.database.connection_timeout == 0 {
            return Err("Connection timeout must be greater than 0".into());
        }

        if self.database.pool_size == 0 {
            return Err("Pool size must be greater than 0".into());
        }

        Ok(())
    }

    /// Configuración para SQLite en memoria
    pub fn for_memory_sqlite() -> Self {
        let mut config = Self::default();
        config.database.backend_type = BackendType::Sqlite;
        config.database.connection_string = ":memory:".to_string();
        config
    }

    /// Configuración para SQLite archivo
    pub fn for_file_sqlite(path: PathBuf) -> Self {
        let mut config = Self::default();
        config.database.backend_type = BackendType::Sqlite;
        config.database.connection_string = path.to_string_lossy().to_string();
        config.global.default_database = Some(path);
        config
    }
}
