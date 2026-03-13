# Codebase Structure

**Analysis Date:** 2026-03-13

## Directory Layout

```
shusei/
├── src/                    # Source code
│   ├── main.rs             # Binary entry point
│   ├── lib.rs              # Library entry point
│   ├── app.rs              # Root component and routing
│   ├── core/               # Platform-agnostic business logic
│   ├── ui/                 # Dioxus UI components
│   └── platform/           # Platform-specific implementations
├── assets/                 # Static assets (models, icons)
│   ├── models/             # ONNX models for OCR/STT
│   │   ├── moonshine/      # Speech-to-text models
│   │   └── ndlocr/         # OCR models
│   └── ocr/                # OCR model files
├── tests/                  # Integration tests
├── platform/               # Platform-specific build configs
│   └── android/            # Android project structure
├── docs/                   # Documentation
├── bin/                    # Binary dependencies
├── lib/                    # Library dependencies
├── include/                # C/C++ headers
├── target/                 # Build output (generated)
├── .planning/              # GSD planning documents
├── Cargo.toml              # Rust package manifest
├── Dioxus.toml             # Dioxus configuration
└── build.rs                # Build script
```

## Directory Purposes

**src/core:**
- Purpose: Platform-agnostic business logic
- Contains: OCR pipeline, STT pipeline, database, storage, models
- Key files: `ocr/mod.rs`, `stt/mod.rs`, `db.rs`, `pdf.rs`, `storage.rs`

**src/ui:**
- Purpose: Dioxus UI components
- Contains: Page components, shared UI elements
- Key files: `camera.rs`, `reader.rs`, `notes.rs`, `vocab.rs`, `components.rs`

**src/platform:**
- Purpose: Platform-specific implementations
- Contains: Android JNI bindings, iOS stubs, desktop platform
- Key files: `mod.rs` (trait definition), `android.rs`

**assets/models:**
- Purpose: ONNX model files for ML inference
- Contains: moonshine/ (STT), ndlocr/ (OCR)
- Key files: Detection model (~45MB), recognition model (~15MB)

**tests:**
- Purpose: Integration tests
- Contains: Large PDF tests, model compatibility tests
- Key files: `large_pdf_test.rs`, `moonshine_tract_test.rs`, `ndlocr_tract_test.rs`

## Key File Locations

**Entry Points:**
- `src/main.rs`: Binary entry, initializes logger and launches Dioxus
- `src/lib.rs`: Library entry, re-exports public API
- `src/app.rs`: Root Dioxus component with routing

**Configuration:**
- `Cargo.toml`: Dependencies, features, build profiles
- `Dioxus.toml`: UI framework configuration (platforms, assets)
- `build.rs`: Build-time configuration

**Core Logic:**
- `src/core/ocr/engine.rs`: NdlocrEngine implementation (872 lines)
- `src/core/stt/engine.rs`: MoonshineEngine implementation (165 lines)
- `src/core/db.rs`: SQLite database operations (922 lines)
- `src/core/pdf.rs`: PDF processing and conversion service (706 lines)
- `src/core/storage.rs`: File storage for images (270 lines)

**UI Pages:**
- `src/ui/camera.rs`: Camera capture page (201 lines)
- `src/ui/reader.rs`: PDF reader and library (333 lines)
- `src/ui/notes.rs`: Sticky notes page
- `src/ui/vocab.rs`: Vocabulary management page
- `src/ui/components.rs`: Shared UI components (289 lines)

**Platform Implementations:**
- `src/platform/mod.rs`: PlatformApi trait definition (135 lines)
- `src/platform/android.rs`: Android JNI bindings (227 lines)

**Testing:**
- `tests/`: Integration tests directory
- Inline `#[cfg(test)]` modules in each source file

## Naming Conventions

**Files:**
- Rust source: `snake_case.rs` (e.g., `engine.rs`, `preprocess.rs`)
- Module directories: `snake_case/` with `mod.rs` (e.g., `core/ocr/mod.rs`)
- Test files: `*_test.rs` for integration tests

**Modules:**
- Crate root modules: `core`, `ui`, `platform`
- Sub-modules: `ocr`, `stt`, nested under parent
- Re-exports: `pub use` at module root for clean API

