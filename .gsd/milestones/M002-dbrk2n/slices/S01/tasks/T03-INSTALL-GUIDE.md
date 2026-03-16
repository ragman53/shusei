# T03: Install APK on Moto G66j 5G

**Status:** APK built successfully; device connection requires physical hardware

## APK Build Complete

The debug APK has been successfully built and is ready for installation:

```
Location: target/dx/shusei/debug/android/app/app/build/outputs/apk/debug/app-debug.apk
Size: 139MB
Package: com.shusei.app
```

## Device Connection Requirements

### Physical Setup (Required)
Since this is a WSL2 environment, USB device passthrough must be configured:

1. **On Windows Host:**
   - Install USB/IP support or use WSL2 USB passthrough tools
   - Connect Moto G66j 5G via USB cable
   - Forward the USB device to WSL2

2. **On Moto G66j 5G:**
   - Enable Developer Options: Settings → About phone → Tap "Build number" 7 times
   - Enable USB Debugging: Settings → Developer options → USB debugging → ON
   - When connected, authorize the computer on the device prompt

3. **Verify Connection:**
   ```bash
   export ANDROID_HOME=/home/devuser/android-sdk
   export PATH=$PATH:$ANDROID_HOME/platform-tools
   adb devices
   # Should show: <serial_number>    device
   ```

### Installation Commands (Ready to Run)

Once device is connected:

```bash
# Install APK
adb install -r target/dx/shusei/debug/android/app/app/build/outputs/apk/debug/app-debug.apk

# Verify installation
adb shell pm list packages | grep com.shusei.app

# Launch app
adb shell am start -n com.shusei.app/.MainActivity

# Check logs
adb logcat | grep -i shusei
```

## Error Handling

If installation fails:

```bash
# INSTALL_FAILED_UPDATE_INCOMPATIBLE - Uninstall first
adb uninstall com.shusei.app
adb install -r target/dx/shusei/debug/android/app/app/build/outputs/apk/debug/app-debug.apk

# device unauthorized - Check device for authorization prompt
# INSTALL_FAILED_INSUFFICIENT_STORAGE - Free device storage
adb shell df /data
```

## APK Contents Verification

```bash
# Check APK structure
unzip -l target/dx/shusei/debug/android/app/app/build/outputs/apk/debug/app-debug.apk | grep -E "lib|MainActivity"

# Verify native library (525MB uncompressed)
unzip -l target/dx/shusei/debug/android/app/app/build/outputs/apk/debug/app-debug.apk | grep libdioxusmain
```

## Build Configuration Fixed

- Java/Kotlin target: 17 (matching installed JDK)
- Lint tasks: Disabled for release builds (AGP 8.8+ compatibility)
- NDK paths: Configured in .cargo/config.toml

## Next Steps

1. Connect Moto G66j 5G with USB debugging enabled
2. Run `adb devices` to verify connection
3. Run `adb install -r ...` command above
4. Verify app launches and functions correctly
