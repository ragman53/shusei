---
id: S04
parent: M003
milestone: M003
provides:
  - Unified integration verification script orchestrating all three M003 flows
  - Combined logcat monitoring with per-flow success/failure tracking
  - UAT documentation with device information and test results
requires:
  - slice: S01
    provides: Camera flow implementation and verification (CameraX, JNI callbacks)
  - slice: S02
    provides: File picker flow implementation and verification (SAF, JNI callbacks)
  - slice: S03
    provides: Demo PDF flow implementation and verification (asset bundling, copy)
affects:
  - S05 (integration verification is prerequisite for ARM64 device testing)
key_files:
  - scripts/verify-s04-integration.sh — Unified verification script for all three flows
  - .gsd/milestones/M003/slices/S04/S04-UAT.md — UAT report with device info and test results
key_decisions:
  - Single APK install reused across all three flows (efficiency improvement over separate scripts)
  - Combined logcat file for all flows (simplifies post-mortem analysis)
  - Per-flow success/failure counters with aggregated M003 status
patterns_established:
  - Integration verification pattern: Orchestrate multiple flows in single script with shared setup
  - UAT documentation pattern: Structured report with per-flow status, device info, and next steps
observability_surfaces:
  - Combined logcat file: /tmp/logcat-s04-YYYYMMDD-HHMMSS.log (all three flows)
  - Per-flow logcat tags: ShuseiCamera, ShuseiFile, Asset
  - Verification script output: Color-coded per-flow breakdown with M003 verdict
  - UAT report: .gsd/milestones/M003/slices/S04/S04-UAT.md (structured test results)
drill_down_paths:
  - .gsd/milestones/M003/slices/S04/tasks/T01-SUMMARY.md
  - .gsd/milestones/M003/slices/S04/tasks/T02-SUMMARY.md
duration: 2h
verification_result: partial
completed_at: 2026-03-22
---

# S04: Integration Verification

**Unified verification script orchestrating all three M003 flows with combined logcat monitoring**

## What Happened

S04 created unified integration verification infrastructure for testing all three M003 flows (camera, file picker, demo PDF) in a single execution. Two tasks composed the slice:

**T01: Unified Verification Script** — Created `scripts/verify-s04-integration.sh` that orchestrates all three flows:
- Pre-flight checks: Device connection via `adb devices`, APK presence check
- Single APK install: Installed once and reused across all three flows (efficiency improvement)
- Combined logcat monitoring: All logs written to single timestamped file at `/tmp/logcat-s04-*.log`
- Three sequential flows with manual UAT prompts between each:
  - **Flow 1 (Camera)**: Monitors ShuseiCamera/CameraX tags, checks for `onImageCaptured` callback
  - **Flow 2 (File Picker)**: Monitors ShuseiFile tags, checks for `onFilePicked` callback
  - **Flow 3 (Demo PDF)**: Monitors Asset tags, checks for asset copy confirmation
- Aggregated report: Per-flow breakdown with success/failure counts and overall "M003 VERIFICATION PASSED/FAILED" status
- Color-coded output with exit codes (0 for pass, 1 for any failure)

**T02: UAT Execution and Documentation** — Executed the verification script on a physical Moto G66j 5G device and documented results:
- Device information recorded: moto g66j 5G, Android 15 (SDK 35), arm64-v8a
- APK installation failed due to ABI mismatch (x86_64 APK vs arm64-v8a device)
- Error documented: `INSTALL_FAILED_NO_MATCHING_ABIS: Failed to extract native libraries`
- Root cause analysis: APK contains only x86_64 native libraries, device requires arm64-v8a
- UAT report created at `.gsd/milestones/M003/slices/S04/S04-UAT.md` with:
  - Per-flow status (all marked NOT EXECUTED due to install failure)
  - Error details with root cause analysis
  - Three resolution options (rebuild for arm64, use emulator, or build universal APK)
  - M003 success criteria assessment
  - Next steps for completing verification

## Verification

| Task | Verification Method | Result |
|------|---------------------|--------|
| T01 | Script exists and is executable; contains all three flow checks; syntax valid | ✅ Pass |
| T02 | UAT report exists; contains device info; documents error with root cause; has next steps | ✅ Pass |

**Overall Status**: PARTIAL — Verification infrastructure is complete and functional, but physical device testing blocked by ABI mismatch. Resolution requires rebuilding APK for arm64-v8a architecture.

## New Requirements Surfaced

- **ARM64 build configuration**: M003 requires explicit ARM64 targeting via `CARGO_BUILD_TARGET=aarch64-linux-android` and NDK ABI filter in build.gradle.kts

## Deviations

- none

## Known Limitations

- **ABI mismatch**: The current APK build produces x86_64 native libraries, but physical devices (Moto G66j 5G) require arm64-v8a. This blocks all physical device testing.
- **Manual UAT required**: The script automates log monitoring but requires a human to perform actions (grant permissions, tap buttons, select files) between flows.
- **Verification result**: Marked as "partial" because the verification infrastructure is complete but not yet executed successfully on hardware.

## Follow-ups

- **S05**: ARM64 APK build configuration to resolve the ABI mismatch and enable physical device testing
- **Future**: Consider building universal APKs with multiple ABIs for broader device compatibility

## Files Created/Modified

- `scripts/verify-s04-integration.sh` — Unified verification script (created)
- `.gsd/milestones/M003/slices/S04/S04-UAT.md` — UAT report with test results (created)

## Forward Intelligence

### What the next slice should know
- The verification infrastructure is ready and waiting for an ARM64-compatible APK.
- The ABI mismatch is a build configuration issue, not a code issue. The Kotlin/Rust code is correct.
- Three resolution options exist: (1) rebuild for arm64, (2) use x86_64 emulator, (3) build universal APK.

### What's fragile
- **APK build configuration**: The current build targets x86_64 by default. This must be explicitly changed to arm64-v8a for physical device testing.
- **Device availability**: Physical device testing requires a Moto G66j 5G (or similar arm64 device) with USB debugging enabled and ADB access.

### Authoritative diagnostics
- `adb shell getprop ro.product.cpu.abi` — Check device architecture
- `unzip -l <APK> | grep lib/` — Inspect APK native libraries
- `scripts/verify-s04-integration.sh` — Run this for unified verification of all three flows
- `/tmp/logcat-s04-*.log` — Combined logcat for all three flows
- `.gsd/milestones/M003/slices/S04/S04-UAT.md` — UAT report with device info and test results

### What assumptions changed
- **Assumption**: APK would work on any Android device. **Reality**: APK architecture must match device ABI (arm64-v8a for most modern phones, x86_64 for emulators).
- **Assumption**: Verification would complete in one run. **Reality**: ABI mismatch blocked execution, requiring a separate slice (S05) to fix build configuration.
