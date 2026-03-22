#!/bin/bash
# android-build.sh — Wrapper for Dioxus Android build with automatic patching

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

# Set NDK environment
export ANDROID_HOME="/home/devuser/android-sdk"
export ANDROID_NDK_HOME="$ANDROID_HOME/ndk/29.0.14206865"
export PATH="$PATH:$ANDROID_NDK_HOME/toolchains/llvm/prebuilt/linux-x86_64/bin"
export CC_aarch64_linux_android="$ANDROID_NDK_HOME/toolchains/llvm/prebuilt/linux-x86_64/bin/aarch64-linux-android21-clang -march=armv8.2-a+fp16"
export CC_x86_64_linux_android="$ANDROID_NDK_HOME/toolchains/llvm/prebuilt/linux-x86_64/bin/x86_64-linux-android21-clang"
export AR_x86_64_linux_android="$ANDROID_NDK_HOME/toolchains/llvm/prebuilt/linux-x86_64/bin/llvm-ar"
export AR_aarch64_linux_android="$ANDROID_NDK_HOME/toolchains/llvm/prebuilt/linux-x86_64/bin/llvm-ar"

RELEASE_MODE=false
if [ "$1" == "--release" ]; then
    RELEASE_MODE=true
fi

echo "=== Dioxus Android Build ==="
echo "Mode: $([ "$RELEASE_MODE" = true ] && echo "release" || echo "debug")"
echo "NDK: $ANDROID_NDK_HOME"
echo ""

# Set Rust target for ARM64 devices
export CARGO_BUILD_TARGET="aarch64-linux-android"

# Set correct NDK paths for Rust cross-compilation
export ANDROID_NDK_HOME="$ANDROID_HOME/ndk/29.0.14206865"
export CC_aarch64_linux_android="$ANDROID_NDK_HOME/toolchains/llvm/prebuilt/linux-x86_64/bin/aarch64-linux-android21-clang"
export AR_aarch64_linux_android="$ANDROID_NDK_HOME/toolchains/llvm/prebuilt/linux-x86_64/bin/llvm-ar"

# Step 1: Run dx build to generate Gradle files
echo "[1/3] Running dx build..."
cd "$PROJECT_ROOT"

# Clean previous build to ensure fresh compilation for correct target
rm -rf "$PROJECT_ROOT/target/dx/shusei/debug/android"

dx build --platform android $([ "$RELEASE_MODE" = true ] && echo "--release" || echo "") || true

# Manually build Rust for ARM64 and copy to Gradle project
echo "Building Rust library for ARM64..."
export CARGO_BUILD_TARGET="aarch64-linux-android"
export ANDROID_NDK_HOME="$ANDROID_HOME/ndk/29.0.14206865"
export CC_aarch64_linux_android="$ANDROID_NDK_HOME/toolchains/llvm/prebuilt/linux-x86_64/bin/aarch64-linux-android21-clang"
export AR_aarch64_linux_android="$ANDROID_NDK_HOME/toolchains/llvm/prebuilt/linux-x86_64/bin/llvm-ar"

cargo build --target aarch64-linux-android --lib 2>&1 | tail -5

# Copy ARM64 .so to Gradle project
ANDROID_JNILIB_DIR="$PROJECT_ROOT/target/dx/shusei/debug/android/app/app/src/main/jniLibs/arm64-v8a"
mkdir -p "$ANDROID_JNILIB_DIR"

# Find the Rust output .so (libshusei.so or similar)
RUST_SO=$(find target/aarch64-linux-android -name "libshusei*.so" -type f 2>/dev/null | head -1)
if [ -n "$RUST_SO" ]; then
    cp "$RUST_SO" "$ANDROID_JNILIB_DIR/libdioxusmain.so"
    echo "Copied $RUST_SO to $ANDROID_JNILIB_DIR/libdioxusmain.so"
else
    # Try finding any .so in the Rust target directory
    RUST_SO=$(find target/aarch64-linux-android/debug/deps -name "libshusei*.so" -type f 2>/dev/null | head -1)
    if [ -n "$RUST_SO" ]; then
        cp "$RUST_SO" "$ANDROID_JNILIB_DIR/libdioxusmain.so"
        echo "Copied $RUST_SO to $ANDROID_JNILIB_DIR/libdioxusmain.so"
    else
        echo "WARNING: Could not find ARM64 Rust .so library"
        find target/aarch64-linux-android -name "*.so" -type f 2>/dev/null | head -5
    fi
fi

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
    ./gradlew assembleRelease || true
    APK_PATH="$ANDROID_DIR/app/build/outputs/apk/release/app-release.apk"
else
    ./gradlew assembleDebug || true
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
