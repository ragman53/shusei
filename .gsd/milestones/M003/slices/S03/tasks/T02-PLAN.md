---
estimated_steps: 4
estimated_files: 3
skills_used:
  - debug-like-expert
---

# T02: Extend android-patch.sh to bundle assets

**Slice:** S03 — Asset Access Fix
**Milestone:** M003

## Description

The demo PDF (`assets/test/medium_pdf_test.pdf`) exists in the project but is not being copied to the APK during the Dioxus build. Android requires assets to be placed in `src/main/assets/` directory to be accessible via `AssetManager`. This task extends the patch script to copy assets and creates a verification script for end-to-end testing.

## Steps

1. Add Step 6 to `scripts/android-patch.sh` that copies `assets/` directory contents to `target/dx/shusei/debug/android/app/app/src/main/assets/`
   - Use `cp -r` to preserve directory structure
   - Make the step idempotent (create target dir if needed, skip if already exists)
   - Log what's being copied

2. Create `scripts/verify-s03-asset.sh` following the S01 verification script pattern:
   - Check device connectivity (`adb devices`)
   - Check APK exists
   - Build and install APK
   - Use `aapt` or `unzip -l` to verify asset is bundled in APK
   - Launch app and monitor logcat for "Asset copied to:" success signal
   - Provide manual UAT steps for pressing "Load Demo PDF" button
   - Output color-coded pass/fail results

3. Test the full flow: `dx build --platform android && bash scripts/android-patch.sh && bash scripts/verify-s03-asset.sh`

## Must-Haves

- [ ] `android-patch.sh` step 6 copies assets directory to APK
- [ ] Verification script exists and is executable
- [ ] APK contains `test/medium_pdf_test.pdf` in assets
- [ ] "Load Demo PDF" works on device (asset copied, PDF imported)

## Verification

- `grep -q "Copy assets" scripts/android-patch.sh` — Step 6 exists
- `test -x scripts/verify-s03-asset.sh` — Verification script is executable
- After build: `unzip -l target/dx/shusei/debug/android/app/app/build/outputs/apk/debug/app-debug.apk | grep -q "test/medium_pdf_test.pdf"` — Asset bundled
- On device: `adb shell ls /data/data/dev.dioxus.main/files/medium_pdf_test.pdf` after loading demo

## Inputs

- `scripts/android-patch.sh` — Existing patch script with 5 steps
- `assets/test/medium_pdf_test.pdf` — Demo PDF to bundle
- `scripts/verify-s01-camera.sh` — Pattern for verification script

## Expected Output

- `scripts/android-patch.sh` — Extended with Step 6 for asset copying
- `scripts/verify-s03-asset.sh` — New verification script for asset access testing

## Observability Impact

**Signals changed:**
- Logcat tag "ShuseiFile" now emits "Asset copied to:" when demo PDF is successfully copied to app files directory
- Logcat emits "Asset not found" if bundled asset cannot be located in APK

**How to inspect:**
- `adb logcat | grep ShuseiFile` — Watch for asset copy confirmation during runtime
- `adb shell ls -la /data/data/dev.dioxus.main/files/medium_pdf_test.pdf` — Verify file exists after loading demo
- `unzip -l <APK_PATH> | grep test/medium_pdf_test.pdf` — Confirm asset is bundled in APK before installation

**Failure states made visible:**
- "Asset not found in bundle" → Asset was not copied to `src/main/assets/` during build
- "Activity not initialized" → JNI Activity reference is null (S01 issue)
- "Permission denied" → Storage permission not granted (should not occur for app-private files)