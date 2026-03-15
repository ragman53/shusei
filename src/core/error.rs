//! Error types for the Shusei application
//!
//! This module defines all error types used throughout the application.

use thiserror::Error;

/// Top-level error type for the Shusei application
#[derive(Error, Debug)]
pub enum ShuseiError {
    #[error("OCR error: {0}")]
    Ocr(#[from] OcrError),

    #[error("STT error: {0}")]
    Stt(#[from] SttError),

    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Platform error: {0}")]
    Platform(String),

    #[error("Model not found: {0}")]
    ModelNotFound(String),

    #[error("Image processing error: {0}")]
    ImageProcessing(String),

    #[error("Audio processing error: {0}")]
    AudioProcessing(String),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Storage error: {0}")]
    Storage(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

impl From<anyhow::Error> for ShuseiError {
    fn from(err: anyhow::Error) -> Self {
        ShuseiError::Storage(err.to_string())
    }
}

/// OCR-specific errors
#[derive(Error, Debug)]
pub enum OcrError {
    #[error("Image preprocessing failed: {0}")]
    Preprocessing(String),

    #[error("Model loading failed: {0}")]
    ModelLoading(String),

    #[error("Text detection failed: {0}")]
    Detection(String),

    #[error("Text recognition failed: {0}")]
    Recognition(String),

    #[error("Direction classification failed: {0}")]
    DirectionClassification(String),

    #[error("Reading order sorting failed: {0}")]
    ReadingOrder(String),

    #[error("Markdown generation failed: {0}")]
    MarkdownGeneration(String),

    #[error("Inference failed: {0}")]
    Inference(String),

    #[error("Invalid image format: {0}")]
    InvalidFormat(String),

    #[error("ONNX operation not supported: {0}")]
    UnsupportedOperation(String),
}

/// STT-specific errors
#[derive(Error, Debug)]
pub enum SttError {
    #[error("Audio preprocessing failed: {0}")]
    Preprocessing(String),

    #[error("Model loading failed: {0}")]
    ModelLoading(String),

    #[error("Encoder inference failed: {0}")]
    Encoder(String),

    #[error("Decoder inference failed: {0}")]
    Decoder(String),

    #[error("Tokenization failed: {0}")]
    Tokenization(String),

    #[error("KV cache error: {0}")]
    KvCache(String),

    #[error("Invalid audio format: {0}")]
    InvalidFormat(String),

    #[error("Audio too long: {0} seconds (max: {1})")]
    AudioTooLong(f32, u32),

    #[error("ONNX operation not supported: {0}")]
    UnsupportedOperation(String),
}

/// Result type alias for Shusei operations
pub type Result<T> = std::result::Result<T, ShuseiError>;
