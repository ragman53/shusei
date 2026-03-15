//! Image preprocessing for OCR

use image::{DynamicImage, GenericImageView, ImageError};
use ndarray::{Array3, Axis};

use crate::core::error::{OcrError, Result};

/// Preprocessing configuration
#[derive(Debug, Clone)]
pub struct PreprocessConfig {
    /// Maximum image dimension (longer side)
    pub max_size: u32,
    
    /// Normalize pixel values to [0, 1] range
    pub normalize: bool,
    
    /// Convert to grayscale
    pub grayscale: bool,
}

impl Default for PreprocessConfig {
    fn default() -> Self {
        Self {
            max_size: 1024,
            normalize: true,
            grayscale: false,
        }
    }
}

/// Preprocess an image for OCR
pub fn preprocess_image(image_data: &[u8], config: &PreprocessConfig) -> Result<Array3<f32>> {
    // Load image from bytes
    let img = image::load_from_memory(image_data)
        .map_err(|e: ImageError| OcrError::Preprocessing(e.to_string()))?;
    
    // Resize if necessary
    let (width, height) = img.dimensions();
    let (new_width, new_height) = if width > config.max_size || height > config.max_size {
        let scale = config.max_size as f32 / width.max(height) as f32;
        ((width as f32 * scale) as u32, (height as f32 * scale) as u32)
    } else {
        (width, height)
    };
    
    let resized = img.resize(new_width, new_height, image::imageops::FilterType::Lanczos3);
    
    // Convert to tensor
    let tensor = if config.grayscale {
        image_to_grayscale_tensor(&resized)
    } else {
        image_to_rgb_tensor(&resized)
    }?;
    
    // Normalize if configured
    let tensor = if config.normalize {
        normalize_tensor(&tensor)
    } else {
        tensor
    };
    
    Ok(tensor)
}

/// Convert RGB image to tensor (C, H, W format)
fn image_to_rgb_tensor(img: &DynamicImage) -> Result<Array3<f32>> {
    let (width, height) = img.dimensions();
    let mut tensor = Array3::<f32>::zeros((3, height as usize, width as usize));
    
    for (x, y, pixel) in img.pixels() {
        let r = pixel[0] as f32;
        let g = pixel[1] as f32;
        let b = pixel[2] as f32;
        
        tensor[[0, y as usize, x as usize]] = r;
        tensor[[1, y as usize, x as usize]] = g;
        tensor[[2, y as usize, x as usize]] = b;
    }
    
    Ok(tensor)
}

/// Convert image to grayscale tensor (1, H, W format)
fn image_to_grayscale_tensor(img: &DynamicImage) -> Result<Array3<f32>> {
    let gray = img.to_luma8();
    let (width, height) = gray.dimensions();
    let mut tensor = Array3::<f32>::zeros((1, height as usize, width as usize));
    
    for (x, y, pixel) in gray.enumerate_pixels() {
        tensor[[0, y as usize, x as usize]] = pixel[0] as f32;
    }
    
    Ok(tensor)
}

/// Normalize tensor values to [0, 1] range
fn normalize_tensor(tensor: &Array3<f32>) -> Array3<f32> {
    let max = tensor.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
    let min = tensor.iter().cloned().fold(f32::INFINITY, f32::min);
    
    if max == min {
        return tensor.clone();
    }
    
    tensor.mapv(|v| (v - min) / (max - min))
}

/// Apply standard normalization (ImageNet mean/std)
pub fn normalize_imagenet(tensor: &Array3<f32>) -> Array3<f32> {
    let mean = [0.485, 0.456, 0.406];
    let std = [0.229, 0.224, 0.225];
    
    let mut result = tensor.clone();
    
    for (c, (m, s)) in mean.iter().zip(std.iter()).enumerate() {
        let mut channel = result.index_axis_mut(Axis(0), c);
        channel.mapv_inplace(|v| (v / 255.0 - m) / s);
    }
    
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_preprocess_config_default() {
        let config = PreprocessConfig::default();
        assert_eq!(config.max_size, 1024);
        assert!(config.normalize);
        assert!(!config.grayscale);
    }
}