# S04: Integration Verification — Research

**Date:** 2026-03-22

## Summary

S04 is the final integration verification slice for M003 (Android Stability). It validates that all previously implemented features (camera capture, PDF file picker, demo PDF asset loading) work together end-to-end on a physical Android device without crashes.

The slice does **not** require new implementation — S01 (CameraX), S02 (File Picker), and S03 (Asset Access) have already delivered the necessary Kotlin code and JNI bridges. S04 focuses on **composing the existing verification scripts** into a unified integration test that exercises the complete user flows:

1. **Camera → OCR → Save flow**: Grant permission → Launch camera → Capture image → JNI callback → Rust processing
2. **PDF Import flow**: Tap Import → Select PDF → Copy to internal storage → JNI callback → Rust processing  
3. **Demo PDF flow**: Tap Load Demo → Asset bundled in APK → Copy to files directory → Import success
4. **Permission denial handling**: Deny camera/storage permission → App shows error → No crash

Three verification scripts already exist (`verify-s01-camera.sh`, `verify-s02-file-picker.sh`, `verify-s03-asset.sh`). S04 creates a **unified integration verification script** that runs all three flows sequentially and produces a single pass/fail report.

## Recommendation

**Create a unified integration verification script** that:
1. Installs the APK once and reuses the session across all tests
2. Runs camera, file picker, and asset tests in sequence
3. Aggregates results into a single summary report
4. Persists combined logs for post-mortem analysis
5. Provides clear pass/fail gates for M003 completion

This approach is recommended because:
- **Efficiency**: Single APK install, shared logcat session, reduced total test time
- **Clarity**: One command to verify the entire milestone, one report to review
- **Completeness**: Ensures all flows work together, not just in isolation
- **Repeatability**: Can be run before each milestone review or PR merge

## Implementation Landscape

### Key Files

- `scripts/verify-s04-integration.sh` — **New**: Unified integration verification script that orchestrates all three flows
- `scripts/verify-s01-camera.sh` — **Existing**: Camera capture verification (reuse logcat filtering patterns)
- `scripts/verify-s02-file-picker.sh` — **Existing**: File picker verification (reuse logcat filtering patterns)
- `scripts/verify-s03-asset.sh` — **Existing**: Asset bundling verification (reuse APK inspection logic)
- `platform/android/app/src/main/kotlin/dev/dioxus/main/MainActivity.kt` — **Existing**: Contains all three feature implementations
- `src/platform/android.rs` — **Existing**: JNI callbacks for all three flows

### Build Order

1. **Verify APK build and patch** — Ensure `android-patch.sh` has been run and APK includes:
   - CameraX dependencies
   - MainActivity.kt with all three feature implementations
   - Assets directory with `test/medium_pdf_test.pdf`

2. **Run unified integration script** — Execute `verify-s04-integration.sh` which:
   - Checks device connection
   - Installs APK once
   - Runs camera flow test (with manual UAT steps)
   - Runs file picker flow test (with manual UAT steps)
   - Runs demo PDF flow test (with manual UAT steps)
   - Aggregates all results

3. **Review combined logs** — Check `/tmp/logcat-s04-*.log` for any failures or warnings

### Verification Approach

The integration script should verify:

| Flow | Success Signals | Failure Signals |
|------|-----------------|-----------------|
| Camera | `CameraX use cases bound successfully`, `onImageCaptured`, `Image captured: X bytes` | `onImageCaptureFailed`, `Camera permission denied`, `Failed to bind camera` |
| File Picker | `File picker result:`, `File copied successfully:`, `onFilePicked` | `onFilePickFailed`, `IO error`, `User cancelled` (if not expected) |
| Demo PDF | `assets/test/medium_pdf_test.pdf` in APK, `Asset copied to:`, file exists in `/data/data/dev.dioxus.main/files/` | `Asset not found`, `Activity not initialized`, file not in files directory |
| Permission Handling | `Permission result: true` or `Permission result: false` (graceful handling) | App crash, `FATAL EXCEPTION`, unhandled `SecurityException` |

**Diagnostic commands:**
```bash
# Run integration verification
bash scripts/verify-s04-integration.sh

# Manual logcat monitoring (alternative)
adb logcat | grep -E "(ShuseiCamera|ShuseiFile|onImageCaptured|onFilePicked|Asset)"

# Check APK contents
unzip -l target/dx/shusei/debug/android/app/app/build/outputs/apk/debug/app-debug.apk | grep assets

# Check files on device
adb shell "ls -la /data/data/dev.dioxus.main/files/"
```

## Don't Hand-Roll

| Problem | Existing Solution | Why Use It |
|---------|------------------|------------|
| Logcat monitoring | Existing scripts use `adb logcat | grep` with PID tracking | Proven pattern, handles background process lifecycle correctly |
| APK installation | `adb install -r` with output capture | Standard Android tooling, handles incremental installs |
| Color-coded output | Existing scripts use ANSI color codes | Improves readability of pass/fail results |
| Log persistence | Copy to `/tmp/logcat-s04-*.log` with timestamp | Enables post-mortem analysis without re-running tests |

## Constraints

- **Physical device required**: Emulator cannot test camera hardware; real PDF files needed for file picker
- **Manual UAT steps**: Cannot automate button taps or file selection without additional tooling (e.g., Appium)
- **Timing dependencies**: 500ms delay in `takePhoto()` and async callbacks require wait periods in verification
- **Package name mismatch**: MainActivity uses `dev.dioxus.main` but some JNI callbacks reference `com.shusei.app` — both must be implemented (already done in `android.rs`)

## Common Pitfalls

- **Activity instance null**: Static methods check `instance != null` but there's a race window during startup. The 500ms delay in `takePhoto()` mitigates this.
- **Logcat buffer overflow**: Long test sessions may overflow the default logcat buffer. Use `adb logcat -G 4M` to increase buffer size if needed.
- **APK not patched**: Running verification without running `android-patch.sh` first will fail (missing CameraX, MainActivity.kt, assets).
- **Wrong package name in JNI callbacks**: Both `dev.dioxus.main` and `com.shusei.app` package callbacks are implemented in `android.rs` to handle Dioxus template variations.

## Open Risks

- **Device-specific camera behavior**: Different Android devices may have slightly different CameraX initialization timing. The 500ms delay may need adjustment on slower devices.
- **Storage permission model**: Android 13+ uses scoped storage; the current implementation uses `READ_EXTERNAL_STORAGE` which may behave differently on newer Android versions.
- **Asset path hardcoding**: The demo PDF path `test/medium_pdf_test.pdf` is hardcoded in multiple places. If the file moves, both the patch script and verification script need updates.

## Skills Discovered

No new skills required — this slice uses established patterns from S01-S03.

## Sources

- S01 Summary: `.gsd/milestones/M003/slices/S01/S01-SUMMARY.md` — CameraX implementation and verification patterns
- S02 Summary: `.gsd/milestones/M003/slices/S02/S02-SUMMARY.md` — File picker implementation (if available)
- S03 Summary: `.gsd/milestones/M003/slices/S03/S03-SUMMARY.md` — Asset access implementation (if available)
- M003 Roadmap: `.gsd/milestones/M003/M003-ROADMAP.md` — Success criteria and verification classes
