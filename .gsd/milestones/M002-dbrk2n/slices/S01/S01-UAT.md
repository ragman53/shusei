# S01: Android Build + Deploy — UAT

**Milestone:** M002-dbrk2n
**Written:** 2026-03-16

## UAT Type

- **UAT mode:** live-runtime (with artifact-driven verification)
- **Why this mode is sufficient:** S01 delivers build infrastructure and a debug APK. Verification requires both automated checks (build success, test passing) and live device testing (APK installation, app launch, persistence). Desktop-based database tests provide strong evidence for persistence logic, but final validation requires physical hardware.

## Preconditions

1. **Android SDK and NDK installed:**
   - `$ANDROID_HOME` points to `/home/devuser/android-sdk`
   - NDK at `/home/devuser/android-ndk/android-ndk-r26d`
   - Platform-33 and build-tools-34.0.0 installed
   - SDK licenses accepted

2. **Debug APK built:**
   - APK exists at `target/dx/shusei/debug/android/app/app/build/outputs/apk/debug/app-debug.apk` (139MB)
   - Run `bash scripts/android-build.sh` if APK not present

3. **Physical device prepared:**
   - Moto G66j 5G with USB debugging enabled
   - WSL2 USB passthrough configured on Windows host
   - Device connected via USB

4. **Verification scripts executable:**
   - `bash scripts/verify-app-launch.sh`
   - `bash scripts/verify-apk-models.sh`

## Smoke Test

**Quick build and install check (5 minutes):**
```bash
# Verify APK exists
ls -lh target/dx/shusei/debug/android/app/app/build/outputs/apk/debug/app-debug.apk
# Expected: 139MB APK file

# Check device connection
adb devices
# Expected: Device listed as "device" (not "unauthorized" or missing)

# Install APK
adb install -r target/dx/shusei/debug/android/app/app/build/outputs/apk/debug/app-debug.apk
# Expected: "Success" message

# Launch app
adb shell am start -n com.shusei.app/.MainActivity
# Expected: "Starting: Intent" message, app appears on device screen
```

## Test Cases

### 1. Build Script Execution

**Purpose:** Verify Gradle patch script and build wrapper work correctly.

1. Run the build script:
   ```bash
   bash scripts/android-build.sh 2>&1 | tee /tmp/android-build.log
   ```

2. Check build output:
   ```bash
   tail -20 /tmp/android-build.log
   ```

3. **Expected:**
   - Patch script logs: "[1/3] Fixing Java version...", "[2/3] Removing manifest attributes...", "[3/3] Disabling lint tasks..."
   - Gradle build completes with "BUILD SUCCESSFUL"
   - APK generated at expected path
   - No Java version errors or lint failures

### 2. APK Model Assets Verification

**Purpose:** Verify NDLOCR and Moonshine models are bundled in APK.

1. Run the model verification script:
   ```bash
   bash scripts/verify-apk-models.sh
   ```

2. Or manually inspect APK contents:
   ```bash
   unzip -l target/dx/shusei/debug/android/app/app/build/outputs/apk/debug/app-debug.apk | grep models
   ```

3. **Expected:**
   - `assets/models/ndlocr/` directory present in APK
   - 4 NDLOCR ONNX files listed (detection + recognition models)
   - Total model size ~147MB
   - Moonshine models present if downloaded (optional for S01)

### 3. App Launch Without Crashes

**Purpose:** Verify app launches on device without FATAL exceptions.

1. Ensure app is installed:
   ```bash
   adb shell pm list packages | grep com.shusei.app
   # Expected: package:com.shusei.app
   ```

2. Launch app and monitor logs:
   ```bash
   adb shell am start -n com.shusei.app/.MainActivity
   adb logcat | grep -i shusei
   ```

3. Wait 10 seconds for app initialization.

