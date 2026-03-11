# Codebase Concerns

**Analysis Date:** 2026-03-11

## Tech Debt

### OCR Engine Not Implemented

**Issue:** The NDLOCR engine in `src/core/ocr/engine.rs` has placeholder implementations that return empty results.

**Files:** `src/core/ocr/engine.rs`, `src/core/ocr/postprocess.rs`

**Impact:**
- `process_image()` returns empty `OcrResult` with no regions, markdown, or plain text
- Text detection, recognition, and direction classification all marked as TODO
- Camera page shows placeholder message instead of actual OCR results

**Fix approach:** Implement tract-onnx model loading and inference pipeline as planned in Week 3-5 roadmap.

---

### STT Engine Not Implemented

**Issue:** The Moonshine STT engine in `src/core/stt/engine.rs` has placeholder implementations.

**Files:** `src/core/stt/engine.rs`, `src/core/stt/decoder.rs`

**Impact:**
- `transcribe()` returns empty `SttResult` with no text
- Mel-spectrogram preprocessing not implemented
- Decoder inference, top-k sampling, and top-p sampling all marked as TODO

**Fix approach:** Implement tract-onnx model loading and decoder pipeline as planned in Week 9-10 roadmap.

---

### UI Components Use Mock Data

**Issue:** All UI components load empty data instead of connecting to the database.

**Files:** 
- `src/ui/notes.rs` - Lines 20, 36
- `src/ui/reader.rs` - Line 17
- `src/ui/vocab.rs` - Line 20
- `src/ui/camera.rs` - Line 186

**Impact:**
- Notes page shows "No notes yet" even after OCR capture
- Reader page cannot load books from database
- Vocab page cannot display saved words
- Save functionality in camera page is stubbed out

**Fix approach:** Wire up UI components to `Database` methods for CRUD operations.

---

### Vocabulary Extractor Depends on Optional Lindera

**Issue:** `WordExtractor` in `src/core/vocab.rs` requires the `lindera` feature for Japanese morphological analysis, but this is an optional dependency.

**Files:** `src/core/vocab.rs`, `Cargo.toml`

**Impact:**
- Lines 45, 57, 84 all have TODOs for lindera implementation
- Default build does not include lindera, breaking vocabulary extraction
- Large dictionary download required for lindera (ipadic ~200MB)

**Fix approach:** Either make lindera a default feature or implement fallback tokenization strategy.

---

## Known Bugs

### Unsafe JNI Code in Android Platform

**Issue:** `src/platform/android.rs:193` uses unsafe conversion of JNI byte arrays.

**Files:** `src/platform/android.rs`

**Trigger:** Any camera capture on Android device.

**Current mitigation:** Wrapped in error handling, but unsafe block could panic if JNI environment is invalid.

**Workaround:** None - requires careful JNI lifecycle management.

---

### FTS Search Not Implemented

**Issue:** Notes search functionality is stubbed out.

**Files:** `src/ui/notes.rs:36`, `src/core/db.rs` (FTS table exists but not used)

**Trigger:** User types in search bar on notes page.

**Current mitigation:** Returns empty list.

**Workaround:** Manual scanning of notes list.

---

## Security Considerations

### No Input Validation on Database Queries

**Issue:** FTS search queries (when implemented) will use user input directly in SQL MATCH clause.

**Files:** `src/core/db.rs:173-181`

**Risk:** Potential FTS injection attacks if user input not sanitized.

**Current mitigation:** None currently visible.

