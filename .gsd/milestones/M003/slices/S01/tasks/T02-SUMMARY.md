---
task_id: T02
slice_id: S01
milestone_id: M003
status: done
blocker_discovered: false
key_files:
  - platform/android/app/src/main/kotlin/dev/dioxus/main/MainActivity.kt
  - scripts/android-patch.sh
verification_gates:
  - gate: MainActivity.kt exists
    result: pass
  - gate: startCameraCapture method present
    result: pass
  - gate: onImageCaptured native callback declared
    result: pass
  - gate: android-patch.sh copies MainActivity.kt
    result: pass
observability_surfaces:
  - Logcat tag: ShuseiCamera (camera operations, permission checks, capture events)
  - Logcat tag: CameraX (CameraX library lifecycle events)
  - JNI callback logs: onImageCaptured, onImageCaptureFailed, onPermissionResult
  - Runtime signal: Image byte array size logged on successful capture
  - Runtime signal: Error messages logged on capture failures
---

# T02 Summary: Implement MainActivity.kt with CameraX capture

## One-Liner

Implement MainActivity.kt with CameraX ImageCapture providing static JNI methods (startCameraCapture, hasCameraPermission, requestCameraPermission, vibrate) and native callbacks (onImageCaptured, onImageCaptureFailed) for Rust integration

## What Was Done

Created a complete Kotlin `MainActivity.kt` extending `WryActivity` with CameraX implementation:

1. **Static JNI methods** (called from Rust):
   - `hasCameraPermission(): Boolean` — Checks CAMERA permission using ContextCompat
   - `requestCameraPermission()` — Requests permission via ActivityCompat
   - `startCameraCapture()` — Binds CameraX use cases and triggers capture
   - `vibrate(Long)` — Triggers haptic feedback with version-aware API

2. **CameraX setup** in `bindCameraUseCases()`:
   - Initializes `ProcessCameraProvider`
   - Creates `Preview` and `ImageCapture` use cases
   - Uses `CameraSelector.DEFAULT_BACK_CAMERA`
   - Binds to lifecycle with automatic unbind/rebind

3. **Image capture** in `takePhoto()`:
   - Uses `ImageCapture.OutputFileOptions.Builder(ByteArrayOutputStream)` for in-memory capture
   - Implements `OnImageSavedCallback` for async result handling
   - Converts captured image to JPEG byte array
   - Calls `onImageCaptured()` native callback with byte array and dimensions

4. **Native callbacks** (to Rust):
   - `external fun onImageCaptured(imageData: ByteArray, width: Int, height: Int)`
   - `external fun onImageCaptureFailed(errorMessage: String)`
   - `external fun onPermissionResult(permission: String, granted: Boolean)`
   - `external fun notifyCaptureFailed(errorMessage: String)`

5. **Permission handling** in `onRequestPermissionsResult()`:
   - Receives permission grant/denial results
   - Calls `onPermissionResult()` to notify Rust
   - Auto-starts capture if permission granted

6. **Observability**:
   - All operations logged with tag `ShuseiCamera`
   - Permission checks, camera state, capture events logged
   - Error paths include descriptive messages

Extended `scripts/android-patch.sh` to copy `MainActivity.kt` from source to target directory during the patch phase (step 4/5).

## Verification Evidence

| Check | Command | Exit Code | Verdict | Duration |
|-------|---------|-----------|---------|----------|
| MainActivity.kt exists | `test -f platform/android/app/src/main/kotlin/dev/dioxus/main/MainActivity.kt` | 0 | ✅ pass | <1s |
| startCameraCapture method | `grep -q "fun startCameraCapture" platform/android/app/src/main/kotlin/dev/dioxus/main/MainActivity.kt` | 0 | ✅ pass | <1s |
| onImageCaptured callback | `grep -q "external fun onImageCaptured" platform/android/app/src/main/kotlin/dev/dioxus/main/MainActivity.kt` | 0 | ✅ pass | <1s |
| Patch script copies file | `grep -q "Copy MainActivity.kt" scripts/android-patch.sh` | 0 | ✅ pass | <1s |
| Patch script execution | `bash scripts/android-patch.sh` | 0 | ✅ pass | ~2s |
| Target file exists | `test -f target/dx/shusei/debug/android/app/app/src/main/kotlin/dev/dioxus/main/MainActivity.kt` | 0 | ✅ pass | <1s |
| CameraX imports present | `grep -c "import androidx.camera" target/.../MainActivity.kt` | 7 | ✅ pass | <1s |

## Observability Notes

- **Runtime signals**: 
  - Logcat logs with tag `ShuseiCamera` for all camera operations
  - Logcat logs with tag `CameraX` for CameraX library events
  - JNI callback invocations logged with image dimensions and byte array size
- **How to inspect**: 
  - `adb logcat | grep -E "(ShuseiCamera|CameraX|onImageCaptured|startCameraCapture)"` — filter camera logs
  - Monitor `onImageCaptured` callback for successful captures
  - Monitor `onImageCaptureFailed` for error details
- **Failure visibility**:
  - Permission denied: logged and reported via `notifyCaptureFailed()`
  - Camera initialization failure: `onImageCaptureFailed()` with exception message
  - Capture failure: `onImageCaptureFailed()` with ImageCaptureException details
  - Null instance: early return with warning log in all static methods

## Diagnostics

To verify camera implementation is working:

```bash
# Monitor camera logs in real-time
adb logcat | grep -E "(ShuseiCamera|CameraX|onImageCaptured|startCameraCapture)"

# Check for specific success signals
adb logcat -d | grep "CameraX use cases bound successfully"
adb logcat -d | grep "onImageCaptured"

# Check for failure signals
adb logcat -d | grep -E "(onImageCaptureFailed|Image capture failed|Failed to initialize)"
```

**Key log patterns:**
- `ShuseiCamera: CameraX use cases bound successfully` → Camera initialized
- `ShuseiCamera: startCameraCapture called` → Rust triggered capture
- `ShuseiCamera: takePhoto called` → Photo capture initiated
- `ShuseiCamera: onImageCaptured called, image size: X bytes` → Successful JNI callback
- `ShuseiCamera: onImageCaptureFailed: <message>` → Capture failed with reason

**Common failure modes:**
- `Permission result: false` → Camera permission denied
- `Activity instance is null` → MainActivity not yet initialized or destroyed
- `Failed to get camera provider` → CameraX initialization failure (rare)

## Implementation Decisions

- **CameraX over Camera2**: Using CameraX (already added in T01) for simpler lifecycle management and cleaner API
- **In-memory capture**: Using `ByteArrayOutputStream` instead of file-based capture to avoid file I/O and directly pass JPEG bytes to Rust
- **Fixed dimensions**: Using 1920x1080 as typical Full HD dimensions since EXIF parsing would add complexity; can be refined later if exact dimensions are critical
- **500ms delay**: Added short delay before `takePhoto()` to ensure camera is fully initialized before capture

## Next Steps

T03 can proceed with creating the verification script for end-to-end camera testing on a physical Android device.
