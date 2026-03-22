# S01: Kotlin Camera Implementation

**Goal:** 「Take Photo」ボタンでカメラが起動し、撮影した画像がRust側に渡る
**Demo:** ユーザーが「Take Photo」ボタンを押す → カメラが起動 → 撮影 → JPEG画像がRust側のonImageCapturedに渡る

## Must-Haves

- CameraX依存関係がbuild.gradle.ktsに追加されている
- MainActivity.ktにstartCameraCapture()メソッドが実装されている
- 撮影した画像がJPEGバイト配列としてRust側のJNIコールバックに渡される
- カメラ権限リクエストが正しく処理される
- android-patch.shがMainActivity.ktを正しく適用する

## Proof Level

- This slice proves: integration
- Real runtime required: yes (Android実機でのカメラ動作)
- Human/UAT required: yes (実機での撮影確認)

## Verification

- `bash scripts/verify-s01-camera.sh` — カメラ起動、撮影、JNIコールバックの検証スクリプト
- `adb logcat | grep -E "(CameraX|onImageCaptured|startCameraCapture)"` — ログでカメラ動作確認

## Observability / Diagnostics

- Runtime signals: Logcat logs (CameraX, ShuseiCamera tags), JNI callback logs with image dimensions
- Inspection surfaces: adb logcat filtering on ShuseiCamera tag
- Failure visibility: Error messages in logcat, onImageCaptureFailed callback with error description
- Redaction constraints: None

## Integration Closure

- Upstream surfaces consumed: `src/platform/android.rs` JNI interface (startCameraCapture call, onImageCaptured callback)
- New wiring introduced in this slice: MainActivity.kt CameraX implementation bridges Rust → Camera → Rust
- What remains before the milestone is truly usable end-to-end: S02 (PDF file picker), S03 (asset access), S04 (integration verification)

## Tasks

- [x] **T01: Add CameraX dependencies to Gradle and implement patch** `est:1h`
  - Why: Dioxus generates empty build.gradle.kts; need CameraX libraries for camera capture
  - Files: `scripts/android-patch.sh`, `platform/android/app/build.gradle.kts` (new)
  - Do: Add CameraX dependencies section to android-patch.sh; extend patch script to append dependencies to generated build.gradle.kts
  - Verify: `grep -q "camera-core" scripts/android-patch.sh`
  - Done when: After `dx build --platform android && bash scripts/android-patch.sh`, the generated build.gradle.kts contains CameraX dependencies

- [x] **T02: Implement MainActivity.kt with CameraX capture** `est:2h`
  - Why: Dioxus generates empty MainActivity.kt; need static methods (startCameraCapture, hasCameraPermission, etc.) that Rust JNI calls
  - Files: `platform/android/app/src/main/kotlin/dev/dioxus/main/MainActivity.kt` (new), `scripts/android-patch.sh`
  - Do: Create MainActivity.kt with CameraX ImageCapture use case; implement startCameraCapture(), hasCameraPermission(), requestCameraPermission(), vibrate(); extend android-patch.sh to copy this file into generated project
  - Verify: `test -f platform/android/app/src/main/kotlin/dev/dioxus/main/MainActivity.kt && grep -q "startCameraCapture" platform/android/app/src/main/kotlin/dev/dioxus/main/MainActivity.kt`
  - Done when: Kotlin file exists with all required JNI methods; patch script copies it to target directory

- [x] **T03: Create verification script and test on device** `est:1h`
  - Why: Need automated way to verify camera capture works end-to-end on physical device
  - Files: `scripts/verify-s01-camera.sh` (new)
  - Do: Create script that installs APK, launches app, monitors logcat for camera capture events, reports success/failure
  - Verify: `test -x scripts/verify-s01-camera.sh`
  - Done when: Script exists and documents manual steps for UAT; provides logcat monitoring for camera events

## Files Likely Touched

- `scripts/android-patch.sh` — Extended to add CameraX dependencies and copy MainActivity.kt
- `platform/android/app/src/main/kotlin/dev/dioxus/main/MainActivity.kt` — New file with CameraX implementation
- `scripts/verify-s01-camera.sh` — New verification script