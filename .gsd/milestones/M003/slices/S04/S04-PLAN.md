# S04: Integration Verification

**Goal:** Verify all M003 features (camera capture, PDF import, demo PDF) work together end-to-end on a physical Android device without crashes.
**Demo:** A single unified verification script runs all three flows sequentially and produces a combined pass/fail report confirming M003 success criteria are met.

## Must-Haves

- Unified integration script (`verify-s04-integration.sh`) that orchestrates camera, file picker, and asset tests
- Single APK install reused across all tests (efficiency improvement)
- Aggregated success/failure report showing overall M003 status
- Log persistence for post-mortem analysis
- Clear pass/fail gates matching M003 success criteria

## Proof Level

- This slice proves: final-assembly
- Real runtime required: yes (physical Android device)
- Human/UAT required: yes (button taps, file selection, camera capture)

## Verification

- `bash scripts/verify-s04-integration.sh` exits with code 0 when all flows pass
- Script produces timestamped log file at `/tmp/logcat-s04-*.log`
- Report shows "M003 VERIFICATION PASSED" when all success criteria are met

## Observability / Diagnostics

- Runtime signals: Combined logcat from all three flows (ShuseiCamera, ShuseiFile, Asset tags)
- Inspection surfaces: `/tmp/logcat-s04-*.log` with all captured logs, per-flow success counts
- Failure visibility: Per-flow breakdown shows which component failed (camera, file picker, or asset)
- Redaction constraints: None (logs contain no secrets)

## Integration Closure

- Upstream surfaces consumed: MainActivity.kt (S01/S02), copyAssetToFiles (S03), JNI callbacks from android.rs
- New wiring introduced in this slice: Unified orchestration script composing existing verification scripts
- What remains before the milestone is truly usable end-to-end: nothing — this is the final verification slice

## Tasks

- [x] **T01: Create Unified Integration Verification Script** `est:1h`
  - Why: Need a single command to verify all M003 features work together, avoiding repeated APK installs and providing one consolidated report
  - Files: `scripts/verify-s04-integration.sh`
  - Do: Create script that installs APK once, runs camera/file-picker/asset verification flows with manual UAT prompts, aggregates results into single report with per-flow breakdown. Reuse logcat patterns from existing scripts. Support incremental testing (skip flows that passed).
  - Verify: `test -x scripts/verify-s04-integration.sh && grep -q "M003 VERIFICATION" scripts/verify-s04-integration.sh`
  - Done when: Script is executable, contains all three flow checks, produces consolidated report

- [x] **T02: Run Integration Verification and Document Results** `est:30m`
  - Why: M003 requires proof that all features work on device; this task captures that proof
  - Files: `scripts/verify-s04-integration.sh`, `.gsd/milestones/M003/slices/S04/S04-UAT.md`
  - Do: Execute unified verification script on physical device, follow manual UAT steps, capture results in S04-UAT.md with timestamp and device info
  - Verify: `grep -q "VERIFICATION PASSED\|VERIFICATION FAILED" .gsd/milestones/M003/slices/S04/S04-UAT.md`
  - Done when: S04-UAT.md documents verification results with pass/fail status for each flow

## Files Likely Touched

- `scripts/verify-s04-integration.sh` — New unified verification script
- `.gsd/milestones/M003/slices/S04/S04-UAT.md` — Verification results documentation