# S02: Camera Book Capture — UAT

**Milestone:** M002-dbrk2n
**Written:** 2026-03-16

## UAT Type

- **UAT mode:** live-runtime + artifact-driven
- **Why this mode is sufficient:** S02 integrates multiple subsystems (database, OCR engine, storage service, UI navigation) that require both automated integration tests (artifact-driven) and manual UI flow verification (live-runtime). Desktop testing validates logic; device testing deferred to S05.

## Preconditions

1. **Desktop environment:**
   - `cargo build` completes successfully
   - `shusei.db` database file exists or can be created
   - Desktop feature enabled: `cargo run --features desktop`

2. **Test data cleanup (if re-running):**
   ```bash
   # Remove existing test database to start fresh
   rm shusei.db
   
   # Or manually clear test data
   sqlite3 shusei.db "DELETE FROM book_pages; DELETE FROM books;"
   ```

3. **Application state:**
   - App launches without errors
   - Library screen displays (initially empty or with existing books)

## Smoke Test

**Quick sanity check (30 seconds):**

1. Launch app: `cargo run --features desktop`
2. Click "Add Book" button
3. Enter title: "Test Book", author: "Test Author"
4. Click "Save" button
5. **Expected:** Form submits, "Saving..." shows briefly, navigates to camera page with URL `/camera/{book_id}`
6. **Expected:** Camera page shows "Loading OCR engine..." banner initially
7. **Expected:** After engine loads, "Run OCR" button becomes enabled
8. Navigate back to library (use back button or navigation)
9. **Expected:** Book card shows "Test Book by Test Author" with "No pages yet" badge and "Capture Pages" button

## Test Cases

### 1. Book Creation with Database Persistence

**Goal:** Verify book creation saves to SQLite and navigates to camera with book_id

1. Launch app with clean database
2. Click "Add Book" button in library screen
3. Enter title: "UAT Test Book"
4. Enter author: "UAT Test Author"
5. Click "Save" button
6. Wait for "Saving..." indicator to complete
7. **Expected:** Navigation to camera page occurs
8. **Expected:** URL contains book_id (e.g., `/camera/1` or `/camera/{uuid}`)
9. Open terminal and run:
   ```bash
   sqlite3 shusei.db "SELECT id, title, author, pages_captured FROM books WHERE title='UAT Test Book';"
   ```
10. **Expected:** Query returns 1 row with correct title, author, and pages_captured=0
11. **Expected:** Console logs show: `INFO Book created: id={id}, title=UAT Test Book`

**Pass criteria:** Book exists in database with correct metadata, navigation includes book_id parameter.

### 2. Camera Page OCR Engine Initialization

**Goal:** Verify OCR engine loads on camera page mount

1. Navigate to camera page via book creation (from Test 1) OR click "Capture Pages" button on existing book card
2. **Expected:** Blue loading banner appears with "Loading OCR engine..." text and spinning hourglass icon
3. **Expected:** "Run OCR" button shows "Loading..." text and is disabled (grayed out)
4. **Expected:** Debug panel shows book_id (e.g., "Book ID: 1")
5. Wait 2-5 seconds for engine initialization
6. **Expected:** Loading banner disappears
7. **Expected:** "Run OCR" button becomes enabled and shows "Run OCR" text
8. **Expected:** Debug panel shows "Engine Ready: true"
9. **Expected:** Console logs show: `INFO OCR engine initialized, ready=true`

**Pass criteria:** Engine loads successfully, UI state transitions from loading → ready, button becomes interactive.

### 3. Page Number Input and Capture Flow

**Goal:** Verify page number input works and integrates with save flow

1. Navigate to camera page for a book
2. Wait for OCR engine to load
3. Enter page number "1" in the "Page Number" input field
4. **Expected:** Input accepts numeric value, min=1 enforced by browser
5. Click "Take Photo" button (simulates camera capture)
6. **Expected:** Image preview appears (or simulated image captured on desktop)
7. Click "Run OCR" button
8. **Expected:** Button shows "Processing..." and is disabled
9. Wait for OCR processing (1-3 seconds on desktop)
10. **Expected:** OCR result text displays in result area
11. **Expected:** Confidence score shows (e.g., "Confidence: 0.85")
12. **Expected:** Console logs show: `INFO OCR completed: {N} chars, confidence {X.XX}`

