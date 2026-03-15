//! Moonshine STT Engine implementation using tract-onnx

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use ndarray::Array2;
use std::sync::Arc;
use std::path::PathBuf;
use tract_onnx::prelude::*;

use crate::core::error::{SttError, Result};
use super::{SttEngine, SttResult, Language, AudioPreprocessor};

/// Moonshine Tiny engine implementation using tract-onnx
pub struct MoonshineEngineTract {
    /// Model directory path
    model_dir: PathBuf,
    
    /// Current language
    language: Language,
    
    /// Whether the engine is initialized
    initialized: bool,
    
    /// Tract model for encoder
    encoder_model: Option<Arc<crate::core::tract_utils::TractModel>>,
    
    /// Tract model for decoder
    decoder_model: Option<Arc<crate::core::tract_utils::TractModel>>,
    
    /// Audio preprocessor (mel-spectrogram)
    preprocessor: AudioPreprocessor,
}

impl MoonshineEngineTract {
    /// Create a new Moonshine engine
    pub fn new(model_dir: impl Into<PathBuf>, language: Language) -> Self {
        Self {
            model_dir: model_dir.into(),
            language,
            initialized: false,
            encoder_model: None,
            decoder_model: None,
            preprocessor: AudioPreprocessor::new(),
        }
    }
    
    /// Initialize the engine (load models)
    pub async fn initialize(&mut self) -> Result<()> {
        log::info!(
            "Initializing Moonshine engine (tract) for {:?} from {:?}",
            self.language,
            self.model_dir
        );
        
        // Check if model files exist
        let encoder_model_path = self.model_dir.join("encoder.onnx");
        let decoder_model_path = self.model_dir.join("decoder.onnx");
        
        // Load encoder model
        if encoder_model_path.exists() {
            let model = crate::core::tract_utils::load_model(&encoder_model_path)
                .map_err(|e| SttError::ModelLoading(format!("Failed to load encoder model: {}", e)))?;
            self.encoder_model = Some(Arc::new(model));
            log::info!("Encoder model loaded: {:?}", encoder_model_path);
        } else {
            return Err(SttError::ModelLoading(format!(
                "Encoder model not found: {:?}",
                encoder_model_path
            )).into());
        }
        
        // Load decoder model
        if decoder_model_path.exists() {
            let model = crate::core::tract_utils::load_model(&decoder_model_path)
                .map_err(|e| SttError::ModelLoading(format!("Failed to load decoder model: {}", e)))?;
            self.decoder_model = Some(Arc::new(model));
            log::info!("Decoder model loaded: {:?}", decoder_model_path);
        } else {
            return Err(SttError::ModelLoading(format!(
                "Decoder model not found: {:?}",
                decoder_model_path
            )).into());
        }
        
        self.initialized = true;
        log::info!("Moonshine engine (tract) initialized successfully");
        
        Ok(())
    }
    
    /// Shutdown the engine (unload models)
    pub fn shutdown(&mut self) {
        self.encoder_model = None;
        self.decoder_model = None;
        self.initialized = false;
        log::info!("Moonshine engine (tract) shutdown");
    }
    
    /// Switch language
    pub fn set_language(&mut self, language: Language) {
        self.language = language;
    }
    
    /// Run encoder on mel-spectrogram
    fn run_encoder(&self, mel_spec: &Array2<f32>) -> Result<Array2<f32>> {
        let encoder_model = self.encoder_model.as_ref()
            .ok_or_else(|| SttError::Encoder("Encoder model not loaded".into()))?;
        
        // Convert mel spectrogram to tensor
        let input_tensor = crate::core::tract_utils::array2_to_tensor(mel_spec)
            .map_err(|e| SttError::Preprocessing(format!("Failed to create encoder input: {}", e)))?;
        
        // Run inference
        let output = crate::core::tract_utils::run_inference(encoder_model, &input_tensor)
            .map_err(|e| SttError::Inference(format!("Encoder inference failed: {}", e)))?;
        
        // Extract output
        let (_shape, data) = crate::core::tract_utils::extract_tensor_data(&output)
            .map_err(|e| SttError::Inference(format!("Failed to extract encoder output: {}", e)))?;
        
        // Reshape output - encoder output shape is [batch, seq_len, hidden_dim]
        // For simplicity, return as 2D array
        let output_rows = output.shape()[1] as usize;
        let output_cols = output.shape()[2] as usize;
        
        let output_array = Array2::from_shape_vec((output_rows, output_cols), data)
            .map_err(|e| SttError::Inference(format!("Failed to reshape encoder output: {}", e)))?;
        
        Ok(output_array)
    }
    
    /// Run decoder autoregressively
    fn run_decoder(&self, encoder_output: &Array2<f32>, max_steps: usize) -> Result<Vec<i32>> {
        let decoder_model = self.decoder_model.as_ref()
            .ok_or_else(|| SttError::Decoder("Decoder model not loaded".into()))?;
        
        // For now, return placeholder tokens
        // Full autoregressive decoding with KV cache is complex
        // This is a simplified implementation
        log::debug!("Running decoder for up to {} steps", max_steps);
        
        // TODO: Implement full autoregressive decoding
        // For S07, we're proving tract can load and run the models
        // Full decoding logic will be refined in S08
        
        Ok(vec![])  // Placeholder - returns empty tokens
    }
}

#[async_trait]
impl SttEngine for MoonshineEngineTract {
    async fn transcribe(&self, audio: &[f32]) -> Result<SttResult> {
        if !self.initialized {
            return Err(SttError::Decoder("Engine not initialized".into()).into());
        }
        
        let start = std::time::Instant::now();
        
        // Calculate audio duration
        let audio_duration_seconds = audio.len() as f32 / 16000.0; // Assuming 16kHz sample rate
        
        // Step 1: Preprocess audio to mel-spectrogram
        let mel_spec = self.preprocessor.preprocess(audio)
            .map_err(|e| SttError::Preprocessing(format!("Mel-spectrogram preprocessing failed: {}", e)))?;
        
        log::debug!("Mel-spectrogram shape: {:?}", mel_spec.dim());
        
        // Step 2: Run encoder
        let encoder_output = match self.run_encoder(&mel_spec) {
            Ok(output) => output,
            Err(e) => {
                log::warn!("Encoder failed: {}", e);
                return Ok(SttResult {
                    text: String::new(),
                    confidence: None,
                    processing_time_ms: start.elapsed().as_millis() as u64,
                    audio_duration_seconds,
                    language: self.language,
                });
            }
        };
        
        // Step 3: Run decoder (autoregressive)
        let tokens = match self.run_decoder(&encoder_output, 512) {
            Ok(tokens) => tokens,
            Err(e) => {
                log::warn!("Decoder failed: {}", e);
                vec![]
            }
        };
        
        // Step 4: Decode tokens to text
        // For now, return placeholder since we don't have tokenizer integration yet
        let text = if tokens.is_empty() {
            String::new()
        } else {
            // TODO: Integrate tokenizer for token-to-text conversion
            format!("[{} tokens]", tokens.len())
        };
        
        let result = SttResult {
            text,
            confidence: None,
            processing_time_ms: start.elapsed().as_millis() as u64,
            audio_duration_seconds,
            language: self.language,
        };
        
        log::info!(
            "STT (tract) completed in {}ms, audio duration: {:.2}s",
            result.processing_time_ms,
            result.audio_duration_seconds
        );
        
        Ok(result)
    }
    
    fn is_ready(&self) -> bool {
        self.initialized
    }
    
    fn name(&self) -> &'static str {
        "Moonshine Tiny (tract)"
    }
    
    fn language(&self) -> Language {
        self.language
    }
}
