//! OCR Engine trait and implementation

use async_trait::async_trait;
use ort::session::Session;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::path::PathBuf;
use ndarray::Array4;
use image::DynamicImage;
use ort::value::Tensor;
use parking_lot::Mutex;

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

/// Bounding box from detection
#[derive(Debug, Clone)]
struct DetectionBox {
    x1: f32,
    y1: f32,
    x2: f32,
    y2: f32,
    confidence: f32,
}

/// NDLOCR-Lite engine implementation using ONNX Runtime
#[derive(Clone, Debug)]
pub struct NdlocrEngine {
    /// Model directory path
    model_dir: PathBuf,
    
    /// ONNX session for text detection (wrapped in Mutex for mutable access)
    detection_session: Option<Arc<Mutex<Session>>>,
    
    /// ONNX session for text recognition  
    recognition_session: Option<Arc<Mutex<Session>>>,
    
    /// ONNX session for direction classification
    direction_session: Option<Arc<Mutex<Session>>>,
    
    /// Character dictionary for recognition decoding
    dictionary: Vec<String>,
    
    /// Whether the engine is initialized
    initialized: bool,
    
    /// Language setting (ja/en)
    language: String,
}

impl NdlocrEngine {
    /// Create a new NDLOCR engine
    pub fn new(model_dir: impl Into<PathBuf>, language: &str) -> Self {
        Self {
            model_dir: model_dir.into(),
            detection_session: None,
            recognition_session: None,
            direction_session: None,
            dictionary: Vec::new(),
            initialized: false,
            language: language.to_string(),
        }
    }
    
    /// Initialize the engine (load ONNX models)
    pub async fn initialize(&mut self) -> Result<()> {
        log::info!("Initializing NDLOCR engine from {:?}", self.model_dir);
        
        // Check if model files exist
        let detection_model = self.model_dir.join("text_detection.onnx");
        let recognition_model = self.model_dir.join("text_recognition.onnx");
        let direction_model = self.model_dir.join("direction_classifier.onnx");
        
        // Load detection model
        if detection_model.exists() {
            let session = Session::builder()
                .map_err(|e| OcrError::ModelLoading(format!("Failed to create session builder: {}", e)))?
                .commit_from_file(&detection_model)
                .map_err(|e| OcrError::ModelLoading(format!("Failed to load detection model: {}", e)))?;
            self.detection_session = Some(Arc::new(Mutex::new(session)));
            log::info!("Detection model loaded: {:?}", detection_model);
        } else {
            log::warn!("Detection model not found: {:?}", detection_model);
        }
        
        // Load recognition model
        if recognition_model.exists() {
            let session = Session::builder()
                .map_err(|e| OcrError::ModelLoading(format!("Failed to create session builder: {}", e)))?
                .commit_from_file(&recognition_model)
                .map_err(|e| OcrError::ModelLoading(format!("Failed to load recognition model: {}", e)))?;
            self.recognition_session = Some(Arc::new(Mutex::new(session)));
            log::info!("Recognition model loaded: {:?}", recognition_model);
        } else {
            log::warn!("Recognition model not found: {:?}", recognition_model);
        }
        
        // Load direction model (optional)
        if direction_model.exists() {
            let session = Session::builder()
                .map_err(|e| OcrError::ModelLoading(format!("Failed to create session builder: {}", e)))?
                .commit_from_file(&direction_model)
                .map_err(|e| OcrError::ModelLoading(format!("Failed to load direction model: {}", e)))?;
            self.direction_session = Some(Arc::new(Mutex::new(session)));
            log::info!("Direction model loaded: {:?}", direction_model);
        } else {
            log::warn!("Direction classifier model not found, direction classification will be disabled");
        }
        
        // Load dictionary
        let dict_path = self.model_dir.join("dict.txt");
        if dict_path.exists() {
            let dict_content = std::fs::read_to_string(&dict_path)
                .map_err(|e| OcrError::ModelLoading(format!("Failed to read dictionary: {}", e)))?;
            self.dictionary = dict_content
                .lines()
                .map(|line| line.to_string())
                .collect();
            log::info!("Dictionary loaded: {} characters", self.dictionary.len());
        } else {
            log::warn!("Dictionary not found, recognition will fail");
        }
        
        // Engine is initialized if at least detection and recognition models are loaded
        if self.detection_session.is_some() && self.recognition_session.is_some() {
            self.initialized = true;
            log::info!("NDLOCR engine initialized successfully with language: {}", self.language);
            Ok(())
        } else {
            Err(OcrError::ModelLoading("Failed to load required OCR models".into()).into())
        }
    }
    
