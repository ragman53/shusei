# Domain Pitfalls: 読書アプリ (Offline Reading App)

**Domain:** Offline-first mobile reading app with OCR, voice memo, and on-device AI
**Tech Stack:** Dioxus + Rust, Android, NDLOCR-Lite, Moonshine Voice, Qwen3.5-08B
**Researched:** 2026-03-11
**Confidence:** HIGH (from Context7 + official docs + verified sources)

---

## Critical Pitfalls

Mistakes that cause rewrites, crashes, or app rejection on low-RAM Android devices.

### Pitfall 1: Loading Full AI Models Into Memory

**What goes wrong:** Loading NDLOCR-Lite (DEIMv2 + PARSeq), Moonshine Voice (123-245M params), and Qwen3.5-08B (8B params) models simultaneously causes OOM kills on Android Go devices (512MB-1GB RAM).

**Why it happens:**
- ONNX Runtime models load fully into RAM before inference starts
- 8B parameter model @ FP16 = ~16GB RAM just for weights
- NDLOCR-Lite layout + text models add additional hundreds of MB
- Android's Dalvik heap limits don't account for native heap usage

**Consequences:**
- App crashes with `SIGABRT` or `SIGKILL` from low memory killer
- System terminates app during OCR or voice processing
- Data loss from interrupted operations

**Prevention:**
1. **Sequential model loading** — never load OCR + Voice + LLM simultaneously
2. **Quantization** — use INT8 quantization via ONNX Runtime's `quantize_dynamic()` to reduce memory by ~75%
3. **Model sharding** — split Qwen3.5-08B across multiple inference sessions or use smaller variants (0.5B, 1.5B, 4B)
4. **Memory pressure callbacks** — implement Android's `onTrimMemory()` equivalents via android-activity crate

**Detection:**
- Monitor `VmRSS` via `/proc/self/status` in native code
- Log memory usage before/after model load
- Test on Android Go emulator with 512MB RAM

**Phase to address:** Phase 1 (Core Infrastructure) — design model loading architecture

---

### Pitfall 2: OCR Image Processing Without Downscaling

**What goes wrong:** Feeding full-resolution camera photos (12MP+ = 4032x3024) directly into NDLOCR-Lite causes memory spikes and processing timeouts.

**Why it happens:**
- NDLOCR-Lite is optimized for book/magazine digitization (document images, not photos)
- Layout detection runs on full image tensor before text recognition
- No built-in memory limit in ONNX Runtime inference session

**Consequences:**
- 12MP RGB image = ~36MB uncompressed
- Preprocessing + model input tensor = 100MB+ spike
- UI thread blocking, ANR (Application Not Responding)

**Prevention:**
1. **Pre-downscale to 2MP max** (1920x1080) before OCR — still sufficient for text
2. **Use image crate with memory-efficient codecs** — avoid loading full image into memory
3. **Stream processing** — process in tiles/chunks if image is large
4. **Preview vs. processing split** — show full-res preview but downscale for OCR

**Detection:**
- Profile memory during camera capture → OCR pipeline
- Log image dimensions before processing
- Monitor for `android.os.DeadObjectException`

**Phase to address:** Phase 2 (OCR Pipeline) — implement preprocessing

---

### Pitfall 3: JNI Reference Leaks in Native Code

**What goes wrong:** Rust code calling Java/Kotlin APIs via JNI accumulates local references without deletion, eventually exhausting the 16-slot local reference limit.

**Why it happens:**
- JNI only guarantees 16 local references per native call frame
- Creating Java objects (Strings, byte arrays) in loops without `DeleteLocalRef`
- Rust JNI bindings (`jni` crate) may not auto-cleanup in async contexts
- Long-running native operations cross multiple JNI calls

**Consequences:**
- `Fatal signal 11 (SIGSEGV)` from JNI table overflow
- App crash during camera/file operations
- Corrupted state in JNI local reference table

**Prevention:**
1. **Use `PushLocalFrame`/`PopLocalFrame`** for batches of JNI operations
2. **Explicit `DeleteLocalRef`** after every Java object creation
3. **Use `NewGlobalRef` sparingly** — convert to global only if truly needed long-term
4. **Use `DirectByteBuffer`** for large data transfers — avoids managed heap

```rust
// BAD: Creates local refs without cleanup
for path in paths {
    let jstring = env.new_string(path)?;
    // ... use jstring
} // Leaks accumulate

// GOOD: Uses frame-based cleanup
env.push_local_frame(32)?;
for path in paths {
    let jstring = env.new_string(path)?;
    // ... use jstring
}
env.pop_local_frame(None)?; // Cleans all locals
```

