# S02: Camera Book Capture — Research

**Date:** 2026-03-16

## Summary

S02 implements the core camera book capture flow: user creates a book (title/author) → captures pages via camera → OCR extracts text → pages saved with book linkage and page number. The slice builds on S01's working APK build infrastructure and JNI camera API.

**Key findings:**

1. **Camera JNI infrastructure exists** — `src/platform/android.rs` has complete JNI camera capture with `capture_image()`, permission handling, and callback wiring (`onImageCaptured`). The Android `MainActivity` Java code must implement `startCameraCapture()` native method call.

2. **OCR engine is ready** — `NdlocrEngineTract` in `src/core/ocr/engine_tract.rs` is fully implemented with tract-onnx backend. Models exist in `assets/models/ndlocr/` (deim-s-1024x1024.onnx for detection, parseq-ndl-16x384-50-tiny-146epoch-tegaki2.onnx for recognition, 77MB total). Engine loads models from `assets/models/ndlocr/` directory.

3. **Camera UI scaffold exists but incomplete** — `src/ui/camera.rs` has a working camera capture UI with image preview and "Run OCR" button, but:
   - No book linkage (doesn't know which book to save pages to)
   - OCR integration is a TODO placeholder
   - No page number input
   - "Save as Note" button doesn't save to database

4. **Database schema supports book pages** — `book_pages` table exists with `book_id`, `page_number`, `image_path`, `ocr_markdown`, `ocr_text_plain`, `confidence` columns. `Database::save_page()` method is implemented and tested.

5. **Storage service ready** — `StorageService::save_page_image()` creates book-organized directory structure (`pages/{book_id}/{timestamp}_{uuid}.jpg`) and returns relative path for database storage.

**Primary recommendation:** Wire together existing components:
1. Modify `CameraPage` to accept `book_id` parameter from book creation flow
2. Add page number input field to camera UI
3. Replace OCR TODO with actual `NdlocrEngine::process_image()` call
4. Implement "Save Page" to call `Database::save_page()` with OCR results
5. Add book creation flow that navigates to camera with `book_id`

## Recommendation

**Implement S02 by integrating existing components rather than building new infrastructure:**

1. **Book Creation Flow** — Modify `AddBookForm` to actually create books via `Database::create_book()` and navigate to camera with `book_id` parameter
2. **Camera Page Enhancement** — Add `book_id` and `page_number` state, wire OCR engine, implement save to database
3. **OCR Integration** — Initialize `NdlocrEngine` on camera page mount, call `process_image()` after capture
4. **Database Wiring** — Use `Database::save_page()` with `StorageService::save_page_image()` for complete persistence

**Why this approach:**
- All core infrastructure exists (JNI camera, OCR engine, database, storage)
- Minimal new code required — mostly wiring and UI enhancements
- Follows established patterns from S01 (database tests, storage service)
- Reduces risk by reusing tested components

## Don't Hand-Roll

| Problem | Existing Solution | Why Use It |
|---------|------------------|------------|
| Camera capture on Android | `AndroidPlatform::capture_image()` with JNI callbacks | Already implemented with permission handling, timeout, error handling |
| OCR processing | `NdlocrEngineTract::process_image()` with tract-onnx | 92 passing tests, models bundled, works on desktop |
| Image storage | `StorageService::save_page_image()` | Creates organized directory structure, returns relative path |
| Page persistence | `Database::save_page()` | Tested CRUD operation, handles UNIQUE(book_id, page_number) constraint |
| Book creation | `Database::create_book()` | Generates UUID-based IDs, handles timestamps |

## Existing Code and Patterns

- `src/platform/android.rs` — JNI camera capture with `startCameraCapture()` native call, `onImageCaptured()` callback, permission request flow. **Pattern to follow:** Use async/await with oneshot channel for callback coordination.
- `src/core/ocr/engine_tract.rs` — NDLOCR engine with `initialize()`, `process_image()`. **Pattern to follow:** Initialize engine once, reuse for multiple inferences.
- `src/core/db.rs` — `save_page()`, `create_book()`, `get_pages_by_book()`. **Pattern to follow:** All DB operations return `Result<T>`, use transactions for multi-step operations.
- `src/core/storage.rs` — `save_page_image()` creates `pages/{book_id}/` directories. **Pattern to follow:** Always store relative paths in database, not absolute.
- `src/ui/camera.rs` — Camera UI scaffold with capture button, image preview, OCR button. **Pattern to extend:** Add book_id param, page number input, wire actual OCR.
- `src/ui/add_book.rs` — Book creation form (currently navigates without saving). **Pattern to fix:** Call `Database::create_book()` before navigation.
- `src/ui/reader.rs` — Example of OCR engine initialization (`NdlocrEngine::new(&app_data_dir, "en")`). **Pattern to follow:** Initialize with app data directory.

## Constraints

- **Dioxus 0.7.3 UI framework** — All UI must use Dioxus component model with `use_signal`, `spawn`, `rsx!` macros. No direct Android View manipulation.
- **JNI thread attachment** — Camera callbacks arrive on Android UI thread; Rust code must handle thread attachment via `JavaVM::attach_current_thread()`.
- **Model bundling** — OCR models bundled via `Dioxus.toml` `resources = ["assets/models/*"]`. Models accessed at runtime from APK assets or copied to files directory.
- **Async runtime** — Tokio runtime available; use `spawn(async move { ... })` for async operations in Dioxus components.
- **Database file location** — On Android, database stored in app data directory (`/data/data/com.shusei.app/files/shusei.db`). Use relative path `"shusei.db"`.
- **Image preprocessing** — Camera returns JPEG bytes; OCR engine expects RGB image, internally resizes to 1024x1024 for detection, 32px height for recognition.

## Common Pitfalls

- **Forgetting to initialize OCR engine** — `NdlocrEngine::new()` only creates the struct; must call `initialize()` (async) to load models. **Avoid:** Initialize engine on component mount, check `is_ready()` before processing.
- **Blocking UI thread with OCR** — OCR inference takes 100ms-2s depending on image size. **Avoid:** Always run `process_image()` in `spawn(async move { ... })` block.
- **Not handling camera permission denial** — User may deny camera permission. **Avoid:** Check `has_camera_permission()` before capture, show user-friendly error if denied.
- **Losing book_id across navigation** — Camera page needs to know which book to save pages to. **Avoid:** Pass `book_id` via route parameter (`/camera/:book_id`) or store in app state.
- **Page number conflicts** — `book_pages` table has `UNIQUE(book_id, page_number)` constraint. **Avoid:** Check for existing page before save, or use `INSERT OR REPLACE`.
- **Memory pressure on mid-range device** — Moto G66j 5G has moderate RAM; large images + OCR may stress memory. **Avoid:** Downscale camera images before OCR (engine does this internally, but be aware of cumulative memory usage).
- **JavaVM not initialized** — JNI callbacks require `JAVA_VM` static to be set via `nativeInit()` or `JNI_OnLoad`. **Avoid:** Verify `nativeInit` is called from Android `MainActivity.onCreate()`.

## Open Risks

- **JNI camera stability on device** — Camera capture + OCR pipeline untested on physical Moto G66j 5G. May encounter memory pressure, camera permission issues, or callback timing problems. **Mitigation:** Test with `adb logcat | grep -i shusei` immediately on device; add timeout handling.
- **OCR model loading time** — Models (77MB) must load from APK assets on first inference. May cause 2-5s delay on first OCR. **Mitigation:** Pre-load OCR engine on app startup or show loading indicator.
- **Camera image format compatibility** — Android camera may return different image formats (JPEG, YUV, etc.). Current code assumes JPEG. **Mitigation:** Test on device; add format detection if needed.
- **Book creation flow not implemented** — `AddBookForm` currently navigates without saving. **Mitigation:** Implement database save before navigation; handle errors gracefully.
- **No book list → camera navigation** — No UI flow from book list to camera capture. **Mitigation:** Add "Capture Pages" button to `LibraryScreen` that navigates to camera with `book_id`.

## Skills Discovered

| Technology | Skill | Status |
|------------|-------|--------|
| Android | wshobson/agents@mobile-android-design | Available (5.2K installs) |
| Dioxus | nevaberry/nevaberry-plugins@dioxus-knowledge-patch | Available (27 installs) |
| Dioxus | daiki48/dotfiles@dioxus-guide | Available (13 installs) |
| Dioxus | sirius-cc-wu/sirius-skills@dioxus-ui-ux | Available (8 installs) |
| Rust | wshobson/agents@rust-async-patterns | Available (4.2K installs) |
| Rust | apollographql/skills@rust-best-practices | Available (2.5K installs) |

**Recommended installs for S02:**
- `wshobson/agents@rust-async-patterns` — Async/await patterns in Rust (camera callbacks, OCR inference)
- `nevaberry/nevaberry-plugins@dioxus-knowledge-patch` — Dioxus 0.7 component patterns

## Sources

- **NDLOCR-Lite OCR engine** — tract-onnx implementation with detection + recognition models (source: `src/core/ocr/engine_tract.rs`)
- **Dioxus Android camera JNI** — Platform abstraction with async callback pattern (source: `src/platform/android.rs`)
- **SQLite book_pages table** — Schema with UNIQUE(book_id, page_number) constraint (source: `src/core/db.rs`)
- **StorageService page image handling** — Directory structure `pages/{book_id}/{timestamp}_{uuid}.jpg` (source: `src/core/storage.rs`)
- **S01 build infrastructure** — Gradle patch script, APK build, model bundling (source: `.gsd/milestones/M002-dbrk2n/slices/S01/S01-SUMMARY.md`)
- **Dioxus 0.7 documentation** — Component state management with `use_signal`, async spawning (source: `resolve_library` query: "dioxus 0.7 use_signal spawn")

---

## Implementation Checklist (for reference)

**Not part of research output — planning aid for S02 execution:**

- [ ] Add route parameter: `/camera/:book_id` in `src/app.rs`
- [ ] Modify `CameraPage` to accept `book_id: String` prop
- [ ] Add page number input field to camera UI
- [ ] Initialize `NdlocrEngine` on camera page mount
- [ ] Wire "Run OCR" button to call `engine.process_image()`
- [ ] Implement "Save Page" to call `Database::save_page()` + `StorageService::save_page_image()`
- [ ] Fix `AddBookForm` to actually create book via `Database::create_book()`
- [ ] Add "Capture Pages" button to `LibraryScreen` that navigates to `/camera/:book_id`
- [ ] Add error handling for OCR failures, database errors
- [ ] Add loading indicators for OCR processing
- [ ] Test on device with `adb logcat | grep -i shusei`
