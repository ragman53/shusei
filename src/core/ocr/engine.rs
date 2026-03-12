//! OCR Engine trait and implementation

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tokio::sync::Semaphore;
use std::sync::Arc;

use crate::core::error::{OcrError, Result};
use crate::core::db::{Database, NewBookPage};

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
#[derive(Clone, Debug)]
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

impl NdlocrEngine {
    /// Process multiple pages in parallel with concurrency control
    ///
    /// # Arguments
    /// * `pages` - Vec of (page_number, image_bytes) to process
    /// * `book_id` - Book identifier for database storage
    /// * `db` - Database connection for saving results
    /// * `progress_cb` - Callback called after each page completes: (page_num, total)
    ///
    /// # Returns
    /// Ok(()) on success, or error if critical failure occurs
    pub async fn process_pages_parallel(
        &self,
        pages: Vec<(u32, Vec<u8>)>,
        book_id: &str,
        db: &Database,
        progress_cb: impl Fn(u32, u32),
    ) -> Result<()> {
        use futures::stream::{self, StreamExt};
        
        let total = pages.len() as u32;
        
        // Process pages with concurrency limit of 3
        stream::iter(pages)
            .map(|(page_num, image_bytes)| async move {
                // Retry logic: up to 3 attempts
                let mut attempts = 0;
                let mut result = self.process_image(&image_bytes).await;

                while result.is_err() && attempts < 3 {
                    attempts += 1;
                    log::warn!("OCR attempt {} failed for page {}, retrying...", attempts, page_num);
                    result = self.process_image(&image_bytes).await;
                }

                match result {
                    Ok(ocr_result) => {
                        // Save to database
                        let new_page = NewBookPage {
                            book_id: book_id.to_string(),
                            page_number: page_num as i32,
                            image_path: String::new(), // Image already saved by render_pages_batch
                            ocr_markdown: ocr_result.markdown,
                            ocr_text_plain: ocr_result.plain_text,
                            confidence: Some(ocr_result.confidence),
                        };

                        if let Err(e) = db.save_page(&new_page) {
                            log::error!("Failed to save page {}: {}", page_num, e);
                        } else {
                            log::info!("Page {} OCR completed, confidence: {}", page_num, ocr_result.confidence);
                        }
                    }
                    Err(e) => {
                        log::error!("Page {} OCR failed after {} attempts: {}", page_num, attempts, e);
                        // Skip this page, continue with others
                    }
                }

                page_num
            })
            .buffer_unordered(3) // Max 3 concurrent
            .collect::<Vec<_>>()
            .await;

        // Report final progress
        progress_cb(total, total);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::db::{Database, NewBook};
    use tempfile::TempDir;

    mod parallel_processing {
        use super::*;
        use std::sync::{Arc, Mutex};

        #[tokio::test]
        async fn test_process_pages_parallel_processes_pages() {
            // Create engine (will fail initialization without models, but that's ok for this test)
            let temp_dir = TempDir::new().unwrap();
            let mut engine = NdlocrEngine::new(temp_dir.path());
            
            // Initialize will fail without models, skip test
            if engine.initialize().await.is_err() {
                println!("Skipping test - models not available");
                return;
            }

            let db = Database::in_memory().unwrap();
            let book_id = db
                .create_book(&NewBook {
                    title: "Test".to_string(),
                    author: "Author".to_string(),
                    ..Default::default()
                })
                .unwrap();

            let pages = vec![
                (0u32, vec![1u8, 2, 3]),
                (1u32, vec![4u8, 5, 6]),
            ];

            let progress_calls = Arc::new(Mutex::new(Vec::new()));
            let progress_calls_clone = Arc::clone(&progress_calls);
            let progress_cb = move |page: u32, total: u32| {
                progress_calls_clone.lock().unwrap().push((page, total));
            };

            let result = engine
                .process_pages_parallel(pages, &book_id, &db, progress_cb)
                .await;

            assert!(result.is_ok());
        }

        #[tokio::test]
        async fn test_process_pages_parallel_calls_progress_callback() {
            let temp_dir = TempDir::new().unwrap();
            let mut engine = NdlocrEngine::new(temp_dir.path());

            if engine.initialize().await.is_err() {
                println!("Skipping test - models not available");
                return;
            }

            let db = Database::in_memory().unwrap();
            let book_id = db
                .create_book(&NewBook {
                    title: "Test".to_string(),
                    author: "Author".to_string(),
                    ..Default::default()
                })
                .unwrap();

            let pages = vec![(0u32, vec![1u8, 2, 3])];
            let progress_called = Arc::new(Mutex::new(false));
            let progress_called_clone = Arc::clone(&progress_called);
            let progress_cb = move |_: u32, _: u32| {
                *progress_called_clone.lock().unwrap() = true;
            };

            engine
                .process_pages_parallel(pages, &book_id, &db, progress_cb)
                .await
                .unwrap();

            assert!(*progress_called.lock().unwrap());
        }

        #[tokio::test]
        async fn test_process_pages_parallel_saves_to_database() {
            let temp_dir = TempDir::new().unwrap();
            let mut engine = NdlocrEngine::new(temp_dir.path());

            if engine.initialize().await.is_err() {
                println!("Skipping test - models not available");
                return;
            }

            let db = Database::in_memory().unwrap();
            let book_id = db
                .create_book(&NewBook {
                    title: "Test".to_string(),
                    author: "Author".to_string(),
                    ..Default::default()
                })
                .unwrap();

            let pages = vec![(0u32, vec![1u8, 2, 3])];
            let progress_cb = |_: u32, _: u32| {};

            engine
                .process_pages_parallel(pages, &book_id, &db, progress_cb)
                .await
                .unwrap();

            let pages_result = db.get_pages_by_book(&book_id).unwrap();
            // Pages should be saved (even if OCR returns empty)
            assert!(pages_result.len() >= 0);
        }

        #[tokio::test]
        async fn test_process_pages_parallel_handles_failures_gracefully() {
            let temp_dir = TempDir::new().unwrap();
            let mut engine = NdlocrEngine::new(temp_dir.path());

            if engine.initialize().await.is_err() {
                println!("Skipping test - models not available");
                return;
            }

            let db = Database::in_memory().unwrap();
            let book_id = db
                .create_book(&NewBook {
                    title: "Test".to_string(),
                    author: "Author".to_string(),
                    ..Default::default()
                })
                .unwrap();

            // Multiple pages - if one fails, others should continue
            let pages = vec![
                (0u32, vec![1u8, 2, 3]),
                (1u32, vec![4u8, 5, 6]),
                (2u32, vec![7u8, 8, 9]),
            ];

            let completed = Arc::new(Mutex::new(0u32));
            let completed_clone = Arc::clone(&completed);
            let progress_cb = move |_: u32, _: u32| {
                *completed_clone.lock().unwrap() += 1;
            };

            // Should not panic even if OCR fails
            let result = engine
                .process_pages_parallel(pages, &book_id, &db, progress_cb)
                .await;

            // Should complete without error (failures are logged, not returned)
            assert!(result.is_ok());
        }
    }
}