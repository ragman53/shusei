---
task_id: T01
slice_id: S03
milestone_id: M003
status: done
blocker_discovered: false
key_files:
  - src/platform/android.rs
---

# T01: Fix package path in get_assets_directory()

## Summary

Fixed the hardcoded package path in `get_assets_directory()` from `com.shusei.app` to `dev.dioxus.main` to match the Dioxus-generated APK package name.

## What Changed

- **File:** `src/platform/android.rs` (line 284)
- **Change:** Updated `get_assets_directory()` return path from `/data/data/com.shusei.app/files` to `/data/data/dev.dioxus.main/files`

## Verification Evidence

| Check | Command | Exit Code | Verdict |
|-------|---------|-----------|---------|
| Package path updated | `grep -q "dev.dioxus.main/files" src/platform/android.rs` | 0 | ✅ pass |
| Code compiles | `cargo check --target aarch64-linux-android` | 0 | ✅ pass |

## Notes

- Compilation completed with 178 warnings (pre-existing, unrelated to this change)
- No errors introduced by this change
- The path fix enables `copy_asset_to_files()` to write to the correct application data directory on Android

## Next Steps

- Deploy to device and verify PDF asset copies successfully
- Run `scripts/verify-s03-asset.sh` for full slice verification
