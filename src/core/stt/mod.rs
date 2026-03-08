//! STT (Speech-to-Text) pipeline
//!
//! This module implements the STT pipeline using Moonshine Tiny ONNX models
//! with the tract inference runtime.

mod engine;
mod decoder;
mod tokenizer;

pub use engine::{SttEngine, SttResult, MoonshineEngine};
pub use decoder::{DecoderState, KvCache};
pub use tokenizer::Tokenizer;

use crate::core::error::SttError;
use serde::{Deserialize, Serialize};

/// Supported languages for STT
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Language {
    English,
    Japanese,
}

impl Default for Language {
    fn default() -> Self {
        Language::Japanese
    }
}

/// STT configuration
#[derive(Debug, Clone)]
pub struct SttConfig {
    /// Maximum audio duration in seconds
    pub max_duration_seconds: u32,
    
    /// Sample rate (Moonshine expects 16kHz)
    pub sample_rate: u32,
    
    /// Number of audio channels (1 for mono)
    pub channels: u16,
    
    /// Language to use for transcription
    pub language: Language,
    
    /// Maximum number of decoder steps
    pub max_decoder_steps: usize,
}

impl Default for SttConfig {
    fn default() -> Self {
        Self {
            max_duration_seconds: 30,
            sample_rate: 16000,
            channels: 1,
            language: Language::Japanese,
            max_decoder_steps: 512,
        }
    }
}