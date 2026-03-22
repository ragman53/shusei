---
task_id: T01
slice_id: S02
milestone_id: M003
status: complete
blocker_discovered: false
files_modified:
  - platform/android/app/src/main/kotlin/dev/dioxus/main/MainActivity.kt
---

# T01-SUMMARY: Add File Picker Implementation to MainActivity.kt

## One-Liner

Implement PDF file picker using Storage Access Framework with JNI callbacks to Rust

## Summary

Implemented the Kotlin side of the PDF file picker in MainActivity.kt, following the established JNI bridge pattern from S01 (camera implementation). The file picker uses Android's Storage Access Framework (SAF) with the `OpenDocument()` contract to launch a system file picker filtered to PDF files.

## Implementation Details

### Changes Made

1. **Added imports**: `ActivityResultLauncher`, `ActivityResultContracts.OpenDocument`, `Uri`, `Context`, `IOException`

2. **Added file picker launcher**: `ActivityResultLauncher<Uri?>` registered in companion object using `registerForActivityResult()` with `OpenDocument()` contract

3. **Implemented `pickPdfFile()` static method**: Launches the file picker with MIME type filter `application/pdf`, includes instance null check, logs with `ShuseiFile` tag

4. **Declared native callbacks**: 
   - `@JvmStatic private external fun onFilePicked(filePath: String)`
   - `@JvmStatic private external fun onFilePickFailed(errorMessage: String)`

5. **Implemented `copyUriToFiles()` helper**: Copies URI content to internal storage (`context.filesDir`) with timestamped filename, returns absolute path, handles IOException

6. **Updated `onCreate()`**: Calls `initializeFilePickerLauncher(this)` to set up the launcher after instance is stored

7. **Added `TAG_FILE` constant**: Uses `ShuseiFile` tag for all file picker logging (separate from camera's `ShuseiCamera` tag)

### Key Patterns Followed

- Instance null checks in all static methods before proceeding
- Consistent logging with `ShuseiFile` tag for file picker operations
- JNI callback pattern matches S01 camera implementation
- Error handling with specific error messages passed to `onFilePickFailed`
- User cancel case handled explicitly with "User cancelled" message

## Verification Evidence

| Check | Command | Exit Code | Verdict |
|-------|---------|-----------|---------|
| pickPdfFile method exists | `grep -q "pickPdfFile" MainActivity.kt` | 0 | ✅ pass |
| onFilePicked native callback declared | `grep -q "onFilePicked" MainActivity.kt` | 0 | ✅ pass |
| ActivityResultLauncher registered | `grep -q "ActivityResultLauncher" MainActivity.kt` | 0 | ✅ pass |
| OpenDocument contract used | `grep -q "OpenDocument" MainActivity.kt` | 0 | ✅ pass |
| ShuseiFile tag used | `grep -q "ShuseiFile" MainActivity.kt` | 0 | ✅ pass |
| onFilePickFailed native callback declared | `grep -q "onFilePickFailed" MainActivity.kt` | 0 | ✅ pass |
| copyUriToFiles helper method exists | `grep -q "copyUriToFiles" MainActivity.kt` | 0 | ✅ pass |

## Observability Impact

- **Logcat tag**: `ShuseiFile` for all file picker operations (launch, result, URI conversion, callback)
- **Debug command**: `adb logcat | grep -E "(ShuseiFile|onFilePicked|pickPdfFile)"`
- **Failure signals**: `onFilePickFailed` callback with specific error messages (user cancelled, IO error, instance null)
- **Success signal**: `onFilePicked` callback with absolute file path

## Manual Testing Required

1. Build and deploy to Android device
2. Tap "Import PDF" button in the app
3. Select a PDF file from the system picker
4. Verify logcat shows:
   - `ShuseiFile: pickPdfFile called`
   - `ShuseiFile: Launching file picker for PDF`
   - `ShuseiFile: File picker result: <uri>`
   - `ShuseiFile: File selected, copying to internal storage...`
   - `ShuseiFile: File copied successfully: <path>`
   - `onFilePicked: file selected` (from Rust side)

## Files Modified

- `platform/android/app/src/main/kotlin/dev/dioxus/main/MainActivity.kt` — Added file picker implementation (imports, launcher, pickPdfFile method, native callbacks, copyUriToFiles helper, onCreate initialization)
