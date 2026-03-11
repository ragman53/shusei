//! Android platform implementation using JNI

use async_trait::async_trait;
use jni::JNIEnv;
use jni::objects::{JClass, JValue, JByteArray, JObject};
use jni::JavaVM;
use std::sync::Mutex;
use tokio::sync::oneshot;
use once_cell::sync::Lazy;
use std::path::PathBuf;

use crate::core::error::{ShuseiError, Result};
use super::{PlatformApi, CameraResult, AudioResult};

static CAMERA_STATE: Mutex<Option<CameraState>> = Mutex::new(None);

static JAVA_VM: Lazy<Mutex<Option<JavaVM>>> = Lazy::new(|| Mutex::new(None));

struct CameraState {
    result_sender: Option<oneshot::Sender<Result<CameraResult>>>,
}

pub struct AndroidPlatform;

impl AndroidPlatform {
    pub fn new() -> Self {
        Self
    }
    
    pub fn with_java_vm(java_vm: JavaVM) -> Self {
        if let Ok(mut guard) = JAVA_VM.lock() {
            *guard = Some(java_vm);
            log::info!("JavaVM stored successfully");
        }
        Self
    }
    
    fn with_env<F, T>(&self, f: F) -> Result<T>
    where
        F: FnOnce(&mut JNIEnv) -> Result<T>,
    {
        let guard = JAVA_VM.lock()
            .map_err(|_| ShuseiError::Platform("Failed to lock JAVA_VM".into()))?;
        
        let java_vm = guard.as_ref()
            .ok_or_else(|| ShuseiError::Platform("JavaVM not initialized".into()))?;
        
        let mut env = java_vm.attach_current_thread()
            .map_err(|e| ShuseiError::Platform(format!("Failed to get JNIEnv: {}", e)))?;
        
        f(&mut env)
    }
    
    fn find_activity_class<'local>(env: &mut JNIEnv<'local>) -> Result<JClass<'local>> {
        env.find_class("com/shusei/app/MainActivity")
            .map_err(|e| ShuseiError::Platform(format!("Failed to find MainActivity class: {}", e)))
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
        
        let (tx, rx) = oneshot::channel();
        
        {
            let mut state = CAMERA_STATE.lock()
                .map_err(|_| ShuseiError::Platform("Failed to lock camera state".into()))?;
            *state = Some(CameraState {
                result_sender: Some(tx),
            });
        }
        
        self.with_env(|env| {
            let class = Self::find_activity_class(env)?;
            env.call_static_method(
                class,
                "startCameraCapture",
                "()V",
                &[],
            ).map_err(|e| ShuseiError::Platform(format!("Failed to call startCameraCapture: {}", e)))?;
            Ok(())
        })?;
        
        tokio::time::timeout(
            std::time::Duration::from_secs(30),
            rx
        ).await
            .map_err(|_| ShuseiError::Platform("Camera capture timeout".into()))?
            .map_err(|_| ShuseiError::Platform("Camera capture channel closed".into()))?
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
        
        let _ = self.with_env(|env| {
            let class = Self::find_activity_class(env)?;
            env.call_static_method(
                class,
                "vibrate",
                "(J)V",
                &[JValue::Long(duration_ms as i64)],
            ).map_err(|e| ShuseiError::Platform(format!("Failed to call vibrate: {}", e)))?;
            Ok(())
        });
    }
    
    async fn has_camera_permission(&self) -> bool {
        self.with_env(|env| {
            let class = Self::find_activity_class(env)?;
            let result = env.call_static_method(
                class,
                "hasCameraPermission",
                "()Z",
                &[],
            ).map_err(|e| ShuseiError::Platform(format!("Failed to call hasCameraPermission: {}", e)))?;
            result.z().map_err(|e| ShuseiError::Platform(format!("Failed to get boolean result: {}", e)))
        }).unwrap_or(false)
    }
    
    async fn has_microphone_permission(&self) -> bool {
        false
    }
    
    async fn request_camera_permission(&self) -> Result<bool> {
        self.with_env(|env| {
            let class = Self::find_activity_class(env)?;
            env.call_static_method(
                class,
                "requestCameraPermission",
                "()V",
                &[],
            ).map_err(|e| ShuseiError::Platform(format!("Failed to call requestCameraPermission: {}", e)))?;
            Ok(true)
        })
    }
    
    async fn request_microphone_permission(&self) -> Result<bool> {
        Ok(false)
    }
}

