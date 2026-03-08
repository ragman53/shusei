//! Android platform implementation using JNI
//!
//! This module provides Android-specific implementations for camera,
//! microphone, and other native features using JNI.

use async_trait::async_trait;
use jni::JNIEnv;
use jni::objects::{JObject, JClass, JString, JValue};
use jni::JavaVM;
use std::sync::{Arc, RwLock, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::sync::oneshot;

use crate::core::error::{ShuseiError, Result};
use super::{PlatformApi, CameraResult, AudioResult};

/// Global state for camera capture
static CAMERA_STATE: RwLock<Option<CameraState>> = RwLock::new(None);
static JAVA_VM: RwLock<Option<JavaVM>> = RwLock::new(None);

/// State for ongoing camera capture operation
struct CameraState {
    /// Channel sender to notify when capture completes
    result_sender: Option<oneshot::Sender<Result<CameraResult>>>,
}

/// Android platform implementation
pub struct AndroidPlatform {
    java_vm: Option<JavaVM>,
}

impl AndroidPlatform {
    /// Create a new Android platform instance
    pub fn new() -> Self {
        // Try to get JavaVM from global storage
        let java_vm = JAVA_VM.read()
            .ok()
            .and_then(|guard| guard.clone());
        
        Self { java_vm }
    }
    
    /// Initialize with JavaVM reference
    pub fn with_java_vm(java_vm: JavaVM) -> Self {
        // Store in global
        if let Ok(mut guard) = JAVA_VM.write() {
            *guard = Some(java_vm.clone());
        }
        Self { java_vm: Some(java_vm) }
    }
    
    /// Get JNI environment
    fn get_env(&self) -> Result<JNIEnv> {
        let java_vm = self.java_vm.as_ref()
            .ok_or_else(|| ShuseiError::Platform("JavaVM not initialized".into()))?;
        
        java_vm.attach_current_thread()
            .map_err(|e| ShuseiError::Platform(format!("Failed to get JNIEnv: {}", e)))
    }
    
    /// Find the main activity class
    fn find_activity_class<'local>(&self, env: &mut JNIEnv<'local>) -> Result<JClass<'local>> {
        env.find_class("com/shusei/app/MainActivity")
            .map_err(|e| ShuseiError::Platform(format!("Failed to find MainActivity class: {}", e)))
    }
    
    /// Call a static method on MainActivity to capture image
    fn call_capture_image(&self, env: &mut JNIEnv) -> Result<()> {
        let class = self.find_activity_class(env)?;
        
        // Call static method to start camera capture
        env.call_static_method(
            class,
            "startCameraCapture",
            "()V",
            &[],
        ).map_err(|e| ShuseiError::Platform(format!("Failed to call startCameraCapture: {}", e)))?;
        
        Ok(())
    }
    
    /// Check and request camera permission via JNI
    fn check_camera_permission(&self, env: &mut JNIEnv) -> Result<bool> {
        let class = self.find_activity_class(env)?;
        
        let result = env.call_static_method(
            class,
            "hasCameraPermission",
            "()Z",
            &[],
        ).map_err(|e| ShuseiError::Platform(format!("Failed to call hasCameraPermission: {}", e)))?;
        
        result.z().map_err(|e| ShuseiError::Platform(format!("Failed to get boolean result: {}", e)))
    }
    
    /// Request camera permission via JNI
    fn request_camera_permission_jni(&self, env: &mut JNIEnv) -> Result<()> {
        let class = self.find_activity_class(env)?;
        
        env.call_static_method(
            class,
            "requestCameraPermission",
            "()V",
            &[],
        ).map_err(|e| ShuseiError::Platform(format!("Failed to call requestCameraPermission: {}", e)))?;
        
        Ok(())
    }
}

impl Default for AndroidPlatform {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl PlatformApi for AndroidPlatform {
    async fn capture_image(&self) -> Result<CameraResult> {
        log::info!("Attempting to capture image via JNI...");
        
        // Create oneshot channel for result
        let (tx, rx) = oneshot::channel();
        
        // Store sender in global state
        {
            let mut state = CAMERA_STATE.write()
                .map_err(|_| ShuseiError::Platform("Failed to lock camera state".into()))?;
            *state = Some(CameraState {
                result_sender: Some(tx),
            });
        }
        
        // Call Java to start camera capture
        let mut env = self.get_env()?;
        self.call_capture_image(&mut env)?;
        
        // Wait for result with timeout
        let result = tokio::time::timeout(
            std::time::Duration::from_secs(30),
            rx
        ).await
            .map_err(|_| ShuseiError::Platform("Camera capture timeout".into()))?
            .map_err(|_| ShuseiError::Platform("Camera capture channel closed".into()))?;
        
        result
    }
    
    async fn record_audio(&self, max_seconds: u32) -> Result<AudioResult> {
        log::info!("Attempting to record audio via JNI (max {}s)...", max_seconds);
        
        Err(ShuseiError::Platform(
            "JNI audio recording not yet implemented.".into()
        ).into())
    }
    
    async fn pick_file(&self, _extensions: &[&str]) -> Result<String> {
        Err(ShuseiError::Platform(
            "File picker not yet implemented on Android.".into()
        ).into())
    }
    
    fn vibrate(&self, duration_ms: u32) {
        log::debug!("Vibrating for {}ms", duration_ms);
        
        // Try to call Java vibration method
        if let Ok(mut env) = self.get_env() {
            if let Ok(class) = self.find_activity_class(&mut env) {
                let _ = env.call_static_method(
                    class,
                    "vibrate",
                    "(J)V",
                    &[JValue::Long(duration_ms as i64)],
                );
            }
        }
    }
    
