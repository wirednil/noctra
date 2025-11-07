//! REPL (Read-Eval-Print Loop) para Noctra

use std::io::{self, Write};
use crate::config::CliConfig;
use noctra_core::NoctraError;
type Result<T> = std::result::Result<T, NoctraError>;

/// Handler del REPL
#[derive(Debug)]
pub struct ReplHandler {
    /// Configuración
    config: CliConfig,
    
    /// Estado del REPL
    state: ReplState,
    
    /// Historial de comandos
    history: Vec<String>,
    
    /// Contador de líneas
    line_count: usize,
}

/// Estado del REPL
#[derive(Debug, Clone)]
pub enum ReplState {
    /// Listo para comando
    Ready,
    
    /// Esperando más líneas (query multi-línea)
    MultiLine,
    
    /// Esperando parámetro
    WaitingParameter(String),
    
    /// Error
    Error,
}

/// REPL principal
#[derive(Debug)]
pub struct Repl {
    /// Configuración
    config: CliConfig,
    
    /// Handler
    handler: ReplHandler,
}

impl Repl {
    /// Crear nuevo REPL
    pub fn new(config: CliConfig, args: ReplArgs) -> Result<Self> {
        let handler = ReplHandler::new(config.clone(), args)?;
        
        Ok(Self {
            config,
            handler,
        })
    }
    
    /// Ejecutar REPL
    pub async fn run(&mut self) -> Result<()> {
        println!("🎯 Noctra REPL iniciado - Escribe 'help' para ayuda");
        
        loop {
            // Mostrar prompt
            let prompt = self.get_prompt();
            
            // Leer input
            let input = read_input(&prompt)?;
            
            // Procesar input
            if self.process_input(&input)? {
                break; // Salir del REPL
            }
        }
        
        println!("👋 ¡Hasta luego!");
        Ok(())
    }
    
    /// Obtener prompt actual
    fn get_prompt(&self) -> String {
        match &self.handler.state {
            ReplState::Ready => self.config.repl.prompt.clone(),
            ReplState::MultiLine => self.config.repl.multiline_prompt.clone(),
            ReplState::WaitingParameter(param) => format!(":param {} => ", param),
            ReplState::Error => "ERROR> ".to_string(),
        }
    }
    
    /// Procesar input del usuario
    fn process_input(&mut self, input: &str) -> Result<bool> {
        let trimmed = input.trim();
        
        // Comandos especiales
        if trimmed.is_empty() {
            return Ok(false);
        }
        
        if trimmed == "quit" || trimmed == "exit" || trimmed == "q" {
            return Ok(true); // Salir
        }
        
        if trimmed == "help" || trimmed == "h" || trimmed == "?" {
            self.show_help();
            return Ok(false);
        }
        
        if trimmed == "clear" || trimmed == "cls" {
            self.clear_screen();
            return Ok(false);
        }
        
        if trimmed.starts_with(':') {
            return self.handle_special_command(trimmed);
        }
        
        // Agregar a historial
        self.handler.history.push(input.to_string());
        
        // Procesar como SQL/RQL
        self.execute_query(input)
    }
    
    /// Manejar comandos especiales
    fn handle_special_command(&mut self, cmd: &str) -> Result<bool> {
        match cmd {
            ":help" => {
                self.show_help();
                Ok(false)
            }
            ":clear" | ":cls" => {
                self.clear_screen();
                Ok(false)
            }
            ":version" | ":ver" => {
                println!("Noctra v0.1.0");
                Ok(false)
            }
            ":config" => {
                self.show_config();
                Ok(false)
            }
            ":status" | ":stats" => {
                self.show_status();
                Ok(false)
            }
            cmd => {
                if cmd.starts_with(":set ") {
                    self.handle_set_command(cmd);
                    Ok(false)
                } else {
                    println!("Comando desconocido: {}", cmd);
                    Ok(false)
                }
            }
        }
    }
    
    /// Ejecutar query (versión simplificada)
    fn execute_query(&mut self, query: &str) -> Result<bool> {
        println!("🔍 Ejecutando query...");
        
        // Por ahora, mostrar el query
        println!("📝 Query: {}", query);
        println!("⚠️  Query execution pendiente de implementación");
        
        Ok(false)
    }
    
    /// Mostrar ayuda
    fn show_help(&self) {
        println!("🐍 Noctra - Comandos disponibles:");
        println!("  help, h, ?       - Mostrar esta ayuda");
        println!("  clear, cls       - Limpiar pantalla");
        println!("  quit, exit, q    - Salir del REPL");
        println!("  :version, :ver   - Mostrar versión");
        println!("  :config          - Mostrar configuración");
        println!("  :status, :stats  - Mostrar estado");
        println!("  :set KEY=VALUE   - Configurar variable");
        println!();
        println!("📋 Ejemplos de SQL:");
        println!("  SELECT * FROM employees WHERE dept = 'IT';");
        println!("  USE payroll;");
        println!("  LET dept = 'SALES';");
        println!("  :dept => IT");
        println!();
    }
    
    /// Limpiar pantalla
    fn clear_screen(&self) {
        print!("\x1B[2J\x1B[H");
        io::stdout().flush().unwrap();
    }
    
    /// Mostrar configuración
    fn show_config(&self) {
        println!("⚙️  Configuración actual:");
        println!("  Database: {:?}", self.config.database.backend_type);
        println!("  Connection: {}", self.config.database.connection_string);
        println!("  Theme: {:?}", self.config.global.theme);
        println!("  Color Mode: {:?}", self.config.global.color_mode);
    }
    
    /// Mostrar estado
    fn show_status(&self) {
        println!("📊 Estado del REPL:");
        println!("  Líneas procesadas: {}", self.handler.line_count);
        println!("  Comandos en historial: {}", self.handler.history.len());
        println!("  Estado: {:?}", self.handler.state);
    }
    
    /// Manejar comando SET
    fn handle_set_command(&mut self, cmd: &str) {
        let parts: Vec<&str> = cmd.splitn(2, ' ').collect();
        if parts.len() == 2 {
            let key_value = parts[1];
            if let Some((key, value)) = key_value.split_once('=') {
                println!("📝 Variable '{}' configurada a '{}'", key.trim(), value.trim());
            } else {
                println!("❌ Formato inválido. Usa: :set KEY=VALUE");
            }
        } else {
            println!("❌ Formato inválido. Usa: :set KEY=VALUE");
        }
    }
}

impl ReplHandler {
    /// Crear nuevo handler
    fn new(config: CliConfig, args: ReplArgs) -> Result<Self> {
        Ok(Self {
            config,
            state: ReplState::Ready,
            history: Vec::new(),
            line_count: 0,
        })
    }
}

/// Leer input con prompt
fn read_input(prompt: &str) -> Result<String> {
    print!("{}", prompt);
    io::stdout().flush()?;
    
    let mut input = String::new();
    io::stdin().read_line(&mut input)
        .map_err(|e| NoctraError::Io(e))?;
    
    Ok(input.trim().to_string())
}

/// Resultado de comando
pub type CommandResult = Result<bool>;

impl Default for ReplArgs {
    fn default() -> Self {
        Self {
            prompt: None,
            no_history: false,
            history: None,
        }
    }
}