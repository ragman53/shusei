//! STT Engine trait and Moonshine implementation
//!
//! Note: In S07, MoonshineEngine is now an alias for MoonshineEngineTract (tract-onnx backend).
//! The original ort-based implementation has been migrated to tract to resolve linker issues.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use ndarray::Array2;

use crate::core::error::{SttError, Result};
use super::Language;

/// STT Engine trait - abstract interface for speech-to-text
#[async_trait]
pub trait SttEngine: Send + Sync {
    /// Transcribe audio data and return text
    async fn transcribe(&self, audio: &[f32]) -> Result<SttResult>;
    
    /// Check if the engine is ready
    fn is_ready(&self) -> bool;
    
    /// Get the engine name
    fn name(&self) -> &'static str;
    
    /// Get the current language
    fn language(&self) -> Language;
}

/// STT processing result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SttResult {
    /// Transcribed text
    pub text: String,
    
    /// Confidence score (if available)
    pub confidence: Option<f32>,
    
    /// Processing time in milliseconds
    pub processing_time_ms: u64,
    
    /// Audio duration in seconds
    pub audio_duration_seconds: f32,
    
    /// Language used for transcription
    pub language: Language,
}

// MoonshineEngine is now re-exported from engine_tract.rs
// See src/core/stt/mod.rs for the actual implementation
