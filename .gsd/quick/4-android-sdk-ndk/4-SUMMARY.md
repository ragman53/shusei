# Quick Task 4: Android SDK/NDK Reinstallation - Summary

**Date:** 2026-03-22  
**Branch:** gsd/quick/4-android-sdk-ndk  
**Status:** ✅ Complete

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

1. **`.cargo/config.toml`** - Updated all Android target linker paths to NDK r29
2. **`target/dx/shusei/debug/android/app/build.gradle.kts`**:
   - Updated AGP: 8.7.0 → 8.9.0
   - Updated Kotlin: 2.0.20 → 2.1.0
3. **`target/dx/shusei/debug/android/app/app/build.gradle.kts`**:
   - Updated `compileSdk`: 33 → 36
   - Updated `targetSdk`: 33 → 36
   - Updated AndroidX dependencies to latest versions
4. **`target/dx/shusei/debug/android/app/gradle.properties`**:
   - Increased JVM memory: 2048m → 4096m
   - Enabled parallel builds
5. **`scripts/android-build.sh`**:
   - Updated `ANDROID_HOME` and `ANDROID_NDK_HOME` paths
   - Removed deprecated lint task exclusions
6. **`scripts/android-patch.sh`**:
   - Added SDK version updates (compileSdk/targetSdk → 36)
   - Added Java toolchain configuration (VERSION_17)
7. **Worktree M003** - Synced all configuration updates

### Additional Tools Installed
- `cargo-ndk` v4.1.2 - Rust Android build tooling

## Files Modified

- `.cargo/config.toml`
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

## Environment Setup

The `~/.bashrc` has been updated with correct Android environment variables:
```bash
export ANDROID_HOME="/home/devuser/android-sdk"
export ANDROID_NDK_HOME="$ANDROID_HOME/ndk/29.0.14206865"
export PATH="$PATH:$ANDROID_HOME/platform-tools:$ANDROID_NDK_HOME/toolchains/llvm/prebuilt/linux-x86_64/bin"
```

**Note:** Open a new terminal or run `source ~/.bashrc` for changes to take effect in existing shells.

## Usage

To run the app on Android:
```bash
# In a new terminal (or after sourcing ~/.bashrc)
dx serve --android --target aarch64-linux-android
```

Or build APK:
```bash
bash scripts/android-build.sh
```
