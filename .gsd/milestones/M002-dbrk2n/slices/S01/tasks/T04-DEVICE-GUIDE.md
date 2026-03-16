# T04 Device Testing Guide

## Prerequisites

### Windows Host Configuration (for WSL2 USB Passthrough)

1. **Enable USB Device Passthrough in WSL2:**
   ```powershell
   # In Windows PowerShell (Admin)
   wsl --update
   ```

2. **Configure USB Device Forwarding:**
   - Install USB Network Gate or similar USB-over-IP software on Windows
   - Or use `usbipd-win` (Microsoft's official USB/IP solution):
     ```powershell
     # Install usbipd
     winget install dorssel.usbipd-win
     
     # List USB devices
     usbipd list
     
     # Bind your Android device (replace BUSID)
     usbipd bind --busid 1-2
     
     # Attach to WSL2
     usbipd attach --wsl --busid 1-2
     ```

3. **Enable Developer Options on Android Device:**
   - Settings → About Phone → Tap "Build Number" 7 times
   - Settings → System → Developer Options → Enable "USB Debugging"

### Verify Device Connection

```bash
# In WSL2 terminal
adb devices

# Expected output:
# List of devices attached
# XXXXXXXX    device
```

## Running Verification

### Option 1: Automated Script

```bash
cd /home/devuser/develop/shusei
bash scripts/verify-app-launch.sh
```

### Option 2: Manual Steps

```bash
export ANDROID_HOME=/home/devuser/android-sdk
export PATH=$PATH:$ANDROID_HOME/platform-tools

# 1. Check device
adb devices

# 2. Install APK
adb install -r target/dx/shusei/debug/android/app/app/build/outputs/apk/debug/app-debug.apk

# 3. Launch app
adb shell am start -n com.shusei.app/.MainActivity

# 4. Check for crashes (wait a few seconds)
adb logcat -d | grep -i "FATAL\|AndroidRuntime"

# 5. Insert test book
TIMESTAMP=$(date +%s)
adb shell sqlite3 /data/data/com.shusei.app/databases/shusei.db \
  "INSERT INTO books (id, title, author, created_at, updated_at) VALUES ('test-manual', 'Manual Test Book', 'Test Author', $TIMESTAMP, $TIMESTAMP);"

# 6. Verify book exists
adb shell sqlite3 /data/data/com.shusei.app/databases/shusei.db \
  "SELECT * FROM books WHERE id='test-manual';"

# 7. Force close app
adb shell am force-stop com.shusei.app

# 8. Reopen app
adb shell am start -n com.shusei.app/.MainActivity

# 9. Verify book persists
adb shell sqlite3 /data/data/com.shusei.app/databases/shusei.db \
  "SELECT COUNT(*) FROM books WHERE id='test-manual';"

# Expected output: 1
```

## Troubleshooting

### No Device Found

```bash
# Check USB connection
lsusb

# Restart adb server
adb kill-server
adb start-server

# Check again
adb devices
```

### Authorization Required

If device shows as "unauthorized":
1. Check your Android device screen
2. Accept the "Allow USB debugging?" prompt
3. Run `adb devices` again

### App Crashes on Launch

```bash
# Get detailed crash logs
adb logcat -d | grep -A 20 "com.shusei.app"

# Check for JNI errors
adb logcat -d | grep -i "jni\|UnsatisfiedLinkError"

# Check app installation
adb shell dumpsys package com.shusei.app
```

### Database Not Accessible

```bash
# Check if database file exists
adb shell ls -la /data/data/com.shusei.app/databases/

# Check database permissions
adb shell ls -la /data/data/com.shusei.app/databases/shusei.db

# If permission denied, try as root (if device is rooted)
adb root
adb shell sqlite3 /data/data/com.shusei.app/databases/shusei.db ".tables"
```

## Expected Results

### Successful App Launch

- No FATAL exceptions in logcat
- Main activity starts without errors
- Library screen renders (check via screenshot or UI automator if needed)

### Successful Persistence Test

```sql
-- Before force close
SELECT COUNT(*) FROM books WHERE id='test-manual';
-- Output: 1

-- After force close + reopen
SELECT COUNT(*) FROM books WHERE id='test-manual';
-- Output: 1 (same count)
```

### Database Schema Verification

```bash
adb shell sqlite3 /data/data/com.shusei.app/databases/shusei.db ".tables"

# Expected tables:
# annotations           processing_progress   vocabulary
# book_pages            sticky_notes          words
# books                 sticky_notes_fts
```

## Cleanup

After testing, remove test data:

```bash
adb shell sqlite3 /data/data/com.shusei.app/databases/shusei.db \
  "DELETE FROM books WHERE id LIKE 'test-%';"

# Or uninstall completely
adb uninstall com.shusei.app
```
