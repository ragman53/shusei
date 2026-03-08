//! iOS platform implementation (stub for future development)
//!
//! This module will provide iOS-specific implementations using Objective-C bridges.

use async_trait::async_trait;

use crate::core::error::{ShuseiError, Result};
use super::{PlatformApi, CameraResult, AudioResult};

/// iOS platform implementation
pub struct IosPlatform;

impl IosPlatform {
    pub fn new() -> Self {
        Self
    }
}

impl Default for IosPlatform {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl PlatformApi for IosPlatform {
    async fn capture_image(&self) -> Result<CameraResult> {
        // TODO: Implement using AVCaptureSession
        Err(ShuseiError::Platform(
            "iOS camera not yet implemented. This will be added in post-MVP.".into()
        ).into())
    }
    
    async fn record_audio(&self, _max_seconds: u32) -> Result<AudioResult> {
        // TODO: Implement using AVAudioRecorder
        Err(ShuseiError::Platform(
            "iOS audio recording not yet implemented.".into()
        ).into())
    }
    
    async fn pick_file(&self, _extensions: &[&str]) -> Result<String> {
        // TODO: Implement using UIDocumentPickerViewController
        Err(ShuseiError::Platform(
            "iOS file picker not yet implemented.".into()
        ).into())
    }
    
    fn vibrate(&self, _duration_ms: u32) {
        // TODO: Implement using UIImpactFeedbackGenerator
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