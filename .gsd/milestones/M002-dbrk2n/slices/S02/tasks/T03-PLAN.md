# T03: OCR + Save Integration

**Slice:** S02 — Camera Book Capture
**Milestone:** M002-dbrk2n

## Description

Wire camera capture to OCR engine and database save. This is the core integration task that replaces TODO placeholders with actual `NdlocrEngine::process_image()` calls and `Database::save_page()` persistence.

## Steps

1. Modify "Run OCR" button handler in `CameraPage`:
   - Check `is_engine_ready` — if false, show error "OCR engine not ready"
   - Get captured image bytes from `captured_image` signal
   - Call `engine.process_image(&image_bytes)` in `spawn(async move { ... })`
   - Handle result: set `ocr_result` signal with extracted text
   - Handle error: set `error_message` signal
   - Update UI to show OCR result with confidence score

2. Modify "Save Page" button handler:
   - Get app data directory for storage service
   - Create `StorageService::new(assets_dir)`
   - Call `storage.save_page_image(&image_bytes, &book_id)` to save JPEG (returns relative path like `pages/{book_id}/{timestamp}_{uuid}.jpg`)
   - Get relative path from result
   - Create `Database::open(db_path)`
   - Create `NewBookPage` struct with: `book_id`, `page_number`, `image_path` (relative path), `ocr_markdown`, `ocr_text_plain`, `confidence`
   - Call `db.save_page(&new_page)`
   - Handle UNIQUE constraint conflict (page already exists):
     - Option A: Show error "Page X already exists, overwrite?"
     - Option B: Auto-increment page number suggestion
     - Option C: Use INSERT OR REPLACE (implement in db.rs)
   - Show success message on completion

3. Add loading states:
   - `is_processing_ocr: Signal<bool>` during OCR inference
   - `is_saving_page: Signal<bool>` during database save
   - Disable buttons appropriately during each state

4. Write integration test `tests/camera_ocr_integration.rs`:
   - Create in-memory database
   - Create test book
   - Simulate camera capture (use test image bytes)
   - Run OCR (use test model or mock)
   - Save page
   - Verify page exists in database with correct book_id linkage

5. Add error handling:
   - OCR engine not ready
   - Image capture failed
   - OCR processing failed
   - Database save failed
   - Storage write failed

## Must-Haves

- [ ] "Run OCR" button calls actual `engine.process_image()` (not placeholder)
- [ ] OCR runs asynchronously (doesn't block UI)
- [ ] OCR result displayed with confidence score
- [ ] "Save Page" saves image via `StorageService::save_page_image()`
- [ ] "Save Page" saves metadata via `Database::save_page()`
- [ ] Page number conflict handled (UNIQUE constraint)
- [ ] Loading indicators for OCR and save operations
- [ ] Error messages for all failure modes

## Verification

- `cargo test --test camera_ocr_integration` — Integration test passes
- `cargo test --lib db::tests::test_create_book_and_save_page` — Database flow test passes
- Manual desktop test: Capture → OCR → Save → verify in database file
- Code inspection: No TODO comments remain in OCR/save handlers

## Observability Impact

- **Signals added/changed:**
  - `log::info!("OCR completed: {} chars, confidence {:.2}", result.text.len(), result.confidence)` — OCR success
  - `log::error!("OCR processing failed: {}", e)` — OCR failure with error details
  - `log::info!("Page saved: book_id={}, page={}, path={}", book_id, page_number, image_path)` — Save success
  - `log::error!("Page save failed: {}", e)` — Database/storage failure
- **How a future agent inspects this:**
  - `adb logcat | grep -i shusei` — Real-time logs on device
  - `sqlite3 shusei.db "SELECT book_id, page_number, ocr_text_plain FROM book_pages;"` — Verify saved pages
  - Check `ocr_result` and `error_message` signals in component state
- **Failure state exposed:**
  - Red error banner with specific error message
  - Button re-enabled after failure (user can retry)
  - Loading indicator cleared on error

## Inputs

- T01: Database integration pattern for error handling
- T02: OCR engine initialized and ready on camera page
- `src/core/ocr/engine_tract.rs` — `process_image()` method signature
- `src/core/storage.rs` — `save_page_image()` method
- `src/core/db.rs` — `save_page()` method with UNIQUE constraint handling

## Expected Output

- `src/ui/camera.rs` — Fully wired OCR + save flow
- `tests/camera_ocr_integration.rs` — New integration test file
- `src/core/db.rs` — New test `test_create_book_and_save_page`
- Working end-to-end flow: capture → OCR → save with database persistence
