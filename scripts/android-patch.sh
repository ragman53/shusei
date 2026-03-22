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

# Fix 1: Update Java version from 8 to 17 and SDK versions
echo "[1/6] Fixing Java version (1.8 → 17) and SDK versions..."
sed -i 's/VERSION_1_8/VERSION_17/g' "$ANDROID_DIR/app/build.gradle.kts"
sed -i 's/jvmTarget = "1.8"/jvmTarget = "17"/g' "$ANDROID_DIR/app/build.gradle.kts"
# Update compileSdk and targetSdk to 36
sed -i 's/compileSdk = [0-9]*/compileSdk = 36/g' "$ANDROID_DIR/app/build.gradle.kts"
sed -i 's/targetSdk = [0-9]*/targetSdk = 36/g' "$ANDROID_DIR/app/build.gradle.kts"

# Add Java toolchain configuration to match Kotlin
if ! grep -q "toolchain" "$ANDROID_DIR/app/build.gradle.kts"; then
    # Add java toolchain configuration after android block
    cat >> "$ANDROID_DIR/app/build.gradle.kts" << 'EOF'

android {
    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_17
        targetCompatibility = JavaVersion.VERSION_17
    }
}
EOF
    echo "  Added Java toolchain configuration"
fi

# Fix 2: Remove deprecated manifest attribute
echo "[2/6] Removing deprecated manifest attributes..."
sed -i 's/ android:extractNativeLibs="false"//g' "$ANDROID_DIR/app/src/main/AndroidManifest.xml"

# Fix 3: Add CameraX dependencies for camera capture
echo "[3/6] Adding CameraX dependencies..."
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

# Fix 4: Copy MainActivity.kt with CameraX and file picker implementation
echo "[4/6] Copying MainActivity.kt..."
MAINACTIVITY_SRC="$PROJECT_ROOT/platform/android/app/src/main/kotlin/dev/dioxus/main/MainActivity.kt"
MAINACTIVITY_DEST="$ANDROID_DIR/app/src/main/kotlin/dev/dioxus/main/MainActivity.kt"

if [ -f "$MAINACTIVITY_SRC" ]; then
    mkdir -p "$(dirname "$MAINACTIVITY_DEST")"
    cp "$MAINACTIVITY_SRC" "$MAINACTIVITY_DEST"
    echo "  Copied MainActivity.kt to target directory"
    echo "  MainActivity.kt lines: $(wc -l < "$MAINACTIVITY_DEST")"
else
    echo "  WARNING: MainActivity.kt not found at $MAINACTIVITY_SRC"
    echo "  Using Dioxus-generated default (JNI methods will be missing)"
fi

# Fix 5: Copy assets to Android project for bundling in APK
echo "[5/6] Copying assets to Android project..."
ASSETS_SRC="$PROJECT_ROOT/assets"
ASSETS_DEST="$ANDROID_DIR/app/src/main/assets"

if [ -d "$ASSETS_SRC" ]; then
    mkdir -p "$ASSETS_DEST"
    cp -r "$ASSETS_SRC"/* "$ASSETS_DEST"/
    echo "  Copied assets from $ASSETS_SRC to $ASSETS_DEST"
    
    # Show what was copied
    echo "  Assets copied:"
    find "$ASSETS_DEST" -type f -exec ls -lh {} \; | awk '{print "    " $5 " " $9}' | head -10
else
    echo "  WARNING: Assets directory not found at $ASSETS_SRC"
fi

# Fix 6: Add NDK ABI filter for ARM64 devices
echo "[6/6] Adding NDK ABI filter (arm64-v8a)..."

# Check if abiFilters already exists
if grep -q "abiFilters" "$ANDROID_DIR/app/build.gradle.kts"; then
    echo "  NDK ABI filter already present"
else
    # Inject NDK ABI filter into defaultConfig block using AWK
    if ! awk '
    /^    defaultConfig \{$/ { in_default=1; print; next }
    in_default && /^    \}$/ { 
        print "        ndk {"
        print "            abiFilters += listOf(\"arm64-v8a\")"
        print "        }"
        in_default=0
    }
    { print }
    ' "$ANDROID_DIR/app/build.gradle.kts" > "$ANDROID_DIR/app/build.gradle.kts.tmp"; then
        echo "  ERROR: Failed to process build.gradle.kts for NDK filter"
        rm -f "$ANDROID_DIR/app/build.gradle.kts.tmp"
        exit 1
    fi
    if ! mv "$ANDROID_DIR/app/build.gradle.kts.tmp" "$ANDROID_DIR/app/build.gradle.kts"; then
        echo "  ERROR: Failed to update build.gradle.kts"
        rm -f "$ANDROID_DIR/app/build.gradle.kts.tmp"
        exit 1
    fi
    echo "  Added NDK ABI filter to app/build.gradle.kts"
fi

# Fix 7: Fix Logger.kt BuildConfig import
echo "[7/7] Fixing Logger.kt BuildConfig import..."
LOGGER_FILE="$ANDROID_DIR/app/src/main/kotlin/dev/dioxus/main/Logger.kt"
if [ -f "$LOGGER_FILE" ]; then
    # Add import for BuildConfig from the correct package
    if ! grep -q "import com.shusei.app.BuildConfig" "$LOGGER_FILE"; then
        sed -i 's/^package dev.dioxus.main$/package dev.dioxus.main\n\nimport com.shusei.app.BuildConfig/' "$LOGGER_FILE"
        echo "  Added BuildConfig import to Logger.kt"
    else
        echo "  Logger.kt already has BuildConfig import"
    fi
else
    echo "  WARNING: Logger.kt not found"
fi

echo "=== Patch Complete ==="
echo "Next: Run gradlew assembleRelease or use android-build.sh wrapper"
