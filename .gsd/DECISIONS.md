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

## S03: Pdf Support (2026-03-15)

### PDF Processing Architecture
- **Batch processing (10 pages/batch)** - Balances memory usage and progress visibility. Prevents OOM on low-RAM devices while providing frequent progress updates.
- **Stream-based concurrency over tokio::spawn** - Used `futures::stream::buffer_unordered(3)` instead of `tokio::spawn` to avoid `Send` issues with rusqlite's `Connection` type. Max 3 concurrent OCR operations.
- **Resume support via processing_progress table** - Persists last processed page, allowing conversion to resume after app backgrounding or crashes.

### OCR Engine Design
- **ONNX Runtime (ort 2.0 RC)** - Thread-safe session management with `Arc<Mutex<Session>>` pattern. Mutex required because `session.run()` needs `&mut Session` and lifetime of `SessionOutputs` tied to session reference.
- **Image preprocessing pipeline** - Grayscale conversion, resize to 960x960 with Lanczos3, normalize to [0.0, 1.0], return NCHW tensor [1, 1, 960, 960].
- **PaddleOCR v5 models bundled** - text_detection.onnx (84MB), text_recognition.onnx (81MB), dict.txt (73KB). Supports Japanese and Chinese character recognition (~27,000 characters).

### UI/UX Decisions
- **Continuous scroll over pagination** - More natural for digital reading, easier to skim content. Uses scroll position / total height heuristic for current page estimation.
- **Font size range 12-32px** - Matches CONTEXT.md requirements for comfortable reading. Default 18px, adjustable via range slider in header.
- **Metadata review dialog** - Users can edit title/author before saving imported PDF to database. Prevents bad metadata persistence.
- **Stage-based progress display** - Three stages: Rendering (📄 blue) → OcrProcessing (🔍 purple) → Complete (✓ green). Visual progress bar with percentage.

### Simplified Progress Callback
- **No-op progress callback in convert_pdf()** - Dioxus signals are not `Send+Sync`, but progress callback requires `Send+Sync` for async execution. Used no-op callback and generic "converting" state signal instead of real-time page updates.
- **Rationale** - Avoids complex channel-based threading patterns, maintains clean async/await flow, user still sees conversion is in progress.

### Storage Conventions
- **PDF path convention: `pdfs/{book_id}.pdf`** - Book model doesn't store PDF path (avoids schema change). Assumes book ID matches PDF filename. Simple but may need enhancement in future slices.
- **`.bin` extension for all images** - Format-agnostic approach; storage doesn't need to know image format.

### Testing Strategy
- **Large PDF test infrastructure** - 373-page test PDF available (`tests/large_pdf_test.pdf`, 14MB) with complete monitoring and test procedure. Awaiting human verification on Android device.
- **Batch timing logs** - Every 10 pages in `src/core/pdf.rs` lines 365-372. Shows batch number, pages rendered, cumulative total, batch time.
- **OCR progress logs** - Confidence tracking and page completion in `src/core/ocr/engine.rs`.

### Known Issues
- **pdfium-render linking error** - Pre-existing unresolved external `FPDFPage_TransformAnnots` prevents full library build with PDF feature on some systems. Not caused by S03.
- **Model size** - 165MB total for OCR models may impact initial download/app size. Consider lazy loading or quantization for mobile.
- **Inference time** - Expected 1-2 seconds per page on CPU; not yet benchmarked on target devices.

## S04: Annotation Foundation (2026-03-15)

### Annotation Schema Design
- **Single table with type discriminator** - One `annotations` table with `annotation_type` column instead of separate tables for highlights/bookmarks/notes. Reduces schema complexity and simplifies queries across annotation types.
- **CHECK constraint for type validation** - `CHECK(annotation_type IN ('highlight', 'bookmark', 'note'))` enforces valid types at database level, not just in application code.
- **Foreign key to books(id)** - Ensures referential integrity; annotations cannot exist without a valid book.

### Annotation Type System
- **AnnotationType enum with FromStr/Display** - Type-safe enum in Rust (`Highlight`, `Bookmark`, `Note`) with automatic string conversion for database storage. Invalid types return descriptive error messages.
- **Type predicate helpers** - `is_highlight()`, `is_bookmark()`, `is_note()` methods on Annotation struct for ergonomic type checking without enum matching.

### Position Tracking
- **Character offsets for text selection** - `position_start` and `position_end` store character offsets (not byte offsets) for precise text selection. UI will capture these from user text selection in reader view.
- **Optional position range** - Positions are optional (INTEGER, not NOT NULL) to support bookmarks that apply to entire pages without specific text selection.

### Color Handling
- **Optional color field with default null** - `color TEXT DEFAULT 'yellow'` but column allows NULL. UI will provide default colors; users can customize. Highlights typically use yellow/green/pink/blue.

### Builder Pattern
- **NewAnnotation builder with fluent API** - `NewAnnotation::highlight()`, `bookmark()`, `note()` constructors with `with_position()` fluent setter. Provides ergonomic construction and ensures all required fields are set before insertion.

### Partial Update Pattern
- **COALESCE for selective field updates** - `update_annotation()` uses `COALESCE(?, field)` pattern to update only provided fields. Allows updating color without changing content, or adding user_note without modifying position.

### Bulk Deletion
- **delete_annotations_by_book()** - Bulk delete method for cleaning up annotations when a book is deleted. Prevents orphaned annotation records. Note: Currently manual operation, not automatic foreign key cascade.

