---
estimated_steps: 6
estimated_files: 3
skills_used:
  - debug-like-expert
  - lint
---

# T01: Implement WryActivity JNI Bindings

**Slice:** S07 — JNI Symbol Fix
**Milestone:** M003

## Description

Create manual JNI bindings for the 10 WryActivity lifecycle methods that Dioxus-generated Kotlin code expects. This works around the Dioxus 0.7.3 framework bug where `tao::android_binding!` is called inside a function body, preventing symbols from being exported.

## Steps

1. Add dependencies to `Cargo.toml`: `ndk = "0.9"`, `ndk-context = "0.1"`, `libc = "0.2"` under `[target.'cfg(target_os = "android")'.dependencies]`
2. Create `src/platform/android_bindings.rs` with:
   - Global state for window manager and pipe file descriptors (using `Mutex<Option<...>>`)
   - 10 `#[no_mangle] extern "system"` JNI functions: `create`, `start`, `resume`, `pause`, `stop`, `save`, `destroy`, `onActivityDestroy`, `memory`, `focus`
   - The `create` function must: (a) get JavaVM from JNIEnv, (b) store activity as global reference, (c) get window manager, (d) initialize ndk_context with `ndk_context::initialize_android_context()`
   - Other lifecycle functions should log their invocation
3. Add `pub mod android_bindings;` to `src/platform/mod.rs` inside the `#[cfg(target_os = "android")]` block
4. Build with: `cargo build --target aarch64-linux-android --lib`
5. Verify symbols: `nm -D target/aarch64-linux-android/debug/libshusei.so | grep "Java_dev_dioxus_main_WryActivity"`
6. Count should be exactly 10

## Must-Haves

- [ ] `src/platform/android_bindings.rs` exists with 10 `#[no_mangle]` JNI functions
- [ ] Each JNI function has correct signature matching WryActivity.kt expectations (e.g., `create(activity: WryActivity)` → `Java_dev_dioxus_main_WryActivity_create(env, class, activity)`)
- [ ] `ndk`, `ndk-context`, `libc` dependencies added for Android target
- [ ] Module registered in mod.rs
- [ ] `nm -D` shows exactly 10 WryActivity JNI symbols

## Verification

- `grep -c "#\[no_mangle\]" src/platform/android_bindings.rs` returns 10
- `nm -D target/aarch64-linux-android/debug/libshusei.so | grep -c "Java_dev_dioxus_main_WryActivity"` returns 10
- `cargo build --target aarch64-linux-android --lib` succeeds with no errors

## Inputs

- `src/platform/mod.rs` — Existing platform module where android_bindings must be registered
- `Cargo.toml` — Existing Cargo manifest where Android dependencies must be added
- `target/dx/shusei/debug/android/app/app/src/main/kotlin/dev/dioxus/main/WryActivity.kt` — Reference for expected JNI method signatures

## Expected Output

- `src/platform/android_bindings.rs` — New file with WryActivity JNI bindings
- `src/platform/mod.rs` — Modified with android_bindings module registration
- `Cargo.toml` — Modified with ndk dependencies for Android target
- `target/aarch64-linux-android/debug/libshusei.so` — Compiled library with exported JNI symbols

## Observability Impact

**New diagnostic signals:**
- Logcat tag `WryActivity` — Each lifecycle method logs when called (e.g., `WryActivity.create called`)
- JNI symbol export — 10 symbols visible via `nm -D` or `objdump -T` on libshusei.so

**How to inspect:**
```bash
# Verify symbols are exported before deployment
nm -D target/aarch64-linux-android/debug/libshusei.so | grep "Java_dev_dioxus_main_WryActivity"

# After deployment, check lifecycle is working
adb logcat | grep -E "(WryActivity\.(create|start|resume|pause|stop|save|destroy)|UnsatisfiedLinkError)"
```

**Failure states made visible:**
- Missing symbols → `nm -D` returns 0 matches → bindings not compiled or wrong module path
- Wrong symbol names → `nm -D` shows different names than expected → JNI function naming mismatch
- UnsatisfiedLinkError on launch → Kotlin looking for symbols in wrong class path or bindings not linked into APK
- Lifecycle methods not called → Bindings present but Dioxus event loop not triggering → framework integration issue