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

# Fix 1: Update Java version from 8 to 17
echo "[1/5] Fixing Java version (1.8 → 17)..."
sed -i 's/VERSION_1_8/VERSION_17/g' "$ANDROID_DIR/app/build.gradle.kts"
sed -i 's/jvmTarget = "1.8"/jvmTarget = "17"/g' "$ANDROID_DIR/app/build.gradle.kts"

# Fix 2: Remove deprecated manifest attribute
echo "[2/5] Removing deprecated manifest attributes..."
sed -i 's/ android:extractNativeLibs="false"//g' "$ANDROID_DIR/app/src/main/AndroidManifest.xml"

# Fix 3: Add CameraX dependencies for camera capture
echo "[3/5] Adding CameraX dependencies..."
CAMERAX_DEPS='    implementation("androidx.camera:camera-core:1.3.4")
    implementation("androidx.camera:camera-camera2:1.3.4")
    implementation("androidx.camera:camera-lifecycle:1.3.4")
    implementation("androidx.camera:camera-view:1.3.4")'

# Check if CameraX dependencies already exist
if grep -q "camera-core" "$ANDROID_DIR/app/build.gradle.kts"; then
    echo "  CameraX dependencies already present"
else
    # Insert CameraX deps before the closing brace of dependencies block
    # Find the line with the closing brace of dependencies block and insert before it
    if ! awk -v deps="$CAMERAX_DEPS" '
    /^dependencies \{$/ { in_deps=1; print; next }
    in_deps && /^}$/ { 
        print deps
        print ""
        in_deps=0
    }
    { print }
    ' "$ANDROID_DIR/app/build.gradle.kts" > "$ANDROID_DIR/app/build.gradle.kts.tmp"; then
        echo "  ERROR: Failed to process build.gradle.kts"
        rm -f "$ANDROID_DIR/app/build.gradle.kts.tmp"
        exit 1
    fi
    if ! mv "$ANDROID_DIR/app/build.gradle.kts.tmp" "$ANDROID_DIR/app/build.gradle.kts"; then
        echo "  ERROR: Failed to update build.gradle.kts"
        rm -f "$ANDROID_DIR/app/build.gradle.kts.tmp"
        exit 1
    fi
    echo "  Added CameraX dependencies to app/build.gradle.kts"
fi

# Fix 4: Disable lint tasks that crash with AGP 8.8+
echo "[4/5] Disabling broken lint tasks..."
# Add lint configuration to app/build.gradle.kts to skip lint on release builds
if ! grep -q "lint {" "$ANDROID_DIR/app/build.gradle.kts"; then
    cat >> "$ANDROID_DIR/app/build.gradle.kts" << 'EOF'

android {
    lint {
        checkReleaseBuilds = false
        abortOnError = false
        disable.addAll(listOf("LintError", "MissingDefaultResource", "UnusedResources", "All"))
    }
}
EOF
    echo "  Added lint configuration to app/build.gradle.kts"
else
    echo "  Lint configuration already present"
fi

# Fix 5: Copy assets to Android project for bundling in APK
echo "[5/5] Copying assets to Android project..."
ASSETS_SRC="$PROJECT_ROOT/assets"
ASSETS_DEST="$ANDROID_DIR/app/src/main/assets"

if [ -d "$ASSETS_SRC" ]; then
    mkdir -p "$ASSETS_DEST"
    cp -r "$ASSETS_SRC"/* "$ASSETS_DEST"/
    echo "  Copied assets from $ASSETS_SRC to $ASSETS_DEST"
    
    # Show what was copied
    echo "  Assets copied:"
    find "$ASSETS_DEST" -type f -exec ls -lh {} \; | awk '{print "    " $5 " " $9}'
else
    echo "  WARNING: Assets directory not found at $ASSETS_SRC"
fi

echo "=== Patch Complete ==="
echo "Next: Run gradlew assembleRelease or use android-build.sh wrapper"
