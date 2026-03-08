//! OCR (Optical Character Recognition) pipeline
//!
//! This module implements the OCR pipeline using NDLOCR-Lite ONNX models
//! with the tract inference runtime.

mod engine;
mod preprocess;
mod postprocess;
mod markdown;

pub use engine::{OcrEngine, OcrResult, TextRegion, NdlocrEngine};
pub use preprocess::{preprocess_image, PreprocessConfig};
pub use postprocess::{detect_text, recognize_text, classify_direction};
pub use markdown::{generate_markdown, ReadingOrder};

use crate::core::error::OcrError;

/// OCR pipeline configuration
#[derive(Debug, Clone)]
pub struct OcrConfig {
    /// Maximum image dimension (longer side)
    pub max_image_size: u32,
    
    /// Confidence threshold for text detection
    pub detection_threshold: f32,
    
    /// Confidence threshold for text recognition
    pub recognition_threshold: f32,
    
    /// Enable direction classification
    pub enable_direction_classification: bool,
}

impl Default for OcrConfig {
    fn default() -> Self {
        Self {
            max_image_size: 1024,
            detection_threshold: 0.5,
            recognition_threshold: 0.5,
            enable_direction_classification: true,
        }
    }
}