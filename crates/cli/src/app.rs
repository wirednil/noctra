//! Aplicación principal de Noctra
//! 
//! Aplicación que integra core, parser, cli, formlib y tui
//! para proporcionar una experiencia SQL interactiva completa.

use std::path::PathBuf;
use clap::Parser;
use log::info;

use crate::config::CliConfig;
use crate::commands::{CommandExecutor, CommandResult};
use crate::repl::Repl;
use noctra_core::{Session, Executor, SqliteBackend};
use noctra_parser::RqlParser;
use noctra_formlib::{FormExecutionContext, load_form_from_path};
use noctra_tui::{TuiApp, TuiConfig, FormComponent};

/// Resultado de aplicación
pub type AppResult<T> = Result<T, Box<dyn std::error::Error>>;

/// Aplicación principal de Noctra
pub struct NoctraApp {
    /// Configuración
    pub config: CliConfig,
    
    /// Sesión actual
    pub session: Option<Session>,
    
    /// Executor de consultas
    pub executor: Option<Executor>,
    
    /// Parser RQL
    pub parser: RqlParser,
}

impl NoctraApp {
    /// Crear nueva aplicación
    pub fn new(config: CliConfig) -> Self {
        Self {
            config,
            session: None,
            executor: None,
            parser: RqlParser::new(),
        }
    }
    
    /// Inicializar aplicación
    pub async fn init(&mut self) -> AppResult<()> {
        info!("🚀 Inicializando Noctra...");
        
        // Conectar a base de datos
        self.connect_database().await?;
        
        // Configurar executor
        self.setup_executor().await?;
        
        info!("✅ Noctra inicializado correctamente");
        Ok(())
    }
    
    /// Conectar a base de datos
    async fn connect_database(&mut self) -> AppResult<()> {
        info!("📡 Conectando a base de datos: {}", self.config.database.connection_string);
        
        // Por ahora, conectar a SQLite
        let sqlite_backend = SqliteBackend::new(&self.config.database.connection_string)?;
        let session = Session::new(self.config.database.connection_string.clone());
        
        self.session = Some(session);
        
        info!("✅ Conectado a base de datos SQLite");
        Ok(())
    }
    
    /// Configurar executor
    async fn setup_executor(&mut self) -> AppResult<()> {
        if let Some(ref session) = self.session {
            // Crear executor con SQLite backend
            let executor = Executor::new(self.config.database.backend_type.clone());
            self.executor = Some(executor);
            
            info!("⚙️ Executor configurado");
        }
        
        Ok(())
    }
    
    /// Ejecutar aplicación en modo REPL
    pub async fn run_repl(&mut self) -> AppResult<()> {
        info!("🎮 Iniciando modo REPL...");
        
        // Crear REPL
        let mut repl = Repl::new(self.config.clone());
        
        // Ejecutar loop REPL
        loop {
            let input = repl.read_line().await?;
            
            if input.trim() == "quit" || input.trim() == "exit" {
                println!("👋 ¡Hasta luego!");
                break;
            }
            
            if !input.trim().is_empty() {
                let result = self.execute_command(&input).await?;
                println!("{}", result.message);
                
                if let Some(data) = result.data {
                    println!("{}", data.to_table());
                }
            }
        }
        
        Ok(())
    }
    
    /// Ejecutar consulta única
    pub async fn run_query(&mut self, sql: &str) -> AppResult<()> {
        let result = self.execute_command(sql).await?;
        println!("{}", result.message);
        
        if let Some(data) = result.data {
            println!("{}", data.to_table());
        }
        
        Ok(())
    }
    
    /// Ejecutar archivo de comandos
    pub async fn run_file(&mut self, file_path: &PathBuf) -> AppResult<()> {
        info!("📁 Ejecutando archivo: {}", file_path.display());
        
        if !file_path.exists() {
            return Err(format!("Archivo no encontrado: {}", file_path.display()).into());
        }
        
        let content = std::fs::read_to_string(file_path)?;
        let lines: Vec<&str> = content.lines().collect();
        
        let mut success_count = 0;
        let mut error_count = 0;
        
        for (line_num, line) in lines.iter().enumerate() {
            let trimmed = line.trim();
            if !trimmed.is_empty() && !trimmed.starts_with('#') {
                match self.execute_command(trimmed).await {
                    Ok(result) => {
                        if result.success {
                            success_count += 1;
                            if !result.message.is_empty() {
                                println!("Línea {}: {}", line_num + 1, result.message);
                            }
                        } else {
                            error_count += 1;
                            println!("❌ Línea {}: {}", line_num + 1, result.message);
                        }
                    }
                    Err(e) => {
                        error_count += 1;
                        println!("❌ Línea {}: Error - {}", line_num + 1, e);
                    }
                }
            }
        }
        
        println!("📊 Resumen: {} exitosas, {} errores", success_count, error_count);
        
        Ok(())
    }
    
    /// Ejecutar formulario
    pub async fn run_form(&mut self, form_path: &PathBuf) -> AppResult<()> {
        info!("📋 Cargando formulario: {}", form_path.display());
        
        let form = load_form_from_path(form_path)?;
        
        println!("📝 Formulario: {}", form.title);
        println!("📊 Campos: {}", form.fields.len());
        println!("⚡ Acciones: {}", form.actions.len());
        
        // Por ahora solo mostrar información del formulario
        // TODO: Implementar ejecución interactiva del formulario
        
        Ok(())
    }
    
