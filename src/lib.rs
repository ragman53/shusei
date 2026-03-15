//! Shusei - Offline reading app with OCR and STT capabilities
//!
//! Library module exposing core functionality.

pub mod app;
pub mod core;
pub mod platform;
pub mod ui;

// Re-export commonly used types
pub use core::error::{ShuseiError, OcrError, SttError};
pub use core::ocr::OcrEngine;
pub use core::stt::SttEngine;
pub use core::db::Database;
pub use platform::PlatformApi;