//! Noctra FormLib - FDL2 (Form Definition Language) Processing
//! 
//! Maneja la carga, validación y ejecución de formularios declarativos
//! definidos en FDL2 (TOML format).

pub mod forms;
pub mod validation;
pub mod loader;

pub use forms::*;
pub use loader::{load_form, load_form_from_path};
pub use validation::ValidationError;