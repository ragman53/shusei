---
task_id: T02
slice_id: S02
milestone_id: M003
status: complete
blocker_discovered: false
files_modified:
  - scripts/verify-s02-file-picker.sh
---

# T02-SUMMARY: Create Verification Script for S02

## One-Liner

Create automated verification script for PDF file picker with logcat monitoring and JNI callback detection

## Summary

Created `scripts/verify-s02-file-picker.sh`, an automated verification script that follows the S01 camera verification pattern. The script checks device connectivity, installs the APK, monitors logcat for file picker operations, detects success/failure signals, and provides color-coded output with timestamped log persistence.

## Implementation Details

### Script Features

1. **Device and APK checks**: Verifies Android device connection via `adb devices` and APK presence at expected path before proceeding

2. **APK installation and app launch**: Installs APK with `adb install -r` and launches MainActivity using `adb shell am start`

3. **Logcat monitoring**: Starts background logcat process filtering for `ShuseiFile|onFilePicked|onFilePickFailed|pickPdfFile` tags with PID tracking for cleanup

4. **Success signal detection**:
   - `pickPdfFile called` - Method invocation
   - `Launching file picker for PDF` - Picker launch
   - `File picker result:` - Result received with URI
   - `File selected, copying to internal storage` - Copy initiation
   - `File copied successfully:` - Copy completion with path
   - `onFilePicked` - JNI callback to Rust (primary success signal)

5. **Failure signal detection**:
   - `onFilePickFailed` - JNI failure callback
   - `User cancelled` - Graceful cancel handling (logged as warning, not failure)
   - ERROR/FATAL/Exception patterns in logs

6. **Manual UAT prompts**: Clear step-by-step instructions for tapping "Import PDF" button, selecting a PDF, or testing cancel flow

7. **Color-coded output**: Green for success, red for failures, yellow for warnings

8. **Timestamped log persistence**: Saves logs to `/tmp/logcat-s02-YYYYMMDD-HHMMSS.log` for post-mortem analysis

### Patterns Followed

- Mirrors S01 verification script structure (`verify-s01-camera.sh`)
- Consistent logcat tag filtering approach
- Same color scheme and output formatting
- Parallel signal detection pattern (success vs failure callbacks)

## Verification Evidence

| Check | Command | Exit Code | Verdict |
|-------|---------|-----------|---------|
| Script exists | `test -f scripts/verify-s02-file-picker.sh` | 0 | ✅ pass |
| Script is executable | `test -x scripts/verify-s02-file-picker.sh` | 0 | ✅ pass |
| Checks for onFilePicked | `grep -q "onFilePicked" scripts/verify-s02-file-picker.sh` | 0 | ✅ pass |
| Checks for ShuseiFile | `grep -q "ShuseiFile" scripts/verify-s02-file-picker.sh` | 0 | ✅ pass |
| Checks for onFilePickFailed | `grep -q "onFilePickFailed" scripts/verify-s02-file-picker.sh` | 0 | ✅ pass |
| Has color-coded output | `grep -q "RED=\|GREEN=\|YELLOW=" scripts/verify-s02-file-picker.sh` | 0 | ✅ pass |
| Saves timestamped logs | `grep -q "/tmp/logcat-s02-" scripts/verify-s02-file-picker.sh` | 0 | ✅ pass |
| Has manual UAT prompts | `grep -q "MANUAL TEST STEPS" scripts/verify-s02-file-picker.sh` | 0 | ✅ pass |

## Observability Impact

- **No runtime code changes**: This task only creates a verification script, no application code is modified
- **Signals inspected**: Logcat tag `ShuseiFile`, JNI callbacks `onFilePicked` and `onFilePickFailed`
- **Failure visibility**: Script detects `onFilePickFailed`, `User cancelled`, ERROR/FATAL/Exception patterns in logs
- **Debugging aid**: Timestamped logs saved to `/tmp/logcat-s02-*.log` for post-mortem analysis

## Files Modified

- `scripts/verify-s02-file-picker.sh` — New automated verification script (8.5KB, 240 lines)
