# S07: JNI Symbol Fix

**Goal:** App launches on Moto G66j 5G without `UnsatisfiedLinkError`; all 10 WryActivity JNI lifecycle symbols exported in native library.

**Demo:** `adb install` succeeds, `adb shell am start` launches app without crash, `adb logcat` shows `WryActivity.create` called and app initialized.

## Must-Haves

- `src/platform/android_bindings.rs` created with 10 `#[no_mangle]` JNI functions for WryActivity lifecycle methods
- Dependencies `ndk`, `ndk-context`, `libc` added to Cargo.toml for Android target
- `android_bindings` module registered in `src/platform/mod.rs`
- All 10 JNI symbols present in compiled `libshusei.so` (verified with `nm -D`)
- APK installs and launches without `UnsatisfiedLinkError`

## Proof Level

- This slice proves: integration
- Real runtime required: yes (device test)
- Human/UAT required: yes (manual device verification)

## Verification

- `nm -D target/aarch64-linux-android/debug/libshusei.so | grep -c "Java_dev_dioxus_main_WryActivity"` returns 10 (all lifecycle symbols present)
- `adb install -r target/dx/shusei/debug/android/app/app/build/outputs/apk/debug/app-debug.apk` succeeds
- `adb shell am start -n com.shusei.app/.MainActivity` returns "Starting activity" (not error)
- `adb logcat -d | grep -E "WryActivity|UnsatisfiedLinkError"` shows WryActivity lifecycle calls and NO UnsatisfiedLinkError

## Observability / Diagnostics

- Runtime signals: `adb logcat` shows WryActivity lifecycle method calls (`WryActivity.create`, `WryActivity.start`, etc.)
- Inspection surfaces:
  - `nm -D libshusei.so | grep Java` — Symbol table showing exported JNI functions
  - `adb logcat | grep -E "WryActivity|MainActivity|Shusei"` — Android runtime logs
- Failure visibility: `UnsatisfiedLinkError` shows exact missing method name; absence of lifecycle logs indicates bindings not called
- Redaction constraints: None (no secrets in JNI layer)

## Integration Closure

- Upstream surfaces consumed: Dioxus-generated `WryActivity.kt` expects 10 JNI lifecycle methods
- New wiring introduced in this slice: Manual JNI bindings connect Android Activity lifecycle to Rust runtime initialization
- What remains before the milestone is truly usable end-to-end: S08 (Full UAT) verifies all three user flows work correctly

## Tasks

- [ ] **T01: Implement WryActivity JNI Bindings** `est:1h`
  - Why: The 10 WryActivity lifecycle JNI methods must be exported for the Dioxus-generated Kotlin code to call; without these, app crashes on launch with `UnsatisfiedLinkError`
  - Files: `src/platform/android_bindings.rs`, `src/platform/mod.rs`, `Cargo.toml`
  - Do: (1) Create `android_bindings.rs` with 10 `#[no_mangle]` JNI functions for create/start/resume/pause/stop/save/destroy/onActivityDestroy/memory/focus, (2) Add `ndk = "0.9"`, `ndk-context = "0.1"`, `libc = "0.2"` to `[target.'cfg(target_os = "android")'.dependencies]`, (3) Add `pub mod android_bindings;` to mod.rs, (4) Initialize ndk_context in create() function, (5) Build with `cargo build --target aarch64-linux-android --lib`, (6) Verify symbols with `nm -D`
  - Verify: `nm -D target/aarch64-linux-android/debug/libshusei.so | grep -c "Java_dev_dioxus_main_WryActivity"` returns 10
  - Done when: All 10 WryActivity JNI symbols are present in the compiled library

- [ ] **T02: Build APK and Verify on Device** `est:30m`
  - Why: Must verify the fix works on the actual target device (Moto G66j 5G) where the original crash occurred
  - Files: `target/dx/shusei/debug/android/app/app/build/outputs/apk/debug/app-debug.apk`, `scripts/android-build.sh`
  - Do: (1) Run `bash scripts/android-build.sh` to build APK, (2) Verify APK contains arm64-v8a libraries, (3) Connect Moto G66j 5G via USB, (4) Install with `adb install -r`, (5) Launch with `adb shell am start -n com.shusei.app/.MainActivity`, (6) Monitor logcat for WryActivity lifecycle calls and no UnsatisfiedLinkError
  - Verify: `adb logcat -d | grep -E "WryActivity|UnsatisfiedLinkError"` shows lifecycle calls and NO UnsatisfiedLinkError
  - Done when: App launches without crash, logcat shows WryActivity lifecycle methods being called

## Files Likely Touched

- `src/platform/android_bindings.rs` — New file with WryActivity JNI bindings
- `src/platform/mod.rs` — Add module registration
- `Cargo.toml` — Add ndk dependencies for Android target