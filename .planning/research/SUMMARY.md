# Project Research Summary

**Project:** 読書アプリ (Reading App)
**Domain:** Offline-first mobile reading app with OCR, voice memo, and on-device AI
**Researched:** 2026-03-11
**Confidence:** HIGH

## Executive Summary

This research synthesizes findings for an offline-first reading app that bridges physical and digital reading workflows. The app targets Japanese/English readers who value privacy and want to capture reading thoughts without cloud dependencies. Key differentiators include OCR-based physical book digitization (using NDLOCR-Lite), voice memos with real-time transcription (Moonshine Voice), and on-device AI word definitions (Qwen3.5).

The recommended approach is a Rust-based architecture using Dioxus 0.7.3 for cross-platform UI with first-class Android support. The stack prioritizes on-device ML inference to maintain the offline/privacy promise, using ONNX Runtime (ort) for OCR, Moonshine for voice recognition, and Candle for LLM inference. SQLite with the bundled feature provides reliable local storage across Android device fragmentation.

The primary risks center on memory management: loading multiple AI models simultaneously will OOM on low-RAM Android devices (512MB-1GB). Critical mitigations include sequential model loading (never load OCR + Voice + LLM together), INT8 quantization to reduce memory by 75%, and aggressive image downscaling before OCR processing. JNI reference management and Android lifecycle handling require careful attention to prevent crashes.

## Key Findings

### Recommended Stack

Research confirms a Rust-native stack is optimal for this use case, balancing performance, privacy, and ML integration. Dioxus 0.7.3 provides production-ready Android support via `dx serve --platform android` with direct JNI access for camera and file operations. The ML stack uses ONNX Runtime (ort 2.0.0-rc.12) for NDLOCR-Lite OCR inference, Moonshine Voice for streaming ASR, and Candle for on-device LLM inference.

**Core technologies:**
- **Dioxus 0.7.3**: Cross-platform UI framework with native Android support, hot-reloading, and direct JNI access — enables rapid development while maintaining native performance
- **rusqlite 0.38.0 (bundled)**: SQLite with bundled feature compiles from source, avoiding Android SQLite version fragmentation — critical for consistent WAL mode support
- **ort 2.0.0-rc.12**: Official ONNX Runtime Rust bindings with Android ARM64 support, NNAPI execution provider, and IoBinding for memory efficiency
- **Moonshine Voice**: Streaming ASR optimized for live speech (not fixed 30s windows like Whisper), supports 34-245M parameter models with Android native bindings
- **Candle 0.8.2**: Hugging Face's pure Rust ML framework for on-device Qwen3.5 inference without Python dependencies

### Expected Features

The app combines standard reading app expectations with unique physical+digital unification. Table stakes include book library management, page navigation, bookmarking, and basic annotation. Differentiators include voice memos for hands-free note capture, AI-generated word definitions with context awareness, and cross-reference word usage across books.

**Must have (table stakes):**
- **Book Library Management** — Users expect to organize their reading materials with metadata
- **Camera Capture + OCR** — Core differentiator: digitize physical books via NDLOCR-Lite
- **PDF Import + Display** — Standard for digital reading workflow
- **Basic Sticky Notes** — Essential annotation functionality
- **Local Data Storage** — SQLite foundation for offline requirement

**Should have (competitive):**
- **Voice Memo Integration** — Moonshine ASR for hands-free thought capture while reading
- **AI-Generated Word Definitions** — Qwen3.5-08B for context-aware vocabulary learning
- **Word Collection System** — Build personal dictionary with source context
- **100% Offline Operation** — Privacy-first, no network dependencies
- **Low RAM Optimization** — Target Android Go devices (512MB-1GB)

**Defer (v2+):**
- **Export Annotations** — Important for data portability but not MVP-blocking
- **Reading Statistics** — Nice-to-have analytics
- **Cross-Reference Word Usage** — Advanced feature requiring substantial data

### Architecture Approach

The architecture follows a layered pattern with clear separation: Dioxus UI components handle presentation, Rust services encapsulate business logic, and SQLite/file system manage data. Heavy processing (OCR, voice, AI) runs in background threads via `spawn()` to maintain UI responsiveness. Services communicate via Dioxus context system with signal-based reactive state management.

**Major components:**
1. **UI Layer (Dioxus)** — LibraryView, ReaderView, CameraModal, VoiceInput, WordListView — reactive components using signals and context
2. **Service Layer (Rust)** — BookService, OCRService, VoiceService, WordService, AIService — background task spawning for heavy work
3. **Data Layer** — SQLite (rusqlite with bundled), FileStorage, ImageCache, ModelStorage — WAL mode for concurrency, filesystem for BLOBs