**Detection:**
- Enable JNI debugging: `-Xcheck:jni` in debug builds
- Log reference counts via `EnsureLocalCapacity`
- Look for `java.lang.OutOfMemoryError: JNI local reference table overflow`

**Phase to address:** Phase 1 (Core Infrastructure) — establish JNI patterns

---

### Pitfall 4: Real-Time Audio Buffer Overflow

**What goes wrong:** Moonshine Voice's streaming ASR buffers audio faster than the model processes it, causing audio data loss and transcription gaps.

**Why it happens:**
- Audio capture thread fills ring buffer continuously (16kHz, ~32KB/sec)
- Inference thread runs slower than real-time on low-end devices
- No backpressure mechanism in audio pipeline
- 245M parameter Moonshine model needs 269ms per chunk on x86 but 800ms+ on Raspberry Pi 5 (Android Go will be slower)

**Consequences:**
- Dropped audio segments = missing words in transcription
- Voice memo captures incomplete thoughts
- User frustration with unreliable voice input

**Prevention:**
1. **Implement adaptive buffer sizing** — increase buffer on slow inference
2. **Use streaming model variants** — Moonshine supports "streaming" vs "non-streaming", choose based on device capability
3. **Audio capture throttling** — drop frames if buffer > threshold instead of crashing
4. **Chunk-based processing** — process audio in 500ms-1s chunks, not continuous stream
5. **Device capability detection** — use `Tiny` (34M) model on low-RAM devices, `Small` (123M) on mid-range

**Detection:**
- Log buffer fill level vs. processing rate
- Monitor `moonshine_voice::Stream` events for `LineUpdated` frequency
- Profile inference time per audio chunk

**Phase to address:** Phase 3 (Voice Memo) — design audio pipeline

---

### Pitfall 5: SQLite Database Without Proper Cleanup

**What goes wrong:** Large BLOBs (page photos, OCR output) accumulate in SQLite without cleanup, causing database bloat and memory pressure during queries.

**Why it happens:**
- Storing both photo + OCR text + AI definitions = large rows
- SQLite stores entire row as BLOB during INSERT/SELECT processing
- No automatic vacuum/pruning configured
- Querying with large result sets loads all data into memory

**Consequences:**
- Database grows to hundreds of MB
- `SQLITE_FULL` errors when exceeding storage limits
- Slow query performance, UI freezes
- Memory spikes during `SELECT * FROM books` with cover images

**Prevention:**
1. **Separate large BLOBs from structured data** — store images in filesystem, paths in SQLite
2. **Use `PRAGMA mmap_size`** — memory-map database for better performance
3. **Regular `VACUUM`** — reclaim space after deletions
4. **Cursor-based pagination** — never load all rows at once
5. **Set runtime limits:**
```rust
sqlite3_limit(db, SQLITE_LIMIT_LENGTH, 10_000_000); // 10MB max string/BLOB
```

**Detection:**
- Monitor database file size growth
- Profile query memory usage
- Check for `SQLITE_TOOBIG` errors

**Phase to address:** Phase 1 (Core Infrastructure) — database schema design

---

### Pitfall 6: Not Handling Android Lifecycle Events

**What goes wrong:** Backgrounding the app during OCR/voice processing doesn't pause operations, leading to OOM kills by Android's low memory killer.

**Why it happens:**
- Native Activity doesn't auto-pause background threads
- ONNX Runtime sessions continue inference in background
- Audio capture continues when app is backgrounded
- Rust async tasks don't respond to Android lifecycle

**Consequences:**
- App killed mid-OCR, corrupted partial data
- Battery drain from background processing
- ANR on resume if operations piled up

**Prevention:**
1. **Implement proper lifecycle handling** via `android-activity`:
```rust
fn android_main(app: AndroidApp) {
    loop {
        app.poll_events(Some(duration), |event| {
            match event {
                PollEvent::Main(MainEvent::Pause) => {
                    // Pause OCR/voice
                }
                PollEvent::Main(MainEvent::Resume) => {
                    // Resume operations
                }
                PollEvent::Main(MainEvent::Destroy) => {
                    // Cleanup and exit
                    return;
                }
                _ => {}
            }
        });
    }
}
```
2. **Cancel long-running tasks** on `Pause`/`Stop`
3. **Save state incrementally** — don't wait for lifecycle to persist
4. **Release native resources** (ONNX sessions, audio buffers) when backgrounded