**Structs and Enums:**
- PascalCase for types (e.g., `OcrEngine`, `NdlocrEngine`, `ConversionStage`)
- New types for creation: `NewStickyNote`, `NewBook`, `NewBookPage`
- Result types: `OcrResult`, `SttResult`, `CameraResult`

**Functions:**
- snake_case for functions (e.g., `process_image`, `save_to_prefs`)
- Async functions: no special suffix, called with `.await`
- Constructor pattern: `new()`, `with_all_fields()`

**Variables:**
- snake_case for locals (e.g., `image_data`, `book_id`)
- SCREAMING_SNAKE_CASE for constants (e.g., `MODEL_DETECTION_PATH`)

## Where to Add New Code

**New Feature:**
- Core logic: `src/core/{feature}/mod.rs`
- UI page: `src/ui/{feature}.rs`
- Add to `src/ui/mod.rs` exports
- Add route in `src/app.rs`

**New Component/Module:**
- Implementation: `src/core/{module}/`
- Create `mod.rs` with public interface
- Add to `src/core/mod.rs` exports

**New Platform Support:**
- Implementation: `src/platform/{platform}.rs`
- Add to `src/platform/mod.rs` with `#[cfg(target_os = "...")]`
- Add feature flag in `Cargo.toml`

**Utilities:**
- Shared helpers: New module in `src/core/`
- UI utilities: `src/ui/components.rs`

**Tests:**
- Unit tests: Inline `#[cfg(test)] mod tests` in source file
- Integration tests: New file in `tests/` directory

## Special Directories

**assets/models:**
- Purpose: ONNX model files for ML inference
- Generated: No (bundled with app)
- Committed: Yes (required for OCR/STT)
- Structure: `moonshine/` for STT, `ndlocr/` for OCR

**assets/ocr/models:**
- Purpose: OCR model files (DEIM detection, PARSeq recognition)
- Generated: No
- Committed: Yes

**target:**
- Purpose: Build output and compiled artifacts
- Generated: Yes
- Committed: No (in .gitignore)
- Structure: `debug/`, `release/`, platform-specific targets

**.planning:**
- Purpose: GSD planning documents (phases, codebase analysis)
- Generated: By GSD commands
- Committed: Yes (tracked in git)
- Structure: `phases/`, `codebase/`, `research/`

**platform/android:**
- Purpose: Android-specific build configuration and Java/Kotlin code
- Generated: Partially (by Dioxus/cargo-apk)
- Committed: Yes
- Structure: `app/src/main/java/com/shusei/app/`

## Module Organization

**Core Module Structure:**
```
src/core/
├── mod.rs          # Re-exports: error, ocr, stt, db, storage, state, models, pdf
├── error.rs        # ShuseiError, OcrError, SttError, Result<T>
├── db.rs           # Database, CRUD operations, data models
├── models.rs       # Book model
├── pdf.rs          # PdfProcessor, PdfConversionService
├── state.rs        # AppState for lifecycle persistence
├── storage.rs      # StorageService for file operations
├── vocab.rs        # Vocabulary management
├── ocr/
│   ├── mod.rs      # OcrConfig, model paths, re-exports
│   ├── engine.rs   # OcrEngine trait, NdlocrEngine
│   ├── preprocess.rs
│   ├── postprocess.rs
│   └── markdown.rs
└── stt/
    ├── mod.rs      # SttConfig, Language enum
    ├── engine.rs   # SttEngine trait, MoonshineEngine
    ├── decoder.rs  # Decoder state, KV cache
    └── tokenizer.rs
```

**UI Module Structure:**
```
src/ui/
├── mod.rs          # Re-exports all page components
├── camera.rs       # CameraPage component
├── reader.rs       # ReaderPage, ReaderBookView components
├── notes.rs        # NotesPage component
├── vocab.rs        # VocabPage component
├── add_book.rs     # AddBookPage component
├── library.rs      # LibraryPage component
└── components.rs   # Shared: LoadingSpinner, Button, Card, etc.
```

---

*Structure analysis: 2026-03-13*