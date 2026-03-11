//! OCR Engine trait and implementation

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::core::error::{OcrError, Result};

/// OCR Engine trait - abstract interface for OCR processing
#[async_trait]
pub trait OcrEngine: Send + Sync {
    /// Process an image and return OCR results
    async fn process_image(&self, image_data: &[u8]) -> Result<OcrResult>;
    
    /// Check if the engine is ready
    fn is_ready(&self) -> bool;
    
    /// Get the engine name
    fn name(&self) -> &'static str;
}

/// OCR processing result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OcrResult {
    /// Markdown formatted output
    pub markdown: String,
    
    /// Plain text output (for FTS)
    pub plain_text: String,
    
    /// Detected text regions
    pub regions: Vec<TextRegion>,
    
    /// Overall confidence score
    pub confidence: f32,
    
    /// Processing time in milliseconds
    pub processing_time_ms: u64,
}

/// A detected text region in the image
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextRegion {
    /// Bounding box coordinates [x1, y1, x2, y2]
    pub bbox: [f32; 4],
    
    /// Recognized text content
    pub text: String,
    
    /// Confidence score for this region
    pub confidence: f32,
    
    /// Text direction (0, 90, 180, 270 degrees)
    pub direction: u32,
    
    /// Whether this is vertical text
    pub is_vertical: bool,
}

/// NDLOCR-Lite engine implementation using tract
pub struct NdlocrEngine {
    /// Model directory path
    model_dir: std::path::PathBuf,
    
    /// Whether the engine is initialized
    initialized: bool,
}

impl NdlocrEngine {
    /// Create a new NDLOCR engine
    pub fn new(model_dir: impl Into<std::path::PathBuf>) -> Self {
        Self {
            model_dir: model_dir.into(),
            initialized: false,
        }
    }
    
    /// Initialize the engine (load models)
    pub async fn initialize(&mut self) -> Result<()> {
        // TODO: Load ONNX models using tract
        // This will be implemented in Week 3-5
        
        log::info!("Initializing NDLOCR engine from {:?}", self.model_dir);
        
        // Check if model files exist
        let detection_model = self.model_dir.join("text_detection.onnx");
        let recognition_model = self.model_dir.join("text_recognition.onnx");
        let direction_model = self.model_dir.join("direction_classifier.onnx");
        
        if !detection_model.exists() {
            return Err(OcrError::ModelLoading(format!(
                "Detection model not found: {:?}",
                detection_model
            )).into());
        }
        
        if !recognition_model.exists() {
            return Err(OcrError::ModelLoading(format!(
                "Recognition model not found: {:?}",
                recognition_model
            )).into());
        }
        
        if !direction_model.exists() {
            log::warn!("Direction classifier model not found, direction classification will be disabled");
        }
        
        self.initialized = true;
        log::info!("NDLOCR engine initialized successfully");
        
        Ok(())
    }
    
    /// Shutdown the engine (unload models)
    pub fn shutdown(&mut self) {
        self.initialized = false;
        log::info!("NDLOCR engine shutdown");
    }
}

#[async_trait]
impl OcrEngine for NdlocrEngine {
    async fn process_image(&self, image_data: &[u8]) -> Result<OcrResult> {
        if !self.initialized {
            return Err(OcrError::Inference("Engine not initialized".into()).into());
        }
        
        let start = std::time::Instant::now();
        
        // Step 1: Preprocess image (downscale to 2MP, enhance contrast)
        let processed_data = super::preprocess::preprocess_image(image_data)?;
        
        // TODO: Implement full OCR pipeline with tract-onnx
        // 2. Detect text regions using text_detection.onnx
        // 3. Classify direction using direction_classifier.onnx  
        // 4. Recognize text using text_recognition.onnx
        // 5. Sort by reading order
        // 6. Generate markdown
        
        // For now, return placeholder with preprocessing completed
        let result = OcrResult {
            markdown: String::new(),
            plain_text: String::new(),
            regions: Vec::new(),
            confidence: 0.0,
            processing_time_ms: start.elapsed().as_millis() as u64,
        };
        
        log::info!("OCR preprocessing completed in {}ms", result.processing_time_ms);
        
        Ok(result)
    }
    
    fn is_ready(&self) -> bool {
        self.initialized
    }
    
    fn name(&self) -> &'static str {
        "NDLOCR-Lite"
    }
}