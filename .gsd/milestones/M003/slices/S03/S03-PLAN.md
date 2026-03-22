# S03: Asset Access Fix

**Goal:** 「Load Demo PDF」ボタンでバンドルされたPDFが読み込める
**Demo:** Tap "Load Demo PDF" → demo PDF appears in library without crash

## Must-Haves

- `get_assets_directory()` returns correct package path (`dev.dioxus.main`)
- Demo PDF (`assets/test/medium_pdf_test.pdf`) bundled into APK
- Verification script confirms asset is accessible after installation

## Proof Level

- This slice proves: integration
- Real runtime required: yes (Android device/emulator)
- Human/UAT required: yes (manual button tap)

## Verification

- `bash scripts/verify-s03-asset.sh` — Checks APK contains asset, runs on device, monitors logcat for success signals
- `grep -q "dev.dioxus.main" src/platform/android.rs` — Confirms package path fix

## Observability / Diagnostics

- Runtime signals: Logcat tag "ShuseiFile" with messages "Asset copied to:", "Asset not found", "Activity not initialized"
- Inspection surfaces: `adb shell ls -la /data/data/dev.dioxus.main/files/` to verify copied PDF
- Failure visibility: Error message in logcat distinguishes between "asset not bundled" vs "Activity reference missing" vs "permission denied"

## Integration Closure

- Upstream surfaces consumed: `ACTIVITY` reference from S01's `nativeInit`, Rust `copy_asset_to_files()` JNI implementation
- New wiring introduced in this slice: `android-patch.sh` step 6 copies assets to APK bundle
- What remains before the milestone is truly usable end-to-end: S04 integration verification of all three input methods (camera, PDF picker, demo PDF)

## Tasks

- [x] **T01: Fix package path in get_assets_directory()** `est:15m`
  - Why: The hardcoded path uses old package name `com.shusei.app` but Dioxus generates `dev.dioxus.main`
  - Files: `src/platform/android.rs`
  - Do: Change `get_assets_directory()` to return `/data/data/dev.dioxus.main/files` instead of `/data/data/com.shusei.app/files`
  - Verify: `grep -q "dev.dioxus.main" src/platform/android.rs`
  - Done when: Path returns correct package name

- [x] **T02: Extend android-patch.sh to bundle assets** `est:30m`
  - Why: Demo PDF exists in `assets/` but isn't copied to APK during Dioxus build
  - Files: `scripts/android-patch.sh`, `scripts/verify-s03-asset.sh`
  - Do: Add Step 6 to copy `assets/` directory to `target/.../src/main/assets/`. Create verification script that checks APK contains asset and tests on device.
  - Verify: `bash scripts/verify-s03-asset.sh` passes (asset bundled, button works)
  - Done when: APK contains `test/medium_pdf_test.pdf` in assets, "Load Demo PDF" button works on device

## Files Likely Touched

- `src/platform/android.rs`
- `scripts/android-patch.sh`
- `scripts/verify-s03-asset.sh` (new)