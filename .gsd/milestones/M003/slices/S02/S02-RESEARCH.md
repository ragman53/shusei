# S02: Kotlin File Picker Implementation — Research

**Date:** 2026-03-22
**Status:** Ready for planning

## Summary

Slice S02 implements PDF file selection on Android using the Storage Access Framework (SAF). The Rust side (`src/platform/android.rs`) already has the `pick_file()` method and JNI callbacks (`onFilePicked`, `onFilePickFailed`) implemented for both `dev.dioxus.main` and `com.shusei.app` packages. The Kotlin side (`MainActivity.kt`) needs to add the file picker implementation.

The recommended approach uses `ActivityResultLauncher` with `OpenDocument()` contract — the modern AndroidX API that replaces the deprecated `startActivityForResult()`. This approach requires no storage permissions on Android 4.4+ (API 19+) because the user explicitly grants access via the system picker.

## Recommendation

**Use Storage Access Framework with ActivityResultLauncher API** for the following reasons:

1. **No permissions required** — SAF uses explicit user consent via system picker, avoiding `READ_EXTERNAL_STORAGE` permission
2. **Modern API** — `ActivityResultLauncher` is the recommended pattern, safe across process death
3. **Cloud storage support** — SAF works with Google Drive, Dropbox, and other document providers
4. **Consistent with camera pattern** — Follows the same JNI bridge pattern established in S01

**Implementation approach:**
1. Add `ActivityResultLauncher<Uri?>` as a companion object property in `MainActivity`
2. Register the launcher with `OpenDocument()` contract in `onCreate()`
3. Add `pickPdfFile()` static method that launches the picker
4. Add `onFilePicked()` and `onFilePickFailed()` external callbacks to Rust
5. Handle URI result in the launcher callback, convert to path string, call JNI callback

## Implementation Landscape

### Key Files

- `/home/devuser/develop/shusei/.gsd/worktrees/M003/target/dx/shusei/debug/android/app/app/src/main/kotlin/dev/dioxus/main/MainActivity.kt` — Add file picker methods (`pickPdfFile`, `onFilePicked`, `onFilePickFailed`) and `ActivityResultLauncher` registration
- `/home/devuser/develop/shusei/.gsd/worktrees/M003/src/platform/android.rs` — Already has `pick_file()` and JNI callbacks (no changes needed)
- `/home/devuser/develop/shusei/.gsd/worktrees/M003/scripts/android-patch.sh` — May need update if new dependencies are required (likely not — `activity-ktx` is already included via `androidx.appcompat`)

### Build Order

1. **Add file picker launcher to MainActivity.kt** — Register `ActivityResultLauncher` in companion object
2. **Add `pickPdfFile()` static method** — Launch the SAF picker with MIME type `application/pdf`
3. **Add JNI callback declarations** — `external fun onFilePicked(filePath: String)` and `external fun onFilePickFailed(errorMessage: String)`
4. **Handle picker result** — In launcher callback, convert URI to path and call Rust callback
5. **Test on device** — Verify picker opens, file selection works, and data reaches Rust

### Verification Approach

```bash
# Build and deploy
bash scripts/android-patch.sh
cd target/dx/shusei/debug/android/app && ./gradlew assembleDebug

# Install on device
adb install -r app/build/outputs/apk/debug/app-debug.apk
adb shell am start -n com.shusei.app/.MainActivity

# Monitor logs
adb logcat | grep -E "(ShuseiFile|onFilePicked|pickPdfFile)"
```

**Observable behaviors:**
- 「Import PDF」button tap opens system file picker
- Selecting a PDF returns the file path to Rust
- Canceling the picker calls `onFilePickFailed` with appropriate message
- Logs show URI conversion and JNI callback execution

## Don't Hand-Roll

| Problem | Existing Solution | Why Use It |
|---------|------------------|------------|
| File picker UI | Storage Access Framework (SAF) | System-provided picker, consistent UX, no permissions needed, supports cloud storage |
| Activity result handling | `ActivityResultLauncher` with `OpenDocument()` | Lifecycle-aware, safe across process death, type-safe contract |
| URI to path conversion | `ContentResolver` + `openInputStream()` | Standard Android pattern, handles all document providers |

## Constraints

- **Minimum SDK 24** — SAF is available from API 19+, so full compatibility
- **Kotlin 1.9+** — Required for Dioxus Android templates
- **No direct file paths** — SAF returns URIs; must use `ContentResolver` to read content
- **MIME type filtering** — Can only filter by MIME type (`application/pdf`), not file extension

## Common Pitfalls

- **URI vs File Path** — SAF returns a `content://` URI, not a `file://` path. The Rust side expects a string path. Solution: Copy the file content to app's internal storage and return the new path, or pass the URI string and let Rust use a different mechanism.
- **Permission persistence** — SAF grants temporary URI permission. Call `takePersistableUriPermission()` if long-term access is needed (not required for one-time import).
- **Null result handling** — User canceling the picker returns `null` URI. Must check for `null` before processing.
- **Launcher registration timing** — `registerForActivityResult()` must be called before activity reaches `STARTED` state. Register in `onCreate()` or as a property initializer.
- **Instance null check** — Like the camera implementation, always check `instance != null` before calling JNI callbacks.

## Open Risks

- **URI to path conversion complexity** — SAF URIs don't have direct filesystem paths. May need to copy file content to internal storage first. This adds I/O overhead but ensures compatibility with Rust's file path expectations.
- **Large file handling** — PDF files can be large (10MB+). Reading into memory via `ContentResolver` may cause OOM on low-RAM devices. Solution: Stream the content directly to a file.

## Skills Discovered

No new skills needed — standard Android patterns already covered by existing knowledge base.

## Sources

- [Access documents and other files from shared storage](https://developer.android.com/training/data-storage/shared/documents-files) — Official Android documentation on Storage Access Framework
- [Get a result from an activity](https://developer.android.com/training/basics/intents/result) — Official documentation on Activity Result APIs
- [ActivityResultContracts.OpenDocument](https://developer.android.com/reference/androidx/activity/result/contract/ActivityResultContracts.OpenDocument) — Contract for opening documents via SAF

## Forward Intelligence (from S01 Camera Implementation)

**Pattern to follow:**
- Static methods for Rust JNI calls (`@JvmStatic fun pickPdfFile()`)
- External callbacks to Rust (`external fun onFilePicked(filePath: String)`)
- Instance null checks before calling JNI methods
- Consistent logging with `ShuseiFile` tag
- Permission checks (not needed for SAF, but check for general readiness)

**Key difference from camera:**
- No permissions required for SAF
- Uses `ActivityResultLauncher` instead of CameraX use cases
- Returns URI string instead of byte array
