//! Post-processing for OCR results
//!
//! This module handles text detection, recognition, direction classification,
//! and quality assessment for OCR results.

use image::GenericImageView;
use ndarray::Array3;

use super::{OcrResult, TextRegion};
use crate::core::error::{OcrError, Result};

/// Quality score thresholds
const BLUR_THRESHOLD: f32 = 100.0;
const BRIGHTNESS_MIN: f32 = 50.0;
const BRIGHTNESS_MAX: f32 = 200.0;
const OCR_CONFIDENCE_RETRY: f32 = 0.5;
const CRITICAL_REGION_CONFIDENCE: f32 = 0.3;

/// Calculate quality score for an image
///
/// Returns a score from 0.0 to 1.0 based on:
/// - Blur detection using Laplacian variance
/// - Brightness assessment (not too dark, not too bright)
///
/// # Arguments
/// * `image_data` - Raw image bytes
///
/// # Returns
/// Quality score (0.0 = poor, 1.0 = excellent)
pub fn calculate_quality_score(image_data: &[u8]) -> Result<f32> {
    // Decode image
    let img = image::load_from_memory(image_data)
        .map_err(|e| OcrError::Preprocessing(format!("Failed to decode image: {}", e)))?;

    // Convert to grayscale for analysis
    let gray = img.to_luma8();

    // Calculate blur score using Laplacian variance approximation
    let blur_score = calculate_laplacian_variance(&gray);

    // Calculate brightness
    let brightness = calculate_mean_brightness(&gray);

    // Combine scores
    let blur_quality = if blur_score < BLUR_THRESHOLD {
        // Blurry image
        (blur_score / BLUR_THRESHOLD) * 0.5
    } else {
        0.5 + ((blur_score - BLUR_THRESHOLD) / BLUR_THRESHOLD).min(0.5)
    };

    let brightness_quality = if brightness < BRIGHTNESS_MIN {
        // Too dark
        (brightness / BRIGHTNESS_MIN) * 0.5
    } else if brightness > BRIGHTNESS_MAX {
        // Too bright
        ((255.0 - brightness) / (255.0 - BRIGHTNESS_MAX)) * 0.5
    } else {
        0.5 + ((brightness - BRIGHTNESS_MIN) / (BRIGHTNESS_MAX - BRIGHTNESS_MIN)) * 0.5
    };

    // Combined score (weighted average)
    let combined = (blur_quality * 0.6) + (brightness_quality * 0.4);

    Ok(combined.clamp(0.0, 1.0))
}

/// Calculate Laplacian variance for blur detection
fn calculate_laplacian_variance(img: &image::ImageBuffer<image::Luma<u8>, Vec<u8>>) -> f32 {
    let (width, height) = img.dimensions();

    if width < 3 || height < 3 {
        return 0.0;
    }

    // Simple Laplacian approximation using convolution
    let mut sum = 0.0f32;
    let mut sum_sq = 0.0f32;
    let mut count = 0;

    for y in 1..(height - 1) {
        for x in 1..(width - 1) {
            let center = img.get_pixel(x, y)[0] as f32;
            let left = img.get_pixel(x - 1, y)[0] as f32;
            let right = img.get_pixel(x + 1, y)[0] as f32;
            let top = img.get_pixel(x, y - 1)[0] as f32;
            let bottom = img.get_pixel(x, y + 1)[0] as f32;

            // Laplacian: 4*center - left - right - top - bottom
            let laplacian = 4.0 * center - left - right - top - bottom;

            sum += laplacian;
            sum_sq += laplacian * laplacian;
            count += 1;
        }
    }

    if count == 0 {
        return 0.0;
    }

    let mean = sum / count as f32;
    let variance = (sum_sq / count as f32) - (mean * mean);

    variance.abs()
}

/// Calculate mean brightness of an image
fn calculate_mean_brightness(img: &image::ImageBuffer<image::Luma<u8>, Vec<u8>>) -> f32 {
    let (width, height) = img.dimensions();
    let mut sum = 0.0f32;
    let mut count = 0;

    for pixel in img.pixels() {
        sum += pixel[0] as f32;
        count += 1;
    }

    if count == 0 {
        return 0.0;
    }

    sum / count as f32
}

