# Architectural and Pattern Decisions

## S01: Core Infrastructure (2026-03-11)

### Storage Architecture
- **Filesystem storage over SQLite BLOBs** - Avoids memory issues on low-RAM Android devices. Store relative file paths in database (e.g., `images/cover_abc123.bin`), not absolute paths.
- **`.bin` extension for all images** - Format-agnostic approach; storage doesn't need to know image format.
- **`{assets_dir}/images/` subdirectory** - Organized storage structure, auto-created on first save.

### Database Design
- **WAL mode enabled** - `PRAGMA journal_mode=WAL` for concurrent read support.
- **`updated_at` column in books table** - Required for tracking modifications, NOT NULL constraint.
- **Parameterized queries only** - No SQL injection risk, all CRUD operations use `params![]` macro.

### State Persistence
- **JSON file over SharedPreferences** - Cross-platform compatibility, easier debugging, human-readable state.
- **`.shusei` subdirectory** - Dedicated directory for app state files, organized and discoverable.
- **AppState fields: route, scroll_position, timestamp** - Minimal viable state for lifecycle restoration.

### Android JNI Patterns
- **PushLocalFrame/PopLocalFrame** - Prevents native memory leaks during lifecycle transitions. Capacity of 16 local references sufficient for state operations.
- **Graceful JavaVM fallback** - When JavaVM not initialized, fallback to `std::env::current_dir()` for desktop development.

### UI Architecture
- **Modal overlay for AddBookForm** - Keeps users in library context, better UX than separate page.
- **Validation from signal state** - Computed `is_valid` from title/author signals, not on submit.
- **Explicit `()` in onclick handlers** - Required by Dioxus 0.7 event handler type system.
- **Placeholder components for router** - Satisfy Dioxus router compilation before full UI implementation.

---

- "Code was pre-existing - verified and fixed test bugs instead of implementing from scratch"
- "Used placeholder components for routes to enable compilation before UI implementation"
- "Implemented validation logic in AddBookForm with is_valid signal"
- "Modal overlay pattern for AddBookForm to maintain context"

## S02: Paper Book Capture (2026-03-15)

### Image Preprocessing
- **2MP downscaling limit** - Balances quality and memory usage for mid-range Android devices. Formula: `scale = sqrt(2M / (w * h))` when `width * height > 2,000,000`.
- **Always enhance contrast** - Histogram-based contrast stretching improves OCR accuracy on book pages.
- **Grayscale conversion** - Most OCR engines work better with grayscale; reduces data from 3 channels to 1.
- **JPEG output at 85% quality** - Smaller file size than PNG for photographic content like book pages.

### Storage Structure
- **`pages/{book_id}/{timestamp}_{uuid}.jpg`** - Organized by book, timestamp ensures chronological ordering, UUID prevents collisions.
- **Relative paths in database** - Storage returns relative path for database; absolute path reconstructed at runtime.

### Database Schema for Pages
- **`book_id` as TEXT** - Matches `books.id` primary key type for foreign key consistency.
- **Separate `ocr_markdown` and `ocr_text_plain`** - Markdown for rich display, plain text for full-text search indexing.
- **Indexes on `book_id` and `page_number`** - Optimizes common queries: "get all pages for a book" and "get page N".
- **`UNIQUE(book_id, page_number)` constraint** - Prevents duplicate pages in same book.

### Quality Detection Algorithms
- **Laplacian variance for blur detection** - Simple 5-point Laplacian, variance < 100 = blurry image. Fast O(n) single-pass algorithm.
- **Brightness optimal range 50-200** - On 0-255 scale; < 50 too dark, > 200 too bright for reliable OCR.
- **60/40 blur/brightness weighting** - Blur more critical for OCR accuracy than brightness.
- **Two-tier retry logic** - Overall confidence threshold (0.5) + critical region threshold (0.3).

### Deferred Items
- **Full tract-onnx OCR pipeline** - Deferred to Week 3-5; requires ONNX model files not yet available.
- **Quality warning UI component** - Backend complete, UI integration deferred to camera UI integration phase.
- **Parallel OCR with tokio::spawn** - Retry logic complete, async integration deferred to UI phase.
- **Page viewer component** - Focus on core capture → OCR → save flow first; viewer can be added later.
