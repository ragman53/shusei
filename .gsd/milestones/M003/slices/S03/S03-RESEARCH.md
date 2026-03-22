# S03: Asset Access Fix — Research

**Date:** 2026-03-22

## Summary

S03 fixes the "Load Demo PDF" button crash on Android devices. The Rust code already has a complete `copy_asset_to_files()` implementation that uses JNI to access APK assets via `AssetManager`. The function copies `test/medium_pdf_test.pdf` from APK assets to the internal files directory, then imports it via `PdfProcessor`.

The issue is that the asset (`assets/test/medium_pdf_test.pdf`) is not being bundled into the APK during the Dioxus build process. The `android-patch.sh` script needs to be extended to copy the `assets/` directory contents to the Android project's `src/main/assets/` directory before building.

Additionally, the Rust `get_assets_directory()` function returns a hardcoded path (`/data/data/com.shusei.app/files`) that doesn't match the actual package name (`dev.dioxus.main`). This needs to be fixed to use the correct package path or derive it dynamically.

## Recommendation

**Approach:** Extend `android-patch.sh` to bundle assets and fix the package path in `get_assets_directory()`.

**Why:** 
1. The Rust asset access code (`copy_asset_to_files`) is already complete and functional
2. The only missing piece is bundling the assets into the APK
3. The `android-patch.sh` pattern is already established for post-generation patching
4. This is the minimal change to make "Load Demo PDF" work

**Implementation:**
1. Add a new step to `android-patch.sh` that copies `assets/` → `target/.../src/main/assets/`
2. Fix `get_assets_directory()` to return the correct package path for `dev.dioxus.main`
3. Verify the asset is accessible via `adb shell` after installation

## Implementation Landscape

### Key Files

- `scripts/android-patch.sh` — Add Step 6/6 to copy assets directory to Android project
- `src/platform/android.rs` — Fix `get_assets_directory()` to use correct package name (`dev.dioxus.main` instead of `com.shusei.app`)
- `assets/test/medium_pdf_test.pdf` — Demo PDF that needs to be bundled (already exists, 856KB)

### Build Order

1. **Fix `get_assets_directory()` first** — This is a simple path fix that unblocks testing
2. **Extend `android-patch.sh`** — Add asset copying step
3. **Rebuild and verify** — Run `dx build --platform android`, then `bash scripts/android-patch.sh`, then install and test

### Verification Approach

```bash
# 1. Rebuild with patched script
dx build --platform android
bash scripts/android-patch.sh

# 2. Install APK
adb install -r target/dx/shusei/debug/android/app/app/build/outputs/apk/debug/app-debug.apk

# 3. Verify asset is bundled
adb shell "ls -la /data/data/dev.dioxus.main/assets/test/"

# 4. Launch app and test "Load Demo PDF" button
adb shell am start -n dev.dioxus.main/.MainActivity

# 5. Monitor logcat for asset access
adb logcat | grep -E "(ShuseiFile|copy_asset|Asset)"
```

**Success signal:** Log shows `Asset copied to: /data/data/dev.dioxus.main/files/medium_pdf_test.pdf` and PDF appears in library.

**Failure signals:**
- `Asset 'test/medium_pdf_test.pdf' not found` → Asset not bundled
- `Activity not initialized` → `nativeInit` not called (check WryActivity.onCreate)
- `Failed to get app directory` → `get_assets_directory()` returns wrong path

## Don't Hand-Roll

| Problem | Existing Solution | Why Use It |
|---------|------------------|------------|
| Asset bundling | Android `src/main/assets/` directory | Standard Android mechanism; `AssetManager.open()` works automatically |
| Path resolution | Use package name from build config | Avoids hardcoding; Dioxus uses `dev.dioxus.main` |

## Constraints

- **Package name mismatch** — Rust code uses `com.shusei.app` but Dioxus generates `dev.dioxus.main`
- **Asset directory structure** — Must preserve `test/medium_pdf_test.pdf` path within APK assets
- **APK size** — Demo PDF is 856KB; acceptable for prototype but should be noted for future optimization

## Common Pitfalls

- **Hardcoded package paths** — The Rust code has `com.shusei.app` hardcoded in multiple places; all need to be updated to `dev.dioxus.main` or made dynamic
- **Asset path casing** — Asset paths are case-sensitive; `test/medium_pdf_test.pdf` must match exactly
- **Timing of asset copy** — `android-patch.sh` must run AFTER `dx build` but BEFORE `gradlew assemble`
- **Null Activity reference** — `copy_asset_to_files()` requires `ACTIVITY` to be initialized via `nativeInit`; verify `WryActivity.onCreate()` calls it

## Open Risks

- **Dioxus asset handling** — Dioxus may have built-in asset bundling that conflicts with manual copying; need to verify no duplication
- **Large asset performance** — Copying 856KB on first load may cause visible delay; should add progress indicator (future enhancement)

## Skills Discovered

No new skills needed — this uses established patterns from S01/S02 (patch script, JNI bridge).

## Sources

- Android AssetManager documentation: https://developer.android.com/reference/android/content/res/AssetManager
- Dioxus Android project structure: `target/dx/shusei/debug/android/app/`
