//! Noctra CLI - Entry point principal integrado
//! 
//! Punto de entrada principal que integra todos los crates de Noctra:
//! - core: Runtime y executor
//! - parser: RQL parser 
//! - cli: Commands y REPL
//! - formlib: Formularios FDL2
//! - tui: Interface de usuario terminal

use clap::Parser;
use std::process::ExitCode;
use log::error;

use noctra_cli::{NoctraArgs, build_app as build_cli};

#[tokio::main]
async fn main() -> ExitCode {
    // Initialize logging
    env_logger::init();

    // Parse CLI arguments
    let args = NoctraArgs::parse();
    
    // Build and run the application
    match build_cli(args).await {
        Ok(_) => {
            println!("👋 ¡Noctra finalizado correctamente!");
            ExitCode::from(0)
        },
        Err(e) => {
            error!("❌ Error crítico: {}", e);
            error!("💡 Para ayuda, prueba: noctra --help");
            ExitCode::from(1)
        }
    }
}