#[no_mangle]
pub extern "system" fn Java_com_shusei_app_MainActivity_nativeInit(
    mut env: JNIEnv,
    _class: JClass,
) {
    log::info!("nativeInit called from Java");
    
    match env.get_java_vm() {
        Ok(java_vm) => {
            if let Ok(mut guard) = JAVA_VM.lock() {
                *guard = Some(java_vm);
                log::info!("JavaVM stored successfully");
            }
        }
        Err(e) => {
            log::error!("Failed to get JavaVM: {}", e);
        }
    }
}

#[no_mangle]
pub extern "system" fn Java_com_shusei_app_MainActivity_onImageCaptured(
    mut env: JNIEnv,
    _class: JClass,
    image_data: jni::sys::jbyteArray,
    width: jni::sys::jint,
    height: jni::sys::jint,
) {
    log::info!("onImageCaptured: {}x{}", width, height);
    
    let byte_array = unsafe { JByteArray::from_raw(image_data) };
    let data: Vec<u8> = match env.convert_byte_array(&byte_array) {
        Ok(d) => d,
        Err(e) => {
            log::error!("Failed to convert byte array: {}", e);
            send_camera_result(Err(ShuseiError::Platform(format!("Failed to convert image data: {}", e))));
            return;
        }
    };
    
    log::info!("Image data size: {} bytes", data.len());
    
    let result = CameraResult {
        image_data: data,
        width: width as u32,
        height: height as u32,
        format: "jpeg".to_string(),
    };
    
    send_camera_result(Ok(result));
}

fn send_camera_result(result: Result<CameraResult>) {
    if let Ok(mut state_guard) = CAMERA_STATE.lock() {
        if let Some(state) = state_guard.take() {
            if let Some(sender) = state.result_sender {
                let _ = sender.send(result);
            }
        }
    }
}

/// Get the assets directory for storing images
/// 
/// On Android, this uses JNI to call Context.getFilesDir()
/// Falls back to current_dir() for non-Android platforms
pub fn get_assets_directory() -> Result<PathBuf> {
    // Try to get Java VM
    let guard = JAVA_VM.lock()
        .map_err(|_| ShuseiError::Platform("Failed to lock JAVA_VM".into()))?;
    
    let java_vm = match guard.as_ref() {
        Some(vm) => vm,
        None => {
            // Not on Android or JavaVM not initialized - fallback to current dir
            log::warn!("JavaVM not initialized, using current directory as fallback");
            return std::env::current_dir()
                .map_err(|e| ShuseiError::Platform(format!("Failed to get current directory: {}", e)).into());
        }
    };
    
    let mut env = java_vm.attach_current_thread()
        .map_err(|e| ShuseiError::Platform(format!("Failed to get JNIEnv: {}", e)))?;
    
    // Get MainActivity instance (assuming it's available via JNI)
    // In real Android app, this would be passed from Java side
    let activity_class = env.find_class("com/shusei/app/MainActivity")
        .map_err(|e| ShuseiError::Platform(format!("Failed to find MainActivity: {}", e)))?;
    
    // Call getFilesDir() on the activity
    // Note: This is a simplified version - in practice, you'd need the activity instance
    let files_dir = env.call_static_method(
        activity_class,
        "getFilesDir",
        "()Ljava/io/File;",
        &[],
    );
    
    match files_dir {
        Ok(dir_obj) => {
            // Convert JValue to JObject and call getAbsolutePath()
            let path_str = env.call_method(
                dir_obj.l()?,
                "getAbsolutePath",
                "()Ljava/lang/String;",
                &[],
            )?;
            
            let path = path_str.l()?;
            let rust_path: String = env.get_string(&path.into())?
                .to_str()?
                .to_string();
            
            Ok(PathBuf::from(rust_path))
        }
        Err(_) => {
            // Fallback to current directory
            log::warn!("Failed to get files directory, using current directory as fallback");
            std::env::current_dir()
                .map_err(|e| ShuseiError::Platform(format!("Failed to get current directory: {}", e)).into())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_get_assets_directory_fallback() {
        // When JavaVM is not initialized, should fallback to current_dir
        let result = get_assets_directory();
        
        // Should succeed with fallback
        assert!(result.is_ok());
        
        // Should be current directory (since JavaVM not initialized in tests)
        let path = result.unwrap();
        let current_dir = std::env::current_dir().unwrap();
        assert_eq!(path, current_dir);
    }
}