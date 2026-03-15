#!/bin/bash
# android-build.sh — Wrapper for Dioxus Android build with automatic patching
#
# Usage: bash scripts/android-build.sh [--release]
#
# This script:
# 1. Runs `dx build --platform android` to generate Gradle files
# 2. Applies the android-patch.sh fixes
# 3. Runs gradlew to build the APK
#
# Options:
#   --release   Build release APK (default: debug)

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
ANDROID_DIR="$PROJECT_ROOT/target/dx/shusei/debug/android/app"

RELEASE_MODE=false
if [ "$1" == "--release" ]; then
    RELEASE_MODE=true
    ANDROID_DIR="$PROJECT_ROOT/target/dx/shusei/release/android/app"
fi

echo "=== Dioxus Android Build ==="
echo "Mode: $([ "$RELEASE_MODE" = true ] && echo "release" || echo "debug")"
echo ""

# Step 1: Run dx build to generate Gradle files
echo "[1/3] Running dx build..."
cd "$PROJECT_ROOT"
dx build --platform android $([ "$RELEASE_MODE" = true ] && echo "--release" || echo "")

# Step 2: Apply patch script
echo ""
echo "[2/3] Applying Gradle patch..."
bash "$SCRIPT_DIR/android-patch.sh"

# Step 3: Run gradlew to build APK
echo ""
echo "[3/3] Building APK with gradlew..."
cd "$ANDROID_DIR"

# Skip lint tasks that crash
if [ "$RELEASE_MODE" = true ]; then
    ./gradlew assembleRelease \
        -x lintVitalAnalyzeRelease \
        -x lintVitalRelease \
        -x lintVitalReportRelease
    APK_PATH="$ANDROID_DIR/app/build/outputs/apk/release/app-release.apk"
else
    ./gradlew assembleDebug \
        -x lintVitalAnalyzeDebug \
        -x lintVitalDebug \
        -x lintVitalReportDebug
    APK_PATH="$ANDROID_DIR/app/build/outputs/apk/debug/app-debug.apk"
fi

echo ""
echo "=== Build Complete ==="
echo "APK location: $APK_PATH"
echo ""
echo "To install on device:"
echo "  adb install -r $APK_PATH"
