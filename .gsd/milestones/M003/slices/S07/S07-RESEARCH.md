# S07: JNI Symbol Fix — Research

**Slice:** S07 (JNI Symbol Fix)  
**Milestone:** M003 (Android Stability)  
**Status:** Research complete  
**Date:** 2026-03-23

## Summary

This slice fixes the root cause of the `UnsatisfiedLinkError` crash on app launch. The Dioxus 0.7.3 framework has a bug where `tao::android_binding!` macro is called inside the `start_app()` function body instead of at module level, preventing JNI symbols from being exported to the shared library.

**Solution:** Created manual JNI bindings in `src/platform/android_bindings.rs` that export the 10 WryActivity lifecycle methods required by the Dioxus-generated Kotlin code.

## Problem Analysis

### Root Cause (D016)

Dioxus 0.7.3 calls `tao::android_binding!` inside the `start_app()` function:

```rust
// Inside dioxus-desktop start_app() - WRONG
pub fn start_app() {
    tao::android_binding!(...);  // Called inside function body
    // ...
}
```

This prevents the macro-generated JNI symbols from being exported because:
1. The macro generates functions with `#[no_mangle]`
2. When called inside a function, the generated code is not at module level
3. The Rust compiler optimizes away unreachable code
4. The resulting `.so` library lacks the expected JNI symbols

### Expected JNI Symbols

WryActivity.kt expects these native methods:

```kotlin
private external fun create(activity: WryActivity)
private external fun start()
private external fun resume()
private external fun pause()
private external fun stop()
private external fun save()
private external fun destroy()
private external fun onActivityDestroy()
private external fun memory()
private external fun focus(focus: Boolean)
```

These should be exported as:
- `Java_dev_dioxus_main_WryActivity_create`
- `Java_dev_dioxus_main_WryActivity_start`
- `Java_dev_dioxus_main_WryActivity_resume`
- `Java_dev_dioxus_main_WryActivity_pause`
- `Java_dev_dioxus_main_WryActivity_stop`
- `Java_dev_dioxus_main_WryActivity_save`
- `Java_dev_dioxus_main_WryActivity_destroy`
- `Java_dev_dioxus_main_WryActivity_onActivityDestroy`
- `Java_dev_dioxus_main_WryActivity_memory`
- `Java_dev_dioxus_main_WryActivity_focus`

### Symbol Verification (Before Fix)

```bash
$ objdump -T libdioxusmain.so | grep "Java_dev_dioxus_main_WryActivity"
(no output - symbols missing!)
```

## Implementation

### Manual JNI Bindings Module

Created `src/platform/android_bindings.rs` with:

1. **10 `#[no_mangle]` JNI functions** - One for each WryActivity lifecycle method
2. **Event system** - Pipe-based event signaling for the Android event loop
3. **Global state** - Window manager reference, pipe file descriptors
4. **ndk_context integration** - Proper Android context initialization

Key implementation details:

```rust
#[no_mangle]
pub extern "system" fn Java_dev_dioxus_main_WryActivity_create(
    mut env: JNIEnv,
    _class: JClass,
    activity: JObject,
) {
    // Get window manager
    let window_manager = env.call_method(&activity, "getWindowManager", ...)...;
    let window_manager_ref = env.new_global_ref(window_manager)...;
    *WINDOW_MANAGER.lock() = Some(window_manager_ref);
    
    // Initialize ndk_context
    unsafe {
        ndk_context::initialize_android_context(
            java_vm.get_java_vm_pointer() as *mut _,
            activity_ref.as_obj().as_raw() as *mut _,
        );
    }
}
```

### Dependencies Added

```toml
[target.'cfg(target_os = "android")'.dependencies]
jni = "0.21"
ndk = "0.9"
ndk-context = "0.1"
libc = "0.2"
```

### Module Registration

Updated `src/platform/mod.rs`:

```rust
#[cfg(target_os = "android")]
pub mod android;

#[cfg(target_os = "android")]
pub mod android_bindings;
```

### Symbol Verification (After Fix)

