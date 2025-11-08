//! CLI principal de Noctra usando clap

use crate::config::CliConfig;
use clap::{Args, Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

/// Argumentos del CLI principal
#[derive(Parser, Debug)]
#[command(name = "noctra")]
#[command(about = "Noctra - Entorno SQL Interactivo")]
#[command(version = "0.1.0")]
#[command(author = "Noctra Team")]
#[command(long_about = None)]
pub struct NoctraArgs {
    /// Archivo de configuración personalizado
    #[arg(short, long, value_name = "FILE")]
    pub config: Option<PathBuf>,

    /// Base de datos SQLite (archivo)
    #[arg(long, value_name = "FILE")]
    pub database: Option<PathBuf>,

    /// Base de datos en memoria
    #[arg(short, long)]
    pub memory: bool,

    /// Modo verbose
    #[arg(short, long)]
    pub verbose: bool,

    /// Modo debug
    #[arg(long)]
    pub debug: bool,

    /// Activar colores
    #[arg(long, value_enum)]
    pub color: Option<ColorChoice>,

    /// Comando a ejecutar
    #[command(subcommand)]
    pub command: Option<NoctraSubcommand>,
}

/// Subcomandos de Noctra
#[derive(Subcommand, Debug)]
pub enum NoctraSubcommand {
    /// Modo interactivo REPL
    #[command(name = "repl")]
    Repl(ReplArgs),

    /// Ejecutar script batch
    #[command(name = "batch")]
    Batch(BatchArgs),

    /// Ejecutar formulario
    #[command(name = "form")]
    Form(FormArgs),

    /// Ejecutar query directo
    #[command(name = "query")]
    Query(QueryArgs),

    /// Información del sistema
    #[command(name = "info")]
    Info(InfoArgs),

    /// Configuración
    #[command(name = "config")]
    Config(ConfigArgs),
}

/// Argumentos del REPL
#[derive(Args, Debug, Clone, Default)]
pub struct ReplArgs {
    /// Prompt personalizado
    #[arg(short, long, value_name = "PROMPT")]
    pub prompt: Option<String>,

    /// No cargar archivo de historial
    #[arg(long)]
    pub no_history: bool,

    /// Historial personalizado
    #[arg(long, value_name = "FILE")]
    pub history: Option<PathBuf>,
}

/// Argumentos de batch processing
#[derive(Args, Debug, Clone)]
pub struct BatchArgs {
    /// Archivo de script RQL
    #[arg(required = true, value_name = "FILE")]
    pub script: PathBuf,

    /// Parámetros del script
    #[arg(short, long, value_name = "KEY=VALUE")]
    pub param: Vec<KeyValueArg>,

    /// Archivo de salida
    #[arg(short, long, value_name = "FILE")]
    pub output: Option<PathBuf>,

    /// Formato de salida
    #[arg(short, long, value_enum)]
    pub format: Option<OutputFormat>,

    /// Modo silencioso
    #[arg(short, long)]
    pub quiet: bool,

    /// Continuar en caso de error
    #[arg(long)]
    pub continue_on_error: bool,
}

/// Argumentos de formulario
#[derive(Args, Debug, Clone)]
pub struct FormArgs {
    /// Subcomando de formulario
    #[command(subcommand)]
    pub command: FormSubcommand,
}

/// Subcomandos de Form
#[derive(Subcommand, Debug, Clone)]
pub enum FormSubcommand {
    /// Cargar y validar formulario
    #[command(name = "load")]
    Load(FormLoadArgs),

    /// Ejecutar formulario interactivamente
    #[command(name = "exec")]
    Exec(FormExecArgs),

    /// Preview de formulario (sin ejecutar)
    #[command(name = "preview")]
    Preview(FormPreviewArgs),
}

/// Argumentos de form load
#[derive(Args, Debug, Clone)]
pub struct FormLoadArgs {
    /// Archivo de formulario TOML
    #[arg(required = true, value_name = "FILE")]
    pub file: PathBuf,

    /// Solo validar (no mostrar)
    #[arg(short, long)]
    pub validate_only: bool,

    /// Mostrar información detallada
    #[arg(short, long)]
    pub info: bool,
}

/// Argumentos de form exec
#[derive(Args, Debug, Clone)]
pub struct FormExecArgs {
    /// Archivo de formulario TOML
    #[arg(required = true, value_name = "FILE")]
    pub file: PathBuf,

    /// Parámetros del formulario
    #[arg(short, long, value_name = "KEY=VALUE")]
    pub param: Vec<KeyValueArg>,

    /// Output file
    #[arg(short, long, value_name = "FILE")]
    pub output: Option<PathBuf>,

    /// Modo no interactivo (usar valores por defecto)
    #[arg(short, long)]
    pub non_interactive: bool,
}

/// Argumentos de form preview
#[derive(Args, Debug, Clone)]
pub struct FormPreviewArgs {
    /// Archivo de formulario TOML
    #[arg(required = true, value_name = "FILE")]
    pub file: PathBuf,

    /// Mostrar valores de ejemplo
    #[arg(short, long)]
    pub with_examples: bool,
}

/// Argumentos de query directo
#[derive(Args, Debug, Clone)]
pub struct QueryArgs {
    /// Query SQL a ejecutar
    #[arg(required = true, value_name = "SQL")]
    pub query: String,

    /// Parámetros del query
    #[arg(short, long, value_name = "KEY=VALUE")]
    pub param: Vec<KeyValueArg>,

    /// Output file
    #[arg(short, long, value_name = "FILE")]
    pub output: Option<PathBuf>,

    /// Formato de salida
    #[arg(short, long, value_enum)]
    pub format: Option<OutputFormat>,

    /// Solo mostrar el SQL generado
    #[arg(long)]
    pub dry_run: bool,
}

/// Argumentos de información
#[derive(Args, Debug, Clone)]
pub struct InfoArgs {
    /// Mostrar información de la base de datos
    #[arg(short, long)]
    pub database: bool,

    /// Mostrar información del sistema
    #[arg(short, long)]
    pub system: bool,

    /// Mostrar versión
    #[arg(short, long)]
    pub version: bool,
}

/// Argumentos de configuración
#[derive(Args, Debug, Clone)]
pub struct ConfigArgs {
    /// Mostrar configuración actual
    #[arg(short, long)]
    pub show: bool,

    /// Editar configuración
    #[arg(short, long)]
    pub edit: bool,

    /// Resetear configuración
    #[arg(short, long)]
    pub reset: bool,
}

/// Choice para colores
#[derive(ValueEnum, Clone, Debug)]
pub enum ColorChoice {
    /// Auto-detectar
    Auto,
    /// Siempre activar
    Always,
    /// Nunca activar
    Never,
}

/// Formatos de salida
#[derive(ValueEnum, Clone, Debug)]
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
}

