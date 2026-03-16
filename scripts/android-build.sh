#!/bin/bash
# android-build.sh — Wrapper for Dioxus Android build with automatic patching

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

# Set NDK environment
export ANDROID_HOME="$HOME/android-ndk/android-ndk-r26d"
export ANDROID_NDK_HOME="$ANDROID_HOME"
export PATH="$PATH:$ANDROID_HOME/toolchains/llvm/prebuilt/linux-x86_64/bin"
export CC_aarch64_linux_android="$ANDROID_HOME/toolchains/llvm/prebuilt/linux-x86_64/bin/aarch64-linux-android21-clang -march=armv8.2-a+fp16"
export CC_x86_64_linux_android="$ANDROID_HOME/toolchains/llvm/prebuilt/linux-x86_64/bin/x86_64-linux-android21-clang"
export AR_x86_64_linux_android="$ANDROID_HOME/toolchains/llvm/prebuilt/linux-x86_64/bin/llvm-ar"
export AR_aarch64_linux_android="$ANDROID_HOME/toolchains/llvm/prebuilt/linux-x86_64/bin/llvm-ar"

RELEASE_MODE=false
if [ "$1" == "--release" ]; then
    RELEASE_MODE=true
fi

echo "=== Dioxus Android Build ==="
echo "Mode: $([ "$RELEASE_MODE" = true ] && echo "release" || echo "debug")"
echo "NDK: $ANDROID_NDK_HOME"
echo ""

# Step 1: Run dx build to generate Gradle files
echo "[1/3] Running dx build..."
cd "$PROJECT_ROOT"
dx build --platform android $([ "$RELEASE_MODE" = true ] && echo "--release" || echo "") || true

# Determine Android dir
if [ "$RELEASE_MODE" = true ]; then
    ANDROID_DIR="$PROJECT_ROOT/target/dx/shusei/release/android/app"
else
    ANDROID_DIR="$PROJECT_ROOT/target/dx/shusei/debug/android/app"
fi

# Step 2: Apply patch script
echo ""
echo "[2/3] Applying Gradle patch..."
bash "$SCRIPT_DIR/android-patch.sh"

# Step 3: Run gradlew to build APK
echo ""
echo "[3/3] Building APK with gradlew..."
cd "$ANDROID_DIR"

if [ "$RELEASE_MODE" = true ]; then
    ./gradlew assembleRelease -x lintVitalAnalyzeRelease -x lintVitalRelease -x lintVitalReportRelease || true
    APK_PATH="$ANDROID_DIR/app/build/outputs/apk/release/app-release.apk"
else
    ./gradlew assembleDebug -x lintVitalAnalyzeDebug -x lintVitalDebug -x lintVitalReportDebug || true
    APK_PATH="$ANDROID_DIR/app/build/outputs/apk/debug/app-debug.apk"
fi

echo ""
echo "=== Build Complete ==="
echo "APK location: $APK_PATH"
if [ -f "$APK_PATH" ]; then
    echo "✓ APK built successfully!"
    echo ""
    echo "To install on device:"
    echo "  adb install -r $APK_PATH"
else
    echo "✗ APK not found - check build logs above"
fi
