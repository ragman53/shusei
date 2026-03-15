//! Android platform implementation using JNI

use async_trait::async_trait;
use jni::JNIEnv;
use jni::objects::{JClass, JValue, JByteArray, JString};
use jni::JavaVM;
use std::sync::Mutex;
use tokio::sync::oneshot;
use once_cell::sync::Lazy;

use crate::core::error::{ShuseiError, Result};
use super::{PlatformApi, CameraResult, AudioResult};

static CAMERA_STATE: Mutex<Option<CameraState>> = Mutex::new(None);

static FILE_PICKER_STATE: Mutex<Option<FilePickerState>> = Mutex::new(None);

static JAVA_VM: Lazy<Mutex<Option<JavaVM>>> = Lazy::new(|| Mutex::new(None));

static ACTIVITY: Lazy<Mutex<Option<jni::objects::GlobalRef>>> = Lazy::new(|| Mutex::new(None));

struct CameraState {
    result_sender: Option<oneshot::Sender<Result<CameraResult>>>,
}

struct FilePickerState {
    result_sender: Option<oneshot::Sender<Result<String>>>,
}

struct AudioRecordState {
    result_sender: Option<oneshot::Sender<Result<AudioResult>>>,
}

static AUDIO_RECORD_STATE: Mutex<Option<AudioRecordState>> = Mutex::new(None);

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
        env.find_class("dev/dioxus/main/MainActivity")
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
        
        // Enforce 30 second hard limit
        let max_seconds = max_seconds.min(30);
        
        let (tx, rx) = oneshot::channel();
        
        {
            let mut state = AUDIO_RECORD_STATE.lock()
                .map_err(|_| ShuseiError::Platform("Failed to lock audio record state".into()))?;
            *state = Some(AudioRecordState {
                result_sender: Some(tx),
            });
        }
        
        // Request microphone permission first
        self.request_microphone_permission().await?;
        
        // Start audio recording via JNI
        self.with_env(|env| {
            let class = Self::find_activity_class(env)?;
            env.call_static_method(
                class,
                "startAudioRecording",
                "(I)V",
                &[JValue::Int(max_seconds as i32)],
            ).map_err(|e| ShuseiError::Platform(format!("Failed to call startAudioRecording: {}", e)))?;
            Ok(())
        })?;
        
        log::info!("Audio recording started, waiting for result (timeout: {}s)", max_seconds + 5);
        
        // Wait for recording to complete (max_seconds + 5s buffer)
        tokio::time::timeout(
            std::time::Duration::from_secs(max_seconds as u64 + 5),
            rx
        ).await
            .map_err(|_| ShuseiError::Platform(format!("Audio recording timeout after {}s", max_seconds)))?
            .map_err(|_| ShuseiError::Platform("Audio recording channel closed".into()))?
    }
    
    async fn pick_file(&self, _extensions: &[&str]) -> Result<String> {
        log::info!("Attempting to pick file via JNI...");
        
        let (tx, rx) = oneshot::channel();
        
        {
            let mut state = FILE_PICKER_STATE.lock()
                .map_err(|_| ShuseiError::Platform("Failed to lock file picker state".into()))?;
            *state = Some(FilePickerState {
                result_sender: Some(tx),
            });
        }
        
        self.with_env(|env| {
            let class = Self::find_activity_class(env)?;
            env.call_static_method(
                class,
                "pickPdfFile",
                "()V",
                &[],
            ).map_err(|e| ShuseiError::Platform(format!("Failed to call pickPdfFile: {}", e)))?;
            Ok(())
        })?;
        
        tokio::time::timeout(
            std::time::Duration::from_secs(60),
            rx
        ).await
            .map_err(|_| ShuseiError::Platform("File picker timeout".into()))?
            .map_err(|_| ShuseiError::Platform("File picker channel closed".into()))?
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
        self.with_env(|env| {
            let class = Self::find_activity_class(env)?;
            let result = env.call_static_method(
                class,
                "hasMicrophonePermission",
                "()Z",
                &[],
            ).map_err(|e| ShuseiError::Platform(format!("Failed to call hasMicrophonePermission: {}", e)))?;
            result.z().map_err(|e| ShuseiError::Platform(format!("Failed to get boolean result: {}", e)))
        }).unwrap_or(false)
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
        self.with_env(|env| {
            let class = Self::find_activity_class(env)?;
            env.call_static_method(
                class,
                "requestMicrophonePermission",
                "()V",
                &[],
            ).map_err(|e| ShuseiError::Platform(format!("Failed to call requestMicrophonePermission: {}", e)))?;
            Ok(true)
        })
    }
}

