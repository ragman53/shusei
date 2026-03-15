//! UI components for the Shusei application
//!
//! This module contains all Dioxus UI components.

mod add_book;
mod camera;
mod library;
mod notes;
mod reader;
mod vocab;
mod components;

pub use add_book::AddBookForm;
pub use camera::CameraPage;
pub use library::LibraryScreen;
pub use notes::NotesPage;
pub use reader::ReaderPage;
pub use vocab::VocabPage;
pub use components::*;