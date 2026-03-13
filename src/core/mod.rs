//! Core business logic (platform-agnostic)
//!
//! This module contains all the core functionality that is completely
//! independent of the UI framework and platform-specific code.

pub mod error;
pub mod ocr;
pub mod stt;
pub mod db;
pub mod vocab;
pub mod storage;
pub mod models;
pub mod state;
pub mod pdf;

pub use error::{ShuseiError, OcrError, SttError};
pub use ocr::OcrEngine;
pub use stt::SttEngine;
pub use db::Database;
pub use storage::StorageService;
pub use state::AppState;