#[no_mangle]
pub extern "system" fn Java_dev_dioxus_main_MainActivity_nativeInit(
    mut env: JNIEnv,
    activity: jni::objects::JObject,
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
    
    let global_ref = env.new_global_ref(activity);
    match global_ref {
        Ok(gref) => {
            if let Ok(mut guard) = ACTIVITY.lock() {
                *guard = Some(gref);
                log::info!("Activity reference stored successfully");
            }
        }
        Err(e) => {
            log::error!("Failed to create global ref for activity: {}", e);
        }
    }
}

#[no_mangle]
pub extern "system" fn Java_com_shusei_app_MainActivity_nativeInit(
    mut env: JNIEnv,
    _class: JClass,
) {
    log::info!("nativeInit (legacy) called - use dev.dioxus.main.MainActivity instead");
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

pub fn copy_asset_to_files(asset_path: &str) -> crate::core::error::Result<std::path::PathBuf> {
    let files_dir = get_assets_directory()?;
    
    let file_name = std::path::Path::new(asset_path)
        .file_name()
        .ok_or_else(|| ShuseiError::Platform("Invalid asset path".into()))?
        .to_str()
        .ok_or_else(|| ShuseiError::Platform("Invalid file name".into()))?;
    
    let target_path = files_dir.join(file_name);
    
    if target_path.exists() {
        log::info!("Asset already copied to: {:?}", target_path);
        return Ok(target_path);
    }
    
    std::fs::create_dir_all(&files_dir)
        .map_err(|e| ShuseiError::Platform(format!("Failed to create files directory: {}", e)))?;
    
    let guard = JAVA_VM.lock()
        .map_err(|_| ShuseiError::Platform("Failed to lock JAVA_VM".into()))?;
    
    let java_vm = guard.as_ref()
        .ok_or_else(|| ShuseiError::Platform("JavaVM not initialized".into()))?;
    
    let mut env = java_vm.attach_current_thread()
        .map_err(|e| ShuseiError::Platform(format!("Failed to get JNIEnv: {}", e)))?;
    
    let activity_guard = ACTIVITY.lock()
        .map_err(|_| ShuseiError::Platform("Failed to lock ACTIVITY".into()))?;
    
    let activity_ref = activity_guard.as_ref()
        .ok_or_else(|| ShuseiError::Platform("Activity not initialized - ensure nativeInit is called with activity reference".into()))?;
    
    let activity_obj = activity_ref.as_obj();
    
    let activity_class = env.find_class("android/app/Activity")
        .map_err(|e| ShuseiError::Platform(format!("Failed to find Activity class: {}", e)))?;
    
    let asset_manager_obj = env.call_method(
        activity_obj,
        "getAssets",
        "()Landroid/content/res/AssetManager;",
        &[],
    ).map_err(|e| ShuseiError::Platform(format!("Failed to get AssetManager: {}", e)))?
    .l()
    .map_err(|e| ShuseiError::Platform(format!("Failed to cast AssetManager: {}", e)))?;
    
    let asset_path_jstr = env.new_string(asset_path)
        .map_err(|e| ShuseiError::Platform(format!("Failed to create string: {}", e)))?;
    
    let input_stream = env.call_method(
        &asset_manager_obj,
        "open",
        "(Ljava/lang/String;)Ljava/io/InputStream;",
        &[JValue::Object(&asset_path_jstr)],
    ).map_err(|e| ShuseiError::Platform(format!("Failed to open asset '{}': {}", asset_path, e)))?
    .l()
    .map_err(|e| ShuseiError::Platform(format!("Failed to cast InputStream: {}", e)))?;
    
    if input_stream.is_null() {
        return Err(ShuseiError::Platform(format!("Asset '{}' not found", asset_path).into()));
    }
    
    let buffer_size = 8192i32;
    let buffer = env.new_byte_array(buffer_size)
        .map_err(|e| ShuseiError::Platform(format!("Failed to create buffer: {}", e)))?;
    
    let mut output_file = std::fs::File::create(&target_path)
        .map_err(|e| ShuseiError::Platform(format!("Failed to create output file: {}", e)))?;
    
    loop {
        let bytes_read = env.call_method(
            &input_stream,
            "read",
            "([B)I",
            &[JValue::Object(&buffer)],
        ).map_err(|e| ShuseiError::Platform(format!("Failed to read from asset: {}", e)))?
        .i()
        .map_err(|e| ShuseiError::Platform(format!("Failed to cast bytes read: {}", e)))?;
        
        if bytes_read <= 0 {
            break;
        }
        
        let bytes = env.convert_byte_array(&buffer)
            .map_err(|e| ShuseiError::Platform(format!("Failed to convert byte array: {}", e)))?;
        
        use std::io::Write;
        output_file.write_all(&bytes[..bytes_read as usize])
            .map_err(|e| ShuseiError::Platform(format!("Failed to write to output file: {}", e)))?;
    }
    
    env.call_method(
        &input_stream,
        "close",
        "()V",
        &[],
    ).map_err(|e| ShuseiError::Platform(format!("Failed to close input stream: {}", e)))?;
    
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
    
    {
        if let Ok(mut guard) = JAVA_VM.lock() {
            *guard = Some(java_vm);
            log::info!("JavaVM initialized successfully in JNI_OnLoad");
        } else {
            log::error!("Failed to lock JAVA_VM mutex in JNI_OnLoad");
            return jni::sys::JNI_VERSION_1_6;
        }
    }
    
    let guard = match JAVA_VM.lock() {
        Ok(g) => g,
        Err(_) => {
            log::error!("Failed to lock JAVA_VM for getting env");
            return jni::sys::JNI_VERSION_1_6;
        }
    };
    
    let stored_vm = match guard.as_ref() {
        Some(vm) => vm,
        None => {
            log::error!("JavaVM not stored");
            return jni::sys::JNI_VERSION_1_6;
        }
    };
    
    let mut env = match stored_vm.attach_current_thread() {
        Ok(e) => e,
        Err(e) => {
            log::error!("JNI_OnLoad: Failed to attach thread: {:?}", e);
            return jni::sys::JNI_VERSION_1_6;
        }
    };
    
    match env.find_class("android/app/ActivityThread") {
        Ok(activity_thread_class) => {
            match env.call_static_method(
                activity_thread_class,
                "currentActivityThread",
                "()Landroid/app/ActivityThread;",
                &[],
            ) {
                Ok(activity_thread_obj) => {
                    match activity_thread_obj.l() {
                        Ok(activity_thread) => {
                            if !activity_thread.is_null() {
                                match env.call_method(
                                    &activity_thread,
                                    "getApplication",
                                    "()Landroid/app/Application;",
                                    &[],
                                ) {
                                    Ok(app_obj) => {
                                        match app_obj.l() {
                                            Ok(application) => {
                                                if !application.is_null() {
                                                    match env.new_global_ref(application) {
                                                        Ok(global_ref) => {
                                                            if let Ok(mut guard) = ACTIVITY.lock() {
                                                                *guard = Some(global_ref);
                                                                log::info!("Application context stored successfully");
                                                            }
                                                        }
                                                        Err(e) => {
                                                            log::error!("Failed to create global ref for application: {}", e);
                                                        }
                                                    }
                                                }
                                            }
                                            Err(e) => {
                                                log::error!("Failed to cast Application: {}", e);
                                            }
                                        }
                                    }
                                    Err(e) => {
                                        log::error!("Failed to get Application: {}", e);
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            log::error!("Failed to cast ActivityThread: {}", e);
                        }
                    }
                }
                Err(e) => {
                    log::error!("Failed to get currentActivityThread: {}", e);
                }
            }
        }
        Err(e) => {
            log::error!("Failed to find ActivityThread class: {}", e);
        }
    }
    
    jni::sys::JNI_VERSION_1_6
}

// File picker JNI callbacks

#[no_mangle]
pub extern "system" fn Java_com_shusei_app_MainActivity_onFilePicked(
    mut env: JNIEnv,
    _class: JClass,
    file_path: jni::sys::jstring,
) {
    log::info!("onFilePicked: file selected");
    
    if file_path.is_null() {
        log::error!("onFilePicked: file_path is null");
        send_file_picker_result(Err(ShuseiError::Platform(
            "File path is null".into()
        )));
        return;
    }
    
    unsafe {
        let j_string = JString::from_raw(file_path);
        let java_str = match env.get_string(&j_string) {
            Ok(s) => s,
            Err(e) => {
                log::error!("Failed to get file path string: {}", e);
                send_file_picker_result(Err(ShuseiError::Platform(
                    "Failed to convert file path".into()
                )));
                return;
            }
        };
        let path = java_str.to_str().unwrap_or("").to_string();
        log::info!("File picked: {}", path);
        send_file_picker_result(Ok(path));
    }
}

#[no_mangle]
pub extern "system" fn Java_com_shusei_app_MainActivity_onFilePickFailed(
    mut env: JNIEnv,
    _class: JClass,
    error_message: jni::sys::jstring,
) {
    log::error!("onFilePickFailed: file picker failed");
    
    if error_message.is_null() {
        log::error!("File picker failed: Unknown error (null message)");
        send_file_picker_result(Err(ShuseiError::Platform("Unknown error".into())));
        return;
    }
    
    unsafe {
        let j_string = JString::from_raw(error_message);
        let java_str = match env.get_string(&j_string) {
            Ok(s) => s,
            Err(e) => {
                log::error!("Failed to get error message string: {}", e);
                send_file_picker_result(Err(ShuseiError::Platform(
                    "Unknown error".into()
                )));
                return;
            }
        };
        let error = java_str.to_str().unwrap_or("Unknown error").to_string();
        log::error!("File picker failed: {}", error);
        send_file_picker_result(Err(ShuseiError::Platform(error)));
    }
}

// Also need the legacy package version for Dioxus

#[no_mangle]
pub extern "system" fn Java_dev_dioxus_main_MainActivity_onFilePicked(
    mut env: JNIEnv,
    _class: JClass,
    file_path: jni::sys::jstring,
) {
    log::info!("onFilePicked (dev.dioxus.main): file selected");
    
    if file_path.is_null() {
        log::error!("onFilePicked (dev.dioxus.main): file_path is null");
        send_file_picker_result(Err(ShuseiError::Platform(
            "File path is null".into()
        )));
        return;
    }
    
    unsafe {
        let j_string = JString::from_raw(file_path);
        let java_str = match env.get_string(&j_string) {
            Ok(s) => s,
            Err(e) => {
                log::error!("Failed to get file path string: {}", e);
                send_file_picker_result(Err(ShuseiError::Platform(
                    "Failed to convert file path".into()
                )));
                return;
            }
        };
        let path = java_str.to_str().unwrap_or("").to_string();
        log::info!("File picked: {}", path);
        send_file_picker_result(Ok(path));
    }
}

#[no_mangle]
pub extern "system" fn Java_dev_dioxus_main_MainActivity_onFilePickFailed(
    mut env: JNIEnv,
    _class: JClass,
    error_message: jni::sys::jstring,
) {
    log::error!("onFilePickFailed (dev.dioxus.main): file picker failed");
    
    if error_message.is_null() {
        log::error!("File picker failed (dev.dioxus.main): Unknown error (null message)");
        send_file_picker_result(Err(ShuseiError::Platform("Unknown error".into())));
        return;
    }
    
    unsafe {
        let j_string = JString::from_raw(error_message);
        let java_str = match env.get_string(&j_string) {
            Ok(s) => s,
            Err(e) => {
                log::error!("Failed to get error message string: {}", e);
                send_file_picker_result(Err(ShuseiError::Platform(
                    "Unknown error".into()
                )));
                return;
            }
        };
        let error = java_str.to_str().unwrap_or("Unknown error").to_string();
        log::error!("File picker failed: {}", error);
        send_file_picker_result(Err(ShuseiError::Platform(error)));
    }
}

fn send_file_picker_result(result: Result<String>) {
    if let Ok(mut state_guard) = FILE_PICKER_STATE.lock() {
        if let Some(state) = state_guard.take() {
            if let Some(sender) = state.result_sender {
                let _ = sender.send(result);
            }
        }
    }
}

// Audio recording JNI callbacks

#[no_mangle]
pub extern "system" fn Java_com_shusei_app_MainActivity_onAudioRecorded(
    mut env: JNIEnv,
    _class: JClass,
    audio_data: jni::sys::jfloatArray,
    sample_rate: jni::sys::jint,
    duration: jni::sys::jfloat,
) {
    log::info!("onAudioRecorded: sample_rate={}, duration={:.2}s", sample_rate, duration);
    
    if audio_data.is_null() {
        log::error!("onAudioRecorded: audio_data is null");
        send_audio_result(Err(ShuseiError::Platform(
            "Audio data is null".into()
        )));
        return;
    }
    
    unsafe {
        let float_array = unsafe { jni::objects::JFloatArray::from_raw(audio_data) };
        
        // Get array length
        let length = match env.get_array_length(&float_array) {
            Ok(len) => len as usize,
            Err(e) => {
                log::error!("Failed to get audio array length: {}", e);
                send_audio_result(Err(ShuseiError::Platform(
                    "Failed to get audio array length".into()
                )));
                return;
            }
        };
        
        // Convert jfloatArray to Vec<f32>
        let mut samples = vec![0.0f32; length];
        if let Err(e) = env.get_float_array_region(&float_array, 0, &mut samples) {
            log::error!("Failed to read audio data: {}", e);
            send_audio_result(Err(ShuseiError::Platform(
                "Failed to read audio data".into()
            )));
            return;
        }
        
        log::info!("Audio data received: {} samples, {} Hz, {:.2}s", samples.len(), sample_rate, duration);
        
        let result = AudioResult {
            samples,
            sample_rate: sample_rate as u32,
            duration_seconds: duration,
        };
        
        send_audio_result(Ok(result));
    }
}

#[no_mangle]
pub extern "system" fn Java_com_shusei_app_MainActivity_onAudioRecordFailed(
    mut env: JNIEnv,
    _class: JClass,
    error_message: jni::sys::jstring,
) {
    log::error!("onAudioRecordFailed: audio recording failed");
    
    if error_message.is_null() {
        log::error!("Audio recording failed: Unknown error (null message)");
        send_audio_result(Err(ShuseiError::Platform("Unknown error".into())));
        return;
    }
    
    unsafe {
        let j_string = JString::from_raw(error_message);
        let java_str = match env.get_string(&j_string) {
            Ok(s) => s,
            Err(e) => {
                log::error!("Failed to get error message string: {}", e);
                send_audio_result(Err(ShuseiError::Platform(
                    "Unknown error".into()
                )));
                return;
            }
        };
        let error = java_str.to_str().unwrap_or("Unknown error").to_string();
        log::error!("Audio recording failed: {}", error);
        send_audio_result(Err(ShuseiError::Platform(error)));
    }
}

// Also need the dev.dioxus.main package version

#[no_mangle]
pub extern "system" fn Java_dev_dioxus_main_MainActivity_onAudioRecorded(
    mut env: JNIEnv,
    _class: JClass,
    audio_data: jni::sys::jfloatArray,
    sample_rate: jni::sys::jint,
    duration: jni::sys::jfloat,
) {
    log::info!("onAudioRecorded (dev.dioxus.main): sample_rate={}, duration={:.2}s", sample_rate, duration);
    
    if audio_data.is_null() {
        log::error!("onAudioRecorded (dev.dioxus.main): audio_data is null");
        send_audio_result(Err(ShuseiError::Platform(
            "Audio data is null".into()
        )));
        return;
    }
    
    unsafe {
        let float_array = unsafe { jni::objects::JFloatArray::from_raw(audio_data) };
        
        // Get array length
        let length = match env.get_array_length(&float_array) {
            Ok(len) => len as usize,
            Err(e) => {
                log::error!("Failed to get audio array length: {}", e);
                send_audio_result(Err(ShuseiError::Platform(
                    "Failed to get audio array length".into()
                )));
                return;
            }
        };
        
        // Convert jfloatArray to Vec<f32>
        let mut samples = vec![0.0f32; length];
        if let Err(e) = env.get_float_array_region(&float_array, 0, &mut samples) {
            log::error!("Failed to read audio data: {}", e);
            send_audio_result(Err(ShuseiError::Platform(
                "Failed to read audio data".into()
            )));
            return;
        }
        
        log::info!("Audio data received: {} samples, {} Hz, {:.2}s", samples.len(), sample_rate, duration);
        
        let result = AudioResult {
            samples,
            sample_rate: sample_rate as u32,
            duration_seconds: duration,
        };
        
        send_audio_result(Ok(result));
    }
}

#[no_mangle]
pub extern "system" fn Java_dev_dioxus_main_MainActivity_onAudioRecordFailed(
    mut env: JNIEnv,
    _class: JClass,
    error_message: jni::sys::jstring,
) {
    log::error!("onAudioRecordFailed (dev.dioxus.main): audio recording failed");
    
    if error_message.is_null() {
        log::error!("Audio recording failed (dev.dioxus.main): Unknown error (null message)");
        send_audio_result(Err(ShuseiError::Platform("Unknown error".into())));
        return;
    }
    
    unsafe {
        let j_string = JString::from_raw(error_message);
        let java_str = match env.get_string(&j_string) {
            Ok(s) => s,
            Err(e) => {
                log::error!("Failed to get error message string: {}", e);
                send_audio_result(Err(ShuseiError::Platform(
                    "Unknown error".into()
                )));
                return;
            }
        };
        let error = java_str.to_str().unwrap_or("Unknown error").to_string();
        log::error!("Audio recording failed: {}", error);
        send_audio_result(Err(ShuseiError::Platform(error)));
    }
}

fn send_audio_result(result: Result<AudioResult>) {
    if let Ok(mut state_guard) = AUDIO_RECORD_STATE.lock() {
        if let Some(state) = state_guard.take() {
            if let Some(sender) = state.result_sender {
                let _ = sender.send(result);
            }
        }
    }
}