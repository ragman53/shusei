//! Manual JNI bindings for WryActivity lifecycle methods
//! 
//! This module provides manual JNI bindings to work around a Dioxus 0.7.3 bug
//! where `tao::android_binding!` called inside `start_app()` function body
//! prevents symbol export.
//! 
//! These bindings replicate what the tao/wry macros would generate:
//! - WryActivity lifecycle: create, start, stop, resume, pause, save, destroy, memory, focus
//! - WryActivity extension: onActivityDestroy
//! 
//! Reference: D016 - Dioxus WryActivity JNI workaround

use jni::JNIEnv;
use jni::objects::{JClass, JObject, GlobalRef};
use jni::sys::jboolean;
use once_cell::sync::OnceCell;
use log::{info, warn};

/// Android package name storage
static PACKAGE: OnceCell<&str> = OnceCell::new();

/// Window manager global reference
static WINDOW_MANAGER: once_cell::sync::Lazy<parking_lot::Mutex<Option<GlobalRef>>> = 
    once_cell::sync::Lazy::new(|| parking_lot::Mutex::new(None));

/// Pipe file descriptors for waking up the main loop
static PIPE_WRITE: OnceCell<i32> = OnceCell::new();
static PIPE_READ: OnceCell<i32> = OnceCell::new();

/// Event types for the Android event loop
#[derive(Clone, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum Event {
    Start,
    Resume,
    SaveInstanceState,
    Pause,
    Stop,
    Destroy,
    LowMemory,
    WindowLostFocus,
    WindowHasFocus,
}

/// Initialize the Android bindings.
/// This must be called before the app is launched.
/// 
/// # Arguments
/// * `package_name` - The Android package name (e.g., "dev.dioxus.main")
pub fn init_android_bindings(package_name: &'static str) {
    PACKAGE.set(package_name).ok();
    
    // Create pipe for event signaling
    let mut pipe: [i32; 2] = [0, 0];
    unsafe {
        libc::pipe(pipe.as_mut_ptr());
    }
    PIPE_READ.set(pipe[0]).ok();
    PIPE_WRITE.set(pipe[1]).ok();
    
    info!("Android bindings initialized for package: {}", package_name);
}

// ============================================================================
// WryActivity Lifecycle JNI Bindings
// ============================================================================

/// JNI binding for WryActivity.create(activity: WryActivity)
#[no_mangle]
pub extern "system" fn Java_dev_dioxus_main_WryActivity_create(
    mut env: JNIEnv,
    _class: JClass,
    activity: JObject,
) {
    info!("WryActivity.create called");
    
    // Get window manager
    let window_manager = match env.call_method(
        &activity,
        "getWindowManager",
        "()Landroid/view/WindowManager;",
        &[],
    ) {
        Ok(result) => match result.l() {
            Ok(wm) => wm,
            Err(e) => {
                warn!("Failed to get window manager: {}", e);
                return;
            }
        },
        Err(e) => {
            warn!("Failed to call getWindowManager: {}", e);
            return;
        }
    };
    
    let window_manager_ref = match env.new_global_ref(window_manager) {
        Ok(gref) => gref,
        Err(e) => {
            warn!("Failed to create global ref for window manager: {}", e);
            return;
        }
    };
    
    *WINDOW_MANAGER.lock() = Some(window_manager_ref);
    
    // Get JavaVM and store activity reference
    let java_vm = match env.get_java_vm() {
        Ok(vm) => vm,
        Err(e) => {
            warn!("Failed to get JavaVM: {}", e);
            return;
        }
    };
    
    let activity_ref = match env.new_global_ref(activity) {
        Ok(gref) => gref,
        Err(e) => {
            warn!("Failed to create global ref for activity: {}", e);
            return;
        }
    };
    
    // Initialize ndk_context
    unsafe {
        ndk_context::initialize_android_context(
            java_vm.get_java_vm_pointer() as *mut _,
            activity_ref.as_obj().as_raw() as *mut _,
        );
    }
    
    info!("WryActivity.create completed");
}

/// JNI binding for WryActivity.start()
#[no_mangle]
pub extern "system" fn Java_dev_dioxus_main_WryActivity_start(
    _env: JNIEnv,
    _class: JClass,
    _activity: JObject,
) {
    info!("WryActivity.start called");
    wake_event(Event::Start);
}

