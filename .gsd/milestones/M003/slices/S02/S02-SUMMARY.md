---
id: S02
parent: M003
milestone: M003
provides:
  - PDF file picker using Storage Access Framework with JNI callbacks
  - MainActivity.kt extended with pickPdfFile(), onFilePicked(), onFilePickFailed()
  - Verification script for file picker testing on physical devices
requires: []
affects:
  - S04 (file picker flow integrated into unified verification)
key_files:
  - platform/android/app/src/main/kotlin/dev/dioxus/main/MainActivity.kt — File picker implementation with JNI callbacks
  - scripts/verify-s02-file-picker.sh — Automated verification with logcat monitoring
key_decisions:
  - Storage Access Framework (SAF) over direct file access to avoid READ_EXTERNAL_STORAGE permission
  - URI-to-file copy pattern for persistent access (URI permissions are temporary)
  - Separate logcat tag "ShuseiFile" for file picker operations (distinct from camera's "ShuseiCamera")
patterns_established:
  - SAF pattern: Register ActivityResultLauncher with OpenDocument(), launch picker, copy URI to internal storage
  - Error handling pattern: Pass specific error messages to Rust via onFilePickFailed callback
  - User cancel handling: Return "User cancelled" message gracefully (not treated as failure)
observability_surfaces:
  - Logcat tag: ShuseiFile (all file picker operations, URI copy, callbacks)
  - JNI callback logs: onFilePicked (success with file path), onFilePickFailed (error message)
  - Verification log file: /tmp/logcat-s02-YYYYMMDD-HHMMSS.log
  - Verification script: scripts/verify-s02-file-picker.sh (color-coded pass/fail output)
drill_down_paths:
  - .gsd/milestones/M003/slices/S02/tasks/T01-SUMMARY.md
  - .gsd/milestones/M003/slices/S02/tasks/T02-SUMMARY.md
duration: 2h
verification_result: passed
completed_at: 2026-03-22
---

# S02: Kotlin File Picker Implementation

**PDF file picker using Storage Access Framework with JNI callbacks to Rust**

## What Happened

S02 delivered a complete Kotlin file picker implementation for PDF import, following the JNI bridge pattern established in S01. Two tasks composed the slice:

**T01: File Picker Implementation** — Extended MainActivity.kt with:
- `ActivityResultLauncher<Uri?>` registered with `OpenDocument()` contract
- Static method `pickPdfFile()` launching system file picker filtered to `application/pdf`
- Native callbacks: `onFilePicked(filePath: String)` and `onFilePickFailed(errorMessage: String)`
- Helper method `copyUriToFiles()` copying URI content to internal storage with timestamped filename
- Instance null checks and comprehensive logging with "ShuseiFile" tag
- User cancel handling with "User cancelled" message (graceful, not an error)

**T02: Verification Script** — Created scripts/verify-s02-file-picker.sh for automated testing:
- Device connectivity and APK presence checks
- APK installation and app launch via `adb shell am start`
- Background logcat monitoring for ShuseiFile tag and JNI callbacks
- Success signal detection: `pickPdfFile called`, `File picker result:`, `File copied successfully:`, `onFilePicked`
- Failure signal detection: `onFilePickFailed`, `User cancelled`, ERROR/FATAL/Exception patterns
- Manual UAT prompts for tapping "Import PDF" button and selecting a PDF
- Color-coded output with timestamped log persistence to `/tmp/logcat-s02-*.log`

The implementation follows the exact pattern from S01 (camera), demonstrating the repeatability of the JNI bridge approach.

## Verification

| Task | Verification Method | Result |
|------|---------------------|--------|
| T01 | grep pickPdfFile in MainActivity.kt; grep onFilePicked callback; grep ActivityResultLauncher | ✅ Pass |
| T02 | Script exists and is executable; contains onFilePicked and ShuseiFile checks; syntax valid | ✅ Pass |

All verification gates passed. The implementation is ready for physical device testing.

## New Requirements Surfaced

- none

## Deviations

- none

## Known Limitations

- **URI permissions are temporary**: The copied file path is returned to Rust, but the original URI permission expires. The copy-to-internal-storage pattern is required for persistent access.
- **SAF limitations**: File picker only shows files from apps that expose documents (Google Drive, Downloads, etc.). Direct filesystem browsing is not available without additional permissions.
- **Manual UAT required**: Full verification requires a human to select a PDF file on a physical device. The verification script automates log monitoring but cannot automate the file selection interaction.

## Follow-ups

- **S04**: File picker flow integrated into unified verification script (verify-s04-integration.sh)
- **Future**: Consider supporting multiple file types (not just PDF) if the use case expands

## Files Created/Modified

- `platform/android/app/src/main/kotlin/dev/dioxus/main/MainActivity.kt` — Extended with file picker implementation (modified)
- `scripts/verify-s02-file-picker.sh` — Automated verification script (created)

## Forward Intelligence

### What the next slice should know
- The JNI bridge pattern is repeatable: T01 followed the exact same structure as S01's camera implementation.
- SAF avoids storage permissions: No READ_EXTERNAL_STORAGE needed, which simplifies the manifest.
- The copy-to-internal-storage pattern is essential: URI permissions expire, so files must be copied for persistent access.

### What's fragile
- **Activity instance lifecycle**: All static methods check `instance != null` but there's a window during app startup where this could be null.
- **URI-to-file copy failures**: IOException during copy is caught and passed to `onFilePickFailed`, but the Rust side must handle this callback gracefully.
- **User cancel handling**: The "User cancelled" message is logged but not treated as a failure. The Rust side should distinguish between user cancel and actual errors.

### Authoritative diagnostics
- `adb logcat | grep ShuseiFile` — First place to check for file picker issues
- `adb logcat | grep onFilePicked` — Confirms successful JNI callback to Rust
- `scripts/verify-s02-file-picker.sh` — Run this for structured verification with saved logs
- `/tmp/logcat-s02-*.log` — Post-mortem analysis of failed test runs

### What assumptions changed
- **Assumption**: Direct file access would be simpler. **Reality**: SAF is cleaner (no permissions) and more user-friendly (familiar system picker).
- **Assumption**: URI could be used directly. **Reality**: URI permissions are temporary, so copying to internal storage is required for persistent access.
