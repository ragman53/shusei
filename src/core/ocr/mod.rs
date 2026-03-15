//! OCR (Optical Character Recognition) pipeline
//!
//! This module implements the OCR pipeline using NDLOCR-Lite ONNX models
//! with the ort inference runtime.

mod engine;
mod preprocess;
mod postprocess;
mod markdown;

pub use engine::{OcrEngine, OcrResult, TextRegion, NdlocrEngine};
pub use preprocess::{preprocess_image, PreprocessConfig};
pub use postprocess::{detect_text, recognize_text, classify_direction};
pub use markdown::{generate_markdown, ReadingOrder};

use crate::core::error::OcrError;
use std::path::PathBuf;

/// Path to the text detection ONNX model (NDLOCR-Lite DEIMv2)
pub const MODEL_DETECTION_PATH: &str = "assets/ocr/models/deim-s-1024x1024.onnx";

/// Path to the text recognition ONNX model (NDLOCR-Lite PARSeq)
pub const MODEL_RECOGNITION_PATH: &str = "assets/ocr/models/parseq-ndl-16x384-50-tiny-146epoch-tegaki2.onnx";

// MODEL_DICT_PATH no longer needed - PARSeq has internal tokenizer

/// Get the full path for a bundled OCR model
pub fn get_model_path(model_name: &str) -> PathBuf {
    match model_name {
        "detection" => PathBuf::from(MODEL_DETECTION_PATH),
        "recognition" => PathBuf::from(MODEL_RECOGNITION_PATH),
        _ => PathBuf::from(format!("assets/ocr/models/{}.onnx", model_name)),
    }
}

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