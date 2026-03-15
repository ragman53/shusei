//! OCR Engine trait and implementation
//! 
//! Note: In S07, NdlocrEngine is now an alias for NdlocrEngineTract (tract-onnx backend).
//! The original ort-based implementation has been migrated to tract to resolve linker issues.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::core::error::Result;

/// OCR Engine trait - abstract interface for OCR processing
#[async_trait]
pub trait OcrEngine: Send + Sync {
    /// Process an image and return OCR results
    async fn process_image(&self, image_data: &[u8]) -> Result<OcrResult>;
    
    /// Check if the engine is ready
    fn is_ready(&self) -> bool;
    
    /// Get the engine name
    fn name(&self) -> &'static str;
    
    /// Process multiple pages in parallel with progress callback
    async fn process_pages_parallel<F>(
        &self,
        pages: Vec<(u32, Vec<u8>)>,  // (page_number, image_data)
        book_id: &str,
        db: &crate::core::db::Database,
        progress_cb: F,
    ) -> Result<()>
    where
        F: Fn(u32, f32) + Send + Sync + 'static,
    {
        // Default implementation: process sequentially
        for (page_num, image_data) in pages {
            let _result = self.process_image(&image_data).await?;
            progress_cb(page_num, 0.0);
        }
        Ok(())
    }
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

// NdlocrEngine is now re-exported from engine_tract.rs
// See src/core/ocr/mod.rs for the actual implementation
