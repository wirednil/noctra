//! Noctra TUI - Terminal User Interface
//!
//! Interfaz de usuario para terminal con componentes para formularios,
//! tablas de resultados y navegación interactiva.

pub mod components;
pub mod form_renderer;
pub mod layout;
pub mod noctra_tui;
pub mod nwm;
pub mod renderer;
pub mod widgets;

pub use components::*;
pub use form_renderer::{FormRenderError, FormRenderer};
pub use layout::LayoutManager;
pub use noctra_tui::{NoctraTui, QueryResults};
pub use nwm::{NoctraWindowManager, NwmConfig, NwmWindow, UiMode, WindowContent};
pub use renderer::{TuiApp, TuiConfig, TuiConfigBuilder, TuiRenderer};
