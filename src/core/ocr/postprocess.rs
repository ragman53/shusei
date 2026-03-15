//! Post-processing for OCR results
//!
//! This module handles text detection, recognition, and direction classification.

use ndarray::Array3;

use crate::core::error::{OcrError, Result};
use super::TextRegion;

/// Detect text regions in an image
pub fn detect_text(
    _tensor: &Array3<f32>,
    _threshold: f32,
) -> Result<Vec<TextRegion>> {
    // TODO: Implement text detection using tract
    // This will use the text_detection.onnx model
    
    log::debug!("Detecting text regions...");
    
    // Placeholder - return empty result
    Ok(Vec::new())
}

/// Recognize text in a region
pub fn recognize_text(
    _tensor: &Array3<f32>,
    _region: &TextRegion,
) -> Result<String> {
    // TODO: Implement text recognition using tract
    // This will use the text_recognition.onnx model
    
    log::debug!("Recognizing text in region...");
    
    // Placeholder - return empty string
    Ok(String::new())
}

/// Classify text direction
pub fn classify_direction(
    _tensor: &Array3<f32>,
    _region: &TextRegion,
) -> Result<u32> {
    // TODO: Implement direction classification using tract
    // This will use the direction_classifier.onnx model
    // Returns 0, 90, 180, or 270 degrees
    
    log::debug!("Classifying text direction...");
    
    // Placeholder - return 0 degrees
    Ok(0)
}

/// Non-Maximum Suppression for text detection
pub fn nms(
    regions: &mut Vec<TextRegion>,
    iou_threshold: f32,
) {
    // Sort by confidence (descending)
    regions.sort_by(|a, b| {
        b.confidence.partial_cmp(&a.confidence).unwrap_or(std::cmp::Ordering::Equal)
    });
    
    let mut keep = vec![true; regions.len()];
    
    for i in 0..regions.len() {
        if !keep[i] {
            continue;
        }
        
        for j in (i + 1)..regions.len() {
            if !keep[j] {
                continue;
            }
            
            let iou = compute_iou(&regions[i].bbox, &regions[j].bbox);
            if iou > iou_threshold {
                keep[j] = false;
            }
        }
    }
    
    // Remove suppressed regions
    let mut idx = 0;
    regions.retain(|_| {
        let k = keep[idx];
        idx += 1;
        k
    });
}

/// Compute Intersection over Union for two bounding boxes
fn compute_iou(bbox1: &[f32; 4], bbox2: &[f32; 4]) -> f32 {
    let x1 = bbox1[0].max(bbox2[0]);
    let y1 = bbox1[1].max(bbox2[1]);
    let x2 = bbox1[2].min(bbox2[2]);
    let y2 = bbox1[3].min(bbox2[3]);
    
    if x2 <= x1 || y2 <= y1 {
        return 0.0;
    }
    
    let intersection = (x2 - x1) * (y2 - y1);
    
    let area1 = (bbox1[2] - bbox1[0]) * (bbox1[3] - bbox1[1]);
    let area2 = (bbox2[2] - bbox2[0]) * (bbox2[3] - bbox2[1]);
    
    let union = area1 + area2 - intersection;
    
    if union <= 0.0 {
        return 0.0;
    }
    
    intersection / union
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_compute_iou_no_overlap() {
        let bbox1 = [0.0, 0.0, 10.0, 10.0];
        let bbox2 = [20.0, 20.0, 30.0, 30.0];
        
        let iou = compute_iou(&bbox1, &bbox2);
        assert_eq!(iou, 0.0);
    }
    
    #[test]
    fn test_compute_iou_full_overlap() {
        let bbox1 = [0.0, 0.0, 10.0, 10.0];
        let bbox2 = [0.0, 0.0, 10.0, 10.0];
        
        let iou = compute_iou(&bbox1, &bbox2);
        assert_eq!(iou, 1.0);
    }
    
    #[test]
    fn test_compute_iou_partial_overlap() {
        let bbox1 = [0.0, 0.0, 10.0, 10.0];
        let bbox2 = [5.0, 5.0, 15.0, 15.0];
        
        let iou = compute_iou(&bbox1, &bbox2);
        assert!(iou > 0.0 && iou < 1.0);
    }
}