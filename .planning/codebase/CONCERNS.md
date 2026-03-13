# Codebase Concerns

**Analysis Date:** 2026-03-13

## Tech Debt

**Unimplemented TODO Comments:**
- Issue: 25+ TODO comments indicating incomplete implementations
- Files: Multiple files across `src/`
- Impact: Core features are placeholder implementations:
  - `src/core/ocr/postprocess.rs:15,29,43` - Text detection, recognition, direction classification using tract
  - `src/core/stt/engine.rs:68,114,136` - ONNX model loading, mel-spectrogram, full STT pipeline
  - `src/core/stt/decoder.rs:152,194,203` - Decoder step, top-k sampling, top-p sampling
  - `src/core/vocab.rs:45,57,84` - Lindera tokenizer integration for Japanese word extraction
  - `src/platform/ios.rs:28,35,42,49` - All iOS platform APIs are stubs
  - `src/ui/camera.rs:78,186` - OCR engine call, database save
  - `src/ui/notes.rs:20,36` - Database loading, FTS search
  - `src/ui/vocab.rs:20,74,82,108` - Database loading, export functions, word deletion
- Fix approach: Prioritize and implement missing functionality in phased approach

**unwrap() Calls in Production Code:**
- Issue: Multiple `.unwrap()` calls that can panic at runtime
- Files: `src/core/db.rs`, `src/core/pdf.rs`, `src/core/models.rs`, `src/core/ocr/engine.rs`
- Impact: Application crashes on edge cases (time errors, missing data)
- Specific locations:
  - `src/core/db.rs:336,343,462,480` - `SystemTime::now().duration_since(UNIX_EPOCH).unwrap()`
  - `src/core/models.rs:23` - Same time calculation
  - `src/core/pdf.rs:161` - ThreadPool build unwrap
  - `src/core/ocr/engine.rs:388,472` - Tensor slice unwrap
- Fix approach: Replace with proper error handling using `Result` and `?` operator

**expect() in Default Implementations:**
- Issue: `Default` implementations use `.expect()` which panics
- Files: `src/core/pdf.rs:229`, `src/core/vocab.rs:108`
- Impact: Application panic if initialization fails (e.g., missing pdfium)
- Fix approach: Remove `Default` impl or use lazy initialization with Result

## Known Bugs

**iOS Platform Not Implemented:**
- Symptoms: All iOS platform methods return error "not yet implemented"
- Files: `src/platform/ios.rs`
- Trigger: Any platform API call on iOS
- Workaround: Desktop/Android only for MVP

**Android Microphone Permission:**
- Symptoms: `has_microphone_permission()` always returns false, `request_microphone_permission()` returns Ok(false)
- Files: `src/platform/android.rs:141-159`
- Trigger: Audio recording feature
- Workaround: None - feature blocked

**Japanese Word Extraction:**
- Symptoms: Returns empty Vec for Japanese text
- Files: `src/core/vocab.rs:83-88`
- Trigger: Extracting words from Japanese text
- Workaround: English-only word extraction works

## Security Considerations

**Unsafe JNI Block:**
- Risk: Potential undefined behavior if JNI contract violated
- Files: `src/platform/android.rs:193`
- Code: `let byte_array = unsafe { JByteArray::from_raw(image_data) };`
- Current mitigation: Used only in controlled callback from Java
- Recommendations: Add validation of jbyteArray pointer before use

**File Path Handling:**
- Risk: No path traversal validation in storage service
- Files: `src/core/storage.rs:68-83`
- Code: `get_image()` accepts arbitrary path string
- Current mitigation: Paths generated internally with UUID
- Recommendations: Add path sanitization to prevent `../` traversal

**Hardcoded Android Path:**
- Risk: Hardcoded path may not work on all Android configurations
- Files: `src/platform/android.rs:226`
- Code: `PathBuf::from("/data/data/com.shusei.app/files")`
- Current mitigation: None
- Recommendations: Use JNI to get actual files directory from Context

**No Input Size Limits:**
- Risk: Memory exhaustion from large images
- Files: `src/ui/camera.rs:87-92`
- Code: Base64 encoding of arbitrary image data without size limits
- Current mitigation: None
- Recommendations: Add maximum image size validation before processing

## Performance Bottlenecks

**Memory Usage in PDF Rendering:**
- Problem: All pages rendered into memory before OCR starts
- Files: `src/core/pdf.rs:324-388`
- Cause: `render_pages_batch` returns Vec of all page images
- Improvement path: Stream pages to OCR as rendered, don't hold all in memory

**Image Clone in Base64 Encoding:**
- Problem: Captured image cloned and encoded to base64 for display
- Files: `src/ui/camera.rs:87-92`
- Cause: UI needs base64 data URI for image display
- Improvement path: Use blob URLs or native image component

**Synchronous Database Operations:**
- Problem: Database operations are synchronous but called from async UI context
- Files: `src/core/db.rs`
- Impact: UI may freeze during large queries
- Improvement path: Wrap database calls in `spawn_blocking` or use async SQLite

**No Caching Strategy:**
- Problem: No LRU cache for rendered PDF pages, no image cache for camera captures
- Files: Multiple
- Impact: Repeated OCR on same image re-runs full pipeline
- Improvement path: Implement caching layer for processed results

## Fragile Areas

**JNI Camera Integration:**
- Files: `src/platform/android.rs:163-223`
- Why fragile: Complex async state management with oneshot channels
- Safe modification: Ensure CAMERA_STATE mutex always released before callback
- Test coverage: Limited - requires Android device/emulator
- Breakage risk: Race conditions if Java callback never fires (30s timeout)

