//! Image preprocessing for OCR

use image::{GenericImageView, ImageError, ImageFormat};

use crate::core::error::{OcrError, Result};

/// Maximum total pixels for OCR processing (2 megapixels)
const MAX_TOTAL_PIXELS: u32 = 2_000_000;

/// Preprocessing configuration
#[derive(Debug, Clone)]
pub struct PreprocessConfig {
    /// Maximum total pixels (width * height)
    pub max_total_pixels: u32,

    /// Normalize pixel values to [0, 1] range
    pub normalize: bool,

    /// Convert to grayscale
    pub grayscale: bool,

    /// Apply contrast enhancement
    pub enhance_contrast: bool,
}

impl Default for PreprocessConfig {
    fn default() -> Self {
        Self {
            max_total_pixels: MAX_TOTAL_PIXELS,
            normalize: true,
            grayscale: false,
            enhance_contrast: true,
        }
    }
}

/// Preprocess an image for OCR
///
/// This function:
/// 1. Decodes JPEG image data
/// 2. Calculates new dimensions maintaining aspect ratio (max 2MP total pixels)
/// 3. Resizes using Lanczos3 filter
/// 4. Applies auto-enhancement: convert to grayscale, increase contrast
/// 5. Returns processed image as JPEG bytes
pub fn preprocess_image(image_data: &[u8]) -> Result<Vec<u8>> {
    // Load image from bytes
    let img = image::load_from_memory(image_data)
        .map_err(|e: ImageError| OcrError::Preprocessing(e.to_string()))?;

    // Get original dimensions
    let (width, height) = img.dimensions();
    let total_pixels = width * height;

    // Calculate new dimensions if image exceeds 2MP
    let (new_width, new_height) = if total_pixels > MAX_TOTAL_PIXELS {
        let scale = (MAX_TOTAL_PIXELS as f32 / total_pixels as f32).sqrt();
        (
            (width as f32 * scale) as u32,
            (height as f32 * scale) as u32,
        )
    } else {
        (width, height)
    };

    // Resize using Lanczos3 for quality
    let resized = img.resize(new_width, new_height, image::imageops::FilterType::Lanczos3);

    // Convert to grayscale for better OCR
    let gray = resized.to_luma8();

    // Apply contrast enhancement
    let enhanced = if true {
        // Always enhance for OCR
        enhance_contrast(&gray)
    } else {
        gray
    };

    // Convert back to JPEG bytes at 85% quality
    let mut output_bytes = Vec::new();
    enhanced
        .write_to(
            &mut std::io::Cursor::new(&mut output_bytes),
            ImageFormat::Jpeg,
        )
        .map_err(|e| OcrError::Preprocessing(format!("Failed to encode JPEG: {}", e)))?;

    Ok(output_bytes)
}

/// Enhance contrast of a grayscale image
fn enhance_contrast(
    img: &image::ImageBuffer<image::Luma<u8>, Vec<u8>>,
) -> image::ImageBuffer<image::Luma<u8>, Vec<u8>> {
    // Simple histogram-based contrast enhancement
    let mut histogram = [0u32; 256];

    // Build histogram
    for pixel in img.pixels() {
        histogram[pixel[0] as usize] += 1;
    }

    // Find min and max non-zero bins
    let mut min_val = 0u8;
    let mut max_val = 255u8;

    for (i, &count) in histogram.iter().enumerate() {
        if count > 0 {
            min_val = i as u8;
            break;
        }
    }

    for (i, &count) in histogram.iter().enumerate().rev() {
        if count > 0 {
            max_val = i as u8;
            break;
        }
    }

    // Apply contrast stretch
    if max_val > min_val {
        let scale = 255.0 / (max_val as f32 - min_val as f32);
        let mut result = image::ImageBuffer::new(img.width(), img.height());
        for (x, y, pixel) in img.enumerate_pixels() {
            let val = pixel[0] as f32;
            let enhanced = ((val - min_val as f32) * scale).clamp(0.0, 255.0) as u8;
            result.put_pixel(x, y, image::Luma([enhanced]));
        }
        result
    } else {
        img.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_preprocess_config_default() {
        let config = PreprocessConfig::default();
        assert_eq!(config.max_total_pixels, MAX_TOTAL_PIXELS);
        assert!(config.normalize);
        assert!(config.enhance_contrast);
    }

    #[test]
    fn test_downscale_large_image() {
        // Create a test image larger than 2MP (e.g., 2000x2000 = 4MP)
        let img = image::ImageBuffer::<image::Rgb<u8>, Vec<u8>>::from_fn(2000, 2000, |x, y| {
            image::Rgb([x as u8, y as u8, 128])
        });

        // Encode to JPEG
        let mut image_data = Vec::new();
        img.write_to(
            &mut std::io::Cursor::new(&mut image_data),
            ImageFormat::Jpeg,
        )
        .unwrap();

        // Preprocess should downscale to < 2MP
        let result = preprocess_image(&image_data).unwrap();
        assert!(result.len() > 0);

        // Decode result to verify dimensions
        let result_img = image::load_from_memory(&result).unwrap();
        let (w, h) = result_img.dimensions();
        assert!(
            w * h <= MAX_TOTAL_PIXELS,
            "Downscaled image should be <= 2MP"
        );
    }

    #[test]
    fn test_small_image_passes_through() {
        // Create a small test image (100x100 = 10KB pixels, well under 2MP)
        let img = image::ImageBuffer::<image::Rgb<u8>, Vec<u8>>::from_fn(100, 100, |x, y| {
            image::Rgb([x as u8, y as u8, 128])
        });

        let mut image_data = Vec::new();
        img.write_to(
            &mut std::io::Cursor::new(&mut image_data),
            ImageFormat::Jpeg,
        )
        .unwrap();

        let result = preprocess_image(&image_data).unwrap();
        assert!(result.len() > 0);

        // Decode result to verify dimensions are similar
        let result_img = image::load_from_memory(&result).unwrap();
        let (w, h) = result_img.dimensions();
        // Should be close to original (may vary slightly due to JPEG compression)
        assert!(
            w > 90 && w < 110,
            "Small image should maintain similar dimensions"
        );
        assert!(
            h > 90 && h < 110,
            "Small image should maintain similar dimensions"
        );
    }

    #[test]
    fn test_preprocess_returns_jpeg() {
        // Create test image
        let img = image::ImageBuffer::<image::Rgb<u8>, Vec<u8>>::from_fn(500, 500, |_, _| {
            image::Rgb([128, 128, 128])
        });

        let mut image_data = Vec::new();
        img.write_to(
            &mut std::io::Cursor::new(&mut image_data),
            ImageFormat::Jpeg,
        )
        .unwrap();

        let result = preprocess_image(&image_data).unwrap();

        // Verify output is valid JPEG by checking magic bytes
        assert!(result.len() > 2, "Output should have data");
        // JPEG files start with 0xFFD8
        assert_eq!(result[0], 0xFF, "Should start with JPEG marker");
        assert_eq!(result[1], 0xD8, "Should be JPEG format");
    }
}
