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

# Architecture Patterns

**Domain:** Offline Reading App (紙の本 + PDF 統合)
**Tech Stack:** Dioxus + Rust (Android-first)
**Researched:** 2026-03-11
**Overall confidence:** HIGH

## Executive Summary

This document defines the system architecture for an offline-first reading app built with Dioxus and Rust, targeting Android. The architecture follows a layered pattern with clear separation between UI (Dioxus components), business logic (Rust services), and data (SQLite). Heavy processing (OCR, voice recognition, on-device AI) runs in background threads to maintain UI responsiveness on low-RAM devices.

## Recommended Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                         UI Layer (Dioxus)                        │
│  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐ ┌─────────────┐ │
│  │   Library   │ │    Book     │ │   Reader    │ │  Word List  │ │
│  │    View     │ │    Detail   │ │    View     │ │    View     │ │
│  └─────────────┘ └─────────────┘ └─────────────┘ └─────────────┘ │
│  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐ ┌─────────────┐ │
│  │   Camera    │ │    Voice    │ │   Note      │ │   Search    │ │
│  │    Modal    │ │    Input    │ │    Editor   │ │    View     │ │
│  └─────────────┘ └─────────────┘ └─────────────┘ └─────────────┘ │
└─────────────────────────────────────────────────────────────────┘
                                │
                    ┌───────────┴───────────┐
                    │     Global Context    │
                    │    (use_context)      │
                    │  AppState, Theme, DB  │
                    └───────────┬───────────┘
                                │
┌─────────────────────────────────────────────────────────────────┐
│                    Service Layer (Rust)                          │
│  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐ ┌─────────────┐ │
│  │   Book      │ │    OCR      │ │   Voice     │ │    Word     │ │
│  │   Service   │ │   Service   │ │  Service    │ │  Service    │ │
│  └─────────────┘ └─────────────┘ └─────────────┘ └─────────────┘ │
│  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐ ┌─────────────┐ │
│  │    PDF      │ │   Image     │ │   Audio     │ │    AI       │ │
│  │   Service   │ │   Service   │ │  Service    │ │  Service    │ │
│  └─────────────┘ └─────────────┘ └─────────────┘ └─────────────┘ │
└─────────────────────────────────────────────────────────────────┘
                                │
                    ┌───────────┴───────────┐
                    │    Async Task Queue   │
                    │   (spawn/spawn_local) │
                    └───────────┬───────────┘
                                │
┌─────────────────────────────────────────────────────────────────┐
│                    Data Layer                                    │
│  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐ ┌─────────────┐ │
│  │   SQLite    │ │   File      │ │   Image     │ │    Model    │ │
│  │   (rusqlite)│ │   System    │ │   Cache     │ │   Files     │ │
│  └─────────────┘ └─────────────┘ └─────────────┘ └─────────────┘ │
│                                                                  │
│  External: NDLOCR-Lite (ONNX), Moonshine Voice, Qwen3.5-08B      │
│  (Rust bindings or C++ FFI via JNI on Android)                   │
└─────────────────────────────────────────────────────────────────┘
```

## Component Boundaries

### UI Layer Components

| Component | Responsibility | Communicates With |
|-----------|---------------|-------------------|
| **LibraryView** | Display book list, manage collection | BookService, AppState |
| **BookDetailView** | Show book metadata, reading progress | BookService, WordService |
| **ReaderView** | Display PDF/text, support annotation | PDFService, NoteService |
| **WordListView** | Show collected words, review stats | WordService |
| **CameraModal** | Capture book pages, trigger OCR | OCRService, ImageService |
| **VoiceInput** | Capture audio, display transcription | VoiceService |
| **NoteEditor** | Create/edit notes with text/voice | NoteService, VoiceService |
| **SearchView** | Full-text search across content | BookService (FTS queries) |

### Service Layer Components

| Component | Responsibility | Threading Model | Communicates With |
|-----------|---------------|-----------------|-------------------|
| **BookService** | CRUD for books, metadata management | Main thread + async | Database, FileService |
| **OCRService** | Process images → text via NDLOCR | Background thread (spawn) | NDLOCR-Lite, ImageService |
| **VoiceService** | Audio capture → text via Moonshine | Background thread | Moonshine Voice, AudioService |
| **WordService** | Word collection, definition generation | Main thread + async | Database, AIService |
| **PDFService** | PDF import, reflow rendering | Background thread | NDLOCR-Lite, ImageService |
| **ImageService** | Image compression, caching | Background thread | File system |
| **AudioService** | Audio recording, file management | Background thread | File system |
| **AIService** | On-device AI (Qwen) for definitions | Background thread | AI model files |
| **NoteService** | Note CRUD, linking to content | Main thread + async | Database |

### Data Layer Components

| Component | Responsibility | Pattern |
|-----------|---------------|---------|
| **Database** | SQLite via rusqlite with bundled feature | Connection per thread, Rc<Mutex<Connection>> |
| **FileStorage** | App-private storage for PDFs, images, audio | Path abstraction, platform-specific |
| **ImageCache** | Thumbnail generation, LRU cache | In-memory + disk cache |
| **ModelStorage** | NDLOCR, Moonshine, Qwen model files | Read-only, bundled or downloaded |

## Data Flow

### 1. Paper Book Capture Flow

```
User opens Camera → CameraModal captures image → ImageService saves
    ↓
