---
id: S07
milestone: M001
status: ready
---

# S07: Performance Polish — Context

<!-- Slice-scoped context. Milestone-only sections (acceptance criteria, completion class,
     milestone sequence) do not belong here — those live in the milestone context. -->

## Goal

Migrate ONNX inference from `ort` to `tract` for NDLOCR and Moonshine models, implement memory optimization to stay under 1GB during heavy operations, and ensure no performance regressions.

## Why this Slice

S07 resolves the pre-existing ONNX Runtime linker bug (`ort-sys` undefined symbols) that has blocked test execution and runtime inference across S02, S03, S04, and S05. By switching to `tract` (already in the dependency tree for NDLOCR), the app gains consistent cross-platform ONNX inference without native linking issues. This slice unblocks:
- S02 OCR pipeline (currently uses placeholder)
- S03 PDF conversion (currently uses hayro fallback)
- S05 Voice Memos (model loading blocked)
- Android deployment (ort linking fails on some systems)

The order matters: S06 (AI Enhancement) depends on working inference before adding Qwen LLM models. S07 ensures the foundation is stable before S06's memory-intensive LLM integration.

## Scope

### In Scope

- **Tract migration for NDLOCR** — Replace `ort` with `tract-onnx` for text detection and recognition models
- **Tract migration for Moonshine** — Replace `ort` with `tract-onnx` for encoder/decoder inference
- **PaddleOCR removal** — Delete unused PaddleOCR models and integration code (not adopted)
- **Memory optimization** — Batch processing for large PDFs (100+ pages), lazy model loading, memory budget < 1GB
- **Auto-retry logic** — Failed OCR/transcription operations retry once with reduced batch size
- **Unit tests** — Prove tract inference works with existing test suite
- **Build verification** — `cargo test --lib` passes, Android APK builds successfully

### Out of Scope

- **Qwen LLM integration** — Belongs to S06 (AI Enhancement), not S07
- **PaddleOCR support** — Explicitly not adopted, code removed
- **UX polish** — No animations, transitions, or visual refinements (backend-only slice)
- **Battery optimization** — Deferred to later performance work
- **Flagship device optimization** — Target is mid-range (2-4GB RAM), not 6GB+ flagship
- **Real device testing** — Desktop tests + Android build sufficient; physical device testing deferred

## Constraints

- **No regressions** — Tract must match or exceed current ort performance (speed, accuracy)
- **Memory budget** — Stay under 1GB during heavy operations (large PDF OCR, voice transcription)
- **Cross-platform consistency** — Tract must work on both desktop (Linux/macOS/Windows) and Android
- **Fallback to ort** — If tract hits major blockers, investigate and fix ort linker issue as backup
- **Lazy model loading** — Models loaded on first use, retained until app backgrounded (Android lifecycle)

## Integration Points

### Consumes

- `src/core/ocr/engine.rs` — Existing NDLOCR engine trait and result structures
- `src/core/stt/engine.rs` — Existing Moonshine engine with mel-spectrogram preprocessing
- `src/core/pdf.rs` — Batch processing infrastructure (10 pages/batch)
- `src/platform/android.rs` — Lifecycle handlers for model unloading on `onPause()`

### Produces

- `src/core/ocr/engine.rs` (modified) — Tract-based NDLOCR inference implementation
- `src/core/stt/engine.rs` (modified) — Tract-based Moonshine encoder/decoder inference
- `src/core/tract_utils.rs` (new) — Shared tract helpers (tensor conversion, session management)
- `assets/ocr/models/` — NDLOCR ONNX models (DEIMv2, PARSeq) verified with tract
- `assets/models/moonshine/` — Moonshine Tiny ONNX models verified with tract
- Unit tests proving tract inference works without linker errors

## Open Questions

- **Tract operator support** — NDLOCR and Moonshine may use operators not supported by tract; need to verify compatibility during implementation. Current thinking: test model loading early, have fallback to ort if unsupported ops found.

- **Quantization strategy** — Tract supports INT8 quantization for memory reduction, but may degrade accuracy. Current thinking: start with FP32 models, add INT8 quantization only if memory exceeds 1GB budget.

- **Android NNAPI acceleration** — Tract supports execution providers; NNAPI could accelerate inference on Android. Current thinking: defer to S07-follow-up, prioritize CPU inference stability first.

- **Model file format** — Tract may require specific ONNX opset versions. Current thinking: test existing models first, convert opset if needed using ONNX model zoo or export scripts.

---

*Context gathered: 2026-03-15*
*Status: Ready for planning*
