//! Noctra CLI - Command Line Interface
//!
//! CLI interactivo y batch para Noctra con REPL, batch processing,
//! form execution y daemon mode.

pub mod app;
pub mod cli;
pub mod commands;
pub mod config;
pub mod interactive_form;
pub mod output;
pub mod repl;

pub use app::{build_cli as build_app, NoctraApp as App};
pub use cli::{build_cli, NoctraApp, NoctraArgs, ReplArgs};
pub use commands::{execute_command, CommandContext, CommandResult};
pub use config::{CliConfig, GlobalConfig};
pub use interactive_form::InteractiveFormExecutor;
pub use output::{format_result_set, CsvFormatter, JsonFormatter, OutputFormatter, TableFormatter};
pub use repl::{Repl, ReplHandler};
