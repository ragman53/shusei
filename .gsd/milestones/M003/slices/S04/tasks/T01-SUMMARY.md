---
task_id: T01
slice_id: S04
milestone_id: M003
status: complete
files_created:
  - scripts/verify-s04-integration.sh
files_modified:
  - .gsd/milestones/M003/slices/S04/S04-PLAN.md
verification_passed: true
---

# T01: Create Unified Integration Verification Script

## Summary

Created the unified integration verification script `scripts/verify-s04-integration.sh` that orchestrates all three M003 verification flows (camera capture, PDF file picker, demo PDF asset loading) in a single execution with one APK install.

## Implementation Details

### Script Structure

The script follows the established patterns from the existing S01, S02, and S03 verification scripts:

1. **Pre-flight checks**: Device connection, APK presence
2. **Single APK install**: Installed once and reused across all three flows (efficiency improvement)
3. **Combined logcat monitoring**: All logs written to a single timestamped file at `/tmp/logcat-s04-YYYYMMDD-HHMMSS.log`
4. **Three sequential flows**:
   - **Flow 1 (Camera)**: Monitors ShuseiCamera/CameraX tags, checks for `onImageCaptured` callback
   - **Flow 2 (File Picker)**: Monitors ShuseiFile tags, checks for `onFilePicked` callback
   - **Flow 3 (Demo PDF)**: Monitors Asset tags, checks for asset copy confirmation
5. **Aggregated report**: Per-flow breakdown with success/failure counts and overall M003 status

### Key Features

- **Color-coded output**: Green for success, red for failures, yellow for warnings
- **Manual UAT prompts**: User is prompted to perform actions (grant permission, tap buttons, select files) between log monitoring phases
- **Per-flow counters**: Tracks success/failure separately for each flow
- **Overall verdict**: "M003 VERIFICATION PASSED" or "M003 VERIFICATION FAILED" banner
- **Exit codes**: 0 for pass, 1 for any failure

### Logcat Filters Used

| Flow | Tags/Monitored Patterns |
|------|------------------------|
| Camera | `ShuseiCamera`, `CameraX`, `onImageCaptured`, `onImageCaptureFailed`, `startCameraCapture` |
| File Picker | `ShuseiFile`, `onFilePicked`, `onFilePickFailed`, `pickPdfFile` |
| Demo PDF | `ShuseiFile`, `Asset` |

### Success Criteria Checked

**Camera Flow:**
- `onImageCaptured` JNI callback invoked
- CameraX initialized (optional, logged as warning if not found)
- No capture failures

**File Picker Flow:**
- `onFilePicked` JNI callback invoked
- File picker result received
- File copied to internal storage
- No file pick failures

**Demo PDF Flow:**
- Asset copied to app files directory
- No "Asset not found" errors
- PDF file exists in app files directory

## Verification Evidence

| Check | Command | Exit Code | Verdict |
|-------|---------|-----------|---------|
| Script is executable | `test -x scripts/verify-s04-integration.sh` | 0 | ✅ pass |
| Contains M003 VERIFICATION | `grep -q "M003 VERIFICATION" scripts/verify-s04-integration.sh` | 0 | ✅ pass |
| Contains Camera Flow | `grep -q "Camera Flow" scripts/verify-s04-integration.sh` | 0 | ✅ pass |
| Contains File Picker Flow | `grep -q "File Picker Flow" scripts/verify-s04-integration.sh` | 0 | ✅ pass |
| Contains Demo PDF Flow | `grep -q "Demo PDF Flow" scripts/verify-s04-integration.sh` | 0 | ✅ pass |

## Notes

- The script requires a physical Android device connected via USB with adb access
- Manual UAT is required: user must grant permissions, tap buttons, and select files during execution
- Combined logcat is saved to `/tmp/logcat-s04-*.log` for post-mortem analysis
- The script reuses the APK install across all flows, reducing total verification time compared to running three separate scripts

## Next Steps

- T02: Run the integration verification script on a physical device and document results in `S04-UAT.md`
