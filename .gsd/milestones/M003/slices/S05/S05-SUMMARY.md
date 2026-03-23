---
id: S05
parent: M003
milestone: M003
provides:
  - ARM64 APK build configuration and verification script
  - Physical device verification infrastructure for Moto G66j 5G
  - JNI symbol analysis and root cause documentation for launch failure
requires:
  - slice: S01
    provides: Camera implementation
  - slice: S02
    provides: File picker implementation
  - slice: S03
    provides: Asset bundling
  - slice: S04
    provides: Integration verification infrastructure
affects: []
key_files:
  - scripts/verify-s05-arm64.sh — Unified ARM64 verification script with combined logcat monitoring
  - .gsd/milestones/M003/slices/S05/S05-UAT.md — UAT report with JNI symbol analysis
key_decisions:
  - ARM64 build via CARGO_BUILD_TARGET=aarch64-linux-android
  - JNI symbol analysis using nm -D and readelf -s for root cause diagnosis
patterns_established:
  - JNI symbol inventory pattern: Document expected vs actual symbols for framework debugging
observability_surfaces:
  - Logcat tags: ShuseiCamera, ShuseiFile, Asset (same as S04)
  - Verification log: /tmp/logcat-s05-YYYYMMDD-HHMMSS.log
  - JNI symbol dump: nm -D output for libshusei.so
drill_down_paths:
  - .gsd/milestones/M003/slices/S05/tasks/T01-SUMMARY.md
  - .gsd/milestones/M003/slices/S05/tasks/T02-SUMMARY.md
duration: 3h
verification_result: partial
completed_at: 2026-03-22
---

# S05: ARM64 APK Build and Device Verification

**ARM64 APK build with physical device verification — blocked by Dioxus/Wry JNI symbol mismatch**

## What Happened

S05 delivered ARM64 APK build infrastructure and executed device verification, discovering a framework-level JNI symbol issue. Two tasks composed the slice:

**T01: Verification Script** — Created `scripts/verify-s05-arm64.sh` for unified ARM64 device testing:
- Pre-flight checks: Device connection, architecture display (arm64-v8a), device model
- APK inspection: Confirms arm64-v8a native libraries via `unzip -l`
- Installation and launch: Uninstall → install → launch MainActivity
- Combined logcat monitoring: All three flows to `/tmp/logcat-s05-*.log`
- Three sequential UAT flows with manual prompts (camera, file picker, demo PDF)
- Aggregated report with "M003 VERIFICATION PASSED/FAILED" status
- Color-coded output with exit codes

**T02: Execution and Root Cause Analysis** — Executed verification on Moto G66j 5G:
- **APK Build**: Successfully built with arm64-v8a libraries (libdioxusmain.so 285MB, libimage_processing_util_jni.so 28KB)
- **Installation**: APK installed successfully via `adb install -r -t`
- **Launch Failure**: App crashes immediately with `UnsatisfiedLinkError`
- **Root Cause**: Missing JNI symbols from Dioxus/Wry framework:
  - Missing: `Java_dev_dioxus_main_WryActivity_create`, `start`, `resume`, `pause`
  - Present: `Java_dev_dioxus_main_MainActivity_nativeInit`, `onImageCaptured`, `onFilePicked`, etc.
- **Analysis**: The build copies `libshusei.so` (app's Rust library) to `libdioxusmain.so`, but WryActivity bindings should come from Dioxus framework
- **Documentation**: Created S05-UAT.md with full technical analysis and JNI symbol inventory

## Verification

| Task | Verification Method | Result |
|------|---------------------|--------|
| T01 | Script syntax valid; contains callback checks; M003 status present | ✅ Pass |
| T02 | APK built with arm64 libs; installed on device; crash documented with root cause | ⚠️ Partial |

**Overall Status**: PARTIAL — Build infrastructure complete, but app launch blocked by framework-level JNI issue.

## New Requirements Surfaced

- **Dioxus framework investigation**: Required to determine correct library generation process for Android

## Deviations

- none

## Known Limitations

- **JNI symbol mismatch**: WryActivity bindings missing from native library — framework-level issue
- **Manual UAT required**: Verification script automates log monitoring but requires human interaction for flows
- **Device availability**: Requires Moto G66j 5G with USB debugging enabled

## Follow-ups

- **Hotfix**: Investigate Dioxus 0.7 Android build process, check for `dioxus-mobile` dependency, file framework issue if needed
- **Future**: Once JNI issue resolved, re-run `verify-s05-arm64.sh` for full UAT

## Files Created/Modified

- `scripts/verify-s05-arm64.sh` — Unified ARM64 verification script (created)
- `.gsd/milestones/M003/slices/S05/S05-UAT.md` — UAT report with JNI analysis (created)

## Forward Intelligence

### What the next milestone should know
- The Kotlin/Rust JNI bridge code (MainActivity.kt, android.rs) is correct — the issue is framework-level
- ARM64 build configuration works: CARGO_BUILD_TARGET=aarch64-linux-android produces valid libraries
- The verification infrastructure is ready and waiting for a framework fix

### What's fragile
- **Dioxus framework coupling**: The app depends on Dioxus generating correct JNI bindings
- **Library naming**: The build script copies libshusei.so to libdioxusmain.so — this may need framework intervention

### Authoritative diagnostics
- `nm -D libshusei.so | grep Java` — Shows actual JNI symbols in library
- `adb logcat | grep UnsatisfiedLinkError` — First place to check for launch failures
- `scripts/verify-s05-arm64.sh` — Run this for structured verification
- `/tmp/logcat-s05-*.log` — Post-mortem analysis of test runs