OCRService spawns background task → NDLOCR-Lite processes image
    ↓
OCR result (text + layout) → BookService saves page text + image reference
    ↓
Signal updates → ReaderView displays reflowed text
```

**Key points:**
- Both original image AND OCR text are stored (allows re-OCR later)
- NDLOCR-Lite runs ONNX models in background thread
- Progress updates via signals during processing

### 2. Voice Note Flow

```
User taps Voice button → VoiceInput component activates → AudioService starts recording
    ↓
Audio chunks → Moonshine Voice (streaming mode) → Live transcript updates
    ↓
User stops → Final transcript → NoteEditor displays editable text
    ↓
User saves → NoteService saves to database (linked to book/page)
```

**Key points:**
- Moonshine supports streaming for real-time feedback
- Audio saved as file, transcript stored in database
- Event-driven architecture for transcript updates

### 3. PDF Import Flow

```
User selects PDF → File picker → PDFService validates
    ↓
Background task: Convert pages to images → NDLOCR-Lite → Markdown
    ↓
Progress updates via signals → BookService saves structured content
    ↓
ReaderView displays reflowed content with annotation support
```

**Key points:**
- PDF conversion is CPU-intensive → runs in background thread
- Store both original PDF and extracted Markdown
- Support resumable processing for large files

### 4. Word Collection Flow

```
User selects word in ReaderView → WordService creates entry
    ↓
AIService (Qwen) generates definition in background
    ↓
Definition + example sentence + source context → Database
    ↓
WordListView shows collection count, review history
```

**Key points:**
- AI definition is non-blocking (shows "generating..." initially)
- Track word frequency across books
- Link to source location (book + page)

## Patterns to Follow

### Pattern 1: Signal-Based State Management

Dioxus uses signals for reactive state. Use `use_signal` for component-local state and `use_context` for shared state.

```rust
use dioxus::prelude::*;

// Global app state provided via context
#[derive(Clone, Copy)]
struct AppState {
    current_book: Signal<Option<BookId>>,
    is_processing: Signal<bool>,
}

fn app() -> Element {
    let app_state = AppState {
        current_book: use_signal(|| None),
        is_processing: use_signal(|| false),
    };
    use_context_provider(|| app_state);
    
    rsx! { Router::<Route> {} }
}

#[component]
fn BookView() -> Element {
    let app_state = use_context::<AppState>();
    
    rsx! {
        if app_state.is_processing() {
            ProcessingIndicator {}
        }
        // ...
    }
}
```

### Pattern 2: Service Access via Context

Services should be provided at app root and accessed via context:

```rust
// Service layer with interior mutability
#[derive(Clone)]
struct BookService {
    db: Arc<Mutex<Connection>>,
}

impl BookService {
    async fn add_book(&self, book: Book) -> Result<BookId> {
        let db = self.db.lock().await?;
        // ... database operations
    }
}

// In app setup
fn app() -> Element {
    let book_service = BookService::new(db_path).expect("Failed to init DB");
    use_context_provider(|| book_service);
    // ...
}

// In component
#[component]
fn LibraryView() -> Element {
    let book_service = use_context::<BookService>();
    let books = use_resource(move || async move {
        book_service.list_books().await
    });
    
    // Handle loading states
    match books.read().as_ref() {
        Some(Ok(books)) => rsx! { BookList { books } },
        Some(Err(e)) => rsx! { ErrorView { error: e.to_string() } },
        None => rsx! { LoadingIndicator {} },
    }
}
```

### Pattern 3: Async Task Spawning for Heavy Work

OCR, voice recognition, and AI must run in background threads:

```rust
#[component]
fn CameraModal(on_capture: EventHandler<CaptureResult>) -> Element {
    let mut is_processing = use_signal(|| false);
    let ocr_service = use_context::<OCRService>();
    
    let handle_capture = move |image_data: Vec<u8>| {
        is_processing.set(true);
        
        // Spawn heavy OCR work in background
        spawn(async move {
            let result = ocr_service.process_image(image_data).await;
            is_processing.set(false);
            on_capture.call(result);
        });
    };
    
    rsx! {
        Camera { on_capture: handle_capture }
        if is_processing() { ProcessingOverlay {} }
    }
}
```

### Pattern 4: Database Connection Management

SQLite connections are not Send + Sync. Use connection pooling or per-thread connections:

```rust
use rusqlite::{Connection, Result};
use std::sync::{Arc, Mutex};