**Detection:**
- Test background/foreground transitions during OCR
- Monitor app survival in background
- Check logs for `ActivityManager: Killing ... due to memory pressure`

**Phase to address:** Phase 1 (Core Infrastructure) — lifecycle architecture

---

### Pitfall 7: Dioxus Signal Memory Retention

**What goes wrong:** Dioxus signals holding large data structures (OCR results, book library) cause memory accumulation that isn't released when components unmount.

**Why it happens:**
- Signals persist across component lifecycles by default
- Large vectors of book data retained even when view changes
- `use_signal` with large initial values never freed
- No explicit signal cleanup mechanism

**Consequences:**
- Memory grows as user navigates between views
- Eventually OOM after extended usage
- Poor performance on low-RAM devices

**Prevention:**
1. **Use `use_memo` for derived data** — recomputes on demand, doesn't store
2. **Limit signal scope** — prefer component-local signals over global
3. **Clear signals on navigation** — explicitly reset to empty/default
4. **Use `use_resource` for async data** — caches but can be invalidated
5. **Implement pagination** — don't load entire library into signals

```rust
// BAD: Global signal holding all books
global_library: Signal<Vec<Book>> = use_signal(|| load_all_books());

// GOOD: Component-local, paginated
let page = use_signal(|| 0);
let books = use_resource(move || async move {
    load_books_page(page())
});
```

**Detection:**
- Profile signal memory usage with `dioxus-logger`
- Monitor heap growth during navigation
- Check for retained large allocations

**Phase to address:** Phase 1 (Core Infrastructure) — state management patterns

---

### Pitfall 8: OCR Without Layout-Aware Preprocessing

**What goes wrong:** Feeding book pages with complex layouts (multi-column, headers, footnotes) directly into NDLOCR-Lite produces garbled reading order.

**Why it happens:**
- NDLOCR-Lite has layout detection (DEIMv2) but expects proper document images
- Camera photos have skew, shadows, curvature
- Reading order determination is separate from text recognition
- "読み順整序" module expects clean input

**Consequences:**
- Text read in wrong order (e.g., footer before main content)
- Unusable OCR output for reading workflow
- User manually re-ordering text

**Prevention:**
1. **Document preprocessing** — deskew, binarize, remove shadows
2. **ROI selection UI** — let user crop/page-detect before OCR
3. **Layout detection first** — use DEIMv2 to identify text regions
4. **Region-based OCR** — process each detected region separately
5. **Manual correction UI** — allow user to fix reading order

**Detection:**
- Test with various book layouts (vertical/horizontal, multi-column)
- Validate reading order of OCR output
- Check XML output structure

**Phase to address:** Phase 2 (OCR Pipeline) — layout handling

---

### Pitfall 9: Storing Raw PDF Content in Memory

**What goes wrong:** Loading entire PDF files into memory for NDLOCR-Lite processing causes memory spikes, especially with image-heavy PDFs.

**Why it happens:**
- PDF → image conversion loads all pages at once
- NDLOCR-Lite processes pages sequentially but input preparation doesn't
- No streaming PDF processing in typical Rust PDF crates
- Large academic PDFs can be 100MB+ with 500+ pages

**Consequences:**
- OOM on PDF import
- App freeze during "converting..." dialog
- Crash with large PDFs

**Prevention:**
1. **Page-by-page processing** — convert PDF to images one page at a time
2. **Temp file streaming** — don't hold all pages in memory
3. **PDF size limits** — warn user if PDF > 50MB
4. **Progressive import** — show progress, allow cancellation
5. **Downscale during conversion** — reduce DPI for OCR (150-200 DPI sufficient)

**Detection:**
- Profile memory during PDF import
- Test with large multi-page PDFs
- Monitor for `java.lang.OutOfMemoryError`

**Phase to address:** Phase 4 (PDF Reading) — import pipeline

---

### Pitfall 10: No On-Device Model Validation

**What goes wrong:** Shipping quantized models that haven't been validated on actual Android devices produces silent accuracy degradation or crashes.

**Why it happens:**
- Quantization (INT8) can degrade OCR accuracy significantly if not calibrated
- Model formats incompatible with mobile ONNX Runtime version
- Dynamic shapes causing allocation failures
- QNN/NNAPI execution provider not available on all devices

**Consequences:**
- OCR produces gibberish on some devices
- Voice recognition fails silently
- App crashes on specific SoC/architecture combinations

**Prevention:**
1. **Test quantized models** on representative device set (low-end, mid-range, flagship)
2. **Validate accuracy metrics** before shipping (CER/WER on test set)
3. **Graceful fallback** — CPU execution if NNAPI fails
4. **Model versioning** — detect incompatible models and refuse to load
5. **A/B testing** — canary release to subset of users

