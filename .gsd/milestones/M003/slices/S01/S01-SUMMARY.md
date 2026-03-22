---
id: S01
parent: M003
milestone: M003
provides:
  - CameraX dependencies injected into Dioxus-generated build.gradle.kts
  - MainActivity.kt with CameraX ImageCapture and JNI bridge methods
  - Verification script for end-to-end camera testing on physical devices
requires: []
affects:
  - S02 (reuses MainActivity.kt pattern)
  - S03 (depends on Activity reference initialized in S01)
key_files:
  - scripts/android-patch.sh — Extended with CameraX dependencies and MainActivity.kt copy
  - platform/android/app/src/main/kotlin/dev/dioxus/main/MainActivity.kt — CameraX implementation with JNI callbacks
  - scripts/verify-s01-camera.sh — Automated verification with logcat monitoring
key_decisions:
  - CameraX over Camera2 for simpler lifecycle management
  - In-memory capture via ByteArrayOutputStream to avoid file I/O
  - Fixed 1920x1080 dimensions (EXIF parsing deferred for simplicity)
  - 500ms delay before takePhoto() to ensure camera initialization
patterns_established:
  - Patch script pattern: Extend android-patch.sh for new dependencies and source files
  - JNI bridge pattern: Static Kotlin methods called from Rust, native callbacks to Rust
  - Logcat tagging: Use "ShuseiCamera" tag for all camera-related operations
  - Verification script pattern: Device check → APK install → logcat monitoring → manual UAT steps
observability_surfaces:
  - Logcat tag: ShuseiCamera (all camera operations, permission checks, capture events)
  - Logcat tag: CameraX (CameraX library lifecycle events)
  - JNI callback logs: onImageCaptured, onImageCaptureFailed, onPermissionResult
  - Verification log file: /tmp/logcat-s01-YYYYMMDD-HHMMSS.log
  - Verification script: scripts/verify-s01-camera.sh (color-coded pass/fail output)
drill_down_paths:
  - .gsd/milestones/M003/slices/S01/tasks/T01-SUMMARY.md
  - .gsd/milestones/M003/slices/S01/tasks/T02-SUMMARY.md
  - .gsd/milestones/M003/slices/S01/tasks/T03-SUMMARY.md
duration: 4h
verification_result: passed
completed_at: 2026-03-22
---

# S01: Kotlin Camera Implementation

**MainActivity.kt with CameraX ImageCapture providing JNI bridge for Rust camera integration**

## What Happened

S01 delivered a complete Kotlin camera implementation bridging Rust and Android CameraX. Three tasks composed the slice:

**T01: CameraX Dependencies** — Verified the existing android-patch.sh already injects CameraX 1.3.4 dependencies (camera-core, camera-camera2, camera-lifecycle, camera-view) into the Dioxus-generated build.gradle.kts. The AWK-based insertion logic ensures idempotent patching.

**T02: MainActivity.kt Implementation** — Created a comprehensive MainActivity.kt extending WryActivity with:
- Static JNI methods (`hasCameraPermission`, `requestCameraPermission`, `startCameraCapture`, `vibrate`) callable from Rust
- CameraX use case binding with Preview and ImageCapture
- In-memory JPEG capture via ByteArrayOutputStream (no file I/O)
- Native callbacks to Rust (`onImageCaptured`, `onImageCaptureFailed`, `onPermissionResult`)
- Permission result handling with auto-capture on grant
- Comprehensive logging with "ShuseiCamera" tag

**T03: Verification Script** — Created scripts/verify-s01-camera.sh for automated end-to-end testing:
- Device connectivity and APK presence checks
- Background logcat monitoring with PID tracking
- Manual UAT steps for permission grant and capture trigger
- Success/failure signal detection with color-coded output
- Timestamped log persistence to /tmp/logcat-s01-*.log

The patch script was extended to copy MainActivity.kt from platform/android/ to the target directory during the patch phase, ensuring the Kotlin source is bundled with every build.

## Verification

| Task | Verification Method | Result |
|------|---------------------|--------|
| T01 | grep camera-core in patch script; grep androidx.camera in generated build.gradle.kts | ✅ Pass |
| T02 | MainActivity.kt exists; startCameraCapture method present; onImageCaptured callback declared; patch script copies file | ✅ Pass |
| T03 | Script exists and is executable; contains onImageCaptured check | ✅ Pass |

All verification gates passed. The implementation is ready for physical device testing.

## New Requirements Surfaced

- none

## Deviations

- none

## Known Limitations

- **Fixed image dimensions**: Using 1920x1080 as typical Full HD dimensions. EXIF parsing for exact dimensions was deferred to reduce complexity. If exact dimensions are critical for OCR preprocessing, this should be revisited.
- **No preview surface**: CameraX Preview use case is bound but not displayed. This is acceptable for the "capture page photo" use case where the user sees the camera UI natively, but a preview widget would improve UX.
- **Manual UAT required**: Full verification requires a human to grant permissions and trigger capture on a physical device. The verification script automates log monitoring but cannot automate the actual camera interaction.

## Follow-ups

- **S02**: Reuse the MainActivity.kt pattern for file picker implementation (add pickPdfFile method and onFilePicked callback)
- **S03**: The Activity reference initialized in MainActivity.kt will be consumed by asset access methods (copyAssetToFiles)
- **Future**: Consider adding EXIF parsing for exact image dimensions if OCR accuracy requires it

## Files Created/Modified

- `scripts/android-patch.sh` — Extended to add CameraX dependencies and copy MainActivity.kt (modified)
- `platform/android/app/src/main/kotlin/dev/dioxus/main/MainActivity.kt` — CameraX implementation with JNI bridge (created)
- `scripts/verify-s01-camera.sh` — Automated verification script with logcat monitoring (created)

## Forward Intelligence

### What the next slice should know
- The JNI bridge pattern is established: static Kotlin methods called from Rust via JNI, native callbacks back to Rust. Follow this pattern for S02's file picker.
- MainActivity.kt is copied by android-patch.sh step 4/5. Any changes to MainActivity.kt require re-running the patch script.
- Logcat filtering with `adb logcat | grep -E "(ShuseiCamera|CameraX|onImageCaptured)"` is the primary debugging tool.

### What's fragile
- **Activity instance lifecycle**: All static methods check `instance != null` but there's a window during app startup where this could be null. The 500ms delay in takePhoto() mitigates this but doesn't eliminate it entirely.
- **Camera permission timing**: If the user denies permission, the app must handle the `onPermissionResult(false)` callback gracefully. The Rust side should implement this callback.
- **Dioxus template changes**: The AWK-based dependency injection assumes a specific build.gradle.kts format. If Dioxus changes the template, the patch may fail silently.

### Authoritative diagnostics
- `adb logcat | grep ShuseiCamera` — First place to check for camera-related issues
- `adb logcat | grep onImageCaptured` — Confirms successful JNI callback to Rust
- `scripts/verify-s01-camera.sh` — Run this for structured verification with saved logs
- `/tmp/logcat-s01-*.log` — Post-mortem analysis of failed test runs

### What assumptions changed
- **Assumption**: Camera2 API would be needed for fine-grained control. **Reality**: CameraX provides sufficient control with much simpler lifecycle management.
- **Assumption**: File-based capture would be required. **Reality**: ByteArrayOutputStream allows in-memory capture, avoiding file I/O and permissions.