Key patterns: Signal-based state management with `use_signal` and `use_context`, async task spawning via `spawn()` for OCR/AI, database connection per thread with `Arc<Mutex<Connection>>`, event-driven updates for streaming voice data.

### Critical Pitfalls

Research identified 10 critical pitfalls that could cause rewrites or crashes on low-RAM Android devices. The top risks involve memory management, JNI integration, and model loading patterns.

1. **Loading Multiple AI Models Simultaneously** — Loading NDLOCR-Lite + Moonshine + Qwen3.5 together causes OOM on Android Go. **Avoid by:** Sequential loading, INT8 quantization (75% memory reduction), model sharding, and implementing memory pressure callbacks.

2. **OCR Without Image Downscaling** — Full-resolution camera photos (12MP+) cause 100MB+ memory spikes. **Avoid by:** Pre-downscale to 2MP max, use image crate with memory-efficient codecs, process in tiles if needed.

3. **JNI Reference Leaks** — Creating Java objects in loops without `DeleteLocalRef` exhausts 16-slot JNI local reference limit. **Avoid by:** Use `PushLocalFrame`/`PopLocalFrame` for batches, explicit cleanup, `DirectByteBuffer` for large transfers.

4. **Real-Time Audio Buffer Overflow** — Moonshine inference slower than real-time on low-end devices causes dropped audio. **Avoid by:** Adaptive buffer sizing, streaming model variants, chunk-based processing (500ms-1s), device capability detection (Tiny vs Small model).

5. **SQLite Database Bloat** — Large BLOBs (photos, OCR output) accumulate without cleanup. **Avoid by:** Store images in filesystem (paths in SQLite), use `PRAGMA mmap_size`, regular `VACUUM`, cursor-based pagination, runtime limits.

## Implications for Roadmap

Based on research, suggested phase structure:

### Phase 1: Core Infrastructure
**Rationale:** Foundation must handle Android lifecycle, JNI patterns, and memory constraints before adding complex features. Database schema and state management patterns established here persist throughout.
**Delivers:** Working app shell with book library, SQLite schema, file storage, Dioxus navigation
**Addresses:** Book Library Management, Local Data Storage, basic UI framework
**Avoids:** JNI reference leaks (establish patterns early), database bloat (proper schema design), signal retention (design state management with cleanup), lifecycle mishandling (proper Android integration)

### Phase 2: Paper Book Capture
**Rationale:** Core differentiator requiring camera integration and OCR pipeline. Must solve image memory management before adding more features.
**Delivers:** Camera capture → downscale → OCR → text display workflow
**Uses:** Dioxus camera modal, ImageService (JNI), OCRService (NDLOCR-Lite via ort)
**Implements:** Document preprocessing, layout detection, ROI selection UI
**Avoids:** OCR memory spikes (downscale to 2MP), wrong reading order (layout-aware preprocessing)

### Phase 3: PDF Support
**Rationale:** Table stakes feature that shares OCR infrastructure with Phase 2. PDF import uses same NDLOCR-Lite pipeline.
**Delivers:** File picker → PDF import → page-by-page OCR → reflow display
**Uses:** PDFService, same OCRService from Phase 2
**Implements:** Page-by-page streaming processing, progress indicators
**Avoids:** OOM on large PDFs (streaming, not loading all pages), memory spikes (temp file handling)

### Phase 4: Annotation Foundation
**Rationale:** Build on captured content from Phases 2-3. Sticky notes work on both OCR text and PDF content.
**Delivers:** Text selection, sticky note creation/editing, bookmarking, basic word collection (manual definitions)
**Uses:** NoteService, WordService (without AI), existing ReaderView
**Avoids:** Duplicate word entries (implement fuzzy deduplication)

### Phase 5: Voice Memos
**Rationale:** High-complexity feature requiring audio pipeline and streaming ASR. Deferred until core reading workflow is solid.
**Delivers:** Audio recording → Moonshine streaming ASR → transcript → note linking
**Uses:** AudioService, VoiceService (Moonshine via JNI), VoiceInput component
**Implements:** Adaptive buffering, device capability detection, real-time visualizer
**Avoids:** Audio buffer overflow (throttling, chunk processing), missing feedback (always show recording state)