**Detection:**
- Automated testing on device farm
- Accuracy regression tests
- Crash analytics for model loading failures

**Phase to address:** Phase 6 (AI Dictionary) — model deployment

---

## Moderate Pitfalls

### Pitfall 11: Camera Preview vs. Capture Resolution Mismatch

**What goes wrong:** Camera preview uses low-res stream but capture uses full-res, confusing user about OCR quality.

**Prevention:** Show actual capture resolution in preview, or downscale capture to match preview.

**Phase:** Phase 2 (OCR Pipeline)

---

### Pitfall 12: Voice Activity Detection Without Visual Feedback

**What goes wrong:** Users don't know if voice memo is recording, leading to truncated recordings.

**Prevention:** Implement real-time visualizer showing audio levels, clear recording indicator.

**Phase:** Phase 3 (Voice Memo)

---

### Pitfall 13: Word Collection Without Deduplication

**What goes wrong:** Same word collected multiple times creates clutter, user can't see unique vocabulary.

**Prevention:** Implement fuzzy matching for word deduplication, show "already collected" indicator.

**Phase:** Phase 5 (Word Collection)

---

### Pitfall 14: No Offline-First Error Handling

**What goes wrong:** Network-related code paths (even if "offline-only") cause crashes when connectivity changes.

**Prevention:** Audit all code for network assumptions, handle all errors gracefully, no network permissions declared.

**Phase:** Phase 1 (Core Infrastructure)

---

## Phase-Specific Warnings

| Phase | Topic | Likely Pitfall | Mitigation |
|-------|-------|----------------|------------|
| Phase 1 | Core Infra | JNI reference leaks | Establish JNI patterns early, use frames |
| Phase 1 | Core Infra | Database bloat | Separate BLOBs, implement pagination |
| Phase 1 | Core Infra | Signal retention | Design state management with cleanup |
| Phase 2 | OCR | Memory spikes from large images | Downscale before processing |
| Phase 2 | OCR | Wrong reading order | Layout detection + manual correction |
| Phase 3 | Voice | Audio buffer overflow | Adaptive buffering, throttling |
| Phase 3 | Voice | Missing visual feedback | Always show recording state |
| Phase 4 | PDF | OOM on large PDFs | Page-by-page streaming |
| Phase 5 | Word Coll | Duplicate entries | Fuzzy deduplication |
| Phase 6 | AI Dict | Model OOM | Quantization, sequential loading |
| Phase 6 | AI Dict | Accuracy degradation | Device validation, fallback |

---

## Memory/Performance Implications for Android

### Low-RAM Device Strategy (Android Go: 512MB-1GB)

| Component | Standard | Android Go | Savings |
|-----------|----------|------------|---------|
| OCR Model | NDLOCR-Lite (Full) | NDLOCR-Lite (Layout-only) + Light OCR | ~200MB |
| Voice Model | Moonshine Small (123M) | Moonshine Tiny (34M) | ~89MB |
| LLM Model | Qwen3.5-4B | Qwen3.5-0.5B or none | ~3.5GB |
| Image Cache | Unbounded | 50MB limit | Variable |
| Database | Default | `PRAGMA mmap_size=10MB` | ~40MB |

### Memory Budget Allocation

```
Android Go (512MB total):
- System overhead:     ~150MB
- App Dalvik heap:     ~100MB
- Native heap usable:  ~200MB
  - UI/State:           ~30MB
  - Models (max 1):    ~150MB
  - Buffers/Work:       ~20MB
```

### Testing Recommendations

1. **Always test on Android Go emulator** (512MB RAM)
2. **Profile with Android Studio Memory Profiler**
3. **Monitor native heap** via `/proc/self/status`
4. **Stress test** — background app during OCR/voice operations
5. **Long-running test** — leave app open for 24 hours, check for leaks

---

## Sources

- Context7: /microsoft/onnxruntime — Quantization, NNAPI execution providers
- Context7: /dioxuslabs/dioxus — Signal memory patterns, reactivity
- Context7: /android/ndk — JNI reference management
- Context7: /rust-mobile/android-activity — Lifecycle handling
- Official: https://github.com/ndl-lab/ndlocr-lite — NDLOCR-Lite architecture
- Official: https://github.com/moonshine-ai/moonshine — Model sizes, latency benchmarks
- Official: https://sqlite.org/limits.html — SQLite memory limits
- Android NDK Documentation — JNI memory management patterns
