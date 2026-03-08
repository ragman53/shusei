//! UI components for the Shusei application
//!
//! This module contains all Dioxus UI components.

mod camera;
mod notes;
mod reader;
mod vocab;
mod components;

pub use camera::CameraPage;
pub use notes::NotesPage;
pub use reader::ReaderPage;
pub use vocab::VocabPage;
pub use components::*;