**Pass criteria:** Page number input functional, OCR processing completes, results displayed with confidence score.

### 4. Page Save with Database Persistence

**Goal:** Verify page save stores image and metadata correctly

1. Complete Test 3 (OCR result displayed)
2. Click "Save Page" button
3. **Expected:** Button shows "Saving..." and is disabled
4. Wait for save operation (1-2 seconds)
5. **Expected:** Success message appears with checkmark icon
6. **Expected:** Console logs show: `INFO Page saved: book_id={id}, page=1, path=pages/{book_id}/{timestamp}_{uuid}.jpg, db_id=1`
7. Navigate back to library screen
8. **Expected:** Book card now shows "1 pages" badge (not "No pages yet")
9. Run database inspection:
   ```bash
   sqlite3 shusei.db "SELECT id, book_id, page_number, ocr_text_plain, confidence FROM book_pages WHERE book_id={book_id};"
   sqlite3 shusei.db "SELECT id, title, pages_captured FROM books WHERE id={book_id};"
   ```
10. **Expected:** book_pages table has 1 row with page_number=1, non-empty ocr_text_plain, confidence > 0
11. **Expected:** books table shows pages_captured=1
12. Verify image file exists:
    ```bash
    ls -la pages/{book_id}/
    ```
13. **Expected:** Directory contains 1 JPEG file with pattern `{timestamp}_{uuid}.jpg`

**Pass criteria:** Page saved to database with OCR text, confidence, image path; books.pages_captured updated; image file exists in storage.

### 5. Multiple Page Capture and Ordering

**Goal:** Verify multiple pages can be captured and are ordered correctly

1. Complete Test 4 (1 page saved)
2. Click "Capture Pages" button on the same book card
3. Enter page number "2" in page number input
4. Capture image → Run OCR → Save Page (repeat from Test 3-4)
5. **Expected:** Success message appears
6. Navigate back to library
7. **Expected:** Book card shows "2 pages" badge
8. Run database inspection:
   ```bash
   sqlite3 shusei.db "SELECT page_number, ocr_text_plain, confidence FROM book_pages WHERE book_id={book_id} ORDER BY page_number;"
   ```
9. **Expected:** 2 rows returned, ordered page_number=1, page_number=2
10. Capture a third page with page number "3"
11. **Expected:** Book card shows "3 pages" badge
12. **Expected:** Database contains 3 rows with correct ordering

**Pass criteria:** Multiple pages save correctly, pages_captured count increments, ordering preserved in database.

### 6. Duplicate Page Number Handling

**Goal:** Verify UNIQUE constraint prevents duplicate page numbers

1. Navigate to camera page for a book that already has page 1 saved
2. Enter page number "1" (duplicate)
3. Capture image → Run OCR → Save Page
4. **Expected:** Red error banner appears with message: "Page 1 already exists for this book. Please use a different page number or overwrite."
5. **Expected:** Button re-enables, allowing retry
6. **Expected:** Console logs show: `ERROR Database save failed: UNIQUE constraint failed`
7. Change page number to "4"
8. Click "Save Page" again
9. **Expected:** Save succeeds (no error banner)
10. **Expected:** Database now contains pages 1, 2, 3, 4 (no duplicates)

**Pass criteria:** Duplicate page numbers rejected with user-friendly error, user can retry with different page number.

### 7. Library Navigation Flow

**Goal:** Verify end-to-end navigation from library → camera → save → library

1. Start at library screen with existing book
2. Click "Capture Pages" button on book card
3. **Expected:** Navigation to `/camera/{book_id}` occurs
4. **Expected:** Console logs show: `DEBUG Navigating to camera for book_id={book_id}`
5. Capture and save a page (Tests 3-4)
6. Navigate back to library (browser back button or app navigation)
7. **Expected:** Book card shows updated page count
8. **Expected:** Console logs show: `DEBUG Book card rendered: {N} pages`
9. Click "Capture Pages" again
10. **Expected:** Same book_id in URL, camera page loads with existing book context

**Pass criteria:** Navigation flow works bidirectionally, book_id preserved throughout flow, page count updates reflect in library.

### 8. Error Handling and Recovery

**Goal:** Verify error states are handled gracefully

