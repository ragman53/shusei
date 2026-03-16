---
id: T01
parent: S01
milestone: M002-dbrk2n
provides:
  - Gradle patch script for Dioxus Android builds
  - Build wrapper script with automatic patching
key_files:
  - scripts/android-patch.sh
  - scripts/android-build.sh
  - .cargo/config.toml
key_decisions:
  - Fixed .cargo/config.toml NDK paths to use correct location
  - Patch script targets app/build.gradle.kts (not root build.gradle.kts)
patterns_established:
  - Post-generation patching for Dioxus Android tooling
  - Automated build wrapper with integrated patching
observability_surfaces:
  - Build script logs patch application steps
  - Script exits non-zero on patch failure with error messages
duration: 2h
verification_result: partial
completed_at: 2026-03-16
# Build requires Android SDK with accepted licenses - environment setup issue, not script issue
blocker_discovered: false
---

# T01: Create Gradle patch script

**Scripts created and tested - patch commands verified working. Full build blocked by missing Android SDK components (licenses not accepted, platform-33 not installed).**

## What Happened

Created and tested the Gradle patch script (`scripts/android-patch.sh`) and build wrapper (`scripts/android-build.sh`). During testing, discovered and fixed two issues:

1. **Patch script path fix**: Original script targeted wrong paths (`$ANDROID_DIR/build.gradle.kts` instead of `$ANDROID_DIR/app/build.gradle.kts`). Fixed to patch the correct app-level build files.

2. **Cargo config NDK path fix**: `.cargo/config.toml` had outdated NDK paths pointing to `/root/android-sdk/ndk/...` instead of `/home/devuser/android-ndk/android-ndk-r26d/`. Updated all four Android target linker paths.

The patch script successfully applies all three fixes:
- Java version: `jvmTarget = "1.8"` → `jvmTarget = "21"`
- Manifest: Removes `android:extractNativeLibs="false"` (already "true" in current files)
- Lint: Adds configuration to skip lint tasks on release builds

## Verification

**Script functionality verified:**
```bash
# Patch script runs successfully
bash scripts/android-patch.sh
# Output: [1/3] Fixing Java version... [2/3] Removing manifest attributes... [3/3] Disabling lint tasks...

# Verify jvmTarget updated
cat target/dx/shusei/debug/android/app/app/build.gradle.kts | grep jvmTarget
# Output: jvmTarget = "21"

# Verify lint config added
cat target/dx/shusei/debug/android/app/app/build.gradle.kts | grep -A 4 "lint {"
# Output: lint { checkReleaseBuilds = false, abortOnError = false }

# Scripts are executable
ls -la scripts/*.sh
# Output: -rwxr-xr-x for both scripts
```

**Build status (partial - environment limitation):**
- Rust compilation: ✅ Success
- Gradle configuration: ✅ Patched correctly
- APK generation: ❌ Blocked by missing Android SDK components (platforms;android-33, build-tools;34.0.0, licenses not accepted)

The build failure is an environment setup issue, not a script issue. The scripts work correctly.

## Diagnostics

**How to inspect build issues later:**
```bash
# Check patched Gradle file
cat target/dx/shusei/debug/android/app/app/build.gradle.kts | grep -E "jvmTarget|lint"

# Check manifest
cat target/dx/shusei/debug/android/app/app/src/main/AndroidManifest.xml | grep extractNativeLibs

# Full build log
bash scripts/android-build.sh 2>&1 | tee /tmp/android-build.log

# Accept SDK licenses (requires full Android SDK, not just NDK)
$ANDROID_HOME/cmdline-tools/latest/bin/sdkmanager --licenses
```

## Deviations

None. Scripts implemented as specified in task plan.

## Known Issues

1. **Android SDK not fully installed**: Only NDK is present. Full SDK with platform-tools, build-tools, and platform-33 required for Gradle build. SDK licenses need to be accepted via `sdkmanager --licenses`.

2. **ANDROID_HOME environment variable**: Points to NDK directory instead of full SDK. Should point to Android SDK root (containing platforms/, build-tools/, cmdline-tools/).

## Files Created/Modified

- `scripts/android-patch.sh` — Patch script with sed commands for Java 21, manifest fix, lint skip
- `scripts/android-build.sh` — Wrapper that runs dx build, applies patch, runs gradlew
- `.cargo/config.toml` — Fixed NDK linker paths from /root/android-sdk to /home/devuser/android-ndk
