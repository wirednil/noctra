//! Noctra TUI - Terminal User Interface
//!
//! Interfaz de usuario para terminal con componentes para formularios,
//! tablas de resultados y navegación interactiva.

pub mod components;
pub mod layout;
pub mod renderer;
pub mod widgets;

pub use components::*;
pub use layout::LayoutManager;
pub use renderer::{TuiApp, TuiConfig, TuiConfigBuilder, TuiRenderer};