1. **OCR Engine Failure (simulated):**
   - Navigate to camera page
   - **Expected:** If engine fails to load, red error banner shows "OCR engine failed to load: {error}"
   - **Expected:** "Run OCR" button remains disabled

2. **Database Error (simulated by corrupting db_path):**
   - Modify code temporarily to use invalid path
   - Attempt to save page
   - **Expected:** Red error banner shows "Database error: {details}"
   - **Expected:** Button re-enables for retry

3. **Validation Error:**
   - Navigate to camera page
   - Try to save without capturing image (if possible)
   - **Expected:** Appropriate error message shown

**Pass criteria:** All error states display user-friendly messages, buttons re-enable after errors, app doesn't crash.

## Edge Cases

### Empty Book List State

1. Start with empty database (no books)
2. **Expected:** Library screen shows empty state message (e.g., "No books yet. Add your first book!")
3. **Expected:** "Add Book" button visible and functional
4. **Expected:** No "Capture Pages" buttons visible (no books to capture)

### Page Number Zero or Negative

1. Navigate to camera page
2. Try to enter page number "0" or "-1"
3. **Expected:** Input field enforces min="1", browser prevents invalid entry
4. **Expected:** If somehow submitted, validation rejects with error

### Very Long OCR Text

1. Capture page with dense text (full page of text)
2. Run OCR
3. **Expected:** OCR result displays (may be truncated in UI for performance)
4. **Expected:** Save succeeds, full text stored in database
5. **Expected:** No UI freezing or performance degradation

### Rapid Repeated Saves

1. Navigate to camera page
2. Capture image → Run OCR → Save Page
3. Immediately click "Save Page" again before first save completes
4. **Expected:** Button remains disabled during save (is_saving prevents double-submit)
5. **Expected:** Only one page saved to database

## Failure Signals

- **Red error banner visible** — Indicates OCR, storage, or database failure
- **"Run OCR" button stays disabled** — OCR engine failed to initialize
- **Navigation to `/camera` without book_id** — Book creation didn't save or didn't pass book_id
- **Page count doesn't update in library** — Database save failed or pages_captured not updated
- **Console errors** — Check `cargo run` output for `ERROR` or `log::error!` messages
- **Database query returns 0 rows** — Save operation failed silently
- **Missing image files in `pages/{book_id}/`** — Storage service failed to save JPEG

## Requirements Proved By This UAT

- **R001 (Camera book capture with page linkage)** — Tests 1-7 prove complete flow: book creation → camera navigation → page capture → OCR → save with book linkage → library page count update.

- **R005 (SQLite data persists across restarts)** — Tests 1, 4, 5 verify database persistence via direct SQLite queries. File-based tests in test suite simulate app restart scenarios.

## Not Proven By This UAT

- **R004 (APK deploys on Moto G66j 5G)** — Desktop testing only; real device JNI camera stability not verified. Deferred to S05 device testing.

- **R006 (Model bundling)** — OCR model loads on desktop from local filesystem; APK asset bundling not verified. Deferred to S05.

- **Performance on mid-range hardware** — OCR latency measured on desktop (1-3s); device performance unknown. Deferred to S05.

- **Camera JNI integration** — Desktop uses simulated image capture; actual Android camera JNI not tested. Deferred to S05.

## Notes for Tester

- **Desktop vs Device:** This UAT runs on desktop with `--features desktop`. Camera capture is simulated; actual device testing requires Moto G66j 5G and will be done in S05.

- **OCR Engine Load Time:** Expect 2-5 seconds for engine initialization on desktop. Device may be slower (target: <5s per M002 success criteria).

- **Database Location:** Desktop uses `shusei.db` in current directory. Android uses `/data/data/com.shusei.app/files/shusei.db`.

- **Storage Path:** Desktop saves to `pages/{book_id}/` relative to cwd. Android saves to app data directory.

- **Known Quirk:** Debug panel shows book_id for troubleshooting — this is intentional for S02, may be hidden in production.

- **Test Data Cleanup:** Use `rm shusei.db` to reset between test runs, or use unique book titles to avoid conflicts.

- **Console Logs:** Run `cargo run 2>&1 | grep -i shusei` to see structured logs during testing.

- **If Tests Fail:** Check `cargo test --test camera_ocr_integration` first — if integration tests pass but UAT fails, issue is likely UI state or navigation, not core logic.
