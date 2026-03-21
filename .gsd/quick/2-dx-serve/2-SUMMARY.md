# Quick Task: アプリをモバイルデバイスで動かしてみる場合 dx serve で試せますか？

**Date:** 2026-03-21
**Branch:** gsd/quick/2-dx-serve

## What Changed

### Investigation Results

**Yes, `dx serve --android` can be used for mobile device testing**, with the following workflow:

1. **Build Command**: `dx serve --android --device <DEVICE_ID>`
   - Builds the Rust code for Android (aarch64-linux-android target)
   - Generates a Gradle Android project in `target/dx/shusei/debug/android/app/`
   - Runs Gradle to bundle the APK
   - Outputs APK to `target/dx/shusei/debug/android/app/app/build/outputs/apk/debug/app-debug.apk`

2. **Prerequisites**:
   - Android SDK installed (`ANDROID_HOME` set)
   - Android NDK installed (`ANDROID_NDK_HOME` set)
   - Java JDK installed (JAVA_HOME recommended)
   - ADB available and device connected
   - Required Rust toolchains: `aarch64-linux-android`, `armv7-linux-androideabi`, `x86_64-linux-android`

3. **Device Detection**:
   - Use `adb devices` to list connected devices
   - Pass device ID with `--device <DEVICE_ID>` flag
   - Without `--device`, dx tries to use the active simulator/emulator

4. **Current Limitations**:
   - `dx serve --android` builds successfully but may not auto-deploy in all environments
   - Manual APK installation via `adb install -r <apk-path>` works as fallback
   - Debug APK size is large (~271MB for this project) - install may take time
   - iOS testing requires macOS with Xcode (not available in current Linux environment)

5. **Recommended Workflow**:
   ```bash
   # Check connected devices
   adb devices
   
   # Build and serve for Android
   dx serve --android --device <DEVICE_ID>
   
   # If auto-deploy doesn't work, manually install:
   adb install -r target/dx/shusei/debug/android/app/app/build/outputs/apk/debug/app-debug.apk
   ```

## Files Modified

- `.gsd/quick/2-dx-serve/2-SUMMARY.md` (created)

## Verification

- ✅ `dx doctor` confirms Android SDK, NDK, and ADB are properly configured
- ✅ `dx serve --android` successfully compiles Rust code for Android target
- ✅ Gradle bundling completes successfully
- ✅ APK generated at expected location
- ✅ Device detected via `adb devices` (ZY32LNFZ8W - moto g66j 5G)
- ⚠️ APK installation via ADB tested but slow due to large APK size (271MB)
- ⚠️ `dx serve --android` auto-deploy behavior may require additional configuration

## Additional Notes

### dx serve Interactive Commands
When running `dx serve`, the following keyboard shortcuts are available:
- `Ctrl+C` - Exit the server
- `r` - Rebuild the app
- `p` - Toggle automatic rebuilds
- `v` - Toggle verbose logging
- `/` - Show more commands and shortcuts

### Build Output Structure
```
target/dx/shusei/debug/android/app/
├── app/
│   ├── build/
│   │   └── outputs/apk/debug/app-debug.apk
│   ├── src/main/
│   │   ├── kotlin/dev/dioxus/main/
│   │   ├── jniLibs/arm64-v8a/
│   │   └── assets/
│   └── build.gradle.kts
├── gradle/
└── gradlew
```
