---
phase: quick
plan: 004
subsystem: mobile-android
tags: [android, pdf, assets, demo]
dependency_graph:
  requires: []
  provides: [bundled-demo-pdf]
  affects: [library-screen]
tech_stack:
  added: [jni-asset-copy]
  patterns: [cfg-target-android, asset-bundling]
key_files:
  created:
    - assets/test/medium_pdf_test.pdf
  modified:
    - Dioxus.toml
    - src/platform/android.rs
    - src/ui/library.rs
    - platform/android/app/src/main/java/com/shusei/app/MainActivity.java
decisions:
  - Use JNI to copy bundled assets from APK to files directory
  - Orange button color to distinguish demo from regular import
metrics:
  duration_minutes: 15
  completed_date: "2026-03-13"
---

# Quick Task 004: Add PDF Test Assets Summary

## One-liner

Added bundled test PDF and Android-specific "Load Demo PDF" button using JNI asset extraction for mobile verification.

## What Was Done

### Task 1: Bundle test PDF in assets
- Created `assets/test/` directory
- Copied `tests/medium_pdf_test.pdf` to `assets/test/medium_pdf_test.pdf`
- Updated `Dioxus.toml` to include `assets/test/*` in resources for APK bundling

### Task 2: Add Load Demo PDF button for Android
- Added `copy_asset_to_files()` JNI function in `src/platform/android.rs`
- Added `copyAssetToFiles()` static method in `MainActivity.java`
- Added `load_demo_pdf` handler in `src/ui/library.rs` (Android-only)
- Added orange "Load Demo PDF" button in Library screen (Android-only)

### Task 3: Checkpoint (Auto-approved)
- Human verification auto-approved due to `auto_advance` setting

## Technical Implementation

### Asset Bundling
```toml
# Dioxus.toml
resources = ["assets/models/*", "assets/test/*"]
```

### JNI Asset Extraction
The Android implementation uses a two-step process:
1. **Rust calls Java**: `copy_asset_to_files()` invokes `MainActivity.copyAssetToFiles()` via JNI
2. **Java copies asset**: Uses Android's `AssetManager` to read from APK and write to files directory

### Button Styling
- Orange (`bg-orange-500`) to distinguish from regular "Import PDF" (green)
- Disabled state shows "Loading..." text
- Only visible on Android via `#[cfg(target_os = "android")]`

## Files Changed

| File | Change |
|------|--------|
| `assets/test/medium_pdf_test.pdf` | Created - bundled test PDF |
| `Dioxus.toml` | Added `assets/test/*` to resources |
| `src/platform/android.rs` | Added `copy_asset_to_files()` function |
| `src/ui/library.rs` | Added demo PDF handler and button |
| `MainActivity.java` | Added `copyAssetToFiles()` JNI method |

## Deviations from Plan

None - plan executed exactly as written.

## How to Verify

1. Run `dx serve --android`
2. Wait for app to launch on device/emulator
3. On Library screen, verify orange "Load Demo PDF" button appears
4. Tap button to import the bundled PDF
5. Verify metadata dialog appears and PDF imports to library

## Commits

- `bd696e1`: feat(quick-004): bundle test PDF for Android demo
- `77eabd9`: feat(quick-004): add Load Demo PDF button for Android

## Self-Check: PASSED

- [x] Test PDF exists in `assets/test/medium_pdf_test.pdf`
- [x] Dioxus.toml includes `assets/test/*` in resources
- [x] `copy_asset_to_files()` function exists in android.rs
- [x] `copyAssetToFiles()` method exists in MainActivity.java
- [x] "Load Demo PDF" button in library.rs (Android-only)
- [x] All commits verified in git log