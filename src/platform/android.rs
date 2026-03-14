//! Android platform implementation using JNI

use async_trait::async_trait;
use jni::JNIEnv;
use jni::objects::{JClass, JValue, JByteArray};
use jni::JavaVM;
use std::sync::Mutex;
use tokio::sync::oneshot;
use once_cell::sync::Lazy;

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

pub fn get_assets_directory() -> crate::core::error::Result<std::path::PathBuf> {
    Ok(std::path::PathBuf::from("/data/data/com.shusei.app/files"))
}

/// Copy a bundled asset from APK to the app's files directory
/// Returns the path to the copied file
pub fn copy_asset_to_files(asset_path: &str) -> crate::core::error::Result<std::path::PathBuf> {
    let files_dir = get_assets_directory()?;
    
    // Create the target path
    let file_name = std::path::Path::new(asset_path)
        .file_name()
        .ok_or_else(|| ShuseiError::Platform("Invalid asset path".into()))?
        .to_str()
        .ok_or_else(|| ShuseiError::Platform("Invalid file name".into()))?;
    
    let target_path = files_dir.join(file_name);
    
    // If already copied, return the path
    if target_path.exists() {
        log::info!("Asset already copied to: {:?}", target_path);
        return Ok(target_path);
    }
    
    // Use JNI to copy from APK assets
    let guard = JAVA_VM.lock()
        .map_err(|_| ShuseiError::Platform("Failed to lock JAVA_VM".into()))?;
    
    let java_vm = guard.as_ref()
        .ok_or_else(|| ShuseiError::Platform("JavaVM not initialized".into()))?;
    
    let mut env = java_vm.attach_current_thread()
        .map_err(|e| ShuseiError::Platform(format!("Failed to get JNIEnv: {}", e)))?;
    
    // Call Java method to copy asset
    let class = env.find_class("com/shusei/app/MainActivity")
        .map_err(|e| ShuseiError::Platform(format!("Failed to find MainActivity class: {}", e)))?;
    
    let asset_path_jstr = env.new_string(asset_path)
        .map_err(|e| ShuseiError::Platform(format!("Failed to create string: {}", e)))?;
    
    let target_path_jstr = env.new_string(target_path.to_str().unwrap())
        .map_err(|e| ShuseiError::Platform(format!("Failed to create target path string: {}", e)))?;
    
    env.call_static_method(
        class,
        "copyAssetToFiles",
        "(Ljava/lang/String;Ljava/lang/String;)Z",
        &[JValue::Object(&asset_path_jstr.into()), JValue::Object(&target_path_jstr.into())],
    ).map_err(|e| ShuseiError::Platform(format!("Failed to call copyAssetToFiles: {}", e)))?
    .z()
    .map_err(|e| ShuseiError::Platform(format!("Failed to get boolean result: {}", e)))?;
    
    log::info!("Asset copied to: {:?}", target_path);
    Ok(target_path)
}

#[no_mangle]
pub extern "system" fn JNI_OnLoad(java_vm: *mut jni::sys::JavaVM, _reserved: *mut std::ffi::c_void) -> jni::sys::jint {
    log::info!("JNI_OnLoad called - initializing JavaVM");
    
    if java_vm.is_null() {
        log::error!("JNI_OnLoad: java_vm is null");
        return jni::sys::JNI_VERSION_1_6;
    }
    
    let java_vm = match unsafe { JavaVM::from_raw(java_vm) } {
        Ok(vm) => vm,
        Err(e) => {
            log::error!("JNI_OnLoad: Failed to create JavaVM from raw pointer: {:?}", e);
            return jni::sys::JNI_VERSION_1_6;
        }
    };
    
    if let Ok(mut guard) = JAVA_VM.lock() {
        *guard = Some(java_vm);
        log::info!("JavaVM initialized successfully in JNI_OnLoad");
    } else {
        log::error!("Failed to lock JAVA_VM mutex in JNI_OnLoad");
    }
    
    jni::sys::JNI_VERSION_1_6
}