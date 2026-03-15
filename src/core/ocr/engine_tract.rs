//! NDLOCR Engine implementation using tract-onnx

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::path::PathBuf;
use ndarray::Array4;
use image::DynamicImage;
use parking_lot::Mutex;
use tract_onnx::prelude::*;

use crate::core::error::{OcrError, Result};
use super::{OcrEngine, OcrResult, TextRegion};

/// Bounding box from detection
#[derive(Debug, Clone)]
struct DetectionBox {
    x1: f32,
    y1: f32,
    x2: f32,
    y2: f32,
    confidence: f32,
}

/// NDLOCR-Lite engine implementation using tract-onnx
#[derive(Clone, Debug)]
pub struct NdlocrEngineTract {
    /// Model directory path
    model_dir: PathBuf,
    
    /// Tract model for text detection
    detection_model: Option<Arc<crate::core::tract_utils::TractModel>>,
    
    /// Tract model for text recognition  
    recognition_model: Option<Arc<crate::core::tract_utils::TractModel>>,
    
    /// Tract model for direction classification
    direction_model: Option<Arc<crate::core::tract_utils::TractModel>>,
    
    /// Character dictionary for recognition decoding
    dictionary: Vec<String>,
    
    /// Whether the engine is initialized
    initialized: bool,
    
    /// Language setting (ja/en)
    language: String,
}

impl NdlocrEngineTract {
    /// Create a new NDLOCR engine
    pub fn new(model_dir: impl Into<PathBuf>, language: &str) -> Self {
        Self {
            model_dir: model_dir.into(),
            detection_model: None,
            recognition_model: None,
            direction_model: None,
            dictionary: Vec::new(),
            initialized: false,
            language: language.to_string(),
        }
    }
    
