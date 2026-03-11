//! UI components for the Shusei application
//!
//! This module contains all Dioxus UI components.

mod add_book;
mod camera;
mod components;
mod library;
mod notes;
mod reader;
mod vocab;

pub use add_book::AddBookForm;
pub use camera::CameraPage;
pub use components::*;
pub use library::LibraryScreen;
pub use notes::NotesPage;
pub use reader::ReaderPage;
pub use vocab::VocabPage;