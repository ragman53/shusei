---
id: S02
parent: M002-dbrk2n
milestone: M002-dbrk2n
title: Camera Book Capture
goal: User creates a book (title/author) → captures pages via camera → OCR runs → pages saved with book linkage and page number
demo: Create book → navigate to camera → capture 2 pages → run OCR → save pages → verify in database
proof_level: integration
requires:
  - S01: Android Build + Deploy (APK, SQLite, JNI camera API)
provides:
  - Book creation flow with database persistence
  - Camera page with book_id parameter and page number input
  - OCR integration with NDLOCR engine
  - Page save flow with StorageService + Database
affects:
  - S05: Model Bundling + Integration (consumes camera → OCR flow)
key_files:
  - src/app.rs
  - src/ui/camera.rs
  - src/ui/add_book.rs
  - src/ui/library.rs
  - src/core/ocr/engine_tract.rs
  - src/core/db.rs
  - src/core/storage.rs
  - tests/camera_ocr_integration.rs
duration: 8h
verification_result: pending
completed_at: null
---

# S02: Camera Book Capture — Plan

**Goal:** User creates a book (title/author) → captures pages via camera → OCR runs → pages saved with book linkage and page number

**Demo:** Create book → navigate to camera → capture 2 pages → run OCR → save pages → verify in database

## Must-Haves

- Book creation actually saves to database via `Database::create_book()`
- Camera page accepts `book_id` route parameter and displays it
- Page number input field added to camera UI
- "Run OCR" button calls actual `NdlocrEngine::process_image()` (not placeholder)
- "Save Page" button saves to `book_pages` table via `Database::save_page()` + `StorageService::save_page_image()`
- Navigation flow: Book List → "Capture Pages" button → Camera with `book_id`
- Error handling for OCR failures, database errors, camera permission denial
- Loading indicators for OCR processing

## Proof Level

- This slice proves: **integration** — wires together JNI camera, OCR engine, database, and storage service
- Real runtime required: **yes** — camera capture, OCR inference, SQLite persistence all must work on real Android runtime
- Human/UAT required: **yes** — manual device testing required for JNI camera stability verification

## Verification

**Unit Tests (Desktop):**
- `cargo test --lib db::tests::test_create_book_and_save_page` — Verifies book + page save flow
- `cargo test --lib ocr::engine_tract::tests` — Verifies OCR engine (92 existing tests)

**Integration Test (New File):**
- `tests/camera_ocr_integration.rs` — End-to-end test: create book → simulate capture → run OCR → save page → verify persistence

**Device Verification Script:**
- `bash scripts/verify-s02-camera.sh` — Automated device test that:
  1. Installs APK on Moto G66j 5G
  2. Creates test book via SQLite
  3. Monitors logcat for camera capture events
  4. Verifies page saved to database after capture + OCR

**Manual UAT Checklist:**
- [ ] Create book with title "Test Book" and author "Test Author"
- [ ] Navigate to book list → tap "Capture Pages" → camera opens
- [ ] Enter page number "1" → tap "Take Photo" → camera opens
- [ ] Capture image → preview shows → tap "Run OCR" → processing indicator shows
- [ ] OCR result displays → tap "Save Page" → success message
- [ ] Navigate back to book list → book shows "1 pages captured"
- [ ] Repeat for page 2 → verify "2 pages captured"
- [ ] Force close app → reopen → verify pages persist

## Observability / Diagnostics

- **Runtime signals:**
  - `log::info!("OCR completed: {} chars, confidence {}", result.text.len(), result.confidence)` — OCR success
  - `log::error!("Camera capture failed: {}", e)` — JNI camera failure
  - `log::info!("Page saved: book_id={}, page_number={}", book_id, page_number)` — Database save success
- **Inspection surfaces:**
  - `adb logcat | grep -i shusei` — Real-time app logs on device
  - `adb shell "sqlite3 /data/data/com.shusei.app/files/shusei.db 'SELECT * FROM book_pages;'"` — Direct database inspection
  - `browser_get_console_logs` — Desktop debugging (if running on desktop)
- **Failure visibility:**
  - Error message displayed in camera UI red banner
  - `error_message` signal populated with user-friendly error text
  - Logcat shows full stack trace for JNI crashes
- **Redaction constraints:** None — no PII or secrets in camera/OCR flow

## Integration Closure

