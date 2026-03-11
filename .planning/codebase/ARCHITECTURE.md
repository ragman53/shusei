# Architecture

**Analysis Date:** 2026-03-11

## Pattern Overview

**Overall:** Layered Architecture with Platform Abstraction Layer

This codebase follows a layered architecture pattern with clear separation of concerns:
- **UI Layer**: Dioxus components for cross-platform rendering
- **Core Layer**: Business logic independent of UI and platform
- **Platform Layer**: Platform-specific abstractions (Android via JNI, iOS, Desktop)

**Key Characteristics:**
- Platform-agnostic core business logic
- Trait-based abstractions for extensibility (OcrEngine, SttEngine, PlatformApi)
- Async/await pattern throughout for non-blocking operations
- Error handling via custom error enums with thiserror
- Feature flags for conditional compilation (android, ios, desktop, web, pdf, lindera)

## Layers

### Core Layer
- **Purpose**: Business logic for OCR, STT, database, and vocabulary management
- **Location**: `src/core/`
- **Contains**: Engine implementations, data models, error types, database operations
- **Depends on**: External crates (tract-onnx, rusqlite, image, ndarray, tokenizers)
- **Used by**: UI layer, Platform layer

**Sub-modules:**
- `ocr/` - OCR pipeline with NDLOCR-Lite ONNX models
- `stt/` - Speech-to-text with Moonshine Tiny models
- `db.rs` - SQLite database with FTS5 for search
- `vocab.rs` - Japanese word extraction and vocabulary management
- `pdf.rs` - PDF rendering (optional, requires `pdf` feature)
- `error.rs` - Centralized error types

### UI Layer
- **Purpose**: Dioxus-based user interface components
- **Location**: `src/ui/`
- **Contains**: Page components, shared UI components
- **Depends on**: Core layer, PlatformApi
- **Used by**: App routing layer

**Sub-modules:**
- `camera.rs` - Camera capture and OCR trigger page
- `notes.rs` - Sticky notes list and search
- `reader.rs` - PDF library and reading view
- `vocab.rs` - Vocabulary list management
- `components.rs` - Reusable UI components (Button, Card, LoadingSpinner, etc.)

### Platform Layer
- **Purpose**: Abstract platform-specific functionality (camera, microphone, file picker)
- **Location**: `src/platform/`
- **Contains**: Platform trait and implementations
- **Depends on**: Core error types
- **Used by**: UI layer for hardware access

**Sub-modules:**
- `android.rs` - JNI-based Android implementation
- `ios.rs` - iOS implementation stub (future)
- `mod.rs` - PlatformApi trait and DesktopPlatform fallback

### App Layer
- **Purpose**: Application routing and page coordination
- **Location**: `src/app.rs`
- **Contains**: Route definitions, page wrappers
- **Depends on**: UI layer components
- **Used by**: Main entry point

## Data Flow

### OCR Pipeline Flow:

1. **Image Capture**: `ui/camera.rs` → `platform/android.rs` (JNI) → Android Camera API
2. **Image Processing**: `core/ocr/preprocess.rs` → Resize, normalize, convert to tensor
3. **Text Detection**: `core/ocr/postprocess.rs` → detect_text() (TODO: tract inference)
4. **Text Recognition**: `core/ocr/postprocess.rs` → recognize_text() (TODO: tract inference)
5. **Direction Classification**: `core/ocr/postprocess.rs` → classify_direction() (TODO)
6. **Markdown Generation**: `core/ocr/markdown.rs` → Sort by reading order, format as Markdown
7. **Storage**: `core/db.rs` → Save to SQLite with FTS5 index

### STT Pipeline Flow:

1. **Audio Recording**: `platform/android.rs` → Android AudioRecord API (TODO)
2. **Audio Preprocessing**: `core/stt/engine.rs` → Mel-spectrogram computation (TODO)
3. **Encoder**: `core/stt/engine.rs` → Run encoder model (TODO: tract)
4. **Decoder**: `core/stt/decoder.rs` → Autoregressive decoding with KV cache (TODO)
5. **Tokenization**: `core/stt/tokenizer.rs` → Decode tokens to text
6. **Storage**: `core/db.rs` → Save transcript to sticky note

### Note Retrieval Flow:

1. **UI Request**: `ui/notes.rs` → User search query
2. **Database Query**: `core/db.rs` → FTS5 search or list all
3. **Display**: `ui/notes.rs` → Render NoteCard components

## Key Abstractions

### OcrEngine Trait
- **Purpose**: Abstract interface for OCR implementations
- **Location**: `src/core/ocr/engine.rs`
- **Pattern**: Trait object for dependency injection
- **Implementations**: `NdlocrEngine` (NDLOCR-Lite with tract)
- **Methods**: `process_image()`, `is_ready()`, `name()`

### SttEngine Trait
- **Purpose**: Abstract interface for speech-to-text
- **Location**: `src/core/stt/engine.rs`
- **Pattern**: Trait object with async support
- **Implementations**: `MoonshineEngine` (Moonshine Tiny with tract)
- **Methods**: `transcribe()`, `is_ready()`, `name()`, `language()`

### PlatformApi Trait
- **Purpose**: Abstract platform hardware access
- **Location**: `src/platform/mod.rs`
- **Pattern**: async_trait for async methods
- **Implementations**: `AndroidPlatform` (JNI), `IosPlatform` (stub), `DesktopPlatform` (fallback)
- **Methods**: `capture_image()`, `record_audio()`, `pick_file()`, permission checks

### Error Types
- **Purpose**: Structured error handling across layers
- **Location**: `src/core/error.rs`
- **Pattern**: thiserror-based enums with From impls
- **Types**: `ShuseiError` (top-level), `OcrError`, `SttError`

## Entry Points

### Desktop/Web Entry Point
- **Location**: `src/main.rs`
- **Triggers**: Direct execution
- **Responsibilities**: Initialize logger, launch Dioxus app

### Library Entry Point
- **Location**: `src/lib.rs`
- **Triggers**: Android JNI, iOS bindings, crate consumers
- **Responsibilities**: Export public API modules and types

### Android JNI Entry Points
- **Location**: `src/platform/android.rs`
- **Triggers**: Java MainActivity callbacks
- **Functions**:
  - `Java_com_shusei_app_MainActivity_nativeInit` - Store JavaVM
  - `Java_com_shusei_app_MainActivity_onImageCaptured` - Handle camera result

## Error Handling

**Strategy**: Structured error propagation with thiserror

**Patterns:**
- All errors convert to `ShuseiError` at boundaries
- Platform errors use `ShuseiError::Platform(String)`
- Module-specific errors (OcrError, SttError) with detailed variants
- `Result<T>` type alias for convenience

**Example:**
```rust
pub type Result<T> = std::result::Result<T, ShuseiError>;

#[derive(Error, Debug)]
pub enum ShuseiError {
    #[error("OCR error: {0}")]
    Ocr(#[from] OcrError),
    // ...
}
```

## Cross-Cutting Concerns

**Logging:** Uses `log` crate with `env_logger` initialization in `main.rs`

**Async Runtime:** Tokio with multi-threaded runtime (`Cargo.toml`)

**Feature Flags:**
- `android` - JNI support
- `ios` - iOS platform stubs
- `desktop` - Desktop-specific features
- `web` - Web platform support
- `pdf` - PDF processing via pdfium-render
- `lindera` - Japanese morphological analysis

**Configuration:**
- `Cargo.toml` - Dependencies and features
- `Dioxus.toml` - UI framework configuration
- `.cargo/config.toml` - Android cross-compilation settings

---

*Architecture analysis: 2026-03-11*
