# Android Build Scripts

## Prerequisites

Before running `android-build.sh`, ensure the following are installed:

### 1. Android SDK
- Install Android Studio or command-line tools
- Set `ANDROID_HOME` environment variable:
  ```bash
  export ANDROID_HOME="$HOME/Library/Android/sdk"  # macOS
  export ANDROID_HOME="$HOME/Android/Sdk"          # Linux
  ```

### 2. Android NDK
- Install via Android Studio SDK Manager or download separately
- Set `ANDROID_NDK_HOME` environment variable:
  ```bash
  export ANDROID_NDK_HOME="$ANDROID_HOME/ndk/27.0.12077973"  # Use your NDK version
  ```

### 3. Java JDK 21
- Required for modern Android Gradle Plugin
- Install: `brew install openjdk@21` (macOS) or `apt install openjdk-21-jdk` (Linux)
- Set `JAVA_HOME`:
  ```bash
  export JAVA_HOME="/opt/homebrew/opt/openjdk@21"  # macOS
  export JAVA_HOME="/usr/lib/jvm/java-21-openjdk"  # Linux
  ```

### 4. Rust Android Targets
```bash
rustup target add aarch64-linux-android
rustup target add armv7-linux-androideabi
rustup target add x86_64-linux-android
rustup target add i686-linux-android
```

### 5. CMake
- Required for NDK builds
- Install: `brew install cmake` or `apt install cmake`

## Verification

Run these commands to verify your setup:

```bash
echo $ANDROID_HOME
echo $ANDROID_NDK_HOME
echo $JAVA_HOME
dx --version
rustup target list | grep android
```

## Usage

```bash
# Debug build
bash scripts/android-build.sh

# Release build
bash scripts/android-build.sh --release

# Install on connected device
adb install -r target/dx/shusei/debug/android/app/app/build/outputs/apk/debug/app-debug.apk
```

## Troubleshooting

### "Android not installed properly"
- Set `ANDROID_NDK_HOME` to your NDK installation path
- Ensure NDK version is 27.x or later

### "Java version mismatch"
- Ensure JDK 21 is installed and JAVA_HOME is set
- The patch script automatically fixes Gradle Java version

### "gradlew: Permission denied"
- Run `chmod +x target/dx/shusei/*/android/app/gradlew`

## Model Files

### NDLOCR (Included)
- `assets/models/ndlocr/deim-s-1024x1024.onnx` (40MB)
- `assets/models/ndlocr/parseq-*.onnx` (35-41MB each)

### Moonshine (Optional for M002)
- Models not required for M002 prototype (voice memos deferred to M003)
- See `assets/models/moonshine/README.md` for acquisition instructions
