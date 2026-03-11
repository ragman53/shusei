# Codebase Concerns

**Analysis Date:** 2026-03-11

## Tech Debt

### Unimplemented TODOs

**OCR Engine (`src/core/ocr/engine.rs`):**
- Line 79: Model loading via tract not implemented - `NdlocrEngine::initialize()` only checks file existence
- Line 129: Full OCR pipeline not implemented - `process_image()` returns empty results

**OCR Postprocessing (`src/core/ocr/postprocess.rs`):**
- Line 15: Text detection using tract not implemented
- Line 29: Text recognition using tract not implemented  
- Line 43: Direction classification using tract not implemented

**STT Engine (`src/core/stt/engine.rs`):**
- Line 68: ONNX model loading via tract not implemented - `MoonshineEngine::initialize()` only checks file existence
- Line 114: Mel-spectrogram preprocessing not implemented - returns raw audio as placeholder
- Line 136: Full STT pipeline not implemented - `transcribe()` returns empty results

**STT Decoder (`src/core/stt/decoder.rs`):**
- Line 152: Decoder step using tract not implemented - returns placeholder token
- Line 194: Top-k sampling not implemented (falls through to greedy)
- Line 203: Top-p sampling not implemented (falls through to greedy)

**UI Pages (`src/ui/notes.rs`, `src/ui/reader.rs`, `src/ui/vocab.rs`):**
- Line 20 in all: Database loading not implemented - all pages show empty state
- Line 36 in notes.rs: FTS search not implemented
- Line 74, 82 in vocab.rs: Markdown/CSV export not implemented
- Line 52, 69 in reader.rs: File picker not implemented

**Vocabulary Module (`src/core/vocab.rs`):**
- Line 45: Lindera tokenizer initialization not implemented
- Line 57: Japanese word extraction not implemented - returns empty Vec

**Platform iOS (`src/platform/ios.rs`):**
- Line 28: Camera using AVCaptureSession not implemented
- Line 35: Audio using AVAudioRecorder not implemented
- Line 42: File picker using UIDocumentPickerViewController not implemented
- Line 49: Vibration using UIImpactFeedbackGenerator not implemented

### Stubs and Placeholders

**Desktop Platform (`src/platform/mod.rs`):**
- Lines 85-118: `DesktopPlatform` returns errors or false for all operations
- Desktop is a fallback that provides no actual functionality
- Development must target Android device/emulator for camera features

**UI Components (`src/ui/components.rs`):**
- Components are well-structured but not yet used in main UI
- Camera page uses inline HTML instead of shared components

## Potential Bugs

### Panic in Default Implementations

**Critical:** Default trait implementations use `.expect()` which will panic:

- `src/core/vocab.rs` line 108: `WordExtractor::default()` calls `.expect("Failed to create WordExtractor")`
- `src/core/pdf.rs` line 108: `PdfProcessor::default()` calls `.expect("Failed to create PdfProcessor")`

**Impact:** Application will crash on startup if these types are instantiated via Default trait.

**Fix approach:** Remove Default implementations or make them return Option/Result.

### Float Comparison Fragility

**Multiple locations use `partial_cmp` with unwrap:**
- `src/core/ocr/markdown.rs` lines 90, 94, 101, 105, 114, 118
- `src/core/ocr/postprocess.rs` line 60
- `src/core/stt/decoder.rs` lines 189-208

**Issue:** `partial_cmp` returns None for NaN values, but code unwraps to Equal. Sorting may produce incorrect results with NaN confidence scores.

### Test-Only unwrap() Calls

**Tests use `.unwrap()` extensively:**
- `src/core/db.rs` lines 293-311: Database tests use unwrap
- `src/core/vocab.rs` lines 215, 225: Vocabulary tests use unwrap
- `src/core/ocr/markdown.rs` lines 190, 201: Markdown generation tests use unwrap

**Risk:** Tests may panic instead of failing gracefully. Not critical for production but indicates test fragility.

### JNI Thread Safety

**Android Platform (`src/platform/android.rs`):**
- Line 14: `CAMERA_STATE` uses std Mutex across async boundaries
- Line 16: `JAVA_VM` stored in Mutex
- Lines 72-78: State mutation without considering reentrancy

**Risk:** Potential deadlocks if camera callbacks happen on unexpected threads.

## Security Concerns

### No Input Validation

**Camera page (`src/ui/camera.rs`):**
- Line 87-92: Base64 encoding of arbitrary image data without size limits
- No validation of image dimensions from JNI callback
- No protection against maliciously large images causing memory exhaustion

**Database (`src/core/db.rs`):**
- Line 115-136: `create_sticky_note()` accepts arbitrary strings without sanitization
- No SQL injection risk (uses parameterized queries), but no content validation

### JNI Security

**Android Platform (`src/platform/android.rs`):**
- Lines 163-181: `nativeInit` gets JavaVM and stores globally
- No validation that caller is authorized Android activity
- Potential for malicious native code injection if APK is tampered

### File Path Exposure

**PDF Processor (`src/core/pdf.rs`):**
- Lines 21-26: Loads system library without path validation
- If pdfium is not bundled, may load from attacker-controlled path

## Performance Issues

### Memory Inefficiency

