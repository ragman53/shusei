---
id: T04
parent: S01
milestone: M002-dbrk2n
provides:
  - App launch and SQLite persistence verification script
  - Database persistence tests (6 new tests added)
  - Comprehensive device verification documentation
key_files:
  - scripts/verify-app-launch.sh
  - src/core/db.rs (new books tests)
  - .gsd/milestones/M002-dbrk2n/slices/S01/tasks/T04-DEVICE-GUIDE.md
key_decisions:
  - Device testing deferred to physical hardware - WSL2 USB passthrough required
  - Desktop-based database tests used to verify persistence logic
patterns_established:
  - Automated verification script for app lifecycle testing
  - File-based database persistence tests simulating app restart
observability_surfaces:
  - adb logcat | grep -i shusei for runtime logs
  - adb shell sqlite3 for database queries
  - cargo test --lib db::tests::books for persistence verification
duration: 1 hour
verification_result: partial
completed_at: 2026-03-16
# Device testing blocked by WSL2 USB passthrough requirement - script ready for hardware
blocker_discovered: false
---

# T04: Verify app launch and SQLite persistence

**Verification script created; device testing requires physical hardware connection**

## What Happened

1. **Device Connection Check:** No physical Android device or emulator available in the WSL2 environment. USB passthrough from Windows host is required for adb device access.

2. **Verification Script Created:** Created `scripts/verify-app-launch.sh` - a comprehensive bash script that performs all T04 verification steps:
   - Checks device connection
   - Installs APK if not present
   - Launches app and monitors for crashes
   - Inserts test book via SQLite
   - Force closes and reopens app
   - Verifies book persists after restart
   - Checks database schema completeness
   - Reports JNI initialization status

3. **Database Persistence Tests Added:** Added 6 new tests to `src/core/db.rs` to verify book CRUD operations and file-based persistence:
   - `test_create_book_persists_to_file` - Simulates app restart by closing and reopening database
   - `test_create_multiple_books` - Tests multiple book creation and sorting
   - `test_get_book_by_id` - Tests book retrieval
   - `test_get_book_returns_none_for_non_existent` - Tests error handling
   - `test_update_book` - Tests book updates
   - `test_delete_book` - Tests book deletion

4. **All Database Tests Pass:** All 29 database tests pass, including the new persistence test that verifies data survives database close/reopen cycle (simulating app lifecycle).

## Verification

**Verified (Passed):**
- ✅ All 29 database tests pass (`cargo test --lib db::`)
- ✅ `test_create_book_persists_to_file` - Book persists after database reopen (simulates app restart)
- ✅ APK built successfully (139MB, from T03)
- ✅ Verification script created and executable
- ✅ Database schema includes all required tables (books, book_pages, words, annotations, sticky_notes, processing_progress, vocabulary)

**Pending (Requires Physical Device):**
- ⏳ `adb devices` shows connected device
- ⏳ App launches on device without FATAL exceptions
- ⏳ SQLite database accessible via `adb shell sqlite3`
- ⏳ Book persists after force close + reopen on device
- ⏳ No JNI initialization errors in logcat

**Verification Commands (ready to run when device connected):**
```bash
# Run the verification script
bash scripts/verify-app-launch.sh

# Or run individual steps manually:
export ANDROID_HOME=/home/devuser/android-sdk
export PATH=$PATH:$ANDROID_HOME/platform-tools

# Check device
adb devices

# Install APK
adb install -r target/dx/shusei/debug/android/app/app/build/outputs/apk/debug/app-debug.apk

# Launch app
adb shell am start -n com.shusei.app/.MainActivity

# Monitor logs
adb logcat | grep -i shusei

# Insert test book
adb shell sqlite3 /data/data/com.shusei.app/databases/shusei.db \
  "INSERT INTO books (id, title, author, created_at, updated_at) VALUES ('test-1', 'Test Book', 'Test Author', 1234567890, 1234567890);"

# Force close and reopen
adb shell am force-stop com.shusei.app
adb shell am start -n com.shusei.app/.MainActivity

# Verify persistence
adb shell sqlite3 /data/data/com.shusei.app/databases/shusei.db "SELECT * FROM books;"
```

**Desktop Verification (Completed):**
```bash
# Run database persistence tests
cargo test --lib db::tests::books -- --nocapture

# Output:
# test core::db::tests::books::test_create_book_persists_to_file ... ok
# test core::db::tests::books::test_create_multiple_books ... ok
# ... (6 tests pass)
```

## Diagnostics

**How to inspect app launch and persistence later:**

```bash
# Check device connection
adb devices

# Check if app is installed
adb shell pm list packages | grep com.shusei.app

# View runtime logs
adb logcat | grep -i shusei

# Check for FATAL exceptions
adb logcat | grep -i "FATAL\|AndroidRuntime"

# Check for JNI errors
adb logcat | grep -i "jni\|UnsatisfiedLinkError"

# Access SQLite database
adb shell sqlite3 /data/data/com.shusei.app/databases/shusei.db

# Query books table
adb shell sqlite3 /data/data/com.shusei.app/databases/shusei.db "SELECT * FROM books;"

# Check database schema
adb shell sqlite3 /data/data/com.shusei.app/databases/shusei.db ".tables"

# Run full verification script
bash scripts/verify-app-launch.sh
```

**Desktop test verification:**
```bash
# Run all database tests
cargo test --lib db::

# Run only books tests
cargo test --lib db::tests::books
```

## Deviations

- **Device testing deferred:** Physical Android device not available in WSL2 environment. USB passthrough from Windows host required. Verification script created for when hardware is available.
- **Desktop persistence tests added:** Instead of only device testing, added file-based database tests that simulate app restart by closing and reopening the database file. This verifies the persistence logic works correctly.

## Known Issues

- **WSL2 USB passthrough:** Device installation and testing requires Windows host configuration for USB device forwarding to WSL2. This is outside the scope of the Linux environment.
- **APK size (139MB):** The native library is 525MB uncompressed. Expected for Dioxus WebView-based apps but may need optimization for release builds.
- **Pre-existing test failures:** 2 STT tests fail (`test_hann_window`, `test_kv_cache_new`) - unrelated to this task, pre-existing issues in the codebase.

## Files Created/Modified

- `scripts/verify-app-launch.sh` — Comprehensive app launch and persistence verification script (executable)
- `src/core/db.rs` — Added 6 books tests including file-based persistence test
- `.gsd/milestones/M002-dbrk2n/slices/S01/tasks/T04-DEVICE-GUIDE.md` — Device testing guide with troubleshooting steps
