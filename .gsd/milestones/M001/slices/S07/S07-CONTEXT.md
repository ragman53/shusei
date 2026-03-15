---
id: S07
milestone: M001
status: ready
---

# S07: Performance Polish — Context

<!-- Slice-scoped context. Milestone-only sections (acceptance criteria, completion class,
     milestone sequence) do not belong here — those live in the milestone context. -->

## Goal

Optimize OCR/PDF throughput and memory management to achieve 1-2 seconds per page processing on mid-range Android devices with graceful degradation under memory pressure.

## Why this Slice

S07 is the final slice of M001 and the MVP launch point. It validates that the performance optimizations established in prior slices (S02 image downscaling, S03 batch processing) actually work on real Android devices across RAM tiers. Without this slice, the app risks OOM crashes on 2GB RAM devices and unacceptably slow OCR on larger PDFs. This slice unblocks shipping by proving the app is production-ready.

## Scope

### In Scope

- **OCR/PDF throughput optimization** — Achieve 1-2 sec/page on 4GB RAM devices, acceptable performance on 2GB devices
- **Memory management with soft limits** — 75% RAM threshold (1.5GB on 2GB device, 3GB on 4GB device) triggers graceful degradation
- **Background resume handling** — App backgrounded mid-OCR resumes from last completed page without data loss
- **Model loading strategy** — Lazy load OCR models on first use, retain until app backgrounded
- **Performance verification** — Unit tests + manual benchmarks with real PDFs (10, 50, 100 pages) on both 2GB and 4GB+ RAM devices
- **Concurrency tuning** — Adjust batch size (currently 10) and parallel operations (currently 3) based on device tier

### Out of Scope

- **Model quantization** — INT8 conversion for OCR/AI models deferred to later optimization pass
- **Export features** — Annotation export (Markdown), backup/restore deferred to S08 or later
- **Reading statistics** — Analytics and usage tracking deferred
- **UI polish** — Loading states, transitions, visual refinements beyond performance feedback deferred
- **Thermal throttling** — Reduce concurrency when device heats up (monitoring only, no active throttling)
- **Image caching** — LRU cache for thumbnails and page previews deferred
- **Background service** — Move OCR to Android background service deferred

## Constraints

- **S07 is MVP launch slice** — Must be production-ready; no "will optimize later" features that block shipping
- **75% RAM soft limit** — Must degrade gracefully (reduce batch size, pause) rather than crash when threshold exceeded
- **Lazy model loading** — OCR models (165MB) loaded on first use, not at startup; retained until background
- **Resume from checkpoint** — Must use existing `processing_progress` table (S03) to resume interrupted conversions
- **No schema changes** — Performance optimizations must work within existing database schema
- **Backwards compatible** — Optimizations must not break books/PDFs processed in prior slices

## Integration Points

### Consumes

- `src/core/pdf.rs` — `render_pages_batch()` and `PdfConversionService` (S03) — optimize batch size and concurrency
- `src/core/ocr/engine.rs` — `process_pages_parallel()` with ONNX Runtime (S03) — tune mutex-wrapped session usage
- `src/core/db.rs` — `processing_progress` table (S03) — resume support for interrupted conversions
- `src/core/storage.rs` — Page image storage with book_id directory structure (S02) — optimize file I/O
- `src/platform/android.rs` — JNI lifecycle handlers (S01) — detect background/foreground for model unloading

### Produces

- `src/core/perf.rs` (new) — Memory monitoring, device tier detection, graceful degradation logic
- `src/core/ocr/engine.rs` (modified) — Lazy model loading, model lifecycle management
- `src/core/pdf.rs` (modified) — Adaptive batch sizing based on available memory
- `tests/perf_benchmark.rs` (new) — Manual benchmark harness for 10/50/100 page PDFs
- `.gsd/milestones/M001/slices/S07/S07-BENCHMARK.md` (new) — Performance results on 2GB and 4GB+ devices

## Open Questions

- **Model loading timing** — RESOLVED: Lazy load on first OCR use, retain in memory until app receives `onPause()` lifecycle event, then unload to free 165MB
- **Device tier detection** — How to detect 2GB vs 4GB+ RAM at runtime? Current thinking: use Android `ActivityManager.getMemoryClass()` via JNI, cache result in AppState
- **Degradation strategy** — When memory > 75% threshold: (1) reduce batch size from 10 to 5, (2) reduce concurrency from 3 to 2, (3) if still > 75%, pause and show "Device memory low" message
- **Benchmark methodology** — Use existing 373-page test PDF (S03) for stress test; create 10-page and 50-page subsets for quick iteration; measure wall-clock time and peak memory via Android profiler

---

*Context gathered: 2026-03-15*
*Status: Ready for planning*
