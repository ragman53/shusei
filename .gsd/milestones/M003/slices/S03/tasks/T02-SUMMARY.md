---
task_id: T02
slice_id: S03
milestone_id: M003
status: done
blocker_discovered: false
key_files:
  - scripts/android-patch.sh
  - scripts/verify-s03-asset.sh
---

# T02 Summary: Extend android-patch.sh to bundle assets

## One-liner
Added Step 6 to android-patch.sh that copies assets/ directory to APK bundle and created verify-s03-asset.sh for end-to-end asset access verification.

## Implementation Narrative

**Step 1: Added Observability Impact section to T02-PLAN.md**
Added the missing Observability Impact section as required by pre-flight checks. Documents the logcat signals ("Asset copied to:", "Asset not found"), inspection commands, and failure states that are now visible.

**Step 2: Extended android-patch.sh with Step 6**
Added a new fix step that:
- Copies the `assets/` directory to `target/dx/shusei/debug/android/app/app/src/main/assets/`
- Uses `cp -rn` for idempotent copying (preserves directory structure, skips existing files)
- Logs the number of files copied and verifies the demo PDF is present
- Creates the target directory if it doesn't exist

Tested the patch script successfully:
```
[6/6] Copying assets directory...
  Copied assets to: /home/devuser/develop/shusei/.gsd/worktrees/M003/target/dx/shusei/debug/android/app/app/src/main/assets
  Total assets in bundle: 7 files
  ✓ Demo PDF bundled: test/medium_pdf_test.pdf
```

**Step 3: Created verify-s03-asset.sh**
Created a comprehensive verification script following the S01 pattern that:
- Checks device connectivity via `adb devices`
- Verifies APK exists at expected path
- Uses `unzip -l` to confirm `assets/test/medium_pdf_test.pdf` is bundled in APK
- Installs APK on device and launches the app
- Monitors logcat for "ShuseiFile" tag and "Asset copied to:" success signal
- Provides manual UAT steps for pressing "Load Demo PDF" button
- Checks if file exists in app files directory after loading
- Outputs color-coded pass/fail results with summary

**Step 4: Verification**
- Both scripts pass bash syntax check (`bash -n`)
- verify-s03-asset.sh is executable (`chmod +x`)
- android-patch.sh Step 6 confirmed via grep
- Patch script runs successfully and copies 7 asset files

**Note on APK rebuild:** The APK in the target directory was built before Step 6 was added. A full rebuild (`dx build --platform android && bash scripts/android-patch.sh`) is required to bundle the assets into a new APK. The verification script will confirm the asset is bundled after a fresh build.

## Verification Evidence

| Check | Command | Exit Code | Verdict |
|-------|---------|-----------|---------|
| Step 6 exists | `grep -q "Copy assets" scripts/android-patch.sh` | 0 | ✅ pass |
| Script executable | `test -x scripts/verify-s03-asset.sh` | 0 | ✅ pass |
| Patch script syntax | `bash -n scripts/android-patch.sh` | 0 | ✅ pass |
| Verify script syntax | `bash -n scripts/verify-s03-asset.sh` | 0 | ✅ pass |
| Assets copied | `bash scripts/android-patch.sh` (Step 6 output) | 0 | ✅ pass |
| APK rebuild | Requires Android SDK (not in worktree) | N/A | ⏳ pending |
| Device verification | Requires connected Android device | N/A | ⏳ pending |

## Observability Impact

As documented in T02-PLAN.md:
- **Runtime signals:** Logcat tag "ShuseiFile" emits "Asset copied to:" on successful copy, "Asset not found" if bundle missing
- **Inspection:** `adb shell ls -la /data/data/dev.dioxus.main/files/` shows copied PDF; `unzip -l <APK> | grep test/medium_pdf_test.pdf` confirms bundling
- **Failure visibility:** Distinguishes "asset not bundled" vs "Activity not initialized" vs "permission denied"

## What Remains

- Run `dx build --platform android && bash scripts/android-patch.sh` to rebuild APK with bundled assets
- Run `bash scripts/verify-s03-asset.sh` on a system with Android SDK and connected device
- Manual UAT: Tap "Load Demo PDF" button in app and verify PDF loads without crash

## Decisions

None. This task followed the established pattern from S01 verification script.

## Knowledge Added

None beyond what's documented in the scripts.
