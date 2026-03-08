# Android Development Setup Guide for Shusei

This guide provides step-by-step instructions for setting up Android Studio, Android SDK, and Android NDK for the Shusei project on Windows 11.

## Table of Contents

1. [Prerequisites](#prerequisites)
2. [Android Studio Installation](#android-studio-installation)
3. [Android SDK Setup](#android-sdk-setup)
4. [Android NDK Installation](#android-ndk-installation)
5. [Environment Variables Configuration](#environment-variables-configuration)
6. [Verification Steps](#verification-steps)
7. [Rust Android Target Setup](#rust-android-target-setup)
8. [Troubleshooting](#troubleshooting)

---

## Prerequisites

Before beginning the setup, ensure you have:

- **Operating System**: Windows 11 (64-bit)
- **Disk Space**: At least 10 GB free space
- **RAM**: Minimum 8 GB (16 GB recommended)
- **Git**: Installed and added to PATH
- **Rust**: Installed via [rustup](https://rustup.rs/)

---

## Android Studio Installation

### Download Android Studio

1. Visit the official Android Studio download page:
   - **URL**: https://developer.android.com/studio

2. Download the latest stable version for Windows:
   - **Recommended**: Android Studio Ladybug (2024.2) or Meerkat (2024.3)
   - **File**: `android-studio-2024.x.x.xx-windows.exe`

### Installation Steps

1. **Run the installer** (`android-studio-*.exe`)

2. **Setup Wizard**:
   - Click "Next" on the welcome screen
   - Choose installation location (default: `C:\Program Files\Android\Android Studio`)
   - Click "Next"

3. **Start Menu Folder**:
   - Leave default or customize
   - Click "Next"

4. **Component Selection**:
   - Ensure "Android Studio" and "Android Virtual Device" are checked
   - Click "Next"

5. **License Agreement**:
   - Accept the terms
   - Click "Next"

6. **Installation**:
   - Click "Install"
   - Wait for installation to complete

7. **First Launch**:
   - Click "Finish" (ensure "Start Android Studio" is checked)
   - On first launch, choose "Do not import settings"
   - Click "OK"

8. **Setup Wizard (First Run)**:
   - Choose "Standard" installation type
   - Select your UI theme (Dark/Light)
   - Click "Next"
   - Accept Android SDK license agreements
   - Click "Finish"

---

## Android SDK Setup

### Using SDK Manager

1. **Open SDK Manager**:
   - In Android Studio, go to `Tools` → `SDK Manager`
   - Or click the SDK Manager icon in the toolbar

2. **SDK Platforms Tab**:
   - Check **Android 14 (API 34)** or **Android 15 (API 35)**
   - For Shusei project, **API Level 34** is recommended for broader compatibility

3. **SDK Tools Tab**:
   - Check the following:
     - ☑ Android SDK Build-Tools 34.0.0
     - ☑ Android SDK Command-line Tools (latest)
     - ☑ Android SDK Platform-Tools
     - ☑ NDK (Side by side) - Version 25.2.9519653 (r25c) or 26.1.10909125 (r26)
     - ☑ Android SDK Tools (Obsolete - if needed for legacy support)
     - ☑ Google USB Driver (for Windows)

4. **SDK Update Sites Tab**:
   - Ensure "https://dl.google.com/android/repository/addons_list-3.xml" is checked

5. Click **Apply** and wait for downloads to complete

### Recommended SDK Versions for Shusei

| Component | Recommended Version | Notes |
|-----------|---------------------|-------|
| Android Studio | Ladybug (2024.2) | Latest stable |
| Compile SDK | API 34 (Android 14) | Compatible with most devices |
| Target SDK | API 34 | Required for Play Store |
| Min SDK | API 24 (Android 7.0) | Moto G66j 5G support |
| Build Tools | 34.0.0 | Stable release |
| NDK | r25c or r26 | Compatible with tract-onnx |
| CMake | 3.22.1 or later | For native builds |

---

## Android NDK Installation

### Method 1: Via SDK Manager (Recommended)

1. Open **SDK Manager** in Android Studio
2. Go to **SDK Tools** tab
3. Check **NDK (Side by side)**
4. Click **Show Package Details**
5. Select version:
   - **r25c** (25.2.9519653) - Recommended for tract-onnx compatibility
   - **r26** (26.1.10909125) - Latest stable
6. Click **Apply**

### Method 2: Manual Installation

1. Download NDK from: https://developer.android.com/ndk/downloads
2. Extract to: `C:\Android\ndk\25.2.9519653` (or your chosen version)
3. Set `ANDROID_NDK_HOME` environment variable (see below)

### NDK Version Compatibility

| NDK Version | tract-onnx Compatibility | Rust Support |
|-------------|-------------------------|--------------|
| r25c | ✅ Full support | ✅ aarch64-linux-android |
| r26 | ✅ Full support | ✅ aarch64-linux-android |
| r24 | ⚠️ Limited | ✅ Compatible |

---

## Environment Variables Configuration

### Setting Up Environment Variables on Windows 11

1. **Open Environment Variables**:
   - Press `Win + S` and search for "Environment Variables"
   - Select "Edit the system environment variables"
   - Click "Environment Variables..." button

2. **Add System Variables**:

   Click "New..." under **System variables** and add:

   | Variable Name | Variable Value |
   |---------------|----------------|
   | `ANDROID_HOME` | `C:\Users\%USERNAME%\AppData\Local\Android\Sdk` |
   | `ANDROID_SDK_ROOT` | `C:\Users\%USERNAME%\AppData\Local\Android\Sdk` |
   | `ANDROID_NDK_HOME` | `C:\Users\%USERNAME%\AppData\Local\Android\Sdk\ndk\25.2.9519653` |
   | `NDK_HOME` | `C:\Users\%USERNAME%\AppData\Local\Android\Sdk\ndk\25.2.9519653` |

   > **Note**: Replace `%USERNAME%` with your actual Windows username, or use the full path.

3. **Update PATH Variable**:

   - Select `Path` under **System variables**
   - Click "Edit..."
   - Click "New" and add the following entries:

   ```
   %ANDROID_HOME%\platform-tools
   %ANDROID_HOME%\cmdline-tools\latest\bin
   %ANDROID_HOME%\build-tools\34.0.0
   %ANDROID_NDK_HOME%
   %ANDROID_NDK_HOME%\toolchains\llvm\prebuilt\windows-x86_64\bin
   ```

   - Click "OK" to save

4. **Apply Changes**:
   - Click "OK" on all open dialogs
   - **Restart your terminal** or VS Code for changes to take effect

### Verify Environment Variables

Open a new Command Prompt and run:

```cmd
echo %ANDROID_HOME%
echo %ANDROID_SDK_ROOT%
echo %ANDROID_NDK_HOME%
echo %NDK_HOME%
```

Each command should output the corresponding path.

---

## Verification Steps

### 1. Verify SDK Manager

Open Command Prompt and run:

```cmd
sdkmanager --list
```

**Expected Output**: A list of available SDK packages with installed packages marked with `| Installed |`.

If you see "sdkmanager is not recognized", ensure:
- Command-line tools are installed via SDK Manager
- `%ANDROID_HOME%\cmdline-tools\latest\bin` is in PATH

### 2. Verify ADB (Android Debug Bridge)

```cmd
adb --version
```

**Expected Output**:
```
Android Debug Bridge version 1.0.41
Version 34.0.5-xxxxxxx
Installed as C:\Users\%USERNAME%\AppData\Local\Android\Sdk\platform-tools\adb.exe
```

### 3. Verify NDK Installation

```cmd
echo %ANDROID_NDK_HOME%
dir %ANDROID_NDK_HOME%\toolchains\llvm\prebuilt\windows-x86_64\bin
```

**Expected Output**: List of toolchain binaries including `clang.exe`, `llvm-ar.exe`, etc.

### 4. Verify Rust Android Targets

```cmd
rustup target list
```

**Expected Output**: Look for these targets (marked as installed if already added):

```
aarch64-linux-android
armv7-linux-androideabi
i686-linux-android
x86_64-linux-android
```

### 5. Test Device Connection (USB Debugging)

1. **Enable Developer Options on Moto G66j 5G**:
   - Go to `Settings` → `About phone`
   - Tap "Build number" 7 times
   - You'll see "You are now a developer!"

2. **Enable USB Debugging**:
   - Go to `Settings` → `System` → `Developer options`
   - Enable "USB debugging"

3. **Connect Device**:
   - Connect Moto G66j 5G via USB
   - On phone, tap "Allow" when prompted for USB debugging authorization

4. **Verify Connection**:
   ```cmd
   adb devices
   ```

   **Expected Output**:
   ```
   List of devices attached
   XXXXXXXX    device
   ```

---

## Rust Android Target Setup

The Shusei project uses Rust with native Android targets. Set up the required targets:

### Install Android Targets

```cmd
rustup target add aarch64-linux-android
rustup target add armv7-linux-androideabi
rustup target add i686-linux-android
rustup target add x86_64-linux-android
```

### Verify Installation

```cmd
rustup target list --installed
```

**Expected Output**:
```
aarch64-linux-android
armv7-linux-androideabi
i686-linux-android
x86_64-linux-android
```

### Install Cargo NDK (Optional but Recommended)

```cmd
cargo install cargo-ndk
```

This tool simplifies building Rust libraries for Android.

---

## Troubleshooting

### "sdkmanager not found" Error

**Symptoms**: Running `sdkmanager --list` returns "'sdkmanager' is not recognized as an internal or external command"

**Solutions**:

1. **Verify Installation**:
   - Open SDK Manager in Android Studio
   - Ensure "Android SDK Command-line Tools (latest)" is installed

2. **Check PATH**:
   ```cmd
   echo %PATH%
   ```
   Verify `%ANDROID_HOME%\cmdline-tools\latest\bin` is present

3. **Manual PATH Addition**:
   - Open Environment Variables
   - Edit `Path` under System variables
   - Add: `%ANDROID_HOME%\cmdline-tools\latest\bin`
   - Restart terminal

4. **Alternative Location**:
   Some installations use:
   ```
   %ANDROID_HOME%\cmdline-tools\bin
   ```
   Add this to PATH if the `latest` folder doesn't exist

### NDK Toolchain Not Found

**Symptoms**: Build errors mentioning "NDK toolchain not found" or "clang not found"

**Solutions**:

1. **Verify NDK Installation**:
   ```cmd
   dir %ANDROID_NDK_HOME%
   ```
   Should show `toolchains`, `sources`, `build`, etc.

2. **Check NDK Version**:
   ```cmd
   type %ANDROID_NDK_HOME%\source.properties
   ```
   Should show `Pkg.Revision = 25.2.9519653` (or your installed version)

3. **Verify Toolchain Path**:
   ```cmd
   dir %ANDROID_NDK_HOME%\toolchains\llvm\prebuilt\windows-x86_64\bin\clang.exe
   ```

4. **Update Environment Variables**:
   Ensure these are set correctly:
   ```
   ANDROID_NDK_HOME = C:\Users\%USERNAME%\AppData\Local\Android\Sdk\ndk\25.2.9519653
   NDK_HOME = C:\Users\%USERNAME%\AppData\Local\Android\Sdk\ndk\25.2.9519653
   ```

5. **Add to PATH**:
   Add to PATH variable:
   ```
   %ANDROID_NDK_HOME%\toolchains\llvm\prebuilt\windows-x86_64\bin
   ```

### USB Debugging Setup for Moto G66j 5G

**Symptoms**: `adb devices` shows "unauthorized" or no devices

**Solutions**:

1. **Enable Developer Options**:
   - `Settings` → `About phone`
   - Tap "Build number" 7 times

2. **Enable USB Debugging**:
   - `Settings` → `System` → `Developer options`
   - Toggle "USB debugging" ON

3. **USB Configuration**:
   - Connect phone via USB
   - Pull down notification shade
   - Tap "Charging this device via USB"
   - Select "File Transfer / Android Auto" or "Transfer files"

4. **Authorize Computer**:
   - A dialog should appear on phone: "Allow USB debugging?"
   - Check "Always allow from this computer"
   - Tap "OK"

5. **Install Google USB Driver** (if device not detected):
   - Open SDK Manager
   - Install "Google USB Driver"
   - Or download from: https://developer.android.com/studio/run/win-usb

6. **Restart ADB Server**:
   ```cmd
   adb kill-server
   adb start-server
   adb devices
   ```

7. **Check Device Manager** (Windows):
   - Press `Win + X` → Device Manager
   - Look for "Android" or your device
   - If there's a yellow exclamation mark:
     - Right-click → Update driver
     - Browse to: `%ANDROID_HOME%\extras\google\usb_driver`

### Build Tools Not Found

**Symptoms**: "Failed to find Build Tools" error in Android Studio

**Solutions**:

1. **Install via SDK Manager**:
   - Open SDK Manager
   - Go to SDK Tools tab
   - Check "Android SDK Build-Tools 34.0.0"
   - Click Apply

2. **Update project configuration**:
   In [`platform/android/app/build.gradle`](platform/android/app/build.gradle):
   ```gradle
   android {
       compileSdkVersion 34
       buildToolsVersion "34.0.0"
   }
   ```

### Rust Target Installation Fails

**Symptoms**: `rustup target add aarch64-linux-android` fails

**Solutions**:

1. **Update Rust**:
   ```cmd
   rustup update
   ```

2. **Install Specific Target**:
   ```cmd
   rustup target add aarch64-linux-android --toolchain stable
   ```

3. **Verify Rust Installation**:
   ```cmd
   rustc --version
   cargo --version
   rustup --version
   ```

### Common Build Errors

| Error | Solution |
|-------|----------|
| `CMake not found` | Install CMake via SDK Manager |
| ` Ninja not found` | Install Ninja via SDK Manager or add to PATH |
| `Java not found` | Ensure JDK is installed and JAVA_HOME is set |
| `Gradle build failed` | Update Gradle version in project settings |

---

## Quick Reference

### Default Installation Paths

| Component | Default Path |
|-----------|-------------|
| Android Studio | `C:\Program Files\Android\Android Studio` |
| Android SDK | `C:\Users\%USERNAME%\AppData\Local\Android\Sdk` |
| NDK | `C:\Users\%USERNAME%\AppData\Local\Android\Sdk\ndk\25.2.9519653` |
| Platform Tools | `C:\Users\%USERNAME%\AppData\Local\Android\Sdk\platform-tools` |
| Build Tools | `C:\Users\%USERNAME%\AppData\Local\Android\Sdk\build-tools\34.0.0` |

### Essential Commands

```cmd
# SDK Management
sdkmanager --list
sdkmanager --install "platform-tools" "build-tools;34.0.0"
sdkmanager --update

# Device Management
adb devices
adb shell
adb logcat
adb install app.apk

# Build Commands
cargo ndk -t arm64-v8a -o platform/android/app/src/main/jniLibs build
cargo build --target aarch64-linux-android --release
```

### Useful Links

- [Android Studio Downloads](https://developer.android.com/studio)
- [Android NDK Downloads](https://developer.android.com/ndk/downloads)
- [Android SDK Platform Tools](https://developer.android.com/studio/releases/platform-tools)
- [Rust Android Guide](https://github.com/rust-lang/rust/wiki/Android)
- [Cargo NDK](https://github.com/bbqsrc/cargo-ndk)

---

## Next Steps

After completing this setup:

1. Open the Shusei project in Android Studio
2. Sync Gradle files
3. Build the project: `Build` → `Make Project`
4. Run on device: `Run` → `Run 'app'` → Select your Moto G66j 5G

For project-specific build instructions, see [`docs/shusei-implementation-plan.md`](docs/shusei-implementation-plan.md).
