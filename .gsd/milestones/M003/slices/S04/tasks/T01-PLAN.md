---
estimated_steps: 8
estimated_files: 1
skills_used:
  - debug-like-expert
---

# T01: Create Unified Integration Verification Script

**Slice:** S04 — Integration Verification
**Milestone:** M003

## Description

Create a unified bash script that orchestrates all three M003 verification flows (camera capture, PDF file picker, demo PDF asset loading) in a single execution. The script should install the APK once, run each flow with manual UAT prompts, and aggregate results into a consolidated report.

## Steps

1. Create `scripts/verify-s04-integration.sh` with bash shebang and set -e
2. Define color codes and counters for aggregated reporting
3. Add device connection check (reuse pattern from existing scripts)
4. Add APK presence check at `target/dx/shusei/debug/android/app/app/build/outputs/apk/debug/app-debug.apk`
5. Install APK once and launch app (single install for all flows)
6. Create three verification sections:
   - **Camera Flow**: Start logcat with ShuseiCamera filter, prompt user to grant permission and tap "Take Photo", check for onImageCaptured callback
   - **File Picker Flow**: Continue logcat with ShuseiFile filter, prompt user to tap "Import PDF" and select a file, check for onFilePicked callback
   - **Demo PDF Flow**: Continue logcat with Asset filter, prompt user to tap "Load Demo PDF", check for asset copy confirmation
7. After all flows complete, produce aggregated report with per-flow success counts and overall M003 status
8. Save combined logcat to `/tmp/logcat-s04-YYYYMMDD-HHMMSS.log`

## Must-Haves

- [ ] Single APK install reused across all three verification flows
- [ ] Per-flow breakdown with success/failure counts
- [ ] Consolidated "M003 VERIFICATION PASSED" or "M003 VERIFICATION FAILED" banner
- [ ] Logcat saved with timestamp for post-mortem analysis
- [ ] Manual UAT prompts between each flow
- [ ] Exit code 0 for pass, non-zero for any failure

## Verification

- `test -x scripts/verify-s04-integration.sh`
- `grep -q "M003 VERIFICATION" scripts/verify-s04-integration.sh`
- `grep -q "Camera Flow" scripts/verify-s04-integration.sh`
- `grep -q "File Picker Flow" scripts/verify-s04-integration.sh`
- `grep -q "Demo PDF Flow" scripts/verify-s04-integration.sh`

## Observability Impact

- Signals added/changed: Combined logcat output with all ShuseiCamera, ShuseiFile, and Asset tags in one file
- How a future agent inspects this: Run `bash scripts/verify-s04-integration.sh` or check `/tmp/logcat-s04-*.log`
- Failure state exposed: Per-flow breakdown shows exactly which component failed (camera, file picker, or asset)

## Inputs

- `scripts/verify-s01-camera.sh` — Existing camera verification script for logcat patterns and success signals
- `scripts/verify-s02-file-picker.sh` — Existing file picker verification script for logcat patterns
- `scripts/verify-s03-asset.sh` — Existing asset verification script for APK inspection and asset checks
- `target/dx/shusei/debug/android/app/app/build/outputs/apk/debug/app-debug.apk` — APK to install and test

## Expected Output

- `scripts/verify-s04-integration.sh` — New unified integration verification script (executable)