/// Determine if OCR result should be retried
///
/// # Arguments
/// * `ocr_result` - OCR result to evaluate
///
/// # Returns
/// true if confidence is too low and retry might help
pub fn should_retry(ocr_result: &OcrResult) -> bool {
    // Check overall confidence
    if ocr_result.confidence < OCR_CONFIDENCE_RETRY {
        return true;
    }

    // Check for critical low-confidence regions
    for region in &ocr_result.regions {
        if region.confidence < CRITICAL_REGION_CONFIDENCE {
            return true;
        }
    }

    false
}

/// Detect text regions in an image
pub fn detect_text(_tensor: &Array3<f32>, _threshold: f32) -> Result<Vec<TextRegion>> {
    // TODO: Implement text detection using tract
    // This will use the text_detection.onnx model

    log::debug!("Detecting text regions...");

    // Placeholder - return empty result
    Ok(Vec::new())
}

/// Recognize text in a region
pub fn recognize_text(_tensor: &Array3<f32>, _region: &TextRegion) -> Result<String> {
    // TODO: Implement text recognition using tract
    // This will use the text_recognition.onnx model

    log::debug!("Recognizing text in region...");

    // Placeholder - return empty string
    Ok(String::new())
}

/// Classify text direction
pub fn classify_direction(_tensor: &Array3<f32>, _region: &TextRegion) -> Result<u32> {
    // TODO: Implement direction classification using tract
    // This will use the direction_classifier.onnx model
    // Returns 0, 90, 180, or 270 degrees

    log::debug!("Classifying text direction...");

    // Placeholder - return 0 degrees
    Ok(0)
}

/// Non-Maximum Suppression for text detection
pub fn nms(regions: &mut Vec<TextRegion>, iou_threshold: f32) {
    // Sort by confidence (descending)
    regions.sort_by(|a, b| {
        b.confidence
            .partial_cmp(&a.confidence)
            .unwrap_or(std::cmp::Ordering::Equal)
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
    use image::{ImageBuffer, Luma};

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

    #[test]
    fn test_calculate_mean_brightness() {
        // Create test image with known brightness
        let img = ImageBuffer::<Luma<u8>, Vec<u8>>::from_fn(10, 10, |_, _| Luma([128]));

        let brightness = calculate_mean_brightness(&img);
        assert!(
            (brightness - 128.0).abs() < 0.1,
            "Brightness should be ~128"
        );
    }

    #[test]
    fn test_calculate_quality_score_good_image() {
        // Create a sharp, well-lit test image
        let img = ImageBuffer::<Luma<u8>, Vec<u8>>::from_fn(100, 100, |x, y| {
            // Checkerboard pattern for high variance
            Luma([if (x + y) % 2 == 0 { 200 } else { 55 }])
        });

        // Encode to JPEG
        let mut data = Vec::new();
        img.write_to(
            &mut std::io::Cursor::new(&mut data),
            image::ImageFormat::Jpeg,
        )
        .unwrap();

        let score = calculate_quality_score(&data).unwrap();
        assert!(
            score > 0.5,
            "Good quality image should score > 0.5, got {}",
            score
        );
    }

    #[test]
    fn test_should_retry_low_confidence() {
        let result = OcrResult {
            markdown: String::new(),
            plain_text: String::new(),
            regions: vec![],
            confidence: 0.3,
            processing_time_ms: 100,
        };

        assert!(
            should_retry(&result),
            "Should retry with low overall confidence"
        );
    }

    #[test]
    fn test_should_retry_critical_region() {
        let result = OcrResult {
            markdown: String::new(),
            plain_text: String::new(),
            regions: vec![TextRegion {
                bbox: [0.0, 0.0, 10.0, 10.0],
                text: "test".to_string(),
                confidence: 0.2,
                direction: 0,
                is_vertical: false,
            }],
            confidence: 0.7,
            processing_time_ms: 100,
        };

        assert!(
            should_retry(&result),
            "Should retry with low-confidence critical region"
        );
    }

    #[test]
    fn test_should_retry_good_result() {
        let result = OcrResult {
            markdown: String::new(),
            plain_text: String::new(),
            regions: vec![TextRegion {
                bbox: [0.0, 0.0, 10.0, 10.0],
                text: "test".to_string(),
                confidence: 0.8,
                direction: 0,
                is_vertical: false,
            }],
            confidence: 0.85,
            processing_time_ms: 100,
        };

        assert!(
            !should_retry(&result),
            "Should not retry with good confidence"
        );
    }
}