/// Key-Value argument
#[derive(Debug, Clone)]
pub struct KeyValueArg {
    pub key: String,
    pub value: String,
}

impl KeyValueArg {
    /// Parsear de string "key=value"
    pub fn parse(s: &str) -> Result<Self, String> {
        let parts: Vec<&str> = s.splitn(2, '=').collect();
        if parts.len() != 2 {
            return Err("Argument must be in format KEY=VALUE".to_string());
        }
        Ok(Self {
            key: parts[0].to_string(),
            value: parts[1].to_string(),
        })
    }
}

impl std::str::FromStr for KeyValueArg {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s)
    }
}

/// Aplicación principal de Noctra
#[derive(Debug)]
pub struct NoctraApp {
    pub args: NoctraArgs,
    pub config: CliConfig,
}

impl NoctraApp {
    /// Crear nueva aplicación desde argumentos
    pub fn new(args: NoctraArgs) -> Result<Self, Box<dyn std::error::Error>> {
        let config = load_config(&args)?;
        Ok(Self { args, config })
    }

    /// Ejecutar aplicación
    pub async fn run(mut self) -> Result<(), Box<dyn std::error::Error>> {
        let command = self.args.command.take();
        let result = match command {
            Some(cmd) => match cmd {
                NoctraSubcommand::Repl(args) => self.run_repl(args).await,
                NoctraSubcommand::Batch(args) => self.run_batch(args).await,
                NoctraSubcommand::Form(args) => self.run_form(args).await,
                NoctraSubcommand::Query(args) => self.run_query(args).await,
                NoctraSubcommand::Info(args) => self.run_info(args),
                NoctraSubcommand::Config(args) => self.run_config(args),
            },
            None => self.run_interactive().await,
        };

        result
    }