4. **Expected:**
   - App launches and shows main UI on device screen
   - No "FATAL EXCEPTION" in logcat
   - No "AndroidRuntime" crashes
   - No "UnsatisfiedLinkError" or JNI initialization failures
   - App remains in foreground (doesn't immediately close)

### 4. SQLite Data Persistence Across App Restart

**Purpose:** Verify books, pages, and words survive app force-close and reopen.

1. Insert test book directly into database:
   ```bash
   adb shell sqlite3 /data/data/com.shusei.app/databases/shusei.db \
     "INSERT INTO books (id, title, author, created_at, updated_at) VALUES ('uat-test-1', 'UAT Test Book', 'Test Author', 1234567890, 1234567890);"
   ```

2. Verify book was inserted:
   ```bash
   adb shell sqlite3 /data/data/com.shusei.app/databases/shusei.db "SELECT * FROM books;"
   # Expected: uat-test-1 | UAT Test Book | Test Author | ...
   ```

3. Force close the app:
   ```bash
   adb shell am force-stop com.shusei.app
   ```

4. Reopen the app:
   ```bash
   adb shell am start -n com.shusei.app/.MainActivity
   ```

5. Wait 5 seconds, then check database again:
   ```bash
   adb shell sqlite3 /data/data/com.shusei.app/databases/shusei.db "SELECT * FROM books;"
   ```

6. **Expected:**
   - Book still exists after force close and reopen
   - Same id, title, author, timestamps
   - No database corruption or schema errors

### 5. Database Schema Completeness

**Purpose:** Verify all required tables exist in SQLite database.

1. Check database schema:
   ```bash
   adb shell sqlite3 /data/data/com.shusei.app/databases/shusei.db ".tables"
   ```

2. **Expected:**
   - `books` table present
   - `book_pages` table present
   - `words` table present
   - `annotations` table present
   - `sticky_notes` table present
   - `processing_progress` table present
   - `vocabulary` table present (if applicable)

### 6. Desktop Database Persistence Tests

**Purpose:** Verify database persistence logic without device (fallback verification).

1. Run database tests:
   ```bash
   cargo test --lib db::tests::books -- --nocapture
   ```

2. **Expected:**
   - All 6 books tests pass
   - `test_create_book_persists_to_file` passes (simulates app restart)
   - `test_create_multiple_books` passes
   - `test_get_book_by_id`, `test_update_book`, `test_delete_book` all pass
   - No test failures or panics

## Edge Cases

### App Launch on Cold Start

1. Uninstall and reinstall app:
   ```bash
   adb uninstall com.shusei.app
   adb install -r target/dx/shusei/debug/android/app/app/build/outputs/apk/debug/app-debug.apk
   ```

2. Launch app for first time:
   ```bash
   adb shell am start -n com.shusei.app/.MainActivity
   adb logcat | grep -i shusei
   ```

3. **Expected:**
   - App creates database on first launch
   - No crashes during initialization
   - Main UI appears within 10 seconds

### Database Lock Contention

1. Launch app normally.

2. While app is running, attempt to write to database via adb:
   ```bash
   adb shell sqlite3 /data/data/com.shusei.app/databases/shusei.db \
     "INSERT INTO books (id, title, author, created_at, updated_at) VALUES ('concurrent-test', 'Concurrent Test', 'Test', 1234567890, 1234567890);"
   ```

3. **Expected:**
   - Insert succeeds (SQLite handles concurrent access)
   - No "database is locked" errors
   - App continues running normally

### Low Memory Scenario

1. Open multiple apps on device to consume memory.

2. Launch Shusei app:
   ```bash
   adb shell am start -n com.shusei.app/.MainActivity
   ```

3. **Expected:**
   - App launches (may be slower)
   - No OutOfMemoryError crashes
   - App survives in background when switching away

## Failure Signals

- **Build failures:**
  - "Java 8 is not supported" or similar JVM errors
  - "Lint found fatal errors" (lint not properly disabled)
  - "SDK location not found" (ANDROID_HOME not set correctly)

- **Installation failures:**
  - "INSTALL_FAILED_UPDATE_INCOMPATIBLE" (uninstall old version first)
  - "INSTALL_FAILED_INSUFFICIENT_STORAGE" (free up device storage)
  - "device unauthorized" (check USB debugging authorization on device)

- **Runtime crashes:**
  - "FATAL EXCEPTION" in logcat
  - "AndroidRuntime: Shutting down VM"
  - "UnsatisfiedLinkError" (JNI library not loading)
  - App immediately closes after launch

- **Persistence failures:**
  - Book disappears after force close
  - "database disk image is malformed"
  - "no such table: books" (schema not created)

## Requirements Proved By This UAT

- **R007 (Android Gradle build patch script)** — Proved by Test Case 1 (build script execution)
- **R004 (APK deploys on Moto G66j 5G)** — Proved by Test Case 3 (app launch without crashes) and Smoke Test (installation)
- **R005 (SQLite data persists across restarts)** — Proved by Test Case 4 (persistence across restart) and Test Case 6 (desktop tests)

## Not Proven By This UAT

- **Camera functionality** — Deferred to S02 (Camera Book Capture)
- **PDF reading functionality** — Deferred to S03 (PDF Reflow Reader)
- **Word collection functionality** — Deferred to S04 (Word Collection)
- **Model inference on device** — Deferred to S05 (Model Bundling + Integration)
- **Long-term stability** — This UAT covers initial launch and single restart cycle only
- **Performance under load** — OCR latency, memory usage during camera capture not tested

## Notes for Tester

- **USB debugging:** Ensure USB debugging is enabled on the Moto G66j 5G. If `adb devices` shows "unauthorized", check the device screen for authorization prompt.
- **WSL2 USB passthrough:** If device doesn't appear in `adb devices`, verify USB passthrough is configured correctly on Windows host. May need to restart adb server: `adb kill-server && adb start-server`.
- **Database access:** Requires root or debuggable app. If sqlite3 commands fail with "Permission denied", the app may need to be built in debug mode (which it is).
- **APK size:** 139MB is expected for debug build. Release builds with ProGuard/R8 will be smaller.
- **First launch:** App may take 5-10 seconds to initialize on first launch (database creation, JNI initialization). Subsequent launches should be faster.
- **Logcat filtering:** Use `adb logcat | grep -i shusei` to see app-specific logs. For full crash diagnostics, use `adb logcat | grep -i "FATAL\|AndroidRuntime"`.
- **Automated verification:** Run `bash scripts/verify-app-launch.sh` for complete automated testing. This script performs all manual test steps automatically.
