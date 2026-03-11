# Phase 2: Paper Book Capture - Context

**Gathered:** 2026-03-11
**Status:** Ready for planning

<domain>
## Phase Boundary

Complete camera → OCR workflow for digitizing physical book pages. Users can capture book pages with camera, process via OCR, and save both the image and extracted text linked to a book.

**In scope:**
- Camera capture with manual trigger
- Image preprocessing (downscale, enhance)
- OCR processing with NDLOCR-Lite
- Page linking to books via cover detection
- Progress feedback during OCR
- Parallel task support during processing

**Out of scope:**
- PDF import (Phase 3)
- Annotations/notes on pages (Phase 4)
- Voice memos linked to pages (Phase 5)

</domain>

<decisions>
## Implementation Decisions

### Camera Workflow
- Manual trigger capture — user taps button for full control
- Single page at a time — capture → review → next flow
- Immediate OCR after capture — seamless one-step flow
- Silent retry on failures — auto-retry, only show error after multiple failures

### Image Processing
- Auto-downscale to 2MP — balance quality and memory for mid-range devices
- Auto-enhance enabled — deskew, contrast adjustment, noise reduction for better OCR accuracy
- JPEG format — smaller file size, good for photos
- Quality warning — detect blur/darkness, suggest retaking before OCR

### OCR Behavior
- Progress spinner — simple loading indicator during processing
- Auto-retry on low confidence — re-process with different parameters
- Auto-detect Japanese — NDLOCR auto-detection, no user action needed
- Parallel task support — OCR runs in background, users can start voice input or other operations during processing

### Page Organization
- Auto-detect book from cover — capture book cover first to auto-create/link book
- Page numbering: OCR auto-extract with manual fallback — detect page numbers from image, prompt user if recognition fails
- Allow duplicates — users can capture same page multiple times, delete later if needed

### OpenCode's Discretion
- Exact enhancement algorithm parameters (deskew angle threshold, contrast levels)
- Specific confidence thresholds for auto-retry
- Duplicate detection similarity threshold (if implemented later)
- Progress spinner animation details

</decisions>

<code_context>
## Existing Code Insights

### Reusable Assets
- **Camera UI component** (`src/ui/camera.rs`) — Existing capture flow with permission handling, state management for captured image
- **OCR engine structure** (`src/core/ocr/`) — NdlocrEngine trait, OcrResult structure with markdown/plain_text output
- **Android camera API** (`src/platform/android.rs`) — JNI-based camera capture with `capture_image()` async method
- **Database schema** (`src/core/db.rs`) — `pages` table with `image_path`, `ocr_markdown`, `ocr_text_plain`, `book_id` fields
- **Storage service** (`src/core/storage.rs`) — `save_image()` method with prefix support for organizing files

### Established Patterns
- **File storage for images** — Images saved to filesystem, paths stored in SQLite (Phase 1 pattern)
- **WAL mode** — Concurrent database reads supported
- **Async/await** — All I/O operations use tokio async runtime
- **Error handling** — anyhow/thiserror pattern throughout codebase
- **Android lifecycle** — State persistence with save/restore pattern

### Integration Points
- **Camera → OCR pipeline** — Connect `CameraPage` component to `NdlocrEngine::process_image()`
- **Page storage** — Use `db.save_page()` pattern (extend db.rs with page save method)
- **Book linking** — Extend cover detection flow to auto-create or link to existing book
- **Parallel processing** — Use tokio spawn for background OCR while keeping UI responsive

</code_context>

<specifics>
## Specific Ideas

- "Auto-detect book from cover" — User captures book cover first, system either finds matching book in library or creates new book entry
- Page numbering should be automatic via OCR detection of page numbers (typically at bottom of page), but gracefully fall back to manual input when OCR confidence is low
- OCR processing should not block other features — user should be able to start voice memo or capture another page while OCR completes in background
- Quality warning before OCR saves time — better to retake a blurry photo than process poor quality image

</specifics>

<deferred>
## Deferred Ideas

- **Batch capture mode** — Continuous capture session for multiple pages, mentioned but deferred to after single-page flow is stable
- **Page similarity detection** — Warn about potential duplicates, deferred to future optimization phase
- **Advanced image enhancement** — ML-based enhancement for difficult lighting, deferred to performance phase
- **Multi-language OCR** — English/Chinese language selection, deferred until Japanese flow is validated

</deferred>

---

*Phase: 02-paper-book-capture*
*Context gathered: 2026-03-11*
