//! Noctra CLI - Command Line Interface
//!
//! CLI interactivo y batch para Noctra con REPL, batch processing,
//! form execution y daemon mode.

pub mod app;
pub mod cli;
pub mod commands;
pub mod config;
pub mod output;
pub mod repl;

pub use app::{build_cli as build_app, NoctraApp as App, NoctraArgs};
pub use cli::{build_cli, NoctraApp};
pub use commands::{execute_command, CommandContext, CommandResult};
pub use config::{CliConfig, GlobalConfig};
pub use output::{CsvFormatter, JsonFormatter, OutputFormatter, TableFormatter};
pub use repl::{Repl, ReplHandler};
