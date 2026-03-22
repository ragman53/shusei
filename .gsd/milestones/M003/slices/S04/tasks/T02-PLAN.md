---
estimated_steps: 5
estimated_files: 1
skills_used:
  - debug-like-expert
---

# T02: Run Integration Verification and Document Results

**Slice:** S04 — Integration Verification
**Milestone:** M003

## Description

Execute the unified integration verification script on a physical Android device, follow the manual UAT steps for each flow, and capture the results in S04-UAT.md. This provides proof that M003 success criteria are met.

## Steps

1. Connect physical Android device via USB (verify with `adb devices`)
2. Execute `bash scripts/verify-s04-integration.sh`
3. Follow manual UAT prompts:
   - Camera Flow: Grant camera permission, tap "Take Photo", verify image captured
   - File Picker Flow: Tap "Import PDF", select a PDF file, verify import succeeds
   - Demo PDF Flow: Tap "Load Demo PDF", verify demo PDF loads
4. Record results in S04-UAT.md with:
   - Timestamp and device info (model, Android version)
   - Per-flow pass/fail status
   - Any errors or warnings encountered
   - Overall M003 verification status
5. If any flow fails, capture logcat excerpt for debugging

## Must-Haves

- [ ] Physical device connected and recognized by adb
- [ ] All three verification flows executed
- [ ] Results documented in S04-UAT.md
- [ ] Pass/fail status clearly recorded for each flow
- [ ] Overall M003 status documented

## Verification

- `grep -q "VERIFICATION PASSED\|VERIFICATION FAILED" .gsd/milestones/M003/slices/S04/S04-UAT.md`
- `grep -q "Camera Flow" .gsd/milestones/M003/slices/S04/S04-UAT.md`
- `grep -q "File Picker Flow" .gsd/milestones/M003/slices/S04/S04-UAT.md`
- `grep -q "Demo PDF Flow" .gsd/milestones/M003/slices/S04/S04-UAT.md`

## Inputs

- `scripts/verify-s04-integration.sh` — Unified verification script created in T01
- `target/dx/shusei/debug/android/app/app/build/outputs/apk/debug/app-debug.apk` — APK to test

## Expected Output

- `.gsd/milestones/M003/slices/S04/S04-UAT.md` — Verification results documentation with pass/fail status for each flow

## Observability Impact

**What signals change:** This task does not modify runtime code; it executes the verification script and documents results. The primary signal is the verification outcome (PASS/FAIL) and the combined logcat log at `/tmp/logcat-s04-*.log`.

**How a future agent inspects this task:** 
- Read `S04-UAT.md` for documented results, device info, and per-flow status
- Review `/tmp/logcat-s04-*.log` for raw logcat output if debugging is needed
- Check the timestamp and device model in the UAT report to correlate with specific test runs

**What failure state becomes visible:**
- Per-flow failure breakdown (Camera, File Picker, or Demo PDF)
- Specific error messages from logcat (e.g., `onImageCaptureFailed`, `onFilePickFailed`, `Asset not found`)
- Device connection issues or APK installation failures
- Manual UAT step failures (permission denied, file not found, etc.)