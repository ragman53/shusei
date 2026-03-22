---
id: S03
parent: M003
milestone: M003
provides:
  - Fixed package path in get_assets_directory() for correct app data directory
  - Asset bundling via android-patch.sh Step 6 (copies assets/ to APK)
  - Verification script for asset access testing on physical devices
requires:
  - slice: S01
    provides: Activity reference initialized in MainActivity.kt (required for asset copy operations)
affects:
  - S04 (demo PDF flow integrated into unified verification)
key_files:
  - src/platform/android.rs — Fixed package path from com.shusei.app to dev.dioxus.main
  - scripts/android-patch.sh — Extended with Step 6 for asset bundling
  - scripts/verify-s03-asset.sh — Automated verification with APK inspection and logcat monitoring
key_decisions:
  - Package path must match Dioxus-generated APK (dev.dioxus.main, not com.shusei.app)
  - cp -rn for idempotent asset copying (preserves directory structure, skips existing files)
  - Asset verification via unzip -l before device install (catches bundling issues early)
patterns_established:
  - Asset bundling pattern: Copy assets/ directory to Gradle project during patch phase
  - Verification pattern: Check APK contents with unzip -l before installing on device
  - Logcat tagging: Use "Asset" tag for asset copy operations
observability_surfaces:
  - Logcat tag: Asset (asset copy operations)
  - Logcat tag: ShuseiFile (file operations for copied assets)
  - APK inspection: unzip -l <APK> | grep assets/ confirms bundling before install
  - Verification log file: /tmp/logcat-s03-YYYYMMDD-HHMMSS.log
  - Verification script: scripts/verify-s03-asset.sh (color-coded pass/fail output)
drill_down_paths:
  - .gsd/milestones/M003/slices/S03/tasks/T01-SUMMARY.md
  - .gsd/milestones/M003/slices/S03/tasks/T02-SUMMARY.md
duration: 1.5h
verification_result: passed
completed_at: 2026-03-22
---

# S03: Asset Access Fix

**Demo PDF bundling and asset access with correct package path**

## What Happened

S03 fixed asset access for bundling demo PDFs and other assets in the APK. Two tasks composed the slice:

**T01: Package Path Fix** — Fixed the hardcoded package path in `get_assets_directory()` from `com.shusei.app/files` to `dev.dioxus.main/files` to match the Dioxus-generated APK package name. This single-line change enables `copy_asset_to_files()` to write to the correct application data directory on Android.

**T02: Asset Bundling and Verification** — Extended the build and verification infrastructure:
- Added Step 6 to `scripts/android-patch.sh` that copies the `assets/` directory to the Gradle project using `cp -rn` for idempotent copying
- Created `scripts/verify-s03-asset.sh` for end-to-end verification:
  - APK inspection via `unzip -l` to confirm assets are bundled before install
  - Device connectivity and APK presence checks
  - APK installation and app launch
  - Logcat monitoring for "Asset copied to:" success signal
  - Manual UAT prompts for tapping "Load Demo PDF" button
  - File existence check in app files directory after loading
  - Color-coded output with timestamped log persistence

The asset bundling pattern ensures that demo PDFs and other assets are included in every APK build without manual intervention.

## Verification

| Task | Verification Method | Result |
|------|---------------------|--------|
| T01 | grep "dev.dioxus.main/files" in android.rs; cargo check passes | ✅ Pass |
| T02 | Step 6 exists in patch script; verify script is executable; syntax valid; assets copied successfully | ✅ Pass |

All verification gates passed. The implementation requires a full APK rebuild (`dx build --platform android && bash scripts/android-patch.sh`) to bundle assets into a new APK.

## New Requirements Surfaced

- none

## Deviations

- none

## Known Limitations

- **APK rebuild required**: The APK in the target directory was built before Step 6 was added. A full rebuild is required to bundle the assets into a new APK.
- **Device verification pending**: Full verification requires a connected Android device. The script is ready but not yet executed on hardware.
- **Asset size impact**: Bundling assets increases APK size. Large assets (e.g., demo PDFs) should be kept minimal or compressed.

## Follow-ups

- **S04**: Demo PDF flow integrated into unified verification script (verify-s04-integration.sh)
- **Future**: Consider asset compression or on-demand download for large assets if APK size becomes a concern

## Files Created/Modified

- `src/platform/android.rs` — Fixed package path (modified)
- `scripts/android-patch.sh` — Extended with Step 6 for asset bundling (modified)
- `scripts/verify-s03-asset.sh` — Automated verification script (created)

## Forward Intelligence

### What the next slice should know
- The package path MUST match the Dioxus-generated APK package name (dev.dioxus.main). If this changes in future Dioxus versions, the path must be updated.
- Asset bundling happens during the patch phase, not the build phase. The assets/ directory must exist before running android-patch.sh.
- The verification script checks APK contents BEFORE installing on device, which catches bundling issues early.

### What's fragile
- **Package path coupling**: The hardcoded path in android.rs is coupled to the Dioxus template. If Dioxus changes the package naming convention, this will break.
- **Asset directory location**: The assets/ directory is expected at the project root. If the project structure changes, the patch script path must be updated.
- **Idempotent copying**: The `cp -rn` command skips existing files, which is good for idempotency but means asset updates require cleaning the target directory first.

### Authoritative diagnostics
- `unzip -l <APK> | grep assets/` — Confirms assets are bundled in APK
- `adb shell ls -la /data/data/dev.dioxus.main/files/` — Shows copied asset files on device
- `adb logcat | grep Asset` — Asset copy operations
- `scripts/verify-s03-asset.sh` — Run this for structured verification with saved logs
- `/tmp/logcat-s03-*.log` — Post-mortem analysis of failed test runs

### What assumptions changed
- **Assumption**: Assets would be bundled automatically. **Reality**: Dioxus doesn't bundle assets by default; manual patching is required.
- **Assumption**: Package path would be configurable. **Reality**: The package path is hardcoded in the Rust code and must match the Dioxus-generated APK.
