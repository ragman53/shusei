# Architecture

**Analysis Date:** 2026-03-13

## Pattern Overview

**Overall:** Layered architecture with platform abstraction

**Key Characteristics:**
- Clear separation between UI, core business logic, and platform-specific code
- Trait-based abstraction for external dependencies (OCR, STT, Platform APIs)
- Async-first design using Tokio runtime
- Cross-platform targeting (Desktop, Android, iOS) via conditional compilation

## Layers

**UI Layer:**
- Purpose: User interface components and routing
- Location: `src/ui/`
- Contains: Dioxus components, pages, reusable UI elements
- Depends on: Core layer for business logic, Platform layer for native features
- Used by: Main application entry point

**Core Layer:**
- Purpose: Platform-agnostic business logic
- Location: `src/core/`
- Contains: OCR pipeline, STT pipeline, database operations, storage, models
- Depends on: External crates (tract, ort, rusqlite, hayro)
- Used by: UI layer, exported as library via `src/lib.rs`

**Platform Layer:**
- Purpose: Platform-specific implementations
- Location: `src/platform/`
- Contains: JNI bindings for Android, iOS platform code, desktop stubs
- Depends on: Core error types, platform-specific crates (jni)
- Used by: UI layer via `PlatformApi` trait

## Data Flow

**PDF Conversion Flow:**

1. User selects PDF via file picker (platform layer)
2. `PdfProcessor` opens PDF using hayro library
3. Pages rendered in parallel batches (10 pages, 3 concurrent)
4. Each page image passed to `NdlocrEngine.process_image()`
5. OCR results saved to SQLite via `Database.save_page()`
6. Progress tracked in `processing_progress` table

**OCR Pipeline Flow:**

1. Image bytes received from camera or PDF renderer
2. `preprocess_image_for_inference()` converts to normalized tensor [1, 3, 1024, 1024]
3. Detection model (DEIM) identifies text regions
4. Each region extracted and resized to [1, 1, 32, W]
5. Recognition model (PARSeq) decodes text with CTC
6. Results aggregated into `OcrResult` with markdown output

**Camera Capture Flow:**

1. UI calls `platform.capture_image()` async method
2. Android: JNI call to `MainActivity.startCameraCapture()`
3. Java camera API captures image
4. Callback `onImageCaptured()` sends bytes to Rust via JNI
5. Result sent through oneshot channel to waiting async task
6. UI receives `CameraResult` with image data

**State Management:**
- Dioxus signals for reactive UI state (`use_signal`)
- SQLite for persistent data (books, notes, vocabulary)
- File system for images (paths stored in DB, not BLOBs)
- `AppState` struct for Android lifecycle state persistence

## Key Abstractions

**OcrEngine Trait:**
- Purpose: Abstract OCR processing for different backends
- Examples: `src/core/ocr/engine.rs`
- Pattern: Async trait with `process_image()`, `is_ready()`, `name()` methods
- Implementation: `NdlocrEngine` using ONNX Runtime (ort crate)

**SttEngine Trait:**
- Purpose: Abstract speech-to-text processing
- Examples: `src/core/stt/engine.rs`
- Pattern: Async trait with `transcribe()`, `is_ready()`, `language()` methods
- Implementation: `MoonshineEngine` (placeholder, uses tract)

**PlatformApi Trait:**
- Purpose: Abstract platform-specific functionality
- Examples: `src/platform/mod.rs`
- Pattern: Async trait for camera, audio, file picker, permissions, vibration
- Implementations: `AndroidPlatform`, `IosPlatform`, `DesktopPlatform`

**PdfConversionService:**
- Purpose: Orchestrate PDF-to-text conversion pipeline
- Examples: `src/core/pdf.rs`
- Pattern: Combines PdfProcessor + OcrEngine + Database + StorageService
- Methods: `convert_pdf()` with progress callback

## Entry Points

**Binary Entry Point:**
- Location: `src/main.rs`
- Triggers: Application launch
- Responsibilities: Initialize logger, launch Dioxus application

**Library Entry Point:**
- Location: `src/lib.rs`
- Triggers: When used as a library
- Responsibilities: Re-export public API (error types, engines, database)

**App Component:**
- Location: `src/app.rs`
- Triggers: Dioxus application mount
- Responsibilities: Route definition, state restoration, root component

**Android JNI Entry Points:**
- Location: `src/platform/android.rs`
- Triggers: Called from Java/Kotlin code
- Functions: `Java_com_shusei_app_MainActivity_nativeInit`, `onImageCaptured`

## Error Handling

**Strategy:** Typed errors with thiserror, propagated via Result

**Patterns:**
- `ShuseiError` - Top-level error enum wrapping all error types
- `OcrError` - OCR-specific errors (preprocessing, detection, recognition)
- `SttError` - STT-specific errors (encoder, decoder, tokenization)
- `Result<T>` - Type alias for `std::result::Result<T, ShuseiError>`
- Error conversion via `From` trait implementations

**Error Flow:**
```rust
// Core returns typed errors
fn process_image(&self, data: &[u8]) -> Result<OcrResult>

// UI handles with match or ?
match engine.process_image(&data).await {
    Ok(result) => { /* handle success */ },
    Err(e) => error_message.set(Some(e.to_string())),
}
```

## Cross-Cutting Concerns

**Logging:** env_logger with configurable filter level
- Init: `env_logger::Builder::from_env(...).init()`
- Usage: `log::info!()`, `log::warn!()`, `log::error!()`

**Validation:** 
- Input validation at UI layer before processing
- Confidence thresholds in OCR (detection: 0.5, recognition: 0.5)
- Max image size: 1024px for detection model

**Authentication:** Not applicable (offline-only application)

**Concurrency:**
- Rayon thread pool (3 threads) for parallel PDF rendering
- Async parallel OCR with `buffer_unordered(3)` for concurrency limit
- Mutex/Arc for thread-safe ONNX session access
- Tokio spawn_blocking for database operations

---

*Architecture analysis: 2026-03-13*