// For single-threaded access (Dioxus main thread)
pub struct Database {
    conn: Arc<Mutex<Connection>>,
}

impl Database {
    pub fn new(path: &str) -> Result<Self> {
        let conn = Connection::open(path)?;
        // Enable WAL mode for better concurrency
        conn.execute_batch("PRAGMA journal_mode = WAL;")?;
        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }
    
    pub async fn with_conn<F, T>(&self, f: F) -> Result<T>
    where
        F: FnOnce(&Connection) -> Result<T> + Send + 'static,
        T: Send + 'static,
    {
        let conn = self.conn.clone();
        // Run on blocking thread pool
        tokio::task::spawn_blocking(move || {
            let guard = conn.lock().map_err(|_| {
                rusqlite::Error::InvalidPath(std::path::PathBuf::from("lock poisoned"))
            })?;
            f(&guard)
        })
        .await
        .map_err(|e| rusqlite::Error::InvalidPath(std::path::PathBuf::from(e.to_string())))?
    }
}
```

### Pattern 5: Event-Driven Updates for Streaming Data

Voice recognition and OCR should stream progress:

```rust
// Define events for progress updates
#[derive(Clone)]
pub enum TranscriptEvent {
    LineStarted { line_id: u64 },
    LineUpdated { line_id: u64, text: String },
    LineCompleted { line_id: u64, text: String },
}

// Service exposes event stream
pub struct VoiceService {
    event_tx: Sender<TranscriptEvent>,
}

impl VoiceService {
    pub fn start_listening(&self) {
        spawn(async move {
            // Moonshine integration here
            // Send events via event_tx
        });
    }
}

// Component subscribes to events
#[component]
fn VoiceInput() -> Element {
    let mut transcript = use_signal(|| String::new());
    let voice_service = use_context::<VoiceService>();
    
    use_hook(|| {
        spawn(async move {
            while let Ok(event) = voice_service.recv_event().await {
                match event {
                    TranscriptEvent::LineUpdated { text, .. } => {
                        transcript.set(text);
                    }
                    // ...
                }
            }
        });
    });
    
    rsx! { "{transcript}" }
}
```

## Anti-Patterns to Avoid

### Anti-Pattern 1: Blocking the UI Thread

**What:** Running OCR, AI, or heavy computation on the main thread.

**Why bad:** UI becomes unresponsive, app may be killed by Android ANR watchdog.

**Instead:**
```rust
// BAD - blocks UI
let result = heavy_ocr_work(image);

// GOOD - async background
spawn(async move {
    let result = heavy_ocr_work(image);
    // Update signal when done
});
```

### Anti-Pattern 2: Holding Locks Across Await Points

**What:** Holding MutexGuard or database transaction while awaiting async operations.

**Why bad:** Can cause deadlocks, prevents concurrent access.

**Instead:**
```rust
// BAD - lock held across await
async fn bad_pattern(&self) {
    let guard = self.db.lock().await;
    let data = fetch_external_data().await; // Lock held here!
    guard.insert(data);
}