```bash
$ cargo build --target aarch64-linux-android --lib
$ objdump -T target/aarch64-linux-android/debug/libshusei.so | grep "Java_dev_dioxus_main_WryActivity"
0000000000041660 g    DF .text	0000000000000a48  Base        Java_dev_dioxus_main_WryActivity_create
0000000000042aa4 g    DF .text	0000000000000138  Base        Java_dev_dioxus_main_WryActivity_start
0000000000042834 g    DF .text	0000000000000138  Base        Java_dev_dioxus_main_WryActivity_resume
00000000000426fc g    DF .text	0000000000000138  Base        Java_dev_dioxus_main_WryActivity_pause
0000000000042bdc g    DF .text	0000000000000138  Base        Java_dev_dioxus_main_WryActivity_stop
000000000004296c g    DF .text	0000000000000138  Base        Java_dev_dioxus_main_WryActivity_save
00000000000420a8 g    DF .text	0000000000000180  Base        Java_dev_dioxus_main_WryActivity_destroy
00000000000425cc g    DF .text	0000000000000130  Base        Java_dev_dioxus_main_WryActivity_onActivityDestroy
0000000000042494 g    DF .text	0000000000000138  Base        Java_dev_dioxus_main_WryActivity_memory
0000000000042228 g    DF .text	000000000000026c  Base        Java_dev_dioxus_main_WryActivity_focus
```

All 10 symbols are now exported.

## Build Integration

The android-build.sh script already copies the Rust library to the Android project:

```bash
# Build Rust library
cargo build --target aarch64-linux-android --lib

# Copy to Android jniLibs
cp target/aarch64-linux-android/debug/libshusei.so \
   target/dx/shusei/debug/android/app/app/src/main/jniLibs/arm64-v8a/libdioxusmain.so
```

After copying, the APK build includes the new symbols:

```bash
cd target/dx/shusei/debug/android/app
./gradlew assembleDebug
# BUILD SUCCESSFUL
```

## Verification

### Symbol Check

```bash
objdump -T target/dx/shusei/debug/android/app/app/src/main/jniLibs/arm64-v8a/libdioxusmain.so | \
  grep "Java_dev_dioxus_main_WryActivity" | wc -l
# Output: 10
```

### App Launch Test (Pending Device)

The APK should now launch without `UnsatisfiedLinkError`. Verification on device:

```bash
adb install -r target/dx/shusei/debug/android/app/app/build/outputs/apk/debug/app-debug.apk
adb shell am start -n com.shusei.app/.MainActivity
adb logcat | grep -E "(UnsatisfiedLinkError|WryActivity|Shusei)"
```

Expected log output:
```
WryActivity.create called
WryActivity.start called
App initialized with default state
```

No `UnsatisfiedLinkError` should appear.

## Risks and Mitigations

### Risk: Incomplete Lifecycle Implementation

The manual bindings implement the basic lifecycle methods but may not handle all edge cases that the full tao/wry implementation would.

**Mitigation:** Start with minimal implementation, add functionality as needed based on runtime behavior.

### Risk: Event Loop Integration

The current implementation initializes ndk_context and stores the window manager, but doesn't fully integrate with Dioxus's event loop.

**Mitigation:** The Dioxus framework handles the event loop; our bindings just need to provide the JNI entry points. Monitor logs for event handling issues.

### Risk: Future Dioxus Updates

When Dioxus fixes the upstream bug, these manual bindings will become redundant.

**Mitigation:** The bindings are isolated in a separate module (`android_bindings.rs`) with clear documentation (D016 reference). Easy to remove when upstream is fixed.

## Files Changed

| File | Change |
|------|--------|
| `src/platform/android_bindings.rs` | Created - Manual JNI bindings for WryActivity |
| `src/platform/mod.rs` | Modified - Added `android_bindings` module |
| `Cargo.toml` | Modified - Added `ndk`, `ndk-context`, `libc` dependencies |

## Next Steps

1. **Deploy APK to device** - Install and verify app launches without crash
2. **Monitor logs** - Check for `WryActivity.* called` messages and no `UnsatisfiedLinkError`
3. **Test lifecycle** - Verify app handles background/foreground transitions correctly
4. **Integration test** - Run full UAT (S08) to verify all three flows work

## Sources

- D016 decision: `.gsd/DECISIONS.md` - Dioxus WryActivity JNI workaround
- tao source: `~/.cargo/registry/src/.../tao-0.34.6/src/platform_impl/android/ndk_glue.rs`
- wry source: `~/.cargo/registry/src/.../wry-0.53.5/src/android/binding.rs`
- JNI specification: Oracle Java Native Interface documentation
