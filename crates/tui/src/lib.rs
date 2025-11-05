//! Noctra TUI - Terminal User Interface
//! 
//! Interfaz de usuario para terminal con componentes para formularios,
//! tablas de resultados y navegación interactiva.

pub mod components;
pub mod widgets;
pub mod renderer;
pub mod layout;

pub use components::*;
pub use renderer::TuiRenderer;
pub use layout::LayoutManager;