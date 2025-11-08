//! Noctra CLI - Entry point principal integrado
//!
//! Punto de entrada principal que integra todos los crates de Noctra:
//! - core: Runtime y executor
//! - parser: RQL parser
//! - cli: Commands y REPL
//! - formlib: Formularios FDL2
//! - tui: Interface de usuario terminal

use clap::Parser;
use log::error;
use std::process::ExitCode;

use noctra_cli::{NoctraApp, NoctraArgs};

#[tokio::main]
async fn main() -> ExitCode {
    // Initialize logging
    env_logger::init();

    // Parse CLI arguments
    let args = NoctraArgs::parse();

    // Build and run the application
    let app = match NoctraApp::new(args) {
        Ok(app) => app,
        Err(e) => {
            error!("❌ Error inicializando aplicación: {}", e);
            return ExitCode::from(1);
        }
    };

    match app.run().await {
        Ok(_) => {
            println!("👋 ¡Noctra finalizado correctamente!");
            ExitCode::from(0)
        }
        Err(e) => {
            error!("❌ Error crítico: {}", e);
            error!("💡 Para ayuda, prueba: noctra --help");
            ExitCode::from(1)
        }
    }
}