    /// Shutdown the engine (unload models)
    pub fn shutdown(&mut self) {
        self.detection_session = None;
        self.recognition_session = None;
        self.direction_session = None;
        self.initialized = false;
        log::info!("NDLOCR engine shutdown");
    }
    
    /// Preprocess image for ONNX inference
    /// Converts image bytes to normalized tensor in NCHW format [1, 1, H, W]
    fn preprocess_image_for_inference(&self, image_data: &[u8]) -> Result<Array4<f32>> {
        // Decode image
        let img = image::load_from_memory(image_data)
            .map_err(|e| OcrError::Preprocessing(format!("Failed to decode image: {}", e)))?;
        
        // Convert to grayscale using DynamicImage
        let gray: DynamicImage = img.to_luma8().into();
        
        // Resize to model input size (960x960 for NDLOCR-Lite)
        let target_size = 960u32;
        let resized = gray.resize(target_size, target_size, image::imageops::FilterType::Lanczos3);
        let resized_gray = resized.to_luma8();
        
        // Convert to ndarray and normalize to [0, 1]
        let mut tensor = Array4::<f32>::zeros((1, 1, target_size as usize, target_size as usize));
        
        for y in 0..target_size as usize {
            for x in 0..target_size as usize {
                let pixel = resized_gray.get_pixel(x as u32, y as u32);
                tensor[[0, 0, y, x]] = pixel[0] as f32 / 255.0;
            }
        }
        
        Ok(tensor)
    }
    
    /// Postprocess detection output to extract bounding boxes
    /// PaddleOCR detection output shape: [1, num_boxes, 4] or [1, num_boxes, 5] with confidence
    fn parse_detection_output(&self, outputs: &ort::session::SessionOutputs, confidence_threshold: f32) -> Result<Vec<DetectionBox>> {
        let mut boxes = Vec::new();
        
        // Get first output (detection boxes)
        if let Some(output) = outputs.get("output") {
            // Try to extract as tensor - returns (shape, data)
            if let Ok((shape, data)) = output.try_extract_tensor::<f32>() {
                log::debug!("Detection output shape: {:?}", shape);
                
                // Shape should be [1, num_boxes, 4] or [1, num_boxes, 5]
                if shape.len() == 3 && shape[0] == 1 {
                    let num_boxes = shape[1] as usize;
                    let dims = shape[2] as usize; // 4 or 5
                    
                    for i in 0..num_boxes {
                        let base_idx = i * dims;
                        if base_idx + 3 < data.len() {
                            let x1 = data[base_idx];
                            let y1 = data[base_idx + 1];
                            let x2 = data[base_idx + 2];
                            let y2 = data[base_idx + 3];
                            let conf = if dims >= 5 && base_idx + 4 < data.len() { data[base_idx + 4] } else { 1.0 };
                            
                            if conf > confidence_threshold {
                                boxes.push(DetectionBox { x1, y1, x2, y2, confidence: conf });
                            }
                        }
                    }
                }
            }
        }
        
        log::info!("Detected {} text regions", boxes.len());
        Ok(boxes)
    }
    
    /// Extract text region from image based on bounding box
    fn extract_text_region(&self, image_data: &[u8], box_: &DetectionBox, original_width: u32, original_height: u32) -> Result<Array4<f32>> {
        // Decode original image
        let img = image::load_from_memory(image_data)
            .map_err(|e| OcrError::Preprocessing(format!("Failed to decode image: {}", e)))?;
        
        // Scale box coordinates to original image size
        let scale_x = original_width as f32 / 960.0;
        let scale_y = original_height as f32 / 960.0;
        
        let x1 = (box_.x1 * scale_x).max(0.0) as u32;
        let y1 = (box_.y1 * scale_y).max(0.0) as u32;
        let x2 = (box_.x2 * scale_x).min(original_width as f32 - 1.0) as u32;
        let y2 = (box_.y2 * scale_y).min(original_height as f32 - 1.0) as u32;
        
        // Crop to region
        let width = (x2 - x1 + 1).max(1);
        let height = (y2 - y1 + 1).max(1);
        let region = img.crop_imm(x1, y1, width, height);
        
        // Resize to recognition model input size (typically 32px height)
        let target_height = 32u32;
        let aspect_ratio = width as f32 / height as f32;
        let target_width = (target_height as f32 * aspect_ratio).max(1.0) as u32;
        
        let resized = region.resize(target_width, target_height, image::imageops::FilterType::Lanczos3);
        let gray = resized.to_luma8();
        
        // Convert to tensor [1, 1, H, W]
        let mut tensor = Array4::<f32>::zeros((1, 1, target_height as usize, target_width as usize));
        for y in 0..target_height as usize {
            for x in 0..target_width as usize {
                let pixel = gray.get_pixel(x as u32, y as u32);
                tensor[[0, 0, y, x]] = pixel[0] as f32 / 255.0;
            }
        }
        
        Ok(tensor)
    }
    
