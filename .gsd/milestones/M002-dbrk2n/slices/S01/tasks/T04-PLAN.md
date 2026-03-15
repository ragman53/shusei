---
estimated_steps: 6
estimated_files: 2
---

# T04: Verify app launch and SQLite persistence

**Slice:** S01 — Android Build + Deploy
**Milestone:** M002-dbrk2n

## Description

Launch app on device, verify no crashes, test SQLite persistence by creating book → close → reopen → book exists.

## Steps

1. Launch app: `adb shell am start -n com.shusei.app/.MainActivity`
2. Monitor logs: `adb logcat | grep -i shusei` — check for FATAL exceptions
3. Navigate to library screen via UI (or use test fixture)
4. Create test book: Insert directly via SQLite for speed:
   ```
   adb shell sqlite3 /data/data/com.shusei.app/databases/shusei.db \
     "INSERT INTO books (id, title, author, created_at, updated_at) VALUES ('test-1', 'Test Book', 'Test Author', 1234567890, 1234567890);"
   ```
5. Force close app: `adb shell am force-stop com.shusei.app`
6. Reopen app: `adb shell am start -n com.shusei.app/.MainActivity`
7. Verify book exists: Query database again or check UI
8. Check logcat for any errors during lifecycle

## Must-Haves

- [ ] App launches without FATAL exceptions
- [ ] Main UI renders (library screen visible)
- [ ] Test book persists after force close + reopen
- [ ] No JNI initialization errors in logcat
- [ ] SQLite database accessible on device

## Verification

- `adb logcat | grep -i "FATAL"` — no fatal errors from com.shusei.app
- `adb shell sqlite3 /data/data/com.shusei.app/databases/shusei.db "SELECT COUNT(*) FROM books;"` returns >= 1
- App survives background/restore cycle without data loss

## Observability Impact

- Signals added/changed: Logcat output with app lifecycle events, JNI init logs, database operations
- How a future agent inspects this: `adb logcat`, `adb shell sqlite3` database queries
- Failure state exposed: Crash logs, database corruption errors, JNI initialization failures

## Inputs

- Installed APK from T03
- M001 database schema (books, book_pages, words, annotations tables)

## Expected Output

- Verified app launch on device
- SQLite persistence confirmed
- Logcat logs showing clean lifecycle