**STT KV Cache (`src/core/stt/decoder.rs`):**
- Lines 32-44: Pre-allocates `max_seq_len` worth of zeros for each layer
- For typical values (4 layers, 8 heads, 64 dim, 512 seq): ~4 * 512 * 512 * 4 bytes = 4MB per cache
- Multiple concurrent transcriptions could exhaust memory

**Image Processing (`src/core/ocr/preprocess.rs`):**
- Line 66-81: `image_to_rgb_tensor()` iterates pixel-by-pixel
- Could be optimized with ndarray operations or parallel iteration

### Synchronous Database Operations

**All database methods (`src/core/db.rs`):**
- Database operations are synchronous but called from async UI context
- UI may freeze during large queries
- No connection pooling for multiple concurrent operations

### No Caching Strategy

- No LRU cache for rendered PDF pages
- No image cache for camera captures
- Repeated OCR on same image re-runs full pipeline

## Fragile Areas

### JNI Bridge (`src/platform/android.rs`)

**Complexity:** 223 lines of unsafe JNI interop

**Fragility points:**
- Line 81-89: Java method call via JNI - method name/signature must match exactly
- Line 183-213: JNI callback from Java to Rust - byte array conversion uses unsafe
- Line 193: `unsafe { JByteArray::from_raw(image_data) }` - assumes valid JNI reference
- Line 215-223: Sender channel may be closed if callback happens unexpectedly

**Safe modification:**
- Add JNI signature validation tests
- Wrap JNI calls in safer abstraction
- Add timeout handling for all JNI operations

### Model Loading (`src/core/ocr/engine.rs`, `src/core/stt/engine.rs`)

**Fragility:**
- Assumes models exist at specific paths relative to executable
- No graceful degradation if models are missing
- No model version checking

**Safe modification:**
- Check model checksums/hashes
- Provide fallback to cloud API if models unavailable
- Add model metadata validation

### Float Operations in Sorting

**Markdown Generation (`src/core/ocr/markdown.rs`):**
- Lines 89-120: Sorting by bbox coordinates with partial_cmp
- Lines 45-46: Threshold comparison for paragraph detection

**Risk:** Page layouts with overlapping or strangely positioned text may sort incorrectly.

## Scaling Limits

### Database

**Current:** Single SQLite file per device
- No migration strategy for schema changes (see `src/core/db.rs` line 40-106)
- No backup/restore mechanism
- FTS5 virtual table may have performance issues with large datasets

**Limit:** Unknown - depends on device storage and FTS5 performance

### Models

**OCR:** NDLOCR-Lite ~8-17MB total
**STT:** Moonshine Tiny ~45-60MB per language

**Current approach:** Load models on demand, no memory management

**Limit:** Can only have one language model loaded at a time

### Image Processing

**Preprocessing (`src/core/ocr/preprocess.rs`):**
- Line 37-46: Images scaled to max_size (1024px) by default
- No memory limit for very large images
- Could OOM on 4K+ images

## Test Coverage Gaps

### Missing Integration Tests

- No end-to-end OCR pipeline test
- No end-to-end STT pipeline test
- No JNI integration tests (requires Android emulator)
- No database migration tests

### Mock-Based Tests Only

- OCR engine tests use placeholder implementations
- STT tests use hardcoded dummy data
- No tests with actual ONNX model loading

### Platform Tests

**iOS (`src/platform/ios.rs`):**
- Entirely stubbed - no tests possible

**Android (`src/platform/android.rs`):**
- JNI callbacks cannot be unit tested
- Requires instrumentation tests on device

## Missing Critical Features

### Error Recovery

- No retry logic for transient failures (JNI calls, model loading)
- No graceful degradation when models unavailable
- No offline queue for failed operations

### Data Persistence

- No automatic backup of user data
- No export/import functionality for notes (only vocabulary export implemented)
- No data migration between app versions

### Input Sanitization

- No XSS protection in markdown rendering
- No validation of OCR output before display
- No limits on note size or vocabulary list size

## Dependency Risks

### tract-onnx (v0.21)

**Risk:** Core dependency for ML inference
- Limited documentation on mobile deployment
- Complex build requirements for cross-compilation
- May have compatibility issues with specific ONNX ops

**Mitigation:** Test files `tests/ndlocr_tract_test.rs` and `tests/moonshine_tract_test.rs` document known issues

### pdfium-render (v0.8)

**Risk:** PDF rendering dependency
- Requires native pdfium library (not bundled by default)
- Platform-specific build complexity
- Optional feature but main use case depends on it

### Dioxus (v0.7)

**Risk:** UI framework
- Relatively new framework, API may change
- Mobile support still maturing
- Router integration is feature-flagged

### jni (v0.21)

**Risk:** Android JNI bindings
- Unsafe code required for all JNI operations
- Complex lifetime management between Java and Rust
- Easy to introduce memory safety bugs

## Recommended Priority Order

1. **High:** Fix panic in Default implementations (`vocab.rs`, `pdf.rs`)
2. **High:** Implement actual model loading in OCR/STT engines
3. **Medium:** Add input validation for image sizes and note content
4. **Medium:** Implement retry logic for JNI operations
5. **Low:** Add database migration strategy
6. **Low:** Optimize image preprocessing with parallel operations

---

*Concerns audit: 2026-03-11*
