#!/bin/bash
# S02 Camera Book Capture Verification Script
# 
# This script verifies the camera book capture flow on a physical Android device.
# It tests: book creation, camera navigation, page save, and data persistence.
#
# Usage: bash scripts/verify-s02-camera.sh
#
# Requirements:
# - Android device connected via USB (adb devices shows device)
# - APK already built (target/dx/shusei/debug/android/app/app/build/outputs/apk/debug/app-debug.apk)
# - WSL2 USB passthrough configured (if running on WSL2)

set -e

echo "========================================="
echo "S02 Camera Book Capture Verification"
echo "========================================="
echo ""

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Check device connection
echo "📱 Checking device connection..."
if ! adb devices | grep -q "device$"; then
    echo -e "${RED}✗ No Android device connected${NC}"
    echo "Connect your Moto G66j 5G via USB and ensure adb recognizes it."
    echo "Run 'adb devices' to verify."
    exit 1
fi
echo -e "${GREEN}✓ Device connected${NC}"
echo ""

# Install APK
echo "📦 Installing APK..."
APK_PATH="target/dx/shusei/debug/android/app/app/build/outputs/apk/debug/app-debug.apk"
if [ ! -f "$APK_PATH" ]; then
    echo -e "${RED}✗ APK not found at $APK_PATH${NC}"
    echo "Run 'bash scripts/android-build.sh' first."
    exit 1
fi

adb install -r "$APK_PATH" > /tmp/adb-install.log 2>&1
if [ $? -eq 0 ]; then
    echo -e "${GREEN}✓ APK installed successfully${NC}"
else
    echo -e "${RED}✗ APK installation failed${NC}"
    cat /tmp/adb-install.log
    exit 1
fi
echo ""

# Launch app
echo "🚀 Launching app..."
adb shell am start -n com.shusei.app/.MainActivity > /tmp/adb-start.log 2>&1
if [ $? -eq 0 ]; then
    echo -e "${GREEN}✓ App launched${NC}"
else
    echo -e "${RED}✗ App launch failed${NC}"
    cat /tmp/adb-start.log
    exit 1
fi
sleep 2
echo ""

# Start logcat monitoring in background
echo "📊 Starting logcat monitoring..."
adb logcat -c # Clear old logs
adb logcat | grep -i shusei > /tmp/logcat-s02.log &
LOGCAT_PID=$!
echo "Logcat PID: $LOGCAT_PID"
sleep 1
echo ""

# Create test book via SQLite
echo "📚 Creating test book via SQLite..."
adb shell "sqlite3 /data/data/com.shusei.app/files/shusei.db \"INSERT INTO books (id, title, author, pages_captured, created_at, updated_at, is_pdf) VALUES ('test-s02-book', 'S02 Test Book', 'Test Author', 0, $(date +%s), $(date +%s), 0);\""
if [ $? -eq 0 ]; then
    echo -e "${GREEN}✓ Test book created${NC}"
else
    echo -e "${YELLOW}⚠ Could not insert test book (table may not exist yet)${NC}"
fi
echo ""

# Manual test instructions
echo "========================================="
echo "📋 MANUAL TEST STEPS"
echo "========================================="
echo ""
echo "The app is now running on your device. Follow these steps:"
echo ""
echo "1. CREATE BOOK:"
echo "   - Tap 'Add Book' button"
echo "   - Enter Title: 'Camera Test Book'"
echo "   - Enter Author: 'Test Author'"
echo "   - Tap 'Add Book'"
echo "   - Expected: Navigates to camera page"
echo ""
echo "2. CAPTURE PAGE 1:"
echo "   - Enter Page Number: 1"
echo "   - Tap 'Take Photo'"
echo "   - Capture an image (point camera at any text)"
echo "   - Wait for image preview to appear"
echo "   - Expected: Image shows in preview"
echo ""
echo "3. RUN OCR:"
echo "   - Tap 'Run OCR' button"
echo "   - Wait for processing (1-5 seconds)"
echo "   - Expected: OCR result text appears"
echo ""
echo "4. SAVE PAGE:"
echo "   - Tap 'Save Page' button"
echo "   - Wait for save confirmation"
echo "   - Expected: Success message shown"
echo ""
echo "5. VERIFY IN DATABASE:"
echo "   - Press Enter when ready to verify..."
read -p ""
echo ""

# Stop logcat
kill $LOGCAT_PID 2>/dev/null || true

# Check database
echo "🔍 Checking database..."
echo ""
echo "Books in database:"
adb shell "sqlite3 /data/data/com.shusei.app/files/shusei.db 'SELECT id, title, pages_captured FROM books;'" 2>/dev/null || echo "Could not query books table"
echo ""
echo "Book pages in database:"
adb shell "sqlite3 /data/data/com.shusei.app/files/shusei.db 'SELECT id, book_id, page_number, substr(ocr_text_plain, 1, 50) FROM book_pages;'" 2>/dev/null || echo "Could not query book_pages table"
echo ""

# Check for errors in logcat
echo "🔍 Checking for errors in logcat..."
ERROR_COUNT=$(grep -c "ERROR\|FATAL\|Exception" /tmp/logcat-s02.log 2>/dev/null || echo "0")
if [ "$ERROR_COUNT" -gt 0 ]; then
    echo -e "${YELLOW}⚠ Found $ERROR_COUNT error(s) in logcat:${NC}"
    grep "ERROR\|FATAL\|Exception" /tmp/logcat-s02.log | tail -20
else
    echo -e "${GREEN}✓ No errors found in logcat${NC}"
fi
echo ""

# Check for success signals
echo "🔍 Checking for success signals..."
if grep -q "OCR completed" /tmp/logcat-s02.log; then
    echo -e "${GREEN}✓ OCR completed successfully${NC}"
else
    echo -e "${YELLOW}⚠ OCR completion not found in logs${NC}"
fi

if grep -q "Page saved" /tmp/logcat-s02.log; then
    echo -e "${GREEN}✓ Page saved successfully${NC}"
else
    echo -e "${YELLOW}⚠ Page save not found in logs${NC}"
fi

if grep -q "Book created" /tmp/logcat-s02.log; then
    echo -e "${GREEN}✓ Book created successfully${NC}"
else
    echo -e "${YELLOW}⚠ Book creation not found in logs${NC}"
fi
echo ""

# Save logs for debugging
echo "💾 Saving logs..."
cp /tmp/logcat-s02.log /tmp/logcat-s02-$(date +%Y%m%d-%H%M%S).log
echo "Logcat saved to: /tmp/logcat-s02-*.log"
echo ""

echo "========================================="
echo "✅ VERIFICATION COMPLETE"
echo "========================================="
echo ""
echo "Next steps:"
echo "- Review database output above"
echo "- Check logcat logs for any errors"
echo "- If tests failed, attach logs to issue report"
echo ""