    async fn has_camera_permission(&self) -> bool {
        if let Ok(mut env) = self.get_env() {
            if let Ok(granted) = self.check_camera_permission(&mut env) {
                return granted;
            }
        }
        false
    }
    
    async fn has_microphone_permission(&self) -> bool {
        // TODO: Check RECORD_AUDIO permission via JNI
        false
    }
    
    async fn request_camera_permission(&self) -> Result<bool> {
        let mut env = self.get_env()?;
        self.request_camera_permission_jni(&mut env)?;
        
        // For now, assume permission request is async
        // In a full implementation, we'd wait for the callback
        Ok(true)
    }
    
    async fn request_microphone_permission(&self) -> Result<bool> {
        Ok(false)
    }
}

// ==================== JNI Native Methods ====================

/// JNI initialization - called from Java when the app starts
#[no_mangle]
pub extern "system" fn Java_com_shusei_app_MainActivity_nativeInit(
    mut env: JNIEnv,
    _class: JClass,
) {
    log::info!("nativeInit called from Java");
    
    // Store JavaVM reference for later use
    match env.get_java_vm() {
        Ok(java_vm) => {
            if let Ok(mut guard) = JAVA_VM.write() {
                *guard = Some(java_vm);
                log::info!("JavaVM reference stored successfully");
            }
        }
        Err(e) => {
            log::error!("Failed to get JavaVM: {}", e);
        }
    }
}

/// Called when camera captures an image (called from Java)
#[no_mangle]
pub extern "system" fn Java_com_shusei_app_MainActivity_onImageCaptured(
    mut env: JNIEnv,
    _class: JClass,
    image_data: jni::sys::jbyteArray,
    width: jni::sys::jint,
    height: jni::sys::jint,
) {
    log::info!("onImageCaptured: {}x{}", width, height);
    
    // Convert Java byte array to Rust Vec
    let data: Vec<u8> = match env.convert_byte_array(image_data) {
        Ok(d) => d,
        Err(e) => {
            log::error!("Failed to convert byte array: {}", e);
            // Send error result
            send_camera_result(Err(ShuseiError::Platform(format!("Failed to convert image data: {}", e))));
            return;
        }
    };
    
    log::info!("Image data size: {} bytes", data.len());
    
    // Create CameraResult
    let result = CameraResult {
        image_data: data,
        width: width as u32,
        height: height as u32,
        format: "jpeg".to_string(), // Java side sends JPEG
    };
    
    // Send result to waiting capture_image call
    send_camera_result(Ok(result));
}

/// Called when camera capture fails (called from Java)
#[no_mangle]
pub extern "system" fn Java_com_shusei_app_MainActivity_onImageCaptureFailed(
    _env: JNIEnv,
    _class: JClass,
    error_message: jni::sys::jstring,
) {
    let error_str = unsafe {
        let msg: JString = JString::from_raw(error_message);
        _env.get_string(&msg)
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_else(|_| "Unknown error".to_string())
    };
    
    log::error!("Image capture failed: {}", error_str);
    
    send_camera_result(Err(ShuseiError::Platform(error_str)));
}

/// Static method called from Java when capture fails without activity instance
#[no_mangle]
pub extern "system" fn Java_com_shusei_app_MainActivity_notifyCaptureFailed(
    _env: JNIEnv,
    _class: JClass,
    error_message: jni::sys::jstring,
) {
    let error_str = unsafe {
        let msg: JString = JString::from_raw(error_message);
        _env.get_string(&msg)
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_else(|_| "Unknown error".to_string())
    };
    
    log::error!("Capture failed (static notification): {}", error_str);
    
    send_camera_result(Err(ShuseiError::Platform(error_str)));
}

/// Helper function to send camera result through the global state
fn send_camera_result(result: Result<CameraResult>) {
    if let Ok(mut guard) = CAMERA_STATE.write() {
        if let Some(state) = guard.take() {
            if let Some(sender) = state.result_sender {
                if sender.send(result).is_err() {
                    log::error!("Failed to send camera result - receiver dropped");
                }
            }
        }
    }
}

/// Called when audio recording completes
#[no_mangle]
pub extern "system" fn Java_com_shusei_app_MainActivity_onAudioRecorded(
    mut env: JNIEnv,
    _class: JClass,
    audio_data: jni::sys::jfloatArray,
    sample_rate: jni::sys::jint,
) {
    log::info!("onAudioRecorded: {} Hz", sample_rate);
    
    // Convert Java float array to Rust Vec
    let data_len = env.get_array_length(audio_data).unwrap_or(0);
    let mut data = vec![0.0f32; data_len as usize];
    
    env.get_float_array_region(audio_data, 0, &mut data)
        .unwrap_or_else(|e| {
            log::error!("Failed to get float array: {}", e);
        });
    
    log::info!("Audio data size: {} samples", data.len());
    
    // TODO: Store the audio data for retrieval by record_audio()
}

/// Called when permission is granted/denied
#[no_mangle]
pub extern "system" fn Java_com_shusei_app_MainActivity_onPermissionResult(
    mut env: JNIEnv,
    _class: JClass,
    permission: jni::sys::jstring,
    granted: jni::sys::jboolean,
) {
    let permission: JString = unsafe { JString::from_raw(permission) };
    let permission_str: String = env.get_string(&permission)
        .unwrap_or_else(|_| "unknown".into())
        .into();
    
    let granted = granted != 0;
    
    log::info!("Permission {} : {}", permission_str, if granted { "granted" } else { "denied" });
    
    // TODO: Store permission result and notify waiting callers
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_android_platform_new() {
        let platform = AndroidPlatform::new();
        assert!(platform.java_vm.is_none());
    }
}