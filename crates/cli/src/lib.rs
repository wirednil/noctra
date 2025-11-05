//! Noctra CLI - Command Line Interface
//!
//! CLI interactivo y batch para Noctra con REPL, batch processing,
//! form execution y daemon mode.

pub mod cli;
pub mod repl;
pub mod commands;
pub mod config;
pub mod output;
pub mod app;

pub use cli::{NoctraApp, build_cli};
pub use repl::{Repl, ReplConfig, ReplHandler};
pub use commands::{execute_command, CommandResult, CommandContext};
pub use config::{CliConfig, GlobalConfig};
pub use output::{OutputFormatter, TableFormatter, CsvFormatter, JsonFormatter};
pub use app::{NoctraApp as App, NoctraArgs, build_cli as build_app};