# Architecture

**Analysis Date:** 2026-03-11

## Pattern Overview

**Overall:** Layered architecture with platform abstraction

**Key Characteristics:**
- Core business logic is platform-agnostic (`src/core/`)
- UI layer uses Dioxus reactive framework (`src/ui/`)
- Platform-specific code behind trait abstraction (`src/platform/`)
- Async-first design with Tokio runtime
- Trait-based engine interfaces for OCR/STT enabling swappable implementations

## Layers

**Platform Abstraction Layer:**
- Purpose: Abstract platform-specific functionality (camera, microphone, permissions)
- Location: `src/platform/`
- Contains: `PlatformApi` trait, Android JNI implementation, iOS stub, desktop fallback
- Depends on: JNI (Android), async-trait
- Used by: UI components via `get_platform_api()`

**Core Business Logic:**
- Purpose: Platform-agnostic business logic and data processing
- Location: `src/core/`
- Contains: OCR engine, STT engine, database operations, vocabulary management, error types
- Depends on: tract-onnx, rusqlite, serde, tokenizers
- Used by: UI layer, platform callbacks

**UI/Presentation Layer:**
- Purpose: User interface components and routing
- Location: `src/ui/`, `src/app.rs`
- Contains: Dioxus components, page views, shared UI components
- Depends on: dioxus, dioxus-router, platform API
- Used by: Dioxus runtime

**Entry Point:**
- Purpose: Application bootstrap
- Location: `src/main.rs`
- Contains: Logger initialization, Dioxus launch
- Depends on: app module, env_logger
- Used by: OS/runtime

## Data Flow

**Camera Capture → OCR → Storage:**

1. User clicks "Take Photo" in `src/ui/camera.rs`
2. `CameraPage` calls `platform.capture_image()` via `get_platform_api()`
3. On Android: JNI calls `MainActivity.startCameraCapture()` → captures image → `onImageCaptured()` JNI callback → Rust receives `CameraResult`
4. UI displays captured image, user clicks "Run OCR"
5. `OcrEngine::process_image()` processes image data (via `NdlocrEngine`)
6. OCR result (markdown + plain text) returned to UI
7. User clicks "Save as Note" → `Database::create_sticky_note()` persists to SQLite

**Audio Recording → STT → Storage:**

1. UI calls `platform.record_audio()` 
2. Platform captures audio, returns `AudioResult` with PCM samples
3. `SttEngine::transcribe()` processes audio (via `MoonshineEngine`)
4. Transcribed text returned to UI
5. Result stored in database via `Database`

**Vocabulary Extraction:**

1. Text input (from OCR or manual) → `WordExtractor::extract_words()`
2. Morphological analysis (lindera for Japanese, whitespace split for English)
3. `VocabularyEntry` created and stored via `Database`

**State Management:**
- Database state: SQLite via `rusqlite` with FTS5 for full-text search
- UI state: Dioxus signals (`use_signal`) for reactive state
- Engine state: Struct fields with `initialized` flags

## Key Abstractions

**`OcrEngine` trait:**
- Purpose: Abstract OCR processing interface
- Examples: `src/core/ocr/engine.rs` (`NdlocrEngine` implements this)
- Pattern: Async trait with `process_image()`, `is_ready()`, `name()` methods

**`SttEngine` trait:**
- Purpose: Abstract speech-to-text interface
- Examples: `src/core/stt/engine.rs` (`MoonshineEngine` implements this)
- Pattern: Async trait with `transcribe()`, `is_ready()`, `language()` methods

**`PlatformApi` trait:**
- Purpose: Abstract platform capabilities (camera, mic, permissions, file picker)
- Examples: `src/platform/mod.rs` (`AndroidPlatform`, `IosPlatform`, `DesktopPlatform`)
- Pattern: Async trait with platform-specific implementations selected at compile-time via `cfg` attributes

**Error Hierarchy:**
- Purpose: Unified error handling across layers
- Examples: `src/core/error.rs`
- Pattern: `ShuseiError` enum with `#[from]` attributes for automatic conversion from underlying errors (rusqlite, io, serde_json, etc.)

## Entry Points

**`src/main.rs`:**
- Location: `src/main.rs`
- Triggers: OS/binary execution
- Responsibilities: Initialize logger, launch Dioxus app with `dioxus::launch(app::App)`

**`src/lib.rs`:**
- Location: `src/lib.rs`
- Triggers: Library imports
- Responsibilities: Re-export core types (`ShuseiError`, `OcrEngine`, `SttEngine`, `Database`, `PlatformApi`)

**JNI Entry Points (Android):**
- Location: `src/platform/android.rs`
- Triggers: Java native method calls
- Responsibilities: `Java_com_shusei_app_MainActivity_nativeInit`, `Java_com_shusei_app_MainActivity_onImageCaptured`

**Java Entry Point (Android):**
- Location: `platform/android/app/src/main/java/com/shusei/app/MainActivity.java`
- Triggers: Android lifecycle
- Responsibilities: Camera2 API management, JNI bridge, permission handling

## Error Handling

**Strategy:** Result-based with custom error types

**Patterns:**
- `Result<T>` type alias for `std::result::Result<T, ShuseiError>` (`src/core/error.rs`)
- `thiserror` derive macros for automatic error conversion
- Error propagation via `?` operator
- UI-level error display via `ErrorMessage` component (`src/ui/components.rs`)
- Logging via `log::error!`/`log::info!` before returning errors

## Cross-Cutting Concerns

**Logging:** `log` crate with `env_logger` initialization in `main.rs`. Filter via `RUST_LOG` env var.

**Validation:** Input validation at module boundaries (e.g., model file existence checks in `OcrEngine::initialize()`)

**Authentication:** Not implemented (offline-first app)

**Serialization:** `serde` with derive macros for all data models (database entities, engine results)

**Async Runtime:** Tokio with `rt-multi-thread`, `sync`, `time`, `fs` features

---

*Architecture analysis: 2026-03-11*
