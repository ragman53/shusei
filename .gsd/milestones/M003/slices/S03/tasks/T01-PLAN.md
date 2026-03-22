---
estimated_steps: 1
estimated_files: 1
skills_used:
  - debug-like-expert
---

# T01: Fix package path in get_assets_directory()

**Slice:** S03 — Asset Access Fix
**Milestone:** M003

## Description

The `get_assets_directory()` function in `src/platform/android.rs` returns a hardcoded path with the old package name `com.shusei.app`, but Dioxus generates the APK with package name `dev.dioxus.main`. This causes the demo PDF copy to fail because the path doesn't exist on device.

## Steps

1. Open `src/platform/android.rs` and locate the `get_assets_directory()` function (around line 226)
2. Change the return path from `/data/data/com.shusei.app/files` to `/data/data/dev.dioxus.main/files`
3. Verify the change compiles with `cargo check`

## Must-Haves

- [ ] `get_assets_directory()` returns `/data/data/dev.dioxus.main/files`
- [ ] Code compiles without errors

## Verification

- `grep -q "dev.dioxus.main/files" src/platform/android.rs`
- `cargo check --target aarch64-linux-android 2>&1 | head -20` shows no errors

## Inputs

- `src/platform/android.rs` — Current implementation with hardcoded `com.shusei.app` path

## Expected Output

- `src/platform/android.rs` — Updated with correct `dev.dioxus.main` package path

## Observability Impact

**Runtime signals changed:**
- `get_assets_directory()` now returns path containing `dev.dioxus.main` instead of `com.shusei.app`
- Logcat messages from `copy_asset_to_files()` will show the correct target path

**How to inspect this task:**
- `grep "dev.dioxus.main/files" src/platform/android.rs` — confirms the path is updated
- On device: `adb shell ls -la /data/data/dev.dioxus.main/files/` — verify copied PDF exists

**Failure state now visible:**
- If asset copy still fails, error will be due to bundling issues (not wrong package path)
- Logcat tag "ShuseiFile" with "Asset copied to:" will show correct destination path