- **Upstream surfaces consumed:**
  - `AndroidPlatform::capture_image()` — JNI camera capture from S01
  - `Database::create_book()`, `Database::save_page()` — SQLite persistence from S01
  - `StorageService::save_page_image()` — Image storage from S01
  - `NdlocrEngine::process_image()` — OCR engine from M001
- **New wiring introduced in this slice:**
  - Route parameter: `/camera/:book_id` in `src/app.rs`
  - `CameraPage` component accepts `book_id: String` prop
  - OCR engine initialization on camera page mount
  - Camera capture → OCR → save pipeline
  - Book list → camera navigation with "Capture Pages" button
- **What remains before the milestone is truly usable end-to-end:**
  - S03: PDF reflow reader with progress tracking
  - S04: Word collection from PDF/OCR text
  - S05: Model bundling verification on device

## Tasks

- [x] **T01: Book Creation Flow** `est:2h`
  - Why: Camera page needs a `book_id` to save pages, but `AddBookForm` currently navigates without saving to database
  - Files: `src/ui/add_book.rs`, `src/core/db.rs`, `src/app.rs`
  - Do: 
    1. Add `Database` dependency to `AddBookForm` component
    2. Call `Database::create_book()` on form submit with title/author
    3. Handle errors (show user-friendly message on failure)
    4. Navigate to `/camera/:book_id` on success (not `/camera` without ID)
  - Verify: `cargo test --lib db::tests::test_create_book_and_save_page` passes; manual test: create book → verify in database
  - Done when: Book creation saves to SQLite and navigates to camera with `book_id` parameter

- [x] **T02: Camera Page Enhancement** `est:2h`
  - Why: Camera page needs to accept `book_id`, add page number input, and prepare for OCR integration
  - Files: `src/ui/camera.rs`, `src/app.rs`
  - Do:
    1. Add route parameter `/camera/:book_id` in `src/app.rs`
    2. Modify `CameraPage` component to accept `book_id: String` prop
    3. Add `page_number` state signal (default: 1)
    4. Add page number input field to UI (numeric input, min=1)
    5. Display `book_id` in UI for debugging (can be hidden later)
    6. Initialize `NdlocrEngine` on component mount (async, check `is_ready()`)
  - Verify: Camera page compiles; route `/camera/test-book-id` works; page number input visible
  - Done when: Camera page accepts `book_id`, has page number input, OCR engine initialized on mount

- [x] **T03: OCR + Save Integration** `est:3h`
  - Why: Core integration risk — camera capture must wire to OCR engine and database save
  - Files: `src/ui/camera.rs`, `src/core/storage.rs`, `src/core/db.rs`, `tests/camera_ocr_integration.rs`
  - Do:
    1. Replace OCR TODO placeholder with actual `engine.process_image(&image_bytes)` call
    2. Run OCR in `spawn(async move { ... })` to avoid blocking UI
    3. Display OCR result (text + confidence) in UI
    4. Implement "Save Page" button:
       - Call `StorageService::save_page_image()` to save JPEG
       - Call `Database::save_page()` with OCR results, book_id, page_number
       - Show success/error message to user
    5. Handle UNIQUE constraint conflicts (page number already exists)
    6. Write integration test `tests/camera_ocr_integration.rs`
  - Verify: `cargo test --test camera_ocr_integration` passes; manual test: capture → OCR → save → verify in database
  - Done when: Camera capture → OCR → save flow works end-to-end with database persistence

- [x] **T04: Library Navigation** `est:1h`
  - Why: Users need a way to navigate from book list to camera capture
  - Files: `src/ui/library.rs`, `src/app.rs`
  - Do:
    1. Add "Capture Pages" button to `LibraryScreen` book card
    2. Button navigates to `/camera/:book_id` on click
    3. Show page count badge on book card (e.g., "3 pages")
    4. Handle empty book list state gracefully
  - Verify: Book list shows "Capture Pages" button; clicking navigates to camera with correct `book_id`
  - Done when: User can navigate from book list → camera → capture → save → return to book list with updated page count

## Files Likely Touched

- `src/app.rs` — Route parameter for camera
- `src/ui/camera.rs` — Camera page enhancement, OCR integration
- `src/ui/add_book.rs` — Book creation flow
- `src/ui/library.rs` — Navigation button
- `src/core/db.rs` — Database tests
- `tests/camera_ocr_integration.rs` — New integration test file
- `scripts/verify-s02-camera.sh` — New device verification script