    /// Initialize the engine (load ONNX models)
    pub async fn initialize(&mut self) -> Result<()> {
        log::info!("Initializing NDLOCR-Lite engine (tract) from {:?}", self.model_dir);
        
        // Check if model files exist (NDLOCR-Lite models)
        let detection_model_path = self.model_dir.join("deim-s-1024x1024.onnx");
        let recognition_model_path = self.model_dir.join("parseq-ndl-16x384-50-tiny-146epoch-tegaki2.onnx");
        let direction_model_path = self.model_dir.join("direction_classifier.onnx");
        
        // Load detection model
        if detection_model_path.exists() {
            let model = crate::core::tract_utils::load_model(&detection_model_path)
                .map_err(|e| OcrError::ModelLoading(format!("Failed to load detection model: {}", e)))?;
            self.detection_model = Some(Arc::new(model));
            log::info!("Detection model loaded: {:?}", detection_model_path);
        } else {
            log::warn!("Detection model not found: {:?}", detection_model_path);
        }
        
        // Load recognition model
        if recognition_model_path.exists() {
            let model = crate::core::tract_utils::load_model(&recognition_model_path)
                .map_err(|e| OcrError::ModelLoading(format!("Failed to load recognition model: {}", e)))?;
            self.recognition_model = Some(Arc::new(model));
            log::info!("Recognition model loaded: {:?}", recognition_model_path);
        } else {
            log::warn!("Recognition model not found: {:?}", recognition_model_path);
        }
        
        // Load direction model (optional)
        if direction_model_path.exists() {
            let model = crate::core::tract_utils::load_model(&direction_model_path)
                .map_err(|e| OcrError::ModelLoading(format!("Failed to load direction model: {}", e)))?;
            self.direction_model = Some(Arc::new(model));
            log::info!("Direction model loaded: {:?}", direction_model_path);
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
        if self.detection_model.is_some() && self.recognition_model.is_some() {
            self.initialized = true;
            log::info!("NDLOCR engine (tract) initialized successfully with language: {}", self.language);
            Ok(())
        } else {
            Err(OcrError::ModelLoading("Failed to load required OCR models".into()).into())
        }
    }
    
    /// Shutdown the engine (unload models)
    pub fn shutdown(&mut self) {
        self.detection_model = None;
        self.recognition_model = None;
        self.direction_model = None;
        self.initialized = false;
        log::info!("NDLOCR engine (tract) shutdown");
    }
    
    /// Preprocess image for ONNX inference
    /// Converts image bytes to normalized tensor in NCHW format [1, 3, 1024, 1024] for NDLOCR-Lite detection
    fn preprocess_image_for_inference(&self, image_data: &[u8]) -> Result<Array4<f32>> {
        // Decode image
        let img = image::load_from_memory(image_data)
            .map_err(|e| OcrError::Preprocessing(format!("Failed to decode image: {}", e)))?;
        
        // Convert to RGB and wrap in DynamicImage for resize (NDLOCR-Lite detection model expects 3 channels)
        let rgb: DynamicImage = img.to_rgb8().into();
        
        // Resize to model input size (1024x1024 for NDLOCR-Lite detection)
        let target_size = 1024u32;
        let resized = rgb.resize(target_size, target_size, image::imageops::FilterType::Lanczos3);
        let resized_rgb = resized.to_rgb8();
        
        // Convert to ndarray and normalize to [0, 1] in NCHW format [1, 3, H, W]
        let mut tensor = Array4::<f32>::zeros((1, 3, target_size as usize, target_size as usize));
        
        for y in 0..target_size as usize {
            for x in 0..target_size as usize {
                let pixel = resized_rgb.get_pixel(x as u32, y as u32);
                tensor[[0, 0, y, x]] = pixel[0] as f32 / 255.0; // R channel
                tensor[[0, 1, y, x]] = pixel[1] as f32 / 255.0; // G channel
                tensor[[0, 2, y, x]] = pixel[2] as f32 / 255.0; // B channel
            }
        }
        
        Ok(tensor)
    }
    
    /// Postprocess detection output to extract bounding boxes
    /// PaddleOCR detection output shape: [1, num_boxes, 4] or [1, num_boxes, 5] with confidence
    fn parse_detection_output(&self, shape: &[i64], data: &[f32], confidence_threshold: f32) -> Result<Vec<DetectionBox>> {
        let mut boxes = Vec::new();
        
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
        
        log::info!("Detected {} text regions", boxes.len());
        Ok(boxes)
    }
    
    /// Extract text region from image based on bounding box
    fn extract_text_region(&self, image_data: &[u8], r#box: &DetectionBox, original_width: u32, original_height: u32) -> Result<Array4<f32>> {
        // Decode original image
        let img = image::load_from_memory(image_data)
            .map_err(|e| OcrError::Preprocessing(format!("Failed to decode image: {}", e)))?;
        
        // Scale box coordinates from detection model output (1024x1024) to original image size
        let scale_x = original_width as f32 / 1024.0;
        let scale_y = original_height as f32 / 1024.0;
        
        let x1 = (r#box.x1 * scale_x).max(0.0) as u32;
        let y1 = (r#box.y1 * scale_y).max(0.0) as u32;
        let x2 = (r#box.x2 * scale_x).min(original_width as f32 - 1.0) as u32;
        let y2 = (r#box.y2 * scale_y).min(original_height as f32 - 1.0) as u32;
        
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
    fn decode_recognition_output(&self, shape: &[i64], data: &[f32]) -> Result<(String, f32)> {
        if self.dictionary.is_empty() {
            return Ok((String::new(), 0.0));
        }
        
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
        
        Ok((String::new(), 0.0))
    }
    
    /// Run detection and recognition inference to extract text
    fn run_inference_and_extract(&self, image_bytes: &[u8]) -> Result<(Vec<String>, Vec<f32>)> {
        // Step 1: Preprocess image to tensor
        let tensor = self.preprocess_image_for_inference(image_bytes)?;
        
        // Step 2: Run detection inference
        let detection_model = self.detection_model.as_ref()
            .ok_or_else(|| OcrError::Inference("Detection model not loaded".into()))?;
        
        let det_input = crate::core::tract_utils::array4_to_tensor(&tensor)
            .map_err(|e| OcrError::Inference(format!("Failed to create detection input: {}", e)))?;
        
        let det_output = crate::core::tract_utils::run_inference(detection_model, &det_input)
            .map_err(|e| OcrError::Inference(format!("Detection inference failed: {}", e)))?;
        
        let (det_shape, det_data) = crate::core::tract_utils::extract_tensor_data(&det_output)
            .map_err(|e| OcrError::Inference(format!("Failed to extract detection output: {}", e)))?;
        
        // Step 3: Parse detection boxes
        let boxes = self.parse_detection_output(&det_shape, &det_data, 0.5)?;
        
        if boxes.is_empty() {
            log::warn!("No text regions detected");
            return Ok((Vec::new(), Vec::new()));
        }
        
        // Step 4: Run recognition on each detected region
        let mut text_lines = Vec::new();
        let mut confidences = Vec::new();
        
        // Get original image dimensions for scaling
        let img = image::load_from_memory(image_bytes)
            .map_err(|e| OcrError::Preprocessing(format!("Failed to decode image: {}", e)))?;
        let original_width = img.width();
        let original_height = img.height();
        
        // Get recognition model
        let recognition_model = self.recognition_model.as_ref()
            .ok_or_else(|| OcrError::Inference("Recognition model not loaded".into()))?;
        
        // Run recognition on each detected box
        for r#box in &boxes {
            // Extract text region from image
            let region_tensor = self.extract_text_region(image_bytes, r#box, original_width, original_height)?;
            
            // Create recognition input
            let rec_shape = [1, 1, 32, region_tensor.shape()[3]];
            let rec_input = crate::core::tract_utils::array4_to_tensor(&region_tensor)
                .map_err(|e| OcrError::Inference(format!("Failed to create recognition input: {}", e)))?;
            
            // Run recognition inference
            let rec_output = crate::core::tract_utils::run_inference(recognition_model, &rec_input)
                .map_err(|e| OcrError::Inference(format!("Recognition inference failed: {}", e)))?;
            
            let (rec_shape, rec_data) = crate::core::tract_utils::extract_tensor_data(&rec_output)
                .map_err(|e| OcrError::Inference(format!("Failed to extract recognition output: {}", e)))?;
            
            // Decode recognition output
            let (text, confidence) = self.decode_recognition_output(&rec_shape, &rec_data)?;
            if !text.is_empty() {
                text_lines.push(text);
                confidences.push(confidence);
            }
        }
        
        Ok((text_lines, confidences))
    }
}

#[async_trait]
impl OcrEngine for NdlocrEngineTract {
    async fn process_image(&self, image_data: &[u8]) -> Result<OcrResult> {
        if !self.initialized {
            return Err(OcrError::Inference("Engine not initialized".into()).into());
        }
        
        let start = std::time::Instant::now();
        
        // Run inference (tract is thread-safe, no mutex needed)
        let (text_lines, confidences) = match self.run_inference_and_extract(image_data) {
            Ok(results) => results,
            Err(e) => {
                log::warn!("OCR inference failed: {}", e);
                (Vec::new(), Vec::new())
            }
        };
        
        // Calculate average confidence
        let avg_confidence = if confidences.is_empty() {
            0.0
        } else {
            confidences.iter().sum::<f32>() / confidences.len() as f32
        };
        
        // Generate markdown output
        let markdown = text_lines.join("\n");
        let plain_text = text_lines.join(" ");
        
        let processing_time_ms = start.elapsed().as_millis() as u64;
        
        let result = OcrResult {
            markdown,
            plain_text,
            regions: Vec::new(),
            confidence: avg_confidence,
            processing_time_ms,
        };
        
        log::info!("OCR (tract) completed in {}ms, confidence: {:.2}", result.processing_time_ms, result.confidence);
        
        Ok(result)
    }
    
    fn is_ready(&self) -> bool {
        self.initialized
    }
    
    fn name(&self) -> &'static str {
        "NDLOCR-Lite (tract)"
    }
}