    /// Ejecutar comando específico
    #[allow(dead_code)]
    async fn run_command(
        self,
        command: NoctraSubcommand,
    ) -> Result<(), Box<dyn std::error::Error>> {
        use NoctraSubcommand::*;

        match command {
            Repl(args) => self.run_repl(args).await,
            Batch(args) => self.run_batch(args).await,
            Form(args) => self.run_form(args).await,
            Query(args) => self.run_query(args).await,
            Info(args) => self.run_info(args),
            Config(args) => self.run_config(args),
        }
    }

    /// Ejecutar modo interactivo
    async fn run_interactive(self) -> Result<(), Box<dyn std::error::Error>> {
        self.run_repl(ReplArgs::default()).await
    }

    /// Ejecutar REPL
    async fn run_repl(self, args: ReplArgs) -> Result<(), Box<dyn std::error::Error>> {
        println!("🐍 Noctra v0.1.0 - Entorno SQL Interactivo");
        println!("Escribe 'help' para comandos disponibles o 'quit' para salir.");

        // Crear e iniciar REPL
        let mut repl = crate::repl::Repl::new(self.config, args)?;
        repl.run().await?;

        Ok(())
    }

    /// Ejecutar batch processing
    async fn run_batch(self, args: BatchArgs) -> Result<(), Box<dyn std::error::Error>> {
        let _script_content = std::fs::read_to_string(&args.script)
            .map_err(|e| format!("Error reading script file: {}", e))?;

        println!("📜 Ejecutando script: {}", args.script.display());

        // Crear parámetros desde argumentos
        let mut parameters = std::collections::HashMap::new();
        for param in args.param {
            parameters.insert(param.key, param.value);
        }

        // TODO: Implementar ejecución de script
        println!("⚠️  Script processing no implementado aún");

        Ok(())
    }

    /// Ejecutar formulario
    async fn run_form(self, args: FormArgs) -> Result<(), Box<dyn std::error::Error>> {
        use FormSubcommand::*;

        match args.command {
            Load(load_args) => self.run_form_load(load_args).await,
            Exec(exec_args) => self.run_form_exec(exec_args).await,
            Preview(preview_args) => self.run_form_preview(preview_args).await,
        }
    }

    /// Cargar y validar formulario
    async fn run_form_load(
        &self,
        args: FormLoadArgs,
    ) -> Result<(), Box<dyn std::error::Error>> {
        use noctra_formlib::load_form_from_path;

        println!("📋 Cargando formulario: {}", args.file.display());

        // Validar que el archivo existe
        if !args.file.exists() {
            return Err(format!("Form file not found: {}", args.file.display()).into());
        }

        // Cargar formulario
        let form = load_form_from_path(&args.file)?;

        if args.validate_only {
            println!("✅ Formulario válido: {}", form.title);
            return Ok(());
        }

        // Mostrar información del formulario
        println!("\n📝 Formulario: {}", form.title);
        if let Some(desc) = &form.description {
            println!("   Descripción: {}", desc);
        }
        if let Some(schema) = &form.schema {
            println!("   Schema: {}", schema);
        }

        println!("\n🔢 Campos ({}):", form.fields.len());
        for (name, field) in &form.fields {
            let required = if field.required { "*" } else { "" };
            println!(
                "   - {}{}: {} ({:?})",
                name, required, field.label, field.field_type
            );
        }

        println!("\n⚡ Acciones ({}):", form.actions.len());
        for (name, action) in &form.actions {
            println!("   - {}: {:?}", name, action.action_type);
        }

        if args.info {
            println!("\n📊 Información Detallada:");
            if let Some(ui_config) = &form.ui_config {
                println!("   Layout: {:?}", ui_config.layout);
                if let Some(width) = ui_config.width {
                    println!("   Width: {}", width);
                }
                if let Some(height) = ui_config.height {
                    println!("   Height: {}", height);
                }
            }
        }

        println!("\n✅ Formulario cargado correctamente");
        Ok(())
    }

