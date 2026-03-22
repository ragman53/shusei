---
estimated_steps: 8
estimated_files: 3
skills_used:
  - debug-like-expert
---

# T02: Implement MainActivity.kt with CameraX capture

**Slice:** S01 — Kotlin Camera Implementation
**Milestone:** M003

## Description

Create a Kotlin MainActivity with CameraX implementation that provides the static JNI methods expected by the Rust side. The Rust code in `src/platform/android.rs` calls static methods on `dev.dioxus.main.MainActivity`: `startCameraCapture()`, `hasCameraPermission()`, `requestCameraPermission()`, and `vibrate()`. The captured image must be passed back via the `onImageCaptured(byte[], int, int)` native callback.

## Steps

1. Create directory structure: `platform/android/app/src/main/kotlin/dev/dioxus/main/`
2. Create `MainActivity.kt` extending `WryActivity` (Dioxus base class)
3. Add companion object with static JNI methods:
   - `hasCameraPermission(): Boolean` — Check CAMERA permission
   - `requestCameraPermission()` — Request CAMERA permission via ActivityCompat
   - `startCameraCapture()` — Initialize CameraX and capture image
   - `vibrate(Long)` — Trigger device vibration
4. Implement CameraX setup in `onCreate()`:
   - Initialize `ProcessCameraProvider`
   - Create `ImageCapture` use case
   - Store reference for later capture
5. Implement `startCameraCapture()`:
   - Check permission first
   - Call `imageCapture.takePicture()` with `OnImageCapturedCallback`
   - In `onCaptureSuccess`: convert `ImageProxy` to JPEG byte array
   - Call `onImageCaptured()` native method with byte array and dimensions
6. Add native method declarations for callbacks:
   - `external fun onImageCaptured(imageData: ByteArray, width: Int, height: Int)`
   - `external fun onImageCaptureFailed(errorMessage: String)`
7. Extend `android-patch.sh` to copy MainActivity.kt into generated project:
   - Source: `platform/android/app/src/main/kotlin/dev/dioxus/main/MainActivity.kt`
   - Target: `target/dx/shusei/debug/android/app/app/src/main/kotlin/dev/dioxus/main/MainActivity.kt`
8. Test compilation with `dx build --platform android && bash scripts/android-patch.sh`

## Must-Haves

- [ ] MainActivity.kt created with correct package name `dev.dioxus.main`
- [ ] All static JNI methods implemented (startCameraCapture, hasCameraPermission, requestCameraPermission, vibrate)
- [ ] CameraX ImageCapture configured with JPEG output
- [ ] Native callback `onImageCaptured` correctly invoked with byte array
- [ ] android-patch.sh copies MainActivity.kt to target directory

## Verification

- `test -f platform/android/app/src/main/kotlin/dev/dioxus/main/MainActivity.kt`
- `grep -q "fun startCameraCapture" platform/android/app/src/main/kotlin/dev/dioxus/main/MainActivity.kt`
- `grep -q "external fun onImageCaptured" platform/android/app/src/main/kotlin/dev/dioxus/main/MainActivity.kt`

## Inputs

- `src/platform/android.rs` — JNI interface defining expected Kotlin method signatures
- `scripts/android-patch.sh` — Patch script to extend for file copy

## Expected Output

- `platform/android/app/src/main/kotlin/dev/dioxus/main/MainActivity.kt` — New Kotlin file with CameraX implementation
- `scripts/android-patch.sh` — Modified to copy MainActivity.kt

## Observability Impact

- **Runtime signals changed**: 
  - Logcat logs with tag `ShuseiCamera` for permission checks, camera state, capture events
  - Logcat logs with tag `CameraX` for CameraX library internal events
  - JNI callback logs showing image dimensions and byte array size
- **How a future agent inspects this task**:
  - `adb logcat | grep -E "(ShuseiCamera|CameraX|onImageCaptured|startCameraCapture)"` — filter camera-related logs
  - Check `onImageCaptured` native callback is invoked with valid byte array
  - Verify `onImageCaptureFailed` is called with descriptive error on failure
- **Failure state visibility**:
  - Permission denied: logged in `hasCameraPermission()` and `requestCameraPermission()`
  - Camera open failure: `onImageCaptureFailed()` with error message
  - Capture failure: `onImageCaptureFailed()` with CameraX error details
  - Null instance: early return with warning log in static methods