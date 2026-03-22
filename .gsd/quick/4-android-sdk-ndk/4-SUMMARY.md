# Quick Task: Android SDK/NDK Reinstallation and Dependency Updates

**Date:** 2026-03-22
**Branch:** gsd/quick/4-android-sdk-ndk

## What Changed

### SDK/NDK Reinstallation
- Removed old Android SDK (`/home/devuser/android-sdk`)
- Removed old Android NDK (`/home/devuser/android-ndk`)
- Installed latest Android SDK command-line tools (v11076708)
- Installed latest components:
  - **NDK**: r29 (29.0.14206865) - Latest stable release
  - **Build Tools**: 36.1.0 - Latest stable
  - **Platform Tools**: 37.0.0 - Latest
  - **Platform**: android-36 (API 36) - Latest
  - **CMake**: 4.1.1 - Latest

### Configuration Updates

1. **`.cargo/config.toml`** - Updated all Android target linker paths:
   - `aarch64-linux-android`: NDK r29 toolchain
   - `armv7-linux-androideabi`: NDK r29 toolchain
   - `i686-linux-android`: NDK r29 toolchain
   - `x86_64-linux-android`: NDK r29 toolchain

2. **`target/dx/shusei/debug/android/app/build.gradle.kts`**:
   - Updated AGP: 8.7.0 → 8.9.0
   - Updated Kotlin: 2.0.20 → 2.1.0

3. **`target/dx/shusei/debug/android/app/app/build.gradle.kts`**:
   - Updated `compileSdk`: 33 → 36
   - Updated `targetSdk`: 33 → 36
   - Updated dependencies:
     - `androidx.webkit`: 1.6.1 → 1.13.0
     - `androidx.appcompat`: 1.6.1 → 1.7.0
     - `com.google.android.material`: 1.8.0 → 1.12.0

4. **`target/dx/shusei/debug/android/app/gradle.properties`**:
   - Increased JVM memory: 2048m → 4096m
   - Enabled parallel builds
   - Added AGP 8.0+ compatibility flag

5. **`scripts/android-build.sh`**:
   - Updated `ANDROID_HOME`: `/home/devuser/android-sdk`
   - Updated `ANDROID_NDK_HOME`: `/home/devuser/android-sdk/ndk/29.0.14206865`
   - Removed deprecated lint task exclusions

6. **`scripts/android-patch.sh`**:
   - Added SDK version updates (compileSdk/targetSdk → 36)
   - Added Java toolchain configuration (VERSION_17)
   - Fixed lint configuration

7. **Worktree updates** (`.gsd/worktrees/M003/`):
   - `.cargo/config.toml` - NDK paths updated
   - `scripts/android-build.sh` - SDK/NDK paths updated
   - `scripts/android-patch.sh` - Synced with main version

### Additional Tools Installed
- `cargo-ndk` v4.1.2 - Rust Android build tooling

## Files Modified

- `.cargo/config.toml`
- `target/dx/shusei/debug/android/app/build.gradle.kts`
- `target/dx/shusei/debug/android/app/app/build.gradle.kts`
- `target/dx/shusei/debug/android/app/gradle.properties`
- `scripts/android-build.sh`
- `scripts/android-patch.sh`
- `.gsd/worktrees/M003/.cargo/config.toml`
- `.gsd/worktrees/M003/scripts/android-build.sh`
- `.gsd/worktrees/M003/scripts/android-patch.sh`

## Verification

### Build Verification
```bash
bash scripts/android-build.sh
```
**Result:** ✅ BUILD SUCCESSFUL in 12s
- APK generated: `target/dx/shusei/debug/android/app/app/build/outputs/apk/debug/app-debug.apk`
- APK size: 622MB (includes model assets)

### Model Assets Verification
```bash
bash scripts/verify-apk-models.sh
```
**Result:** ✅ All required model files present
- NDLOCR models: 4/4 present
  - deim-s-1024x1024.onnx ✅
  - parseq-ndl-16x256-30-tiny-192epoch-tegaki3.onnx ✅
  - parseq-ndl-16x384-50-tiny-146epoch-tegaki2.onnx ✅
  - parseq-ndl-16x768-100-tiny-165epoch-tegaki2.onnx ✅
- Total model size: 220MB

### SDK Components Verification
```bash
sdkmanager --list_installed
```
**Installed:**
- build-tools;36.1.0
- cmake;4.1.1
- ndk;29.0.14206865
- platform-tools;37.0.0
- platforms;android-36

## Notes

- The old NDK r26d has been completely removed
- All configuration files now reference the new SDK structure at `/home/devuser/android-sdk`
- The NDK is now installed within the SDK at `ndk/29.0.14206865` (side-by-side layout)
- Java/Kotlin compilation now targets Java 17 for compatibility with modern Android libraries
- CameraX dependencies added for camera capture functionality