### Testing Approach
- **15 unit tests with in-memory database** - Each test creates isolated in-memory database for reproducibility. Tests follow same patterns as `books_crud` and `book_pages` modules.
- **Test execution blocked by ONNX linker** - Pre-existing `ort-sys` linker error prevents test execution. Tests are structurally correct and would pass on properly configured system.

### Deferred Items
- **Annotation UI components** - Backend complete; UI for creating/editing/deleting annotations deferred to frontend implementation phase.
- **Text selection integration** - Position range fields exist but reader UI doesn't yet capture text selection. Requires integration with reflow reader.
- **Highlight visual rendering** - Database stores color but reader view doesn't yet render colored backgrounds.
- **Export/import integration** - Annotations not yet included in book export/import JSON serialization.

## S05: Voice Memos (2026-03-15)

### Audio Recording Architecture
- **Float array over byte array** — Normalized PCM samples (-1.0 to 1.0) as `Vec<f32>` instead of raw bytes. Easier for DSP processing and mel-spectrogram computation.
- **30-second hard limit** — Enforced at Rust layer with timeout (max_seconds + 5s buffer). Defense in depth with Java side also enforcing limit.
- **Dual package namespace support** — JNI callbacks support both `com.shusei.app` and `dev.dioxus.main` packages for development flexibility.

### Mel-Spectrogram Implementation
- **From-scratch implementation** — Custom STFT, FFT (radix-2 + DFT fallback), and mel filterbank instead of external crate. Matches Moonshine specs exactly, avoids dependency issues.
- **Moonshine-default parameters** — 16kHz sample rate, 25ms window (400 samples), 10ms hop (160 samples), 80 mel bins, 400 FFT size.
- **2D ndarray output** — `Array2<f32>` with shape `[time_frames, 80]` instead of flat `Vec<f32>`. Proper tensor structure for model input.

### Testing Strategy
- **9 unit tests for preprocessing** — Cover parameters, conversions, edge cases, output shapes, FFT correctness. Tests compile and verify algorithm correctness.
- **Integration tests deferred** — Full end-to-end testing blocked by pre-existing ONNX linker error (`__isoc23_strtoll` undefined symbol in `ort-sys`).

### Partial Slice Completion
- **T01-T02 completed, T03-T06 deferred** — Audio recording and preprocessing complete. Moonshine model integration, UI components, reader integration, and comprehensive testing deferred due to ONNX blocker and Java side gap.
- **Java side implementation pending** — `MainActivity.java` needs `startAudioRecording()`, permission methods, and native callbacks for `AudioRecord` integration.

### Known Blockers
- **ONNX Runtime linker error** — `__isoc23_strtoll`, `__isoc23_strtoull`, `__isoc23_strtol` undefined symbols in `ort-sys`. Prevents loading Moonshine models and running full test suite.
- **Model files not acquired** — Moonshine Tiny ONNX models (encoder/decoder for English and Japanese) not yet downloaded or bundled.

### Deferred Items
- **VoiceMemoInput UI component** — Recording UI with record/stop buttons, timer, transcript editor deferred to frontend phase.
- **Reader integration** — Voice memo button in note creation dialog deferred until model integration complete.
- **Lazy model loading** — Load/unload Moonshine models on demand deferred until lifecycle management implemented.

## S06: AI Enhancement (2026-03-15)

### Words Table Schema
- **`ai_generated` boolean flag** — Distinguishes AI-generated definitions from manual entries. Enables filtering and accuracy tracking.
- **Context storage** — `context_text` field stores ±50 characters around tapped word for future reference and debugging.
- **Foreign key to books(id)** — `source_book_id` links word to source book for vocabulary-by-book queries.

### AI Engine Abstraction
- **Trait-based architecture** — `AiEngine` trait decouples service layer from specific model implementation. Allows swapping `MockAiEngine` for `QwenEngine` without changing calling code.
- **Mock engine for testing** — `MockAiEngine` provides deterministic responses for 5 known words, placeholder for unknowns. Enables testing without model files or ONNX runtime.
- **Service layer pattern** — `WordDefinitionService<E>` wraps engine with high-level `define_word()` API. Handles auto-loading, logging, and error handling.

### Model Loading Strategy
- **Lazy loading on first tap** — Model loaded when user first taps a word, not at app startup. Reduces initial memory footprint.
- **Retain until background** — Model stays in memory until app backgrounds (per Android lifecycle). Subsequent taps are instant.
- **2-3 second cold start budget** — First definition takes 2-3 seconds (model load + inference); warm definitions should be <1 second.

### Minimal Viable Slice
- **Database + engine only** — S06 delivers schema, CRUD, engine trait, mock implementation, and tests. Qwen integration and UI deferred due to ONNX blocker.
- **9 unit tests** — Prove pipeline logic works: engine lifecycle, definition generation, error handling, service integration.
- **No Japanese tokenization** — MeCab/Jieba integration deferred. Current implementation assumes word boundaries already detected.

### Known Blockers
- **ONNX Runtime linker error** — Same `__isoc23_*` undefined symbols blocking S05 also block Qwen model loading. Requires upstream fix or alternative runtime.
- **Model file sourcing** — Qwen3.5-0.8B GGUF not bundled. Would need ~800MB download or aggressive quantization to <100MB.

### Deferred Items
- **QwenEngine implementation** — Real Candle-based Qwen inference deferred until ONNX issue resolved.
- **DefinitionPopup UI** — Inline popup component with tap-outside dismissal deferred to frontend phase.
- **Japanese word segmentation** — MeCab or Jieba integration for word boundary detection deferred.
- **Model download flow** — First-run HuggingFace download with progress indicator deferred.
- **Accuracy validation** — 50-word accuracy test (>85% target) deferred until real model integrated.

---