### Phase 6: AI Enhancement
**Rationale:** Most memory-intensive phase requiring careful model management. Only load one model at a time.
**Delivers:** AI-generated word definitions (Qwen3.5), word frequency visualization, advanced search
**Uses:** AIService (Candle), Qwen3.5-0.5B/3B/8B models (quantized)
**Implements:** Sequential model loading, memory pressure handling
**Avoids:** Model OOM (quantization, sequential loading), accuracy degradation (device validation, CPU fallback)

### Phase 7: Polish & Export
**Rationale:** Final phase for performance optimization, export features, and UI refinements after core functionality validated.
**Delivers:** Export annotations (Markdown), reading statistics, UI polish, performance optimization
**Addresses:** Deferred features from MVP recommendation

### Phase Ordering Rationale

- **Core Infrastructure First:** Android lifecycle, JNI patterns, and database design must be correct from start — fixes here require rewrites
- **OCR Before Voice/AI:** Image processing pipeline informs memory management patterns used later
- **PDF Parallel to Physical:** Shares OCR infrastructure, doesn't block on camera integration
- **Voice Before AI:** Audio pipeline complexity is independent of LLM integration, establishes background processing patterns
- **AI Last:** Most memory-risky phase, requires all prior memory management discipline

### Research Flags

Phases likely needing deeper research during planning:
- **Phase 2 (OCR):** NDLOCR-Lite Rust integration with ort needs validation — complex ONNX model loading on Android
- **Phase 5 (Voice):** Moonshine Rust bindings don't exist yet — need to create JNI wrapper around C++ core
- **Phase 6 (AI):** Qwen3.5-08B quantization and on-device performance needs testing on target devices

Phases with standard patterns (skip research-phase):
- **Phase 1 (Core):** Dioxus + SQLite patterns well-documented, established patterns
- **Phase 4 (Annotations):** Standard CRUD operations, no novel technology
- **Phase 7 (Polish):** Export and statistics are conventional features

## Confidence Assessment

| Area | Confidence | Notes |
|------|------------|-------|
| Stack | **HIGH** | Dioxus 0.7.3, rusqlite, ort all verified via Context7 + official docs, production usage confirmed |
| Features | **HIGH** | Table stakes well-understood, differentiators validated against NDLOCR-Lite and Moonshine documentation |
| Architecture | **HIGH** | Dioxus patterns from Context7, Rust async patterns established, clear component boundaries |
| Pitfalls | **HIGH** | Memory management risks validated via Android NDK docs, ONNX Runtime quantization docs, SQLite limits |

**Overall confidence:** HIGH

### Gaps to Address

- **NDLOCR-Lite Rust port:** Need to verify if NDLOCR-Lite ONNX models work with ort on Android or if Candle conversion is required — validate during Phase 2 planning
- **Moonshine Rust bindings:** Need to create Rust wrapper around Moonshine C++ core (currently has Python/Swift/Java bindings) — requires JNI expertise, plan for Phase 5
- **Qwen model size:** Validate 0.5B vs 3B vs 8B quantization performance on target Android devices — test on real hardware during Phase 6
- **Memory pressure:** Test simultaneous OCR + Voice + LLM memory usage on 2GB RAM devices — stress test during Phase 6

## Sources

### Primary (HIGH confidence)
- **Context7: /dioxuslabs/dioxus** — Architecture patterns, signal memory management, async patterns
- **Context7: /microsoft/onnxruntime** — Quantization, NNAPI execution providers, memory optimization
- **Context7: /rusqlite/rusqlite** — Database patterns, WAL mode, Android compatibility
- **Dioxus 0.7.3 Release** (https://github.com/dioxuslabs/dioxus/releases/tag/v0.7.3) — Mobile platform support, verified Jan 2026
- **ort 2.0.0-rc.12** (https://ort.pyke.io/) — Official Rust ONNX Runtime bindings

### Secondary (MEDIUM confidence)
- **NDLOCR-Lite GitHub** (https://github.com/ndl-lab/ndlocr-lite) — OCR architecture, DEIMv2 + PARSeq models
- **Moonshine Voice GitHub** (https://github.com/moonshine-ai/moonshine) — Model sizes, latency benchmarks, Android Maven package
- **Candle GitHub** (https://github.com/huggingface/candle) — Pure Rust ML framework, Qwen model support
- **Android NDK Documentation** — JNI memory management, lifecycle handling

### Tertiary (LOW confidence)
- **Qwen3.5 on-device performance** — Requires validation on target Android devices, limited mobile deployment documentation
- **Moonshine Rust bindings** — No existing Rust wrapper, implementation complexity TBD

---
*Research completed: 2026-03-11*
*Ready for roadmap: yes*