    /// Ejecutar formulario en modo TUI
    pub async fn run_tui_form(&mut self, form_path: &PathBuf) -> AppResult<()> {
        info!("🎨 Cargando formulario TUI: {}", form_path.display());
        
        let form = load_form_from_path(form_path)?;
        
        // Crear componente de formulario
        let form_component = FormComponent::new(form);
        
        // Crear aplicación TUI
        let mut tui_config = TuiConfig::default();
        tui_config.title = Some("Noctra Form".to_string());
        
        let mut tui_app = TuiApp::new(tui_config);
        tui_app.register_component(Box::new(form_component));
        
        // Ejecutar TUI (implementación futura)
        println!("🎨 Modo TUI iniciado (implementación en desarrollo)");
        
        Ok(())
    }
    
    /// Ejecutar comando individual
    async fn execute_command(&mut self, input: &str) -> AppResult<CommandResult> {
        let mut executor = CommandExecutor::new(self.config.clone());
        executor.context.session = self.session.clone();
        executor.context.executor = self.executor.clone();
        executor.context.parser = self.parser.clone();
        
        let result = executor.execute_command(input).await;
        
        // Actualizar estado de la aplicación
        self.session = executor.context.session;
        self.executor = executor.context.executor;
        self.parser = executor.context.parser;
        
        Ok(result)
    }
    
    /// Obtener información de estado
    pub fn get_status(&self) -> String {
        let db_status = if self.session.is_some() { "Conectado" } else { "Desconectado" };
        let executor_status = if self.executor.is_some() { "Activo" } else { "Inactivo" };
        
        format!(
            "🐍 Noctra Status:\n\
             📡 Base de datos: {}\n\
             ⚙️ Executor: {}\n\
             🎯 Parser: RQL v1.0\n\
             📦 Crates: core, parser, cli, formlib, tui\n\
             🚀 Versión: 0.1.0",
            db_status,
            executor_status
        )
    }
}

/// Aplicación CLI argumentos
#[derive(Parser, Debug)]
#[command(name = "noctra")]
#[command(about = "🐍 Noctra - Entorno SQL Interactivo en Rust")]
#[command(version = "0.1.0")]
pub struct NoctraArgs {
    /// Query SQL a ejecutar
    #[arg(short, long)]
    pub query: Option<String>,
    
    /// Archivo de comandos a ejecutar
    #[arg(short, long)]
    pub file: Option<PathBuf>,
    
    /// Formulario a ejecutar
    #[arg(short, long)]
    pub form: Option<PathBuf>,
    
    /// Formulario en modo TUI
    #[arg(short, long)]
    pub tui: Option<PathBuf>,
    
    /// Base de datos SQLite a usar
    #[arg(short, long, default_value = "sqlite:noctra.db")]
    pub database: String,
    
    /// Modo verbose
    #[arg(short, long)]
    pub verbose: bool,
    
    /// No iniciar REPL (solo ejecutar y salir)
    #[arg(short, long)]
    pub batch: bool,
    
    /// Mostrar versión y salir
    #[arg(short, long)]
    pub version: bool,
}

/// Construir y ejecutar aplicación CLI
pub async fn build_cli(args: NoctraArgs) -> AppResult<()> {
    if args.verbose {
        println!("🐍 Noctra v0.1.0 - Entorno SQL Interactivo en Rust");
        println!("📚 Crates: core, parser, cli, formlib, tui");
        println!("🗄️ Base de datos: {}", args.database);
    }
    
    // Mostrar versión si se solicita
    if args.version {
        println!("Noctra v0.1.0");
        println!("Entorno SQL Interactivo en Rust");
        println!("Crates: core, parser, cli, formlib, tui");
        return Ok(());
    }
    
    // Crear configuración
    let config = CliConfig::from_args(&args)?;
    
    // Crear aplicación
    let mut app = NoctraApp::new(config);
    app.init().await?;
    
    // Ejecutar según argumentos
    if let Some(query) = args.query {
        // Query única
        app.run_query(&query).await?;
    } else if let Some(file) = args.file {
        // Archivo de comandos
        app.run_file(&file).await?;
    } else if let Some(form) = args.form {
        // Formulario FDL2
        app.run_form(&form).await?;
    } else if let Some(tui_form) = args.tui {
        // Formulario TUI
        app.run_tui_form(&tui_form).await?;
    } else {
        // Modo REPL (por defecto)
        if args.batch {
            println!("❌ Modo batch requiere --file o --query");
            std::process::exit(1);
        } else {
            println!("🎮 Iniciando modo REPL...");
            println!("💡 Escribe 'help' para ver comandos disponibles");
            println!("💡 Escribe 'quit' para salir");
            println!();
            
            app.run_repl().await?;
        }
    }
    
    Ok(())
}

impl CliConfig {
    /// Crear configuración desde argumentos CLI
    fn from_args(args: &NoctraArgs) -> AppResult<Self> {
        let mut config = Self::default();
        
        // Configurar base de datos
        config.database.connection_string = args.database.clone();
        config.database.backend_type = if args.database.starts_with("sqlite:") {
            "sqlite".to_string()
        } else {
            "postgres".to_string()
        };
        
        // Configurar verbosidad
        config.global.verbose = args.verbose;
        
        // Configurar modo batch
        config.global.batch_mode = args.batch;
        
        Ok(config)
    }
}