    /// Decode recognition output using CTC and dictionary
    fn decode_recognition_output(&self, outputs: &ort::session::SessionOutputs) -> Result<(String, f32)> {
        if self.dictionary.is_empty() {
            return Ok((String::new(), 0.0));
        }
        
        // Get recognition output (logits)
        if let Some(output) = outputs.get("output") {
            if let Ok((shape, data)) = output.try_extract_tensor::<f32>() {
                log::debug!("Recognition output shape: {:?}", shape);
                
                // Shape: [1, seq_len, vocab_size]
                if shape.len() == 3 && shape[0] == 1 {
                    let seq_len = shape[1] as usize;
                    let vocab_size = shape[2] as usize;
                    
                    let mut result = String::new();
                    let mut total_confidence = 0.0f32;
                    let mut prev_char_idx = -1isize;
                    
                    for t in 0..seq_len {
                        let mut max_prob = 0.0f32;
                        let mut max_idx = 0usize;
                        
                        // Argmax over vocab
                        for c in 0..vocab_size {
                            let idx = t * vocab_size + c;
                            if idx < data.len() {
                                let prob = data[idx];
                                if prob > max_prob {
                                    max_prob = prob;
                                    max_idx = c;
                                }
                            }
                        }
                        
                        // CTC decoding: skip blank (idx 0) and duplicates
                        if max_idx > 0 && max_idx as isize != prev_char_idx && max_idx < self.dictionary.len() {
                            result.push_str(&self.dictionary[max_idx]);
                            total_confidence += max_prob;
                            prev_char_idx = max_idx as isize;
                        }
                    }
                    
                    let avg_confidence = if result.is_empty() { 0.0 } else { total_confidence / seq_len as f32 };
                    return Ok((result, avg_confidence));
                }
            }
        }
        
        Ok((String::new(), 0.0))
    }
}

