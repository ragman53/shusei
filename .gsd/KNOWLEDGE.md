# Knowledge Base

<!-- Append-only learnings that save future agents from repeating investigation -->

## Android CameraX JNI Bridge Pattern

**Context:** M003/S01 implemented CameraX integration for Rust → Kotlin → Camera → Rust flow.

**Pattern:**
1. Kotlin MainActivity extends Dioxus-generated WryActivity
2. Static methods exposed for Rust JNI calls: `hasCameraPermission()`, `startCameraCapture()`
3. Native callbacks declared as `external fun` for Rust to implement: `onImageCaptured(imageData: ByteArray, width: Int, height: Int)`
4. Activity instance saved in `onCreate()` for static methods to access

**Key implementation details:**
- Always null-check `instance` before use — there's a window during startup where it's null
- Use `ByteArrayOutputStream` for in-memory capture to avoid file I/O
- Add small delay (500ms) before capture to ensure CameraX is fully initialized
- Log everything with a consistent tag (`ShuseiCamera`) for easy filtering

**Diagnostic command:**
```bash
adb logcat | grep -E "(ShuseiCamera|CameraX|onImageCaptured)"
```

## Dioxus Android Patch Script Pattern

**Context:** M002/M003 use post-generation patching for Dioxus Android builds.

**Pattern:**
1. Generate Android project: `dx build --platform android`
2. Run patch script: `bash scripts/android-patch.sh`
3. Patch script performs idempotent modifications:
   - Java version update (17 or 21)
   - Lint skip for faster builds
   - Manifest fixes
   - Dependency injection (CameraX, etc.)
   - Source file copying (MainActivity.kt)

**Idempotency technique:**
```bash
grep -q "marker" target/file || patch_command
```

**AWK dependency injection:**
```bash
awk '/dependencies \{/{found=1} found && /^\}/{print "    implementation(\"dep\")"; found=0} 1' file
```

**When to extend:**
- New Android dependencies → Add to patch script
- New Kotlin source files → Copy in patch script step
- Build configuration changes → Add conditional patch

## Android Verification Script Pattern

**Context:** M003/S01 created device testing automation.

**Pattern:**
1. Device check: `adb devices | grep -q "device$"`
2. APK presence check
3. Install and launch: `adb install -r && adb shell am start`
4. Background logcat: `adb logcat -c && adb logcat &`
5. Manual UAT prompts with `read -p`
6. Log analysis with `grep` for success/failure signals
7. Color-coded output (green=pass, red=fail, yellow=warning)
8. Timestamped log persistence to `/tmp/`

**Success signal detection:**
```bash
if grep -q "success_pattern" "$LOG_FILE"; then
    echo -e "${GREEN}✅ PASS${NC}: description"
fi
```

**Failure signal detection:**
```bash
if grep -qi "error\|fatal\|exception" "$LOG_FILE"; then
    echo -e "${RED}❌ FAIL${NC}: description"
fi
```

**Log persistence:**
```bash
LOG_FILE="/tmp/logcat-$(date +%Y%m%d-%H%M%S).log"
adb logcat > "$LOG_FILE" &
```

## JNI Callback Reliability

**Context:** M003/S01 implemented Rust ↔ Kotlin JNI callbacks.

**Learnings:**
- Declare callbacks as `external fun` in Kotlin, implement in Rust with `#[no_mangle]`
- Callback names must match exactly (case-sensitive)
- Threading: CameraX callbacks run on background thread; use `runOnUiThread` if updating UI
- Null safety: Always check Activity instance before calling callbacks

**Rust side signature:**
```rust
#[no_mangle]
pub extern "C" fn onImageCaptured(image_data: *const u8, len: usize, width: i32, height: i32) {
    // Convert to Vec<u8> and process
}
```

**Kotlin side declaration:**
```kotlin
external fun onImageCaptured(imageData: ByteArray, width: Int, height: Int)
```

## CameraX Initialization Timing

**Context:** M003/S01 encountered race condition between permission grant and camera capture.

**Problem:** Calling `takePhoto()` immediately after permission grant sometimes fails because CameraX use cases aren't fully bound.

**Solution:** Add 500ms delay before capture:
```kotlin
Handler(Looper.getMainLooper()).postDelayed({
    takePhoto()
}, 500)
```

**Alternative:** Listen for `CameraX use cases bound successfully` log before triggering capture (more complex, not implemented).

**When to revisit:** If users report intermittent capture failures on slower devices, increase delay or implement proper synchronization.

## Android Storage Access Framework (SAF) Pattern

**Context:** M003/S02 implemented PDF file picker using Android's Storage Access Framework.

**Pattern:**
1. Register `ActivityResultLauncher<Uri?>` with `ActivityResultContracts.OpenDocument()`
2. Launch picker with MIME type filter: `launcher.launch(arrayOf("application/pdf"))`
3. Handle result in callback: URI → copy to internal storage → return absolute path
4. Use `context.contentResolver.openInputStream()` to read URI content
5. Copy to `context.filesDir` with timestamped filename

