# S01: Kotlin Camera Implementation — Research

**Date:** 2026-03-22

## Summary

This slice implements camera capture functionality in Kotlin for the Shusei Android app. The Rust JNI interface (`src/platform/android.rs`) already defines the contract: `startCameraCapture()` triggers camera, `onImageCaptured()` callback returns JPEG byte array to Rust. The current `MainActivity.kt` is empty (extends `WryActivity`), requiring full CameraX implementation.

**Primary approach:** Use CameraX 1.3+ with `ImageCapture` use case. The implementation captures images in-memory using `OnImageCapturedCallback`, converts to JPEG byte array, and passes to Rust via JNI. This avoids file I/O overhead and matches the existing Rust JNI signature that expects `byte[]` data.

**Key requirements:**
- Add CameraX dependencies to `build.gradle.kts`
- Implement `startCameraCapture()` that launches camera preview and captures image
- Implement `onImageCaptured()` JNI callback that sends JPEG bytes to Rust
- Handle runtime permissions (CAMERA)
- Store Activity reference for CameraX lifecycle binding

## Recommendation

**Use CameraX with in-memory capture** for the following reasons:

1. **Matches existing JNI contract** — Rust expects `byte[]` image data, not file path
2. **Modern Android standard** — CameraX is the recommended approach, handles device quirks
3. **Lifecycle-aware** — Automatically binds to Activity lifecycle
4. **No file cleanup needed** — In-memory capture avoids temp file management

**Implementation pattern:**
1. Add CameraX dependencies (`camera-core`, `camera-camera2`, `camera-lifecycle`, `camera-view`)
2. Add `PreviewView` to layout (or use invisible preview for background capture)
3. Initialize `ImageCapture` use case in `onCreate`
4. `startCameraCapture()` binds camera and triggers `takePicture()`
5. `OnImageCapturedCallback.onCaptureSuccess()` converts `ImageProxy` to JPEG byte array
6. Call `onImageCaptured()` JNI static method with byte array

**Don't hand-roll:**
- Permission handling — extend existing `PermissionHelper.kt`
- Camera lifecycle — use CameraX `ProcessCameraProvider`
- Image conversion — use `ImageProxy.toBitmap()` + `Bitmap.compress()`

## Implementation Landscape

### Key Files

- `target/dx/shusei/debug/android/app/app/src/main/kotlin/dev/dioxus/main/MainActivity.kt` — Empty class; add CameraX initialization, `startCameraCapture()`, `onImageCaptured()` callback
- `target/dx/shusei/debug/android/app/app/src/main/kotlin/dev/dioxus/main/PermissionHelper.kt` — Extend with `requestCameraPermission()` helper
- `target/dx/shusei/debug/android/app/app/build.gradle.kts` — Add CameraX dependencies
- `target/dx/shusei/debug/android/app/app/src/main/AndroidManifest.xml` — Add CAMERA permission (already in `platform/android/AndroidManifest.xml`, needs to be in patch script)
- `src/platform/android.rs` — JNI interface already defined; verify callback signatures match Kotlin implementation
- `scripts/android-patch.sh` — Add permission declarations and copy assets

### Build Order

1. **Add CameraX dependencies** — Unblocks all camera work
2. **Extend PermissionHelper** — Permission check/request needed before camera access
3. **Implement MainActivity camera methods** — Core functionality
4. **Update AndroidManifest via patch script** — Ensures permissions survive rebuilds
5. **Verify JNI callback signatures** — Ensure Rust and Kotlin match

### Verification Approach

**Compile verification:**
```bash
dx build --platform android
# Should compile without errors
```

**Runtime verification (manual on device):**
1. Install APK on Moto G66j 5G
2. Tap "Take Photo" button
3. Grant camera permission if prompted
4. Verify camera preview appears (or shutter sound if no preview)
5. Verify image is captured and OCR processing starts
6. Check logcat for JNI callback logs: `onImageCaptured: <width>x<height>`

**Logcat monitoring:**
```bash
adb logcat | grep -E "(CameraX|onImageCaptured|startCameraCapture|Permission)"
```

## Don't Hand-Roll

| Problem | Existing Solution | Why Use It |
|---------|------------------|------------|
| Camera API | CameraX 1.3+ | Android recommended; handles device-specific quirks; lifecycle-aware |
| Permission requests | PermissionHelper.kt (existing) | Already in codebase; follows Android best practices |
| Image conversion | ImageProxy.toBitmap() + Bitmap.compress() | Standard Android pattern; reliable JPEG encoding |
| Lifecycle binding | ProcessCameraProvider.bindToLifecycle() | CameraX handles lifecycle automatically |

## Constraints

- **Kotlin 1.9+** — Dioxus generates Kotlin 1.9 config
- **minSdk 24** — CameraX requires minSdk 21+, compatible
- **No camera preview UI** — App uses webview; camera may need invisible preview or background capture
- **JNI thread safety** — Callback must run on main thread; Rust side handles async
- **Image size** — OCR efficiency suggests ~1024px width; CameraX default may be larger

## Common Pitfalls

- **Activity reference null** — Ensure `nativeInit()` is called before camera access; use `JNI_OnLoad` fallback
- **Permission timing** — Request permission before binding camera; handle denial gracefully
- **Image rotation** — CameraX provides rotation info; must apply to bitmap before sending to Rust
- **Memory pressure** — Large images (12MP+) may OOM; consider downsampling before OCR
- **PreviewView requirement** — CameraX needs a surface; use invisible PreviewView or `ImageAnalysis` only

## Open Risks

- **PreviewView in webview app** — Dioxus uses webview; need to verify CameraX can work without visible preview
- **Image format mismatch** — Rust expects JPEG; verify `Bitmap.compress()` produces compatible format
- **Callback thread** — JNI callback may need to run on specific thread; verify Rust side thread safety

## Skills Discovered

No additional skills needed — standard Android CameraX implementation.

## Sources

- CameraX Take Photo (source: [Android Developers](https://developer.android.com/media/camera/camerax/take-photo))
- CameraX Getting Started (source: [Android Codelab](https://developer.android.com/codelabs/camerax-getting-started))
- ImageProxy to ByteArray conversion (source: [Stack Overflow](https://stackoverflow.com/questions/63245975/camerax-image-analysis-convert-image-to-bytearray-or-bytebuffer))
