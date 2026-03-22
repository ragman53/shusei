---
estimated_steps: 4
estimated_files: 1
skills_used:
  - debug-like-expert
  - test
---

# T02: Create Verification Script for S02

**Slice:** S02 — Kotlin File Picker Implementation
**Milestone:** M003

## Description

Create an automated verification script for the file picker implementation that follows the pattern established in S01 (`verify-s01-camera.sh`). The script should check device connectivity, install APK, monitor logcat for success/failure signals, and present color-coded results.

## Steps

1. **Create script structure with device and APK checks**
   - Check `adb devices` for connected device
   - Verify APK exists at expected path
   - Install APK with `adb install -r`
   - Launch app with `adb shell am start`

2. **Add logcat monitoring for ShuseiFile tag**
   - Clear logcat before starting
   - Start background logcat process filtering for `ShuseiFile|onFilePicked|onFilePickFailed|pickPdfFile`
   - Track PID for cleanup

3. **Define success/failure signal detection**
   - Success signals: `onFilePicked` in logs, `File picked:` message
   - Failure signals: `onFilePickFailed` in logs, `User cancelled` message, ERROR/FATAL/Exception keywords
   - Color-coded output (green=pass, red=fail, yellow=warning)

4. **Add manual UAT instructions and summary**
   - Prompt user to tap "Import PDF" button
   - Prompt user to select a PDF file or cancel
   - Stop logcat, analyze logs, present summary
   - Save timestamped logs to `/tmp/logcat-s02-*.log`

## Must-Haves

- [ ] Device connectivity check with meaningful error message
- [ ] APK presence check and installation
- [ ] Logcat monitoring with `ShuseiFile` tag filter
- [ ] Success signal detection: `onFilePicked` callback logged
- [ ] Failure signal detection: `onFilePickFailed` or errors logged
- [ ] Color-coded output (green/red/yellow)
- [ ] Timestamped log persistence to `/tmp/`
- [ ] Manual UAT prompts for file selection

## Verification

- `test -f scripts/verify-s02-file-picker.sh` — Script exists
- `test -x scripts/verify-s02-file-picker.sh` — Script is executable
- `grep -q "onFilePicked" scripts/verify-s02-file-picker.sh` — Checks for JNI callback
- `grep -q "ShuseiFile" scripts/verify-s02-file-picker.sh` — Checks for logcat tag

## Inputs

- `scripts/verify-s01-camera.sh` — S01 verification script for pattern reference
- `platform/android/app/src/main/kotlin/dev/dioxus/main/MainActivity.kt` — Expected logcat tags (ShuseiFile)

## Expected Output

- `scripts/verify-s02-file-picker.sh` — Automated verification script with logcat monitoring

## Observability Impact

- **No runtime code changes**: This task only creates a verification script, no application code is modified
- **Signals inspected**: Logcat tag `ShuseiFile`, JNI callbacks `onFilePicked` and `onFilePickFailed`
- **Failure visibility**: Script detects `onFilePickFailed`, `User cancelled`, ERROR/FATAL/Exception patterns in logs
- **Debugging aid**: Timestamped logs saved to `/tmp/logcat-s02-*.log` for post-mortem analysis