**Key implementation details:**
- SAF avoids storage permissions — no READ_EXTERNAL_STORAGE needed
- URI permissions are temporary — copy to app storage for persistent access
- Handle user cancel gracefully: return "User cancelled" message via onFilePickFailed
- IOException handling: catch and pass error message to Rust via onFilePickFailed

**Diagnostic command:**
```bash
adb logcat | grep -E "(ShuseiFile|onFilePicked|pickPdfFile)"
```

## ARM64 APK Build Configuration

**Context:** M003/S05 fixed ABI mismatch (x86_64 APK vs arm64-v8a device) for physical device deployment.

**Required configuration:**
1. **NDK ABI filter** (in build.gradle.kts via android-patch.sh):
   ```kotlin
   ndk {
       abiFilters += listOf("arm64-v8a")
   }
   ```
2. **Rust target** (in android-build.sh or environment):
   ```bash
   export CARGO_BUILD_TARGET="aarch64-linux-android"
   dx build --platform android
   ```

**Verification:**
```bash
# Check device architecture
adb shell getprop ro.product.cpu.abi

# Inspect APK native libraries
unzip -l app-debug.apk | grep lib/

# Expected output: lib/arm64-v8a/*.so (no x86_64, armeabi-v7a, or x86)
```

**Common issues:**
- `INSTALL_FAILED_NO_MATCHING_ABIS`: APK contains wrong architecture libraries
- Fix: Set CARGO_BUILD_TARGET before dx build, add NDK ABI filter

**APK size impact:** Single-ABI APK (~149MB) vs universal APK (~500MB+)

## M003 Verification Script Taxonomy

**Context:** M003 created 5 verification scripts for different testing scopes.

| Script | Purpose | When to use |
|--------|---------|-------------|
| verify-s01-camera.sh | Camera flow only | Debugging camera issues |
| verify-s02-file-picker.sh | File picker flow only | Debugging PDF import issues |
| verify-s03-asset.sh | Asset bundling only | Debugging demo PDF loading |
| verify-s04-integration.sh | All three flows (unified) | Integration testing |
| verify-s05-arm64.sh | ARM64 validation + all flows | Pre-deployment verification |

**Script structure:**
1. Device architecture check (S05 only)
2. APK native library inspection (S05 only)
3. Device connection check
4. APK installation
5. App launch via `adb shell am start`
6. Combined logcat monitoring
7. Manual UAT prompts
8. Success/failure signal detection
9. Color-coded output with aggregated status

**Log file naming:**
- `/tmp/logcat-s01-YYYYMMDD-HHMMSS.log` (camera)
- `/tmp/logcat-s02-YYYYMMDD-HHMMSS.log` (file picker)
- `/tmp/logcat-s03-YYYYMMDD-HHMMSS.log` (asset)
- `/tmp/logcat-s04-YYYYMMDD-HHMMSS.log` (integration)
- `/tmp/logcat-s05-YYYYMMDD-HHMMSS.log` (ARM64 verification)

## Activity Instance Lifecycle Management

**Context:** M003/S01-S02 use static Kotlin methods that require Activity instance reference.

**Pattern:**
```kotlin
companion object {
    private var instance: MainActivity? = null
    
    fun initialize() {
        instance = this
    }
    
    @JvmStatic
    fun someMethod() {
        val context = instance?.context ?: run {
            Log.e(TAG, "Activity instance is null")
            return
        }
        // Proceed with context
    }
}
```

**Lifecycle considerations:**
- Instance is set in `onCreate()` after `super.onCreate()`
- Instance becomes stale if Activity is recreated (config changes, process death)
- Static methods must check `instance != null` before use
- There's a window during startup where instance is null — add delays if needed

**When to revisit:** If Activity recreation causes issues, consider using Application context or WeakReference.

## JNI Bridge Pattern for Rust ↔ Kotlin Communication

**Context:** M003 established a repeatable pattern for Rust ↔ Kotlin JNI bridges (camera, file picker, asset access).

**Kotlin side:**
```kotlin
companion object {
    // Static method callable from Rust
    @JvmStatic
    external fun someMethod(param: String)
    
    // Native callback implemented in Rust
    @JvmStatic
    private external fun onResult(data: ByteArray)
}
```

**Rust side:**
```rust
// Call Kotlin static method
#[jni_fn]
pub fn call_kotlin_method(env: &JNIEnv, class: JClass, param: JString) {
    // Method implementation
}

// Implement Kotlin callback
#[no_mangle]
pub extern "C" fn onResult(env: &JNIEnv, class: JClass, data: jbyteArray) {
    // Convert jbyteArray to Vec<u8> and process
}
```

**Key considerations:**
- Method names must match exactly (case-sensitive)
- Threading: Kotlin callbacks to Rust run on calling thread (often background)
- Data ownership: ByteArray is copied — Rust owns the data after callback
- Error handling: Pass error messages via separate failure callbacks

**Established callbacks in M003:**
- `onImageCaptured(imageData: ByteArray, width: Int, height: Int)`
- `onImageCaptureFailed(errorMessage: String)`
- `onPermissionResult(granted: Boolean)`
- `onFilePicked(filePath: String)`
- `onFilePickFailed(errorMessage: String)`
