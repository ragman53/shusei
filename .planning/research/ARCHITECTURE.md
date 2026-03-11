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
