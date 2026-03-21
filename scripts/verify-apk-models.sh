#!/bin/bash
# verify-apk-models.sh — Verify APK contains model assets
#
# Usage: bash scripts/verify-apk-models.sh [path-to-apk]
#
# If no APK path is provided, uses the default debug APK location.

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

# Default APK path
APK_PATH="${1:-$PROJECT_ROOT/target/dx/shusei/debug/android/app/app/build/outputs/apk/debug/app-debug.apk}"

echo "=== APK Model Assets Verification ==="
echo "APK: $APK_PATH"
echo ""

# Check if APK exists
if [ ! -f "$APK_PATH" ]; then
    echo "❌ ERROR: APK not found at $APK_PATH"
    echo ""
    echo "Build the APK first:"
    echo "  bash scripts/android-build.sh"
    exit 1
fi

# List all .onnx files in the APK
echo "[1/4] Checking for ONNX model files in APK..."
echo ""

ONNX_FILES=$(unzip -l "$APK_PATH" | grep -E "\.onnx$" || true)

if [ -z "$ONNX_FILES" ]; then
    echo "❌ No ONNX files found in APK"
    echo ""
    echo "Full APK contents:"
    unzip -l "$APK_PATH" | head -50
    exit 1
fi

echo "$ONNX_FILES"
echo ""

# Count model files
NDLOCR_COUNT=$(echo "$ONNX_FILES" | grep -c "ndlocr" || echo "0")
MOONSHINE_COUNT=$(echo "$ONNX_FILES" | grep -c "moonshine" || echo "0")

echo "[2/4] Model file counts:"
echo "  NDLOCR models: $NDLOCR_COUNT"
echo "  Moonshine models: $MOONSHINE_COUNT"
echo ""

# Check for specific required files
echo "[3/4] Checking for required model files..."
echo ""

# NDLOCR models (detection + recognition)
REQUIRED_NDLOCR=(
    "deim-s-1024x1024.onnx"
    "parseq-ndl-16x256-30-tiny-192epoch-tegaki3.onnx"
    "parseq-ndl-16x384-50-tiny-146epoch-tegaki2.onnx"
    "parseq-ndl-16x768-100-tiny-165epoch-tegaki2.onnx"
)

NDLOCR_PASS=0
NDLOCR_FAIL=0

for model in "${REQUIRED_NDLOCR[@]}"; do
    if unzip -l "$APK_PATH" | grep -q "$model"; then
        echo "  ✅ $model"
        NDLOCR_PASS=$((NDLOCR_PASS + 1))
    else
        echo "  ❌ $model (MISSING)"
        NDLOCR_FAIL=$((NDLOCR_FAIL + 1))
    fi
done

echo ""

# Moonshine models (deferred to M003 - not required for S05)
# REQUIRED_MOONSHINE=(
#     "moonshine-tiny-en-encoder.onnx"
#     "moonshine-tiny-en-decoder.onnx"
#     "moonshine-tiny-ja-encoder.onnx"
#     "moonshine-tiny-ja-decoder.onnx"
# )

MOONSHINE_PASS=0
MOONSHINE_FAIL=0
REQUIRED_MOONSHINE=()

# Skip Moonshine checks for S05
echo "  ℹ️  Moonshine models deferred to M003 (skipping check)"

echo ""

# Calculate total model size
echo "[4/4] Calculating model sizes..."
echo ""

TOTAL_SIZE=$(unzip -l "$APK_PATH" | grep -E "\.onnx$" | awk '{sum += $1} END {print sum}')
TOTAL_SIZE_MB=$((TOTAL_SIZE / 1024 / 1024))

echo "  Total model size: ${TOTAL_SIZE_MB}MB"
echo ""

# Summary
echo "=== Summary ==="
echo ""
echo "NDLOCR models: $NDLOCR_PASS/${#REQUIRED_NDLOCR[@]} present"
echo "Moonshine models: $MOONSHINE_PASS/${#REQUIRED_MOONSHINE[@]} present"
echo "Total model size: ${TOTAL_SIZE_MB}MB"
echo ""

# Check if under 50MB limit
if [ $TOTAL_SIZE_MB -lt 50 ]; then
    echo "✅ Model size under 50MB limit"
else
    echo "⚠️  Model size exceeds 50MB (consider optimization)"
fi

echo ""

# Final verdict
if [ $NDLOCR_FAIL -eq 0 ] && [ $MOONSHINE_FAIL -eq 0 ]; then
    echo "✅ All required model files present in APK"
    exit 0
else
    echo "❌ Missing model files in APK"
    echo ""
    echo "Troubleshooting:"
    echo "  1. Ensure model files exist in assets/models/"
    echo "  2. Verify Dioxus.toml includes: resources = [\"assets/models/*\"]"
    echo "  3. Rebuild APK: bash scripts/android-build.sh"
    echo "  4. Re-run verification: bash scripts/verify-apk-models.sh"
    exit 1
fi
