---
estimated_steps: 5
estimated_files: 1
skills_used:
  - debug-like-expert
  - best-practices
---

# T01: Add File Picker Implementation to MainActivity.kt

**Slice:** S02 — Kotlin File Picker Implementation
**Milestone:** M003

## Description

Add PDF file picker functionality to MainActivity.kt using the Storage Access Framework (SAF). The Rust side (`src/platform/android.rs`) already has the JNI callbacks implemented (`onFilePicked`, `onFilePickFailed`). This task implements the Kotlin side to launch the file picker, handle the result, and bridge back to Rust.

Follow the established JNI bridge pattern from S01 (camera implementation): static methods called from Rust, native callbacks to Rust, instance null checks, consistent logging.

## Steps

1. **Add ActivityResultLauncher registration in companion object**
   - Register `ActivityResultLauncher<Uri?>` using `registerForActivityResult()` with `OpenDocument()` contract
   - Set MIME type filter to `application/pdf`
   - Handle result: if null (user cancelled), call `onFilePickFailed("User cancelled")`; if URI, proceed to copy file

2. **Implement pickPdfFile() static method**
   - Add `@JvmStatic fun pickPdfFile()` that launches the registered launcher
   - Check `instance != null` before proceeding
   - Log with tag `ShuseiFile`

3. **Declare native callbacks to Rust**
   - Add `@JvmStatic private external fun onFilePicked(filePath: String)`
   - Add `@JvmStatic private external fun onFilePickFailed(errorMessage: String)`
   - These match the JNI callback signatures in `android.rs`

4. **Implement URI-to-path conversion**
   - Create helper method `copyUriToFiles(uri: Uri): String?`
   - Use `ContentResolver.openInputStream()` to read file content
   - Copy to `context.filesDir` with unique filename (e.g., `picked_<timestamp>.pdf`)
   - Return the absolute path string
   - Handle IOException with appropriate error message

5. **Wire up the launcher callback**
   - In the launcher's callback, call `copyUriToFiles(uri)`
   - On success: call `onFilePicked(path)`
   - On failure: call `onFilePickFailed(errorMessage)`

## Must-Haves

- [ ] `ActivityResultLauncher<Uri?>` registered in companion object with `OpenDocument()` contract
- [ ] `pickPdfFile()` static method launches the file picker
- [ ] `onFilePicked(filePath: String)` native callback declared
- [ ] `onFilePickFailed(errorMessage: String)` native callback declared
- [ ] URI content copied to internal storage and path returned
- [ ] Null/cancel case handled with `onFilePickFailed("User cancelled")`
- [ ] Instance null check in all static methods
- [ ] Logging with `ShuseiFile` tag

## Verification

- `grep -q "pickPdfFile" platform/android/app/src/main/kotlin/dev/dioxus/main/MainActivity.kt` — Method exists
- `grep -q "onFilePicked" platform/android/app/src/main/kotlin/dev/dioxus/main/MainActivity.kt` — Native callback declared
- `grep -q "ActivityResultLauncher" platform/android/app/src/main/kotlin/dev/dioxus/main/MainActivity.kt` — Launcher registered
- `grep -q "OpenDocument" platform/android/app/src/main/kotlin/dev/dioxus/main/MainActivity.kt` — SAF contract used
- Manual: Build, deploy, tap "Import PDF", select file, verify logcat shows `onFilePicked` with path

## Observability Impact

- Signals added/changed: Logcat tag `ShuseiFile` for picker launch, URI result, copy progress, JNI callbacks
- How a future agent inspects this: `adb logcat | grep ShuseiFile`
- Failure state exposed: `onFilePickFailed` with specific error message (user cancelled, IO error, null instance)

## Inputs

- `platform/android/app/src/main/kotlin/dev/dioxus/main/MainActivity.kt` — Current MainActivity with camera implementation from S01
- `src/platform/android.rs` — Rust JNI callbacks (already implemented, for reference)

## Expected Output

- `platform/android/app/src/main/kotlin/dev/dioxus/main/MainActivity.kt` — Updated with file picker methods and JNI bridge