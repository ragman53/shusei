# S01: Kotlin Camera Implementation — UAT

**Milestone:** M003
**Written:** 2026-03-22

## UAT Type

- UAT mode: mixed (artifact-driven + live-runtime + human-experience)
- Why this mode is sufficient: Camera capture requires physical device interaction (human), JNI callback verification requires runtime log analysis (live-runtime), and build artifacts must be correct (artifact-driven)

## Preconditions

1. Android device connected via USB with debugging enabled (Motorola Moto G66j 5G or equivalent)
2. ADB accessible from host machine (WSL2 with USB passthrough configured)
3. APK built: `dx build --platform android && bash scripts/android-patch.sh`
4. APK exists at: `target/dx/shusei/debug/android/app/app/build/outputs/apk/debug/app-debug.apk`

## Smoke Test

```bash
# Quick check that verification script runs
bash scripts/verify-s01-camera.sh
```

Expected: Script detects device, installs APK, launches app, and prompts for manual test steps.

## Test Cases

### 1. Camera Permission Request Flow

1. Fresh install APK on device (uninstall first if previously installed)
2. Launch app
3. Navigate to "Take Photo" flow (create book or add page)
4. Tap camera trigger button

**Expected:** 
- System permission dialog appears requesting "Allow Shusei to take pictures and record video?"
- Logcat shows: `ShuseiCamera: Requesting camera permission`
- If denied: Logcat shows `ShuseiCamera: Permission result: android.permission.CAMERA, granted: false` and `notifyCaptureFailed: Camera permission denied`

### 2. Camera Capture Success

1. Grant camera permission when prompted
2. Wait for camera to initialize (500ms delay built in)
3. Capture photo

**Expected:**
- Logcat shows sequence:
  - `ShuseiCamera: startCameraCapture called`
  - `ShuseiCamera: CameraX use cases bound successfully`
  - `ShuseiCamera: takePhoto called`
  - `ShuseiCamera: onImageCaptured called, image size: X bytes, width: 1920, height: 1080`
- Rust side receives `onImageCaptured` callback with JPEG byte array
- No crash or ANR (Application Not Responding)

### 3. JNI Callback Verification

1. Run verification script: `bash scripts/verify-s01-camera.sh`
2. Follow prompts to grant permission and trigger capture
3. Press Enter when prompted to analyze logs

**Expected:**
- Script outputs: `✅ PASS: CameraX initialization detected`
- Script outputs: `✅ PASS: Camera trigger detected`
- Script outputs: `✅ PASS: Photo capture detected`
- Script outputs: `✅ PASS: JNI callback received`
- Exit code 0
- Log file saved to `/tmp/logcat-s01-YYYYMMDD-HHMMSS.log`

### 4. Permission Denial Handling

1. Fresh install APK
2. Launch app and trigger camera
3. Deny permission when prompted

**Expected:**
- Logcat shows: `ShuseiCamera: Permission result: android.permission.CAMERA, granted: false`
- Logcat shows: `ShuseiCamera: notifyCaptureFailed: Camera permission denied`
- App does not crash
- User can retry (trigger camera again to show permission dialog)

### 5. Build Verification

1. Run: `bash scripts/android-patch.sh`
2. Check generated build.gradle.kts: `grep androidx.camera target/dx/shusei/debug/android/app/app/build.gradle.kts`
3. Check MainActivity.kt copied: `test -f target/dx/shusei/debug/android/app/app/src/main/kotlin/dev/dioxus/main/MainActivity.kt`

**Expected:**
- Four CameraX dependencies present (camera-core, camera-camera2, camera-lifecycle, camera-view)
- MainActivity.kt exists in target directory
- All imports resolve (no red squiggles in Android Studio if opened)

## Edge Cases

### Rapid Permission Toggle

1. Grant permission, capture photo
2. Go to system settings, revoke camera permission
3. Return to app, trigger camera again

**Expected:** Permission dialog appears again; no crash; denial handled gracefully.

### Background During Capture

1. Trigger camera capture
2. Immediately press home button (background the app)

**Expected:** App handles lifecycle gracefully; no crash when returning to foreground. (Camera capture may fail, but app remains stable.)

### Low Storage

1. Fill device storage to near capacity
2. Attempt camera capture

**Expected:** Capture may fail with appropriate error logged; app does not crash.

## Failure Signals

- **App crashes on camera trigger**: Check logcat for `FATAL` or `Exception` in `ShuseiCamera` or `CameraX` tags
- **No JNI callback received**: Check Rust side implementation of `onImageCaptured` in `src/platform/android.rs`
- **Permission dialog not shown**: Check AndroidManifest.xml contains `<uses-permission android:name="android.permission.CAMERA" />`
- **CameraX initialization failure**: Check logcat for `Failed to get camera provider` or `CameraX use cases failed to bind`
- **Verification script fails**: Check `/tmp/logcat-s01-*.log` for detailed error messages

## Not Proven By This UAT

- OCR processing of captured image (proven in M002/S02)
- Image quality or resolution accuracy (fixed 1920x1080 assumed)
- Multiple rapid captures in succession (stress testing)
- Different camera lenses (wide, telephoto) — uses DEFAULT_BACK_CAMERA only
- Front camera functionality — back camera only

## Notes for Tester

- **USB debugging**: Ensure device shows "USB debugging connected" notification
- **WSL2 passthrough**: If using WSL2, ensure `adb devices` shows the device from within WSL2
- **First launch delay**: First camera initialization after install may take 1-2 seconds longer
- **Known rough edge**: The 500ms delay before capture is a heuristic; on slower devices, the camera may need more time to initialize. If capture fails with "Camera not ready" errors, increase the delay in MainActivity.kt.
- **Ignore**: Warnings about "CameraX is not ready" in logcat during normal operation — these are informational, not errors.