    /// Ejecutar formulario interactivamente
    async fn run_form_exec(
        &self,
        args: FormExecArgs,
    ) -> Result<(), Box<dyn std::error::Error>> {
        use noctra_formlib::load_form_from_path;

        println!("🚀 Ejecutando formulario: {}", args.file.display());

        // Cargar formulario
        let form = load_form_from_path(&args.file)?;

        if args.non_interactive {
            // Modo no interactivo (batch)
            use noctra_tui::FormRenderer;
            let mut renderer = FormRenderer::new(form);

            // Usar valores por defecto o de parámetros
            for param in args.param {
                renderer.set_field_value(&param.key, param.value)?;
            }

            // Validar
            renderer.validate_form()?;
            println!("✅ Formulario validado correctamente");

            // Mostrar valores
            println!("\n📊 Valores:");
            for (key, value) in renderer.get_values() {
                println!("   {}: {}", key, value);
            }
        } else {
            // Modo interactivo con TUI
            use crate::interactive_form::InteractiveFormExecutor;

            println!("\n🎯 Modo interactivo");
            println!("   TAB/Shift+TAB: Navegar entre campos");
            println!("   Escribir: Editar valor del campo");
            println!("   Backspace: Borrar carácter");
            println!("   Enter: Validar y continuar");
            println!("   ESC: Cancelar\n");

            println!("Presiona cualquier tecla para continuar...");
            std::thread::sleep(std::time::Duration::from_secs(2));

            let mut executor = InteractiveFormExecutor::new(form);
            match executor.run()? {
                Some(values) => {
                    println!("\n✅ Formulario completado");
                    println!("\n📊 Valores:");
                    for (key, value) in &values {
                        println!("   {}: {}", key, value);
                    }

                    // Guardar output si se especifica
                    if let Some(output_file) = args.output {
                        let json = serde_json::to_string_pretty(&values)?;
                        std::fs::write(&output_file, json)?;
                        println!("\n💾 Guardado en: {}", output_file.display());
                    }
                }
                None => {
                    println!("\n❌ Formulario cancelado");
                }
            }
        }

        Ok(())
    }

    /// Preview de formulario
    async fn run_form_preview(
        &self,
        args: FormPreviewArgs,
    ) -> Result<(), Box<dyn std::error::Error>> {
        use noctra_formlib::load_form_from_path;
        use noctra_tui::FormRenderer;

        println!("👁️  Preview de formulario: {}", args.file.display());

        // Cargar formulario
        let form = load_form_from_path(&args.file)?;
        let mut renderer = FormRenderer::new(form);

        // Agregar valores de ejemplo si se solicita
        if args.with_examples {
            // Colectar campos primero para evitar borrow issues
            let field_examples: Vec<(String, String)> = renderer
                .form
                .fields
                .iter()
                .map(|(name, field)| {
                    let example_value = match field.field_type {
                        noctra_formlib::FieldType::Text => "Example Text".to_string(),
                        noctra_formlib::FieldType::Int => "42".to_string(),
                        noctra_formlib::FieldType::Float => "3.14".to_string(),
                        noctra_formlib::FieldType::Boolean => "true".to_string(),
                        noctra_formlib::FieldType::Email => "user@example.com".to_string(),
                        noctra_formlib::FieldType::Date => "2025-11-08".to_string(),
                        noctra_formlib::FieldType::DateTime => "2025-11-08 14:30:00".to_string(),
                        noctra_formlib::FieldType::Password => "password123".to_string(),
                        noctra_formlib::FieldType::Select { ref options } => {
                            options.first().cloned().unwrap_or_default()
                        }
                        _ => String::new(),
                    };
                    (name.clone(), example_value)
                })
                .collect();

            // Ahora establecer los valores
            for (name, value) in field_examples {
                let _ = renderer.set_field_value(&name, value);
            }
        }

        // Renderizar
        println!("\n{}", renderer.render());

        println!("\n✨ Este es un preview del formulario.");
        println!("   Usa 'noctra form exec {}' para ejecutarlo.", args.file.display());

        Ok(())
    }

    /// Ejecutar query directo
    async fn run_query(self, args: QueryArgs) -> Result<(), Box<dyn std::error::Error>> {
        println!("🔍 Ejecutando query...");

        if args.dry_run {
            println!("📝 SQL generado:");
            println!("{}", args.query);
            return Ok(());
        }

        // TODO: Implementar ejecución de query
        println!("⚠️  Query execution no implementado aún");

        Ok(())
    }

    /// Ejecutar comando info
    fn run_info(self, args: InfoArgs) -> Result<(), Box<dyn std::error::Error>> {
        if args.version {
            println!("Noctra v0.1.0");
        }

        if args.system {
            self.show_system_info();
        }

        if args.database {
            self.show_database_info();
        }

        Ok(())
    }

