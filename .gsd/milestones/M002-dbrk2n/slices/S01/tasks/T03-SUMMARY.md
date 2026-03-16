---
id: T03
parent: S01
milestone: M002-dbrk2n
provides:
  - Debug APK built and ready for installation (139MB)
  - Device installation guide with adb commands
  - Fixed Java/Kotlin target version (17) in patch script
key_files:
  - target/dx/shusei/debug/android/app/app/build/outputs/apk/debug/app-debug.apk
  - scripts/android-patch.sh (updated Java version)
  - .gsd/milestones/M002-dbrk2n/slices/S01/tasks/T03-INSTALL-GUIDE.md
key_decisions:
  - Changed Java/Kotlin target from 21 to 17 to match installed JDK
  - Android SDK command-line tools installed separately from NDK
patterns_established:
  - APK build wrapper script with automatic patching
  - Device installation documented for WSL2 environment
observability_surfaces:
  - adb logcat | grep -i shusei for runtime logs
  - adb shell pm list packages | grep com.shusei.app for installation verification
duration: 2 hours
verification_result: partial
completed_at: 2026-03-16
# APK build complete; device installation blocked by WSL2 USB passthrough requirement
blocker_discovered: false
---

# T03: Install APK on Moto G66j 5G

**Debug APK built successfully; device installation requires physical hardware connection**

## What Happened

1. **Android SDK Setup:** Installed Android SDK command-line tools and platform-tools (adb) separately from the NDK. Accepted SDK licenses and installed platform-33 and build-tools-34.0.0.

2. **APK Build:** Ran the build process (`dx build --platform android` → patch script → gradlew assembleDebug). Initial build failed due to:
   - Missing lint task exclusions (fixed in T01)
   - JVM target mismatch between Kotlin (21) and Java (1.8)
   - Java 21 not available (system has Java 17)

3. **Build Configuration Fixed:** Updated `scripts/android-patch.sh` to use Java 17 instead of 21, matching the installed JDK. Also added `compileOptions` block to `build.gradle.kts` for consistent Java/Kotlin targeting.

4. **APK Built Successfully:** Debug APK generated at 139MB containing:
   - Native library: lib/x86_64/libdioxusmain.so (525MB uncompressed)
   - DEX files: classes.dex, classes2.dex
   - Android resources and manifest

5. **Device Connection:** No physical device connected to WSL2 environment. USB passthrough from Windows host required for adb device access.

## Verification

**Verified (Passed):**
- ✅ `dx build --platform android` completes successfully
- ✅ `bash scripts/android-patch.sh` applies fixes correctly
- ✅ `./gradlew assembleDebug` builds APK without errors
- ✅ APK exists at expected location (139MB)
- ✅ APK contains native library and DEX files

**Pending (Requires Device):**
- ⏳ `adb devices` shows connected device
- ⏳ `adb install -r ...` succeeds
- ⏳ `adb shell pm list packages | grep com.shusei.app` returns package
- ⏳ App launches without crash
- ⏳ App icon appears in device app drawer

**Verification Commands (ready to run when device connected):**
```bash
export ANDROID_HOME=/home/devuser/android-sdk
export PATH=$PATH:$ANDROID_HOME/platform-tools
adb devices
adb install -r target/dx/shusei/debug/android/app/app/build/outputs/apk/debug/app-debug.apk
adb shell pm list packages | grep com.shusei.app
adb shell am start -n com.shusei.app/.MainActivity
```

## Diagnostics

**How to inspect installation later:**
```bash
# Check if device is connected
adb devices

# Check installation status
adb shell pm list packages | grep com.shusei.app
adb shell dumpsys package com.shusei.app

# View runtime logs
adb logcat | grep -i shusei

# Check APK contents
unzip -l target/dx/shusei/debug/android/app/app/build/outputs/apk/debug/app-debug.apk | grep -E "lib|assets"

# Full build log
bash scripts/android-build.sh 2>&1 | tee /tmp/android-build.log
```

## Deviations

- **Java version changed from 21 to 17:** The patch script originally targeted Java 21, but the system has Java 17 installed. Updated to match available tooling.
- **Device installation not completed:** Physical device connection requires WSL2 USB passthrough configuration on Windows host, which is outside the scope of this environment.

## Known Issues

- **APK size (139MB):** The native library alone is 525MB uncompressed. This is expected for a Dioxus WebView-based app but may be concerning for distribution. Consider release builds with ProGuard/R8 for size optimization.
- **Moonshine models not bundled:** Only NDLOCR models are present in assets/models/. Moonshine models need to be downloaded from Hugging Face (documented in T02-SUMMARY.md).
- **WSL2 USB passthrough:** Device installation requires Windows host configuration for USB device forwarding to WSL2.

## Files Created/Modified

- `scripts/android-patch.sh` — Updated Java target from 21 to 17
- `target/dx/shusei/debug/android/app/app/build.gradle.kts` — Added compileOptions for Java 17
- `target/dx/shusei/debug/android/app/app/build/outputs/apk/debug/app-debug.apk` — Built debug APK (139MB)
- `.gsd/milestones/M002-dbrk2n/slices/S01/tasks/T03-INSTALL-GUIDE.md` — Device installation guide
- `.gsd/milestones/M002-dbrk2n/slices/S01/tasks/T03-SUMMARY.md` — This summary