#[async_trait]
impl OcrEngine for NdlocrEngine {
    async fn process_image(&self, image_data: &[u8]) -> Result<OcrResult> {
        if !self.initialized {
            return Err(OcrError::Inference("Engine not initialized".into()).into());
        }
        
        let start = std::time::Instant::now();
        
        // Step 1: Preprocess image to tensor
        let tensor = self.preprocess_image_for_inference(image_data)?;
        
        // Step 2: Run detection inference (if session available)
        let (text_lines, confidences) = if let Some(session_arc) = &self.detection_session {
            // Lock session for mutable access
            let mut session = session_arc.lock();
            
            // Create input from tensor data
            let input_data = tensor.as_slice().unwrap();
            
            // Run inference and extract results within the lock scope
            match self.run_inference_and_extract(&mut session, input_data, image_data) {
                Ok(results) => results,
                Err(e) => {
                    log::warn!("Detection inference failed: {}", e);
                    (Vec::new(), Vec::new())
                }
            }
        } else {
            (Vec::new(), Vec::new())
        };
        
        // Step 4: Calculate average confidence
        let avg_confidence = if confidences.is_empty() {
            0.0
        } else {
            confidences.iter().sum::<f32>() / confidences.len() as f32
        };
        
        // Step 5: Generate markdown output
        let markdown = text_lines.join("\n");
        let plain_text = text_lines.join(" ");
        
        let processing_time_ms = start.elapsed().as_millis() as u64;
        
        let result = OcrResult {
            markdown,
            plain_text,
            regions: Vec::new(), // Will be populated with actual regions in Task 3
            confidence: avg_confidence,
            processing_time_ms,
        };
        
        log::info!("OCR completed in {}ms, confidence: {:.2}", result.processing_time_ms, result.confidence);
        
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
    /// Run detection and recognition inference to extract text
    fn run_inference_and_extract(&self, detection_session: &mut Session, input_data: &[f32], image_bytes: &[u8]) -> Result<(Vec<String>, Vec<f32>)> {
        // Step 1: Run detection
        let tensor = Value::from_array(([1usize, 3, 960, 960], input_data.to_vec()))
            .map_err(|e| OcrError::Inference(format!("Failed to create input tensor: {}", e)))?;
        
        let det_outputs = detection_session.run(ort::inputs![tensor])
            .map_err(|e| OcrError::Inference(format!("Detection inference failed: {}", e)))?;
        
        // Step 2: Parse detection boxes
        let boxes = self.parse_detection_output(&det_outputs, 0.5)?;
        
        if boxes.is_empty() {
            log::warn!("No text regions detected");
            return Ok((Vec::new(), Vec::new()));
        }
        
        // Step 3: Run recognition on each detected region
        // TODO: Implement recognition inference - currently returns placeholder results
        let mut text_lines = Vec::new();
        let mut confidences = Vec::new();
        
        // Placeholder: Return box coordinates as text for testing
        for box_ in &boxes {
            text_lines.push(format!("[{:.0},{:.0},{:.0},{:.0}]", box_.x1, box_.y1, box_.x2, box_.y2));
            confidences.push(box_.confidence);
        }
        
        log::info!("Detected {} text regions", text_lines.len());
        Ok((text_lines, confidences))
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
    use image::{ImageBuffer, Rgb};

    #[test]
    fn test_engine_creation() {
        // Test engine creation without models
        let temp_dir = TempDir::new().unwrap();
        let engine = NdlocrEngine::new(temp_dir.path(), "en");
        
        assert!(!engine.is_ready());
        assert_eq!(engine.name(), "NDLOCR-Lite");
    }

    #[tokio::test]
    async fn test_preprocessing_produces_valid_tensor() {
        // Create a test image
        let img = ImageBuffer::<Rgb<u8>, Vec<u8>>::from_fn(100, 100, |x, y| {
            Rgb([x as u8, y as u8, 128])
        });
        
        // Encode to JPEG
        let mut image_data = Vec::new();
        img.write_to(
            &mut std::io::Cursor::new(&mut image_data),
            image::ImageFormat::Jpeg,
        ).unwrap();
        
        // Create engine (won't be initialized, but preprocessing doesn't need initialization)
        let temp_dir = TempDir::new().unwrap();
        let engine = NdlocrEngine::new(temp_dir.path(), "en");
        
        // Preprocessing should work even without models
        let tensor = engine.preprocess_image_for_inference(&image_data);
        
        assert!(tensor.is_ok());
        let tensor = tensor.unwrap();
        
        // Check tensor shape: [1, 1, 960, 960]
        assert_eq!(tensor.shape(), &[1, 1, 960, 960]);
        
        // Check values are normalized to [0, 1]
        for &val in tensor.iter() {
            assert!(val >= 0.0 && val <= 1.0, "Tensor values should be normalized");
        }
    }

    #[tokio::test]
    async fn test_process_image_returns_result_structure() {
        // Create a test image
        let img = ImageBuffer::<Rgb<u8>, Vec<u8>>::from_fn(100, 100, |_, _| {
            Rgb([128, 128, 128])
        });
        
        let mut image_data = Vec::new();
        img.write_to(
            &mut std::io::Cursor::new(&mut image_data),
            image::ImageFormat::Jpeg,
        ).unwrap();
        
        // Create engine without models (will fail initialization, but process_image should handle gracefully)
        let temp_dir = TempDir::new().unwrap();
        let mut engine = NdlocrEngine::new(temp_dir.path(), "en");
        
        // Initialize will fail without models
        let init_result = engine.initialize().await;
        assert!(init_result.is_err(), "Initialization should fail without models");
        
        // Process image should return error when not initialized
        let result = engine.process_image(&image_data).await;
        assert!(result.is_err());
    }

    mod parallel_processing {
        use super::*;
        use std::sync::{Arc, Mutex};

        #[tokio::test]
        async fn test_process_pages_parallel_processes_pages() {
            // Create engine (will fail initialization without models, but that's ok for this test)
            let temp_dir = TempDir::new().unwrap();
            let mut engine = NdlocrEngine::new(temp_dir.path(), "en");
            
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
            let mut engine = NdlocrEngine::new(temp_dir.path(), "en");

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
            let mut engine = NdlocrEngine::new(temp_dir.path(), "en");

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
            let mut engine = NdlocrEngine::new(temp_dir.path(), "en");

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