    /// Ejecutar comando config
    fn run_config(mut self, args: ConfigArgs) -> Result<(), Box<dyn std::error::Error>> {
        if args.show {
            self.show_config();
        } else if args.edit {
            self.edit_config()?;
        } else if args.reset {
            self.reset_config()?;
        } else {
            println!("Usa --help para ver opciones de configuración");
        }

        Ok(())
    }

    /// Mostrar información del sistema
    fn show_system_info(&self) {
        println!("📊 Información del Sistema:");
        println!("  Versión de Noctra: {}", env!("CARGO_PKG_VERSION"));
        println!("  Sistema Operativo: {}", std::env::consts::OS);
        println!("  Arquitectura: {}", std::env::consts::ARCH);

        // Mostrar información de Rust
        println!(
            "  Rust Version: {}",
            std::env::var("RUSTC_VERSION").unwrap_or_else(|_| "Unknown".to_string())
        );
    }

    /// Mostrar información de la base de datos
    fn show_database_info(&self) {
        println!("💾 Información de la Base de Datos:");
        println!("  Backend: {:?}", self.config.database.backend_type);
        println!("  Connection: {}", self.config.database.connection_string);
        println!("  Timeout: {}s", self.config.database.connection_timeout);
        println!("  Pool Size: {}", self.config.database.pool_size);
    }

    /// Mostrar configuración actual
    fn show_config(&self) {
        println!("⚙️  Configuración Actual:");
        println!("  Database: {:?}", self.config.database.backend_type);
        println!("  Connection: {}", self.config.database.connection_string);
        println!("  Default Timeout: {}s", self.config.global.default_timeout);
        println!(
            "  Default Format: {:?}",
            self.config.global.default_output_format
        );
        println!("  Color Mode: {:?}", self.config.global.color_mode);
        println!("  Theme: {:?}", self.config.global.theme);
    }

    /// Editar configuración
    fn edit_config(&self) -> Result<(), Box<dyn std::error::Error>> {
        let config_path = CliConfig::default_config_path()?;
        println!("Editando configuración en: {}", config_path.display());

        // Crear directorio si no existe
        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        // Guardar configuración por defecto si no existe
        if !config_path.exists() {
            self.config.save_to_file(&config_path)?;
            println!("📝 Archivo de configuración creado");
        }

        // TODO: Abrir editor externo
        println!("⚠️  Editor configuration no implementado aún");

        Ok(())
    }

    /// Resetear configuración
    fn reset_config(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.config = CliConfig::default();
        println!("🔄 Configuración reseteada a valores por defecto");
        Ok(())
    }
}

/// Cargar configuración desde argumentos
fn load_config(args: &NoctraArgs) -> Result<CliConfig, Box<dyn std::error::Error>> {
    let mut config = if let Some(config_file) = &args.config {
        CliConfig::load_from_file(config_file)?
    } else {
        // Intentar cargar configuración por defecto
        match CliConfig::default_config_path() {
            Ok(default_path) => {
                if default_path.exists() {
                    CliConfig::load_from_file(&default_path)?
                } else {
                    CliConfig::default()
                }
            }
            Err(_) => CliConfig::default(),
        }
    };

    // Aplicar overrides de argumentos CLI
    apply_cli_overrides(&mut config, args);

    config.validate()?;
    Ok(config)
}

/// Aplicar overrides de CLI a la configuración
fn apply_cli_overrides(config: &mut CliConfig, args: &NoctraArgs) {
    // Database overrides
    if args.memory {
        config.database.backend_type = crate::config::BackendType::Sqlite;
        config.database.connection_string = ":memory:".to_string();
    } else if let Some(db_path) = &args.database {
        config.database.backend_type = crate::config::BackendType::Sqlite;
        config.database.connection_string = db_path.to_string_lossy().to_string();
    }

    // Verbose/Debug
    config.global.verbose = args.verbose;
    config.global.debug = args.debug;

    // Color mode
    if let Some(color_choice) = &args.color {
        config.global.color_mode = match color_choice {
            ColorChoice::Auto => crate::config::ColorMode::Auto,
            ColorChoice::Always => crate::config::ColorMode::Always,
            ColorChoice::Never => crate::config::ColorMode::Never,
        };
    }
}

/// Construir parser de CLI
pub fn build_cli() -> clap::Command {
    use clap::CommandFactory;
    NoctraArgs::command()
}
