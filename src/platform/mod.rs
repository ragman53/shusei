//! Platform abstraction layer
//!
//! This module provides platform-specific implementations for camera,
//! microphone, and file access across different platforms.

#[cfg(target_os = "android")]
pub mod android;

#[cfg(target_os = "ios")]
pub mod ios;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::core::error::Result;

/// Platform API trait - abstract interface for platform-specific functionality
#[async_trait]
pub trait PlatformApi: Send + Sync {
    /// Capture an image from the camera
    async fn capture_image(&self) -> Result<CameraResult>;
    
    /// Record audio from the microphone
    async fn record_audio(&self, max_seconds: u32) -> Result<AudioResult>;
    
    /// Pick a file using the system file picker
    async fn pick_file(&self, extensions: &[&str]) -> Result<String>;
    
    /// Vibrate the device
    fn vibrate(&self, duration_ms: u32);
    
    /// Check if camera permission is granted
    async fn has_camera_permission(&self) -> bool;
    
    /// Check if microphone permission is granted
    async fn has_microphone_permission(&self) -> bool;
    
    /// Request camera permission
    async fn request_camera_permission(&self) -> Result<bool>;
    
    /// Request microphone permission
    async fn request_microphone_permission(&self) -> Result<bool>;
}

/// Camera capture result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CameraResult {
    /// Image data in PNG or JPEG format
    pub image_data: Vec<u8>,
    
    /// Image width
    pub width: u32,
    
    /// Image height
    pub height: u32,
    
    /// Image format (e.g., "png", "jpeg")
    pub format: String,
}

/// Audio recording result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioResult {
    /// Audio samples (16kHz mono PCM as f32)
    pub samples: Vec<f32>,
    
    /// Sample rate
    pub sample_rate: u32,
    
    /// Duration in seconds
    pub duration_seconds: f32,
}

/// Desktop platform implementation (for testing/development)
pub struct DesktopPlatform;

impl DesktopPlatform {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl PlatformApi for DesktopPlatform {
    async fn capture_image(&self) -> Result<CameraResult> {
        use crate::core::error::ShuseiError;
        Err(ShuseiError::Platform("Camera not available on desktop".into()).into())
    }
    
    async fn record_audio(&self, _max_seconds: u32) -> Result<AudioResult> {
        use crate::core::error::ShuseiError;
        Err(ShuseiError::Platform("Audio recording not implemented on desktop".into()).into())
    }
    
    async fn pick_file(&self, _extensions: &[&str]) -> Result<String> {
        use crate::core::error::ShuseiError;
        Err(ShuseiError::Platform("File picker not implemented".into()).into())
    }
    
    fn vibrate(&self, _duration_ms: u32) {
        // No vibration on desktop
    }
    
    async fn has_camera_permission(&self) -> bool {
        false
    }
    
    async fn has_microphone_permission(&self) -> bool {
        false
    }
    
    async fn request_camera_permission(&self) -> Result<bool> {
        Ok(false)
    }
    
    async fn request_microphone_permission(&self) -> Result<bool> {
        Ok(false)
    }
}

/// Get the platform-specific API implementation
#[cfg(target_os = "android")]
pub fn get_platform_api() -> impl PlatformApi {
    android::AndroidPlatform::new()
}

#[cfg(target_os = "ios")]
pub fn get_platform_api() -> impl PlatformApi {
    ios::IosPlatform::new()
}

#[cfg(not(any(target_os = "android", target_os = "ios")))]
pub fn get_platform_api() -> impl PlatformApi {
    DesktopPlatform::new()
}