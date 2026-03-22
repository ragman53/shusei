# S02: Kotlin File Picker Implementation

**Goal:** 「Import PDF」ボタンでファイル選択ダイアログが開き、選択したPDFがRust側に渡る
**Demo:** User taps "Import PDF" button → System file picker opens → User selects a PDF → File path is passed to Rust via JNI callback

## Must-Haves

- `pickPdfFile()` static method that launches SAF file picker with `application/pdf` MIME type
- `ActivityResultLauncher<Uri?>` registered with `OpenDocument()` contract in companion object
- Native callback declarations: `onFilePicked(filePath: String)` and `onFilePickFailed(errorMessage: String)`
- URI to file path conversion: Copy file content from SAF URI to internal storage, return the internal path
- Null/cancel handling: When user cancels the picker, call `onFilePickFailed` with appropriate message
- Instance null check: All static methods check `instance != null` before proceeding

## Proof Level

- This slice proves: integration
- Real runtime required: yes
- Human/UAT required: yes

## Verification

- `bash scripts/verify-s02-file-picker.sh` — Automated verification with logcat monitoring and success/failure signal detection
- Manual UAT: Tap "Import PDF" button → Select PDF file → Verify logcat shows `onFilePicked` callback with file path

## Observability / Diagnostics

- Runtime signals: Logcat tag `ShuseiFile` for all file picker operations (launch, result, URI conversion, callback)
- Inspection surfaces: `adb logcat | grep -E "(ShuseiFile|onFilePicked|pickPdfFile)"` — Primary debugging tool
- Failure visibility: `onFilePickFailed` callback with specific error message (null URI, IO error, permission issue)
- Redaction constraints: None (file paths are not sensitive)

## Integration Closure

- Upstream surfaces consumed: `src/platform/android.rs` JNI callbacks (`onFilePicked`, `onFilePickFailed`) — already implemented
- New wiring introduced in this slice: `pickPdfFile()` Kotlin method called from Rust via `pick_file()` in `android.rs`
- What remains before the milestone is truly usable end-to-end: S03 (Asset Access), S04 (Integration Verification)

## Tasks

- [x] **T01: Add File Picker Implementation to MainActivity.kt** `est:1h`
  - Why: The Rust side has JNI callbacks ready but the Kotlin side needs to implement the SAF file picker and wire up the JNI bridge
  - Files: `platform/android/app/src/main/kotlin/dev/dioxus/main/MainActivity.kt`
  - Do: Add `ActivityResultLauncher` registration in companion object, implement `pickPdfFile()` static method, declare native callbacks (`onFilePicked`, `onFilePickFailed`), implement URI-to-path conversion by copying content to internal storage, handle null/cancel cases
  - Verify: `grep -q "pickPdfFile\|onFilePicked\|ActivityResultLauncher" platform/android/app/src/main/kotlin/dev/dioxus/main/MainActivity.kt`
  - Done when: File picker launches from Rust call, selected PDF path is passed back via JNI callback, cancellation is handled gracefully

- [x] **T02: Create Verification Script for S02** `est:30m`
  - Why: Need automated verification with logcat monitoring to detect success/failure signals during manual UAT
  - Files: `scripts/verify-s02-file-picker.sh`
  - Do: Create verification script following S01 pattern: device check, APK install, logcat monitoring with `ShuseiFile` tag, success/failure signal detection (`onFilePicked` = success, `onFilePickFailed` = failure), color-coded output, timestamped log persistence
  - Verify: `test -x scripts/verify-s02-file-picker.sh && grep -q "onFilePicked\|ShuseiFile" scripts/verify-s02-file-picker.sh`
  - Done when: Script exists, is executable, contains checks for file picker JNI callbacks

## Files Likely Touched

- `platform/android/app/src/main/kotlin/dev/dioxus/main/MainActivity.kt` — Add file picker methods and ActivityResultLauncher
- `scripts/verify-s02-file-picker.sh` — Automated verification script (new file)