/// JNI binding for WryActivity.resume()
#[no_mangle]
pub extern "system" fn Java_dev_dioxus_main_WryActivity_resume(
    _env: JNIEnv,
    _class: JClass,
    _activity: JObject,
) {
    info!("WryActivity.resume called");
    wake_event(Event::Resume);
}

/// JNI binding for WryActivity.pause()
#[no_mangle]
pub extern "system" fn Java_dev_dioxus_main_WryActivity_pause(
    _env: JNIEnv,
    _class: JClass,
    _activity: JObject,
) {
    info!("WryActivity.pause called");
    wake_event(Event::Pause);
}

/// JNI binding for WryActivity.stop()
#[no_mangle]
pub extern "system" fn Java_dev_dioxus_main_WryActivity_stop(
    _env: JNIEnv,
    _class: JClass,
    _activity: JObject,
) {
    info!("WryActivity.stop called");
    wake_event(Event::Stop);
}

/// JNI binding for WryActivity.save()
#[no_mangle]
pub extern "system" fn Java_dev_dioxus_main_WryActivity_save(
    _env: JNIEnv,
    _class: JClass,
    _activity: JObject,
) {
    info!("WryActivity.save called");
    wake_event(Event::SaveInstanceState);
}

/// JNI binding for WryActivity.destroy()
#[no_mangle]
pub extern "system" fn Java_dev_dioxus_main_WryActivity_destroy(
    _env: JNIEnv,
    _class: JClass,
    _activity: JObject,
) {
    info!("WryActivity.destroy called");
    wake_event(Event::Destroy);
    
    // Release ndk_context
    unsafe {
        ndk_context::release_android_context();
    }
    
    // Clear window manager
    *WINDOW_MANAGER.lock() = None;
}

/// JNI binding for WryActivity.memory() - called on low memory
#[no_mangle]
pub extern "system" fn Java_dev_dioxus_main_WryActivity_memory(
    _env: JNIEnv,
    _class: JClass,
    _activity: JObject,
) {
    info!("WryActivity.memory called");
    wake_event(Event::LowMemory);
}

/// JNI binding for WryActivity.focus(focus: Boolean)
#[no_mangle]
pub extern "system" fn Java_dev_dioxus_main_WryActivity_focus(
    _env: JNIEnv,
    _class: JClass,
    _activity: JObject,
    has_focus: jboolean,
) {
    let event = if has_focus == 0 {
        info!("WryActivity.focus called: lost focus");
        Event::WindowLostFocus
    } else {
        info!("WryActivity.focus called: has focus");
        Event::WindowHasFocus
    };
    wake_event(event);
}

/// JNI binding for WryActivity.onActivityDestroy()
#[no_mangle]
pub extern "system" fn Java_dev_dioxus_main_WryActivity_onActivityDestroy(
    _env: JNIEnv,
    _class: JClass,
    _activity: JObject,
) {
    info!("WryActivity.onActivityDestroy called");
    // This is handled by the destroy() method
}

// ============================================================================
// Event Wake Mechanism
// ============================================================================

/// Wake up the main event loop with an event
fn wake_event(event: Event) {
    if let Some(pipe_fd) = PIPE_WRITE.get() {
        unsafe {
            let size = std::mem::size_of::<Event>();
            let res = libc::write(
                *pipe_fd,
                &event as *const _ as *const _,
                size,
            );
            if res != size as libc::ssize_t {
                warn!("Failed to write event to pipe: {}", res);
            }
        }
    }
}

/// Read events from the wake pipe
pub fn poll_events() -> Option<Event> {
    if let Some(pipe_fd) = PIPE_READ.get() {
        unsafe {
            let size = std::mem::size_of::<Event>();
            let mut event = Event::Start;
            if libc::read(
                *pipe_fd,
                &mut event as *mut _ as *mut _,
                size,
            ) == size as libc::ssize_t {
                return Some(event);
            }
        }
    }
    None
}

/// Get the window manager global reference
pub fn get_window_manager() -> Option<GlobalRef> {
    WINDOW_MANAGER.lock().clone()
}