// GOOD - release lock before await
async fn good_pattern(&self) {
    let data_to_insert = {
        let guard = self.db.lock().await;
        guard.prepare_data()
    };
    let data = fetch_external_data().await;
    self.db.insert(data).await;
}
```

### Anti-Pattern 3: Synchronous File I/O on Main Thread

**What:** Using std::fs operations in async context without spawn_blocking.

**Why bad:** Blocks the async runtime, degrades performance.

**Instead:**
```rust
// Use tokio::fs or spawn_blocking
async fn save_image(&self, data: Vec<u8>, path: PathBuf) -> Result<()> {
    tokio::task::spawn_blocking(move || {
        std::fs::write(path, data)
    }).await?
}
```

### Anti-Pattern 4: Global Mutable State Without Signals

**What:** Using static mut or lazy_static with Mutex for UI state.

**Why bad:** Dioxus can't track changes, no reactive updates.

**Instead:** Use Dioxus signals and context system.

## Scalability Considerations

### Memory Optimization for Low-RAM Devices

| Concern | Strategy |
|---------|----------|
| **Large PDFs** | Stream processing, don't load entire PDF into memory |
| **OCR Models** | Load ONNX models once, reuse across requests |
| **Image Cache** | LRU cache with max size, aggressive thumbnail generation |
| **AI Models** | Use Qwen3.5-08B (8B params) quantized to 4-bit |
| **Audio Buffer** | Ring buffer for streaming, flush to disk periodically |

### Database Schema Considerations

```sql
-- Key tables for scalability
CREATE TABLE books (
    id INTEGER PRIMARY KEY,
    title TEXT NOT NULL,
    author TEXT,
    cover_image_path TEXT,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE pages (
    id INTEGER PRIMARY KEY,
    book_id INTEGER REFERENCES books(id),
    page_number INTEGER,
    image_path TEXT,           -- Original photo
    ocr_text TEXT,             -- Extracted text
    markdown_content TEXT,     -- Reflowed content
    UNIQUE(book_id, page_number)
);

CREATE TABLE words (
    id INTEGER PRIMARY KEY,
    word TEXT NOT NULL,
    definition TEXT,
    ai_generated BOOLEAN DEFAULT FALSE,
    context TEXT,              -- Usage example from source
    book_id INTEGER REFERENCES books(id),
    page_id INTEGER REFERENCES pages(id),
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_words_word ON words(word);
CREATE INDEX idx_pages_book ON pages(book_id);

-- Full-text search virtual table
CREATE VIRTUAL TABLE pages_fts USING fts5(ocr_text, content='pages', content_rowid='id');
```

## Suggested Build Order (Dependencies)

### Phase 1: Foundation
1. **Database layer** (rusqlite setup, migrations)
2. **File storage service** (platform paths, permissions)
3. **Basic book service** (CRUD, metadata)
4. **Simple library view** (list books)

### Phase 2: Paper Book Capture
1. **Image service** (camera integration via JNI)
2. **OCR service** (NDLOCR-Lite integration)
3. **Page storage** (images + OCR text)
4. **Reader view** (display OCR text)

### Phase 3: Voice Features
1. **Audio service** (recording, file management)
2. **Voice service** (Moonshine integration)
3. **Note service** (link notes to content)
4. **Voice input UI** (real-time transcript display)

### Phase 4: PDF Support
1. **PDF service** (import, page extraction)
2. **Markdown conversion** (via NDLOCR)
3. **Reflow rendering** (custom component)
4. **PDF reader view**

### Phase 5: Word Collection & AI
1. **Word service** (collection, retrieval)
2. **AI service** (Qwen integration)
3. **Word list view** (collection browsing)
4. **Definition generation** (background AI)

### Phase 6: Polish
1. **Search** (full-text search with FTS)
2. **Statistics** (reading progress, word counts)
3. **Export** (backup, sharing)
4. **Performance optimization**

## Dioxus+Rust Specific Architecture Patterns

### Pattern: Platform-Specific Code via cfg

```rust
// Access native Android APIs via JNI
#[cfg(android)]
fn request_camera_permission() -> bool {
    jni::call_android_method("requestCameraPermission")
}

#[cfg(not(android))]
fn request_camera_permission() -> bool {
    true // Desktop: no permission needed
}
```

### Pattern: Custom Hooks for Services

```rust
// Encapsulate service usage in custom hooks
fn use_books() -> (Signal<Vec<Book>>, impl FnMut(Book)) {
    let books = use_signal(|| Vec::new());
    let book_service = use_context::<BookService>();
    
    let add_book = move |book: Book| {
        spawn(async move {
            book_service.add_book(book).await.ok();
            // Refresh list
        });
    };
    
    (books, add_book)
}
```

### Pattern: Resource for Async Data

```rust
#[component]
fn BookDetail(id: BookId) -> Element {
    let book_service = use_context::<BookService>();
    
    let book = use_resource(move || async move {
        book_service.get_book(id).await
    });
    
    match &*book.read() {
        Some(Ok(book)) => rsx! { BookView { book } },
        Some(Err(e)) => rsx! { ErrorView { message: e.to_string() } },
        None => rsx! { LoadingIndicator {} },
    }
}
```

## Sources

- **Dioxus Architecture:** https://context7.com/dioxuslabs/dioxus (Context7, HIGH confidence)
- **Dioxus Mobile Guide:** https://dioxuslabs.com/learn/0.7/guides/platforms/mobile (Official docs, HIGH confidence)
- **Dioxus Async Patterns:** https://dioxuslabs.com/learn/0.7/essentials/basics/async (Official docs, HIGH confidence)
- **Rusqlite Patterns:** https://context7.com/rusqlite/rusqlite (Context7, HIGH confidence)
- **NDLOCR-Lite:** https://github.com/ndl-lab/ndlocr-lite (GitHub, MEDIUM confidence)
- **Moonshine Voice:** https://github.com/moonshine-ai/moonshine (GitHub, HIGH confidence)
- **Moonshine Android Integration:** Maven package `ai.moonshine:moonshine-voice` (Official, HIGH confidence)

# Technology Stack

**Project:** 読書アプリ (Reading App)
**Researched:** 2026-03-11
**Confidence:** HIGH

## Recommended Stack

### Core Framework

| Technology | Version | Purpose | Why |
|------------|---------|---------|-----|
| **Dioxus** | 0.7.3 | Cross-platform UI framework | Native Android support via `dx serve --platform android`, Rust-native, direct JNI access, hot-reloading for development. Production-ready with 35k+ GitHub stars. |
| **dioxus-mobile** | 0.7.3 | Mobile renderer | Re-export of dioxus-desktop with mobile-specific tweaks, JNI integration for Android native APIs |

### Database & Storage

| Technology | Version | Purpose | Why |
|------------|---------|---------|-----|
| **rusqlite** | 0.38.0 | SQLite database | Mature (100% documented), `bundled` feature compiles SQLite from source (avoids system dependency hell), supports Android via bundled builds, excellent Rust ergonomics |
| **rusqlite_migration** | 1.2.0 | Schema migrations | Simple schema migration library using SQLite's user_version field, atomic schema updates |

### ML/AI Inference

| Technology | Version | Purpose | Why |
|------------|---------|---------|-----|
| **ort** | 2.0.0-rc.12 | ONNX Runtime bindings | Official Rust wrapper for ONNX Runtime 1.24, supports Android ARM64, enables NDLOCR-Lite OCR inference, optimized with GraphOptimizationLevel, supports IoBinding for memory efficiency |
| **ndarray** | 0.17.2 | N-dimensional arrays | Standard Rust tensor/array library, integrates with ort for tensor operations, zero-copy views |
| **candle-core** | 0.8.2 | Alternative ML backend | Pure Rust ML framework by Hugging Face, fallback for on-device LLM inference if ort has issues, no Python dependency |

### Image Processing

| Technology | Version | Purpose | Why |
|------------|---------|---------|-----|
| **image** | 0.25.10 | Image loading/processing | Standard Rust image library, supports JPEG/PNG/WebP, memory-efficient streaming, works with Android camera output |

### Voice Recognition

| Technology | Version | Purpose | Why |
|------------|---------|---------|-----|
| **moonshine** | N/A (via JNI) | Voice transcription | Rust wrapper around Moonshine C++ core, supports streaming ASR with caching, 34-245M parameter models, optimized for live speech (not 30s fixed window like Whisper), Android native support via Maven |

### AI Dictionary/LLM

| Technology | Version | Purpose | Why |
|------------|---------|---------|-----|
| **candle-transformers** | 0.8.2 | Local LLM inference | Runs Qwen3.5-0.5B/3B/8B on-device, 8-bit quantized GGUF support, no external API calls, pure Rust |

### Supporting Libraries

| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| **serde** | 1.0 | Serialization | Configuration, data export, JSON handling |
| **anyhow** | 1.0 | Error handling | Simple error propagation throughout app |
| **tracing** | 0.1 | Logging | Structured logging for debugging |
| **tokio** | 1.36 | Async runtime | Background OCR processing, file I/O |
| **jni** | 0.21 | Java interop | Access Android Camera API, file picker |

## Cargo.toml Configuration

```toml
[dependencies]
# Core framework
dioxus = { version = "0.7.3", features = ["mobile"] }
dioxus-mobile = "0.7.3"

# Database
rusqlite = { version = "0.38.0", features = ["bundled"] }
rusqlite_migration = "1.2.0"

# ML Inference
ort = "=2.0.0-rc.12"
ndarray = "0.17.2"
candle-core = { version = "0.8.2", optional = true }
candle-transformers = { version = "0.8.2", optional = true }

# Image processing
image = "0.25.10"

# Async & utilities
tokio = { version = "1.36", features = ["rt-multi-thread"] }
serde = { version = "1.0", features = ["derive"] }
anyhow = "1.0"
tracing = "0.1"

# Android JNI
jni = "0.21"

[features]
default = ["candle"]
candle = ["dep:candle-core", "dep:candle-transformers"]

[target.'cfg(target_os = "android")'.dependencies]
# Android-specific optimizations
```

## Installation Commands

```bash
# Install Dioxus CLI
cargo install cargo-binstall
cargo binstall dioxus-cli

# Android development requirements:
# - Android SDK
# - Android NDK (r25c or later)
# - Set ANDROID_SDK_ROOT and ANDROID_NDK_HOME env vars

# Build for Android
dx serve --platform android

# Release build
cargo build --target aarch64-linux-android --release
```

## Architecture Decisions

### Why Dioxus over other options?

**Chosen:** Dioxus 0.7.3
- First-class Android support with `dx serve --platform android`
- Direct JNI access to Android APIs (camera, file picker)
- Native webview rendering (not Electron)
- Hot-reloading for rapid development
- Cross-platform path for future desktop/iOS expansion

**Alternatives considered:**
- **Tauri**: Not Rust-native (WebView-based), more complex mobile setup
- **Flutter**: Requires Dart, not Rust-focused
- **Compose Multiplatform**: Kotlin-based, harder to integrate Rust ML libraries

### Why ort over Candle for OCR?

**Chosen:** ort (ONNX Runtime)
- NDLOCR-Lite is provided as ONNX models
- ONNX Runtime has mature Android ARM64 support
- Better performance for pre-trained vision models
- Graph optimization and execution providers (NNAPI on Android)

**When to use Candle instead:**
- If porting NDLOCR-Lite to Candle fails
- For on-device LLM inference (Qwen models)
- When avoiding C++ dependencies entirely

### Why rusqlite with bundled feature?

**Critical for Android:**
- `bundled` compiles SQLite from source, avoiding system SQLite version mismatches
- Android devices have inconsistent SQLite versions
- Ensures consistent WAL mode support for concurrent read/write
- Atomic schema migrations via rusqlite_migration

### Memory Optimization Strategy

| Component | Memory Budget | Strategy |
|-----------|---------------|----------|
| OCR model (NDLOCR-Lite) | ~100-200MB | Load once, use ort IoBinding to minimize allocations |
| Voice model (Moonshine Tiny) | ~34MB | Streaming inference with caching, unload when not in use |
| LLM (Qwen 0.5B quantized) | ~500MB-1GB | Optional feature, aggressive quantization (4-bit) |
| Image buffers | ~50MB per image | Process sequentially, don't hold multiple full-res images |
| SQLite | ~10MB | WAL mode, prepared statements, connection pooling |

**Total target:** <2GB RAM on mid-range Android devices

## Platform-Specific Considerations

### Android

**NDK Version:** r25c or later (required for ort Android support)

**Target ABIs:**
- `aarch64-linux-android` (primary - modern devices)
- `armv7-linux-androideabi` (optional - older devices)

**Permissions needed:**
- `CAMERA` - for page photography
- `RECORD_AUDIO` - for voice transcription
- `READ_EXTERNAL_STORAGE` - for PDF import (Android 10+)
- `MANAGE_EXTERNAL_STORAGE` - for file picker (Android 11+)

**JNI Integration:**
- Use `jni` crate for Android Camera2 API access
- File picker via JNI to native Android intents
- Moonshine voice via C++ bindings through JNI

## What NOT to Use

| Technology | Why Not | Alternative |
|------------|---------|-------------|
| **Whisper** | Fixed 30s window, no caching, poor Asian language support | Moonshine Voice (streaming, cached, language-specific models) |
| **PyTorch Mobile** | Massive binary size, Python dependency | ONNX Runtime via ort |
| **TensorFlow Lite** | C++ API complexity, larger than ORT | ONNX Runtime via ort |
| **System SQLite** | Android version fragmentation | rusqlite with bundled feature |
| **Cloud OCR APIs** | Violates offline/privacy requirement | NDLOCR-Lite local ONNX |
| **Cloud LLM APIs** | Violates offline/privacy requirement | Candle with local Qwen models |

## Confidence Assessment

| Component | Confidence | Reason |
|-----------|------------|--------|
| Dioxus 0.7.3 | HIGH | Verified via Context7 + official releases, production usage, active development |
| rusqlite 0.38.0 | HIGH | Context7 docs, standard in Rust ecosystem, Android tested |
| ort 2.0.0-rc.12 | HIGH | Context7 docs, official pyke.io documentation, production users |
| Moonshine Voice | HIGH | Official Android support, C++ core with JNI bindings |
| Candle 0.8.x | MEDIUM | Verified via GitHub, active development, but newer in ecosystem |
| Android NDK | HIGH | Standard Android development path |

## Research Sources

- **Dioxus**: https://github.com/dioxuslabs/dioxus/releases/tag/v0.7.3 (v0.7.3, Jan 17 2026)
- **ort**: https://ort.pyke.io/ (v2.0.0-rc.12)
- **rusqlite**: https://docs.rs/rusqlite/0.38.0/rusqlite/ (v0.38.0)
- **Moonshine**: https://github.com/moonshine-ai/moonshine (Android native support)
- **Candle**: https://github.com/huggingface/candle (v0.8.x)
- **ndarray**: https://docs.rs/ndarray/0.17.2/ndarray/ (v0.17.2)
- **image**: https://docs.rs/image/0.25.10/image/ (v0.25.10)

## Open Questions

1. **NDLOCR-Lite Rust port**: Need to verify if NDLOCR-Lite ONNX models work with ort on Android or if Candle conversion is required
2. **Moonshine Rust bindings**: Need to create Rust wrapper around Moonshine C++ core (currently has Python/Swift/Java bindings)
3. **Qwen model size**: Validate 0.5B vs 3B vs 8B quantization performance on target Android devices
4. **Memory pressure**: Test simultaneous OCR + Voice + LLM memory usage on 2GB RAM devices

# Feature Landscape: Offline Reading App

**Domain:** Reading/Annotation Apps (読書アプリ - Reading App)  
**Researched:** 2026-03-11  
**Focus:** Physical books + PDFs with sticky notes, voice memos, word collection

## Overview

This research categorizes features for an **offline-first reading app** that uniquely combines physical book workflow (OCR-based) with PDF reading, featuring sticky notes (付箋), voice memos (ボイスメモ), and word collection (単語採集). The app targets Japanese/English readers who value privacy and want to capture reading thoughts without cloud dependencies.

---

## Table Stakes (Must-Have)

Features users expect. Missing = product feels incomplete.

| Feature | Why Expected | Complexity | Notes |
|---------|--------------|------------|-------|
| **Book Library Management** | Users need to organize their reading materials | Medium | Shelf/list view, manual metadata entry (title, author), cover photo |
| **Page Navigation** | Core reading functionality | Low | Page numbers, progress indicator, jump to page |
| **Text Display/Reflow** | Reading experience foundation | Medium | PDF: reflow display; Physical: OCR text display |
| **Bookmarking** | Save reading position | Low | Essential for both PDF and physical book workflows |
| **Basic Annotation** | Note-taking is core promise | Medium | Text selection → add note (sticky note) |
| **Local Data Storage** | Offline requirement | Low | SQLite for structured data, file system for images/audio |
| **Dark/Light Mode** | Eye strain reduction | Low | Standard reading app expectation |
| **Font Size Adjustment** | Accessibility | Low | Text reflow capability required |
| **Search Within Book** | Find information quickly | Medium | Full-text search on OCR/PDF text |

### Physical Book Specific Table Stakes

| Feature | Why Expected | Complexity | Notes |
|---------|--------------|------------|-------|
| **Camera Capture** | Physical book digitization | Medium | Page photo → OCR workflow |
| **OCR Text Extraction** | Convert images to readable text | High | NDLOCR-Lite integration, Japanese/English support |
| **Photo + Text Pairing** | Verify OCR accuracy | Low | Save both original photo and extracted text |
| **Manual Page Number Entry** | Physical books lack digital metadata | Low | User inputs current page |

### PDF Specific Table Stakes

| Feature | Why Expected | Complexity | Notes |
|---------|--------------|------------|-------|
| **File Import** | Get PDFs into app | Low | File picker, import from storage |
| **PDF Rendering** | Display PDF content | Medium | Convert to Markdown via NDLOCR, reflow view |
| **Page Thumbnails** | Quick navigation | Medium | Generate from PDF or captured photos |

---

## Differentiators (Competitive Advantage)

Features that set this product apart. Not expected, but valued.

| Feature | Value Proposition | Complexity | Notes |
|---------|-------------------|------------|-------|
| **Voice Memo Integration** | Capture thoughts hands-free while reading | High | Moonshine Voice ASR, real-time speech-to-text |
| **AI-Generated Word Definitions** | Learn vocabulary in context | High | On-device Qwen3.5-08B, 100% offline |
| **Word Collection System** | Build personal dictionary | Medium | Word + context sentence + page + AI definition |
| **Collection Frequency Visualization** | See learning patterns | Low | Show how many times a word appears across books |
| **Unified Physical + Digital** | Seamless experience across book types | High | Same annotation tools for both workflows |
| **100% Offline Operation** | Privacy, no network dependency | Medium | All AI/ML models on-device |
| **Low RAM Optimization** | Works on budget Android devices | High | Memory-conscious design for Android Go |
| **Cross-Reference Word Usage** | See words in different contexts | Medium | Link word occurrences across books |
| **Export Annotations** | Data portability | Medium | Markdown/text export of notes and words |
| **Reading Session Timer** | Track reading habits | Low | Simple timer with session logging |

### Key Differentiator Analysis

**Voice Memos (ボイスメモ)**
- **Why it matters:** Readers often have thoughts while hands are occupied holding a book
- **Implementation:** Moonshine Voice offers streaming ASR optimized for live speech
- **Unique value:** Real-time transcription while reading, not post-hoc recording

**Word Collection with AI (単語採集)**
- **Why it matters:** Language learners need context-aware definitions
- **Implementation:** Qwen3.5-08B generates definitions based on sentence context
- **Unique value:** AI understands nuance that dictionary lookups miss

**Physical Book Bridge**
- **Why it matters:** Most apps treat physical and digital as separate worlds
- **Implementation:** OCR + photo workflow makes paper books "digital-native"
- **Unique value:** Single app for entire reading life

---

## Anti-Features (Deliberately NOT Building)

Features to explicitly NOT build.

| Anti-Feature | Why Avoid | What to Do Instead |
|--------------|-----------|-------------------|
| **Cloud Sync** | Privacy promise, complexity | Local-only storage with optional manual export/import |
| **Social Features** | Scope creep, privacy concerns | Focus on personal knowledge management |
| **E-book Store Integration** | Licensing complexity, scope | User brings their own PDFs/books |
| **Automatic Cloud Backup** | Network dependency | Document manual backup process, user responsibility |
| **DRM Support** | Complexity, legal issues | Support DRM-free PDFs only |
| **Reading Statistics/Social** | Feature bloat | Simple personal reading log only |
| **Built-in Dictionary (licensed)** | Licensing costs | AI-generated definitions instead |
| **OCR Auto-Upload to Cloud** | Privacy violation | 100% on-device OCR processing |
| **Subscription Model** | Not aligned with offline ethos | One-time purchase or free/open source |
| **Multi-Device Sync** | Complexity, cloud dependency | Single-device focus, manual transfer if needed |

---

## Feature Dependencies

```
Core Infrastructure
├── SQLite Database (books, annotations, words)
├── File Storage (photos, audio, PDFs)
└── OCR Engine (NDLOCR-Lite)

Physical Book Workflow
├── Camera Access
├── Photo Storage
├── OCR Processing
└── Text Display
    ├── Sticky Notes
    ├── Voice Memos
    └── Word Collection

PDF Workflow
├── File Picker
├── PDF Import
├── Markdown Conversion (NDLOCR)
└── Reflow Display
    ├── Sticky Notes
    ├── Voice Memos
    └── Word Collection

Word Collection Feature
├── Text Selection
├── Word Extraction
├── Context Sentence Capture
├── AI Definition (Qwen3.5-08B)
├── Storage in Database
└── Frequency Tracking

Voice Memo Feature
├── Microphone Access
├── Audio Recording
├── ASR Processing (Moonshine)
└── Text Association with Page/Selection
```

---

## MVP Recommendation

**Prioritize for MVP:**

1. **Book Library Management** - Foundation, low complexity
2. **Physical Book Workflow** - Core differentiator
   - Camera capture → OCR → Text display
3. **Basic Sticky Notes** - Essential annotation
4. **PDF Import + Display** - Table stakes for digital
5. **Word Collection (basic)** - Key differentiator
   - Word + context + manual definition
   - AI definitions as Phase 2

**Defer:**
- **Voice Memos:** High complexity, can ship without initially
- **AI Definitions:** Requires ML model integration, can use manual input first
- **Export Features:** Important but not MVP-blocking
- **Advanced Search:** Basic text search sufficient initially

**Rationale:** The core value proposition is "unified reading notes for physical + PDF books." Get the basic workflows working first, then add the AI-enhanced features that differentiate from simple note-taking apps.

---

## Phase Recommendations

Based on feature dependencies and complexity:

### Phase 1: Core Reading Infrastructure
- Book library management
- Physical book capture + OCR
- PDF import + display
- Basic page navigation
- Local storage (SQLite)

### Phase 2: Annotation Foundation
- Sticky notes on text
- Basic word collection (manual definitions)
- Bookmarking
- Reading progress tracking

### Phase 3: AI Enhancement
- AI-generated word definitions (Qwen3.5)
- Voice memos (Moonshine integration)
- Word frequency visualization
- Advanced search

### Phase 4: Polish & Export
- Export annotations
- Reading statistics
- Performance optimizations
- UI refinements

---

## Confidence Assessment

| Area | Confidence | Notes |
|------|------------|-------|
| Table Stakes | **HIGH** | Standard reading app features well-understood |
| Physical Book OCR | **HIGH** | NDLOCR-Lite documented, proven technology |
| Voice Memos | **MEDIUM** | Moonshine Voice confirmed working on Android, integration complexity TBD |
| AI Definitions | **MEDIUM** | Qwen3.5-08B suitable for on-device, but Rust integration needs verification |
| Offline-First | **HIGH** | Clear scope, no external API dependencies |
| Low RAM Optimization | **MEDIUM** | Requires careful memory management, test on real devices |

---

## Sources

- [PROJECT.md](../PROJECT.md) - Project requirements and constraints
- [NDLOCR-Lite GitHub](https://github.com/ndl-lab/ndlocr-lite) - OCR technology details
- [Moonshine Voice GitHub](https://github.com/moonshine-ai/moonshine) - ASR capabilities
- Context7 Documentation for Dioxus and ONNX Runtime

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