**Parallel PDF Processing:**
- Files: `src/core/pdf.rs:137-181`, `src/core/ocr/engine.rs:526-596`
- Why fragile: Thread pool with fixed 3 threads, assumes availability
- Safe modification: Keep batch sizes consistent with memory limits
- Test coverage: Integration tests in `tests/large_pdf_test.rs`
- Breakage risk: Memory exhaustion on very large PDFs if concurrency too high

**OCR Session Locking:**
- Files: `src/core/ocr/engine.rs:383-400,471-507`
- Why fragile: Mutex-wrapped ONNX sessions, lock scope management critical
- Safe modification: Always extract tensor data before releasing lock
- Test coverage: Unit tests with mock models
- Breakage risk: Deadlock if lock held during async operation

**Float Comparison in Sorting:**
- Files: `src/core/ocr/markdown.rs:89-120`
- Why fragile: `partial_cmp` returns None for NaN, code unwraps to Equal
- Safe modification: Handle None case explicitly in sort comparisons
- Test coverage: Unit tests with normal floats
- Breakage risk: Incorrect sorting with NaN confidence scores

## Scaling Limits

**PDF Page Processing:**
- Current capacity: Tested with 373-page PDF
- Limit: Memory exhaustion on very large PDFs (1000+ pages)
- Scaling path: Implement true streaming with page-at-a-time processing

**SQLite FTS Index:**
- Current capacity: Suitable for thousands of notes
- Limit: FTS5 performance degrades with millions of records
- Scaling path: Partition database per-book, or migrate to dedicated search

**ONNX Model Memory:**
- Current capacity: Single model loaded at a time
- Limit: Multiple concurrent OCR/STT operations share same model sessions
- Scaling path: Load multiple model instances or queue operations

**STT KV Cache:**
- Current capacity: Pre-allocates `max_seq_len` for each layer
- Limit: ~4MB per cache (4 layers, 512 seq, 64 dim)
- Scaling path: Dynamic cache growth, memory-mapped tensors

## Dependencies at Risk

**hayro (v0.4):**
- Risk: Relatively new PDF library, may have rendering bugs
- Impact: PDF processing failures for edge-case PDFs
- Migration plan: Fallback to pdfium if hayro issues persist

**ort (v2.0.0-rc.12):**
- Risk: Release candidate, not stable version
- Impact: Potential API changes, instability
- Migration plan: Pin version, test thoroughly before upgrading

**tract-onnx (v0.21):**
- Risk: Alternative inference runtime, different behavior than ort
- Impact: Inconsistency between test and production inference
- Migration plan: Standardize on ort for production

**jni (v0.21):**
- Risk: Unsafe code required for all JNI operations
- Impact: Easy to introduce memory safety bugs
- Migration plan: Wrap in safer abstraction layer

## Missing Critical Features

**STT Pipeline:**
- Problem: Speech-to-text engine is placeholder only
- Files: `src/core/stt/engine.rs`, `src/core/stt/decoder.rs`
- Blocks: Voice note transcription feature

**iOS Support:**
- Problem: All iOS platform methods return errors
- Files: `src/platform/ios.rs`
- Blocks: iOS deployment

**Japanese Morphological Analysis:**
- Problem: Lindera tokenizer not initialized
- Files: `src/core/vocab.rs`
- Blocks: Japanese word extraction from text

**UI-Database Integration:**
- Problem: Most UI pages use placeholder data, not database
- Files: `src/ui/notes.rs`, `src/ui/vocab.rs`, `src/ui/camera.rs`
- Blocks: Actual data persistence and retrieval

**Database Migration:**
- Problem: No migration strategy for schema changes
- Files: `src/core/db.rs:40-106`
- Blocks: Safe app updates

## Test Coverage Gaps

**Android Platform Tests:**
- What's not tested: JNI callbacks, camera capture flow, permission handling
- Files: `src/platform/android.rs`
- Risk: Runtime failures on Android only
- Priority: High - critical for mobile deployment

**STT Engine Tests:**
- What's not tested: Audio transcription pipeline (not implemented)
- Files: `src/core/stt/`
- Risk: Feature won't work when implemented
- Priority: Medium - post-MVP feature

**iOS Platform Tests:**
- What's not tested: Everything (stub implementation)
- Files: `src/platform/ios.rs`
- Risk: Feature missing entirely
- Priority: Low - post-MVP platform

**Error Path Testing:**
- What's not tested: Error recovery, partial failures, edge cases
- Files: All modules
- Risk: Poor user experience on errors
- Priority: Medium

**Database Migration:**
- What's not tested: Schema migrations, version upgrades
- Files: `src/core/db.rs`
- Risk: Data loss on app update
- Priority: High for production release

## Recommended Priority Order

1. **Critical:** Fix panic in Default implementations (`src/core/vocab.rs:108`, `src/core/pdf.rs:229`)
2. **Critical:** Replace unwrap() with proper error handling in time calculations
3. **High:** Implement actual model loading in OCR/STT engines
4. **High:** Add database migration strategy
5. **Medium:** Add input validation for image sizes and note content
6. **Medium:** Implement retry logic for JNI operations
7. **Medium:** Add path traversal protection in storage service
8. **Low:** Optimize image preprocessing with parallel operations
9. **Low:** Implement caching layer for processed results

---

*Concerns audit: 2026-03-13*