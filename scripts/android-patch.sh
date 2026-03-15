#!/bin/bash
# android-patch.sh — Post-generation patch for Dioxus Android Gradle files
#
# Fixes Dioxus 0.7.3 generated Gradle config for modern Android tooling.
# Based on GitHub issue #5251 workaround.
#
# Usage: bash scripts/android-patch.sh
# Run this AFTER `dx build --platform android` generates the Gradle files.

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
ANDROID_DIR="$PROJECT_ROOT/target/dx/shusei/debug/android/app"

echo "=== Android Gradle Patch ==="
echo "Patching: $ANDROID_DIR"

if [ ! -d "$ANDROID_DIR" ]; then
    echo "ERROR: Android directory not found. Run 'dx build --platform android' first."
    exit 1
fi

# Fix 1: Update Java version from 8 to 21
echo "[1/3] Fixing Java version (1.8 → 21)..."
sed -i 's/VERSION_1_8/VERSION_21/g' "$ANDROID_DIR/build.gradle.kts"
sed -i 's/jvmTarget = "1.8"/jvmTarget = "21"/g' "$ANDROID_DIR/build.gradle.kts"

# Fix 2: Remove deprecated manifest attribute
echo "[2/3] Removing deprecated manifest attributes..."
sed -i 's/ android:extractNativeLibs="false"//g' "$ANDROID_DIR/src/main/AndroidManifest.xml"

# Fix 3: Disable lint tasks that crash with AGP 8.8+
echo "[3/3] Disabling broken lint tasks..."
# Add lint configuration to build.gradle.kts to skip lint on release builds
if ! grep -q "lint {" "$ANDROID_DIR/build.gradle.kts"; then
    cat >> "$ANDROID_DIR/build.gradle.kts" << 'EOF'

android {
    lint {
        checkReleaseBuilds = false
        abortOnError = false
    }
}
EOF
    echo "  Added lint configuration to build.gradle.kts"
else
    echo "  Lint configuration already present"
fi

echo "=== Patch Complete ==="
echo "Next: Run gradlew assembleRelease or use android-build.sh wrapper"
