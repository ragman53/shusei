---
id: T01
parent: S01
milestone: M002-dbrk2n
provides:
  - scripts/android-patch.sh — Gradle patch for Java 21, manifest, lint fixes
  - scripts/android-build.sh — Wrapper build script with automatic patching
  - scripts/README.md — Environment setup documentation
requires:
  - slice: none
    provides: N/A (first task)
affects: [S01, S02, S03, S04, S05]
key_files:
  - scripts/android-patch.sh
  - scripts/android-build.sh
  - scripts/README.md
key_decisions:
  - "Used sed-based post-generation patch (not dx template modification) — upstream fix not available, workaround confirmed in GitHub issue #5251"
patterns_established:
  - "Build scripts in scripts/ directory with bash shebang and usage documentation"
drill_down_paths:
  - .gsd/milestones/M002-dbrk2n/slices/S01/tasks/T01-PLAN.md
duration: 45min
verification_result: partial (scripts created, build blocked by incomplete NDK)
completed_at: 2026-03-15T22:00:00Z
---

# T01: Create Gradle patch script

**Created Gradle patch script and build wrapper for Dioxus Android builds**

## What Happened

Created two shell scripts to automate Android APK builds with the GitHub issue #5251 workaround:

1. **android-patch.sh** — Post-generation patch that fixes:
   - Java version: VERSION_1_8 → VERSION_21
   - Kotlin JVM target: "1.8" → "21"
   - Manifest: Removes deprecated `android:extractNativeLibs="false"`
   - Lint: Disables broken lintVital tasks for AGP 8.8+

2. **android-build.sh** — Wrapper script that:
   - Runs `dx build --platform android`
   - Automatically applies android-patch.sh
   - Runs gradlew with lint tasks skipped
   - Supports `--release` flag for release builds

3. **README.md** — Documents:
   - Prerequisites (Android SDK, NDK, JDK 21, Rust targets, CMake)
   - Environment variable setup (ANDROID_HOME, ANDROID_NDK_HOME, JAVA_HOME)
   - Usage examples
   - Troubleshooting guide

## Verification Status

**Scripts created:** ✅ PASS
- Both scripts exist and are executable
- Syntax verified with bash -n

**Build execution:** 🔴 BLOCKED
- Android NDK at `/opt/android-sdk/ndk/26.1.10909125` is incomplete
- Missing linker tools: `x86_64-linux-android-ar`, `x86_64-linux-android28-clang`
- This is an environment issue, not a script defect

## Deviations

None. Scripts match the plan exactly.

## Files Created/Modified

- `scripts/android-patch.sh` (new, 1.7KB) — Gradle patch script
- `scripts/android-build.sh` (new, 1.8KB) — Build wrapper script
- `scripts/README.md` (new, 2.5KB) — Setup documentation

## Required Environment Fix

Before builds will work, the NDK must be fixed:

**Option A (Recommended):** Use Android Studio SDK Manager
1. Open Android Studio → Tools → SDK Manager
2. Select "SDK Tools" tab
3. Check "NDK (Side by side)" → select version 26.x or 27.x
4. Click Apply to download complete NDK

**Option B:** Download NDK directly
1. Visit https://developer.android.com/ndk/downloads
2. Download NDK r26 or r27 for Linux
3. Extract to `/opt/android-sdk/ndk/` or custom location
4. Update `ANDROID_NDK_HOME` environment variable

After NDK is fixed, run:
```bash
bash scripts/android-build.sh
```

## Next Steps

T02 will verify model files exist (already confirmed - 150MB NDLOCR models present). Once NDK is fixed:
1. Run build script
2. Verify APK structure
3. Install on Moto G66j 5G
4. Test app launch and persistence
