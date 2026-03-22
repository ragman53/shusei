---
estimated_steps: 4
estimated_files: 1
skills_used:
  - debug-like-expert
---

# T03: Create verification script and test on device

**Slice:** S01 — Kotlin Camera Implementation
**Milestone:** M003

## Description

Create a verification script that installs the APK on a physical Android device, launches the app, and monitors logcat for camera capture events. The script should detect success signals (camera opened, image captured, JNI callback invoked) and failure signals (errors, exceptions, permission denials). This provides automated verification for the camera capture flow.

## Steps

1. Create `scripts/verify-s01-camera.sh` based on existing `verify-s02-camera.sh` pattern
2. Add APK installation check and auto-install if APK exists
3. Add app launch with `adb shell am start -n com.shusei.app/.MainActivity`
4. Add logcat monitoring for key events:
   - `CameraX` tag for camera lifecycle events
   - `onImageCaptured` log message for successful capture
   - `ERROR`/`FATAL`/`Exception` for failure detection
5. Add manual test instructions for UAT:
   - Navigate to camera capture page
   - Tap "Take Photo" button
   - Grant camera permission if prompted
   - Capture image
   - Verify OCR processing starts
6. Add summary output showing success/failure counts

## Must-Haves

- [ ] Script exists at `scripts/verify-s01-camera.sh`
- [ ] Script checks for connected device
- [ ] Script installs APK if needed
- [ ] Script monitors logcat for camera events
- [ ] Script provides clear pass/fail output

## Verification

- `test -x scripts/verify-s01-camera.sh`
- `grep -q "onImageCaptured" scripts/verify-s01-camera.sh`

## Inputs

- `scripts/verify-s02-camera.sh` — Existing pattern for device verification

## Expected Output

- `scripts/verify-s01-camera.sh` — New verification script for camera capture testing

## Observability Impact

- **Runtime signals added**: None (this is a verification script, not runtime code)
- **How to inspect**: 
  - Run `bash scripts/verify-s01-camera.sh` to execute automated verification
  - Script monitors logcat for `ShuseiCamera`, `CameraX`, `onImageCaptured`, `onImageCaptureFailed` tags
  - Logs saved to `/tmp/logcat-s01-*.log` for debugging
- **Failure visibility**:
  - Script detects permission denials, camera initialization failures, capture errors
  - Clear pass/fail output with color-coded results
  - Error messages extracted from logcat and displayed in summary