**Recommendations:** 
- Sanitize FTS query input (escape special characters: `"`, `-`, `+`, `(`, `)`, `{`, `}`, `[`, `]`, `^`, `~`, `*`, `?`, `:`, `\`)
- Add query length limits

---

### File Path Handling

**Issue:** Image paths and book file paths stored as plain strings without validation.

**Files:** `src/core/db.rs` (schema), `src/core/pdf.rs`

**Risk:** Path traversal attacks if user-controlled paths used in file operations.

**Current mitigation:** None visible.

**Recommendations:**
- Validate file paths are within app data directory
- Use `PathBuf` with canonicalization before file access

---

## Performance Bottlenecks

### Database Queries Lack Pagination

**Issue:** `get_all_sticky_notes()` loads all notes into memory at once.

**Files:** `src/core/db.rs:151-162`

**Cause:** No LIMIT/OFFSET clause in query.

**Impact:** Performance degradation as note count grows (1000+ notes).

**Improvement path:** Add pagination parameters to all list queries.

---

### No Database Connection Pooling

**Issue:** Single `Connection` object in `Database` struct.

**Files:** `src/core/db.rs:14-16`

**Cause:** rusqlite `Connection` is not thread-safe by default.

**Impact:** Blocking I/O on async operations; cannot handle concurrent requests efficiently.

**Improvement path:** Use `r2d2` connection pool or wrap `Connection` in `Arc<Mutex<>>`.

---

### Image Encoding in Memory

**Issue:** Camera captures full-resolution images and encodes to base64 for UI display.

**Files:** `src/ui/camera.rs:88-92`

**Cause:** `general_purpose::STANDARD.encode(&data)` creates full base64 string in memory.

**Impact:** Large images (4MB+) cause memory pressure and UI lag.

**Improvement path:** 
- Downscale images before encoding
- Use streaming/base64 chunking
- Consider native image display via platform APIs

---

## Fragile Areas

### Platform Feature Detection

**Files:** `src/platform/mod.rs`, `src/platform/android.rs`, `src/platform/ios.rs`

**Why fragile:** 
- iOS platform returns `false` for all permission checks
- Android platform has partial implementation (audio recording, file picker not implemented)
- No compile-time feature flags to prevent calling unimplemented methods

**Safe modification:**
- Always check return values from platform APIs
- Add feature flags (`android`, `ios`, `desktop`) to gate functionality
- Implement fallback behavior for missing features

**Test coverage:** No platform integration tests detected.

---

### Optional Feature Dependencies

**Files:** `Cargo.toml:65-74`

**Why fragile:**
- `lindera`, `pdf`, `android`, `ios` features are optional
- Default build excludes critical functionality
- No runtime checks for feature availability

**Safe modification:**
- Add runtime feature detection helpers
- Document required features for each use case
- Consider making `lindera` and `pdf` default features

---

## Scaling Limits

### SQLite Single-Writer Limitation

**Current capacity:** One write transaction at a time.

**Limit:** WAL mode allows concurrent reads, but writes are serialized.

**Scaling path:** 
- Enable WAL mode explicitly (`PRAGMA journal_mode=WAL`)
- Batch writes where possible
- Consider async queue for write operations

---

### Model Loading Memory

**Current capacity:** Models loaded into memory on initialization.

**Limit:** ONNX models (NDLOCR + Moonshine) may exceed 500MB combined.

**Scaling path:**
- Implement lazy model loading
- Add model unloading on low-memory warnings
- Consider quantized model variants

---

## Dependencies at Risk

### tract-onnx Compatibility

**Risk:** `tract-onnx = "0.21"` - ONNX operator support may be incomplete for NDLOCR/Moonshine models.

**Impact:** Model inference may fail if operators not supported.

**Migration plan:** 
- Test model compatibility early (Week 3 for OCR, Week 9 for STT)
- Have fallback to ONNX Runtime (`ort` crate) if tract insufficient

---

### pdfium-render Static Linking

**Risk:** `pdfium-render = { version = "0.8", features = ["static"] }` - Large binary blob, potential licensing issues.

**Impact:** Increased binary size (~50MB), potential GPL licensing conflicts.

**Migration plan:** Review PDFium license; consider alternative PDF libraries if needed.

---

## Test Coverage Gaps

### Platform Integration Tests

**What's not tested:** 
- JNI camera capture flow
- Permission request flows
- File picker integration
- Audio recording

**Files:** `src/platform/android.rs`, `src/platform/ios.rs`

**Risk:** Platform-specific bugs only discovered on real devices.

**Priority:** High - blocks mobile deployment.

---

### OCR/STT Pipeline Tests

**What's not tested:**
- Full OCR pipeline (preprocess → detect → recognize → postprocess)
- Full STT pipeline (audio → mel-spectrogram → encode → decode)
- Model loading failures
- Error handling in inference

**Files:** `src/core/ocr/`, `src/core/stt/`

**Risk:** Core functionality may break without detection.

**Priority:** High - these are core value propositions.

---

### Database Migration Tests

**What's not tested:**
- Schema upgrades
- Data migration between versions
- FTS index rebuilding

**Files:** `src/core/db.rs`

**Risk:** App updates may corrupt user data.

**Priority:** Medium - becomes critical after user data exists.

---

## Missing Critical Features

### Error Display in UI

**Problem:** Errors logged but not shown to users.

**Blocks:** User troubleshooting; understanding what went wrong.

**Files:** `src/ui/camera.rs` (partial), other UI components

---

### Offline Model Management

**Problem:** No mechanism to download/update ONNX models.

**Blocks:** Initial app setup; model updates.

**Files:** Not implemented

---

### Settings/Configuration UI

**Problem:** No UI for app settings (language, quality, storage location).

**Blocks:** User customization.

**Files:** Not implemented

---

*Concerns audit: 2026-03-11*
