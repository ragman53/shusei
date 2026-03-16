---
id: T03
parent: S02
milestone: M002-dbrk2n
provides:
  - OCR + Save integration with actual NdlocrEngine::process_image() calls
  - Database persistence via Database::save_page() with UNIQUE constraint handling
  - Storage service integration via StorageService::save_page_image()
  - Loading states for OCR and save operations
  - End-to-end integration tests
key_files:
  - src/ui/camera.rs
  - tests/camera_ocr_integration.rs
  - src/core/db.rs
key_decisions:
  - Used separate loading states (is_processing_ocr, is_saving_page) for better UX feedback
  - Parse OCR confidence from display format rather than storing raw OcrResult in signal
  - Handle UNIQUE constraint violations with user-friendly error message suggesting page number conflict
  - Update pages_captured count in books table after successful page save
patterns_established:
  - Async spawn with error handling for OCR and save operations
  - StorageService + Database integration pattern for persistence
  - User-friendly error messages for all failure modes (storage, database, OCR engine)
observability_surfaces:
  - log::info!("OCR completed: {} chars, confidence {:.2}", result.plain_text.len(), result.confidence)
  - log::info!("Page saved: book_id={}, page={}, path={}, db_id={}", book_id, page_num, new_page.image_path, page_id)
  - log::error!("OCR processing failed: {}", e)
  - log::error!("Storage save failed: {}", e)
  - log::error!("Database save failed: {}", e)
  - Red error banner in UI with specific error messages
duration: 2h
verification_result: passed
completed_at: 2026-03-16
# Set blocker_discovered: true only if execution revealed the remaining slice plan
# is fundamentally invalid (wrong API, missing capability, architectural mismatch).
# Do NOT set true for ordinary bugs, minor deviations, or fixable issues.
blocker_discovered: false
---

# T03: OCR + Save Integration

**Wired camera capture to actual OCR engine and database save with full error handling and integration tests.**

## What Happened

Implemented the core OCR + Save integration flow in CameraPage:

1. **Modified "Run OCR" button handler:**
   - Added engine readiness check before processing
   - Calls actual `engine.process_image(&image_bytes)` asynchronously
   - Sets ocr_result signal with extracted text and confidence score
   - Handles errors with user-friendly error messages
   - Uses `is_processing_ocr` signal for loading state

2. **Modified "Save Page" button handler:**
   - Gets app data directory for storage service
   - Creates `StorageService::new(assets_dir)` and saves JPEG image
   - Opens database with `Database::open(db_path)`
   - Creates `NewBookPage` struct with book_id, page_number, image_path, ocr_markdown, ocr_text_plain, confidence
   - Calls `db.save_page(&new_page)` with UNIQUE constraint handling
   - Updates `pages_captured` count in books table after successful save
   - Shows success message with checkmark on completion

3. **Added loading states:**
   - `is_processing_ocr: Signal<bool>` during OCR inference
   - `is_saving_page: Signal<bool>` during database save
   - Buttons disabled appropriately during each state
   - Loading indicators show "Processing..." and "Saving..."

4. **Created integration test `tests/camera_ocr_integration.rs`:**
   - `test_camera_ocr_integration_end_to_end` — Full flow: create book → save image → save page → verify persistence
   - `test_camera_ocr_multiple_pages` — Save 3 pages and verify ordering
   - `test_camera_ocr_duplicate_page_number` — Verify UNIQUE constraint enforcement
   - `test_storage_service_page_image_organization` — Verify directory structure: `pages/{book_id}/{timestamp}_{uuid}.jpg`

5. **Added database test `test_create_book_and_save_page` in src/core/db.rs:**
   - Verifies complete flow: create book → save page → verify linkage → update pages_captured

6. **Error handling implemented:**
   - OCR engine not ready → "OCR engine not ready"
   - Storage initialization failed → "Storage error: <details>"
   - Image save failed → "Failed to save image: <details>"
   - Database open failed → "Database error: <details>"
   - UNIQUE constraint violation → "Page X already exists for this book. Please use a different page number or overwrite."
   - Generic database save failed → "Failed to save to database: <details>"

## Verification

**All tests passed:**

```bash
# Database flow test
cargo test --lib db::tests::book_pages::test_create_book_and_save_page
# Result: ok. 1 passed; 0 failed

# Integration tests (4 tests)
cargo test --test camera_ocr_integration
# Result: ok. 4 passed; 0 failed
# - test_storage_service_page_image_organization ... ok
# - test_camera_ocr_duplicate_page_number ... ok
# - test_camera_ocr_integration_end_to_end ... ok
# - test_camera_ocr_multiple_pages ... ok

# Build verification
cargo check
# Result: Finished dev profile [unoptimized + debuginfo] target(s) in 1.31s
```

**Code inspection:**
- No TODO comments remain in OCR/save handlers
- All async operations wrapped in `spawn(async move { ... })`
- Error messages displayed in red banner at top of form
- Buttons re-enabled after errors, allowing user retry

## Diagnostics

**How to inspect what this task built:**

1. **Check OCR processing logs:**
   ```bash
   # Desktop
   cargo run 2>&1 | grep -i "OCR completed"
   # Expected: INFO OCR completed: 123 chars, confidence 0.85
   
   # Device
   adb logcat | grep -i shusei | grep -i OCR
   ```

2. **Check page save logs:**
   ```bash
   cargo run 2>&1 | grep -i "Page saved"
   # Expected: INFO Page saved: book_id=test-123, page=1, path=pages/test-123/1234567890_abc.jpg, db_id=1
   ```

3. **Verify database persistence:**
   ```bash
   sqlite3 shusei.db "SELECT book_id, page_number, ocr_text_plain, confidence FROM book_pages;"
   # Should show saved pages with OCR text and confidence scores
   
   sqlite3 shusei.db "SELECT id, title, pages_captured FROM books;"
   # Should show updated pages_captured count
   ```

4. **Check storage directory structure:**
   ```bash
   ls -la pages/{book_id}/
   # Should show: {timestamp}_{uuid}.jpg files
   ```

5. **UI state verification:**
   - Navigate to `/camera/test-book-123`
   - Capture image → "Run OCR" button enabled after engine loads
   - Click "Run OCR" → button shows "Processing..." → OCR result displays with confidence
   - Click "Save Page" → button shows "Saving..." → success message with checkmark
   - Navigate back to book list → book shows "1 pages captured"

**Failure states exposed:**
- Red error banner with specific error message for each failure mode
- Button re-enabled after failure (user can retry)
- Loading indicator cleared on error
- UNIQUE constraint violation shows user-friendly message about page number conflict

## Deviations

None. Implementation followed the task plan exactly.

## Known Issues

None discovered during implementation.

## Files Created/Modified

- `src/ui/camera.rs` — Wired OCR + save flow with actual engine.process_image() and db.save_page() calls, added loading states, error handling
- `tests/camera_ocr_integration.rs` — New integration test file with 4 tests covering end-to-end flow, multiple pages, duplicate detection, and storage organization
- `src/core/db.rs` — Added test_create_book_and_save_page test to verify book + page save flow
