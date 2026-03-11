# Codebase Structure

**Analysis Date:** 2026-03-11

## Directory Layout

```
shusei/
├── src/                    # Rust source code
│   ├── core/               # Platform-agnostic business logic
│   │   ├── ocr/            # OCR pipeline (NDLOCR-Lite)
│   │   ├── stt/            # STT pipeline (Moonshine)
│   │   ├── db.rs           # SQLite database operations
│   │   ├── error.rs        # Error type definitions
│   │   ├── vocab.rs        # Vocabulary management
│   │   ├── pdf.rs          # PDF processing (optional)
│   │   └── mod.rs          # Core module exports
│   ├── ui/                 # Dioxus UI components
│   │   ├── camera.rs       # Camera capture page
│   │   ├── notes.rs        # Notes list page
│   │   ├── reader.rs       # PDF reader page
│   │   ├── vocab.rs        # Vocabulary list page
│   │   ├── components.rs   # Shared UI components
│   │   └── mod.rs          # UI module exports
│   ├── platform/           # Platform abstraction layer
│   │   ├── android.rs      # Android JNI implementation
│   │   ├── ios.rs          # iOS stub implementation
│   │   └── mod.rs          # Platform trait and dispatch
│   ├── app.rs              # Root app component with routing
│   ├── main.rs             # Binary entry point
│   └── lib.rs              # Library entry point
├── platform/               # Native platform code
│   └── android/
│       └── app/src/main/java/com/shusei/app/
│           └── MainActivity.java   # Android camera/mic bridge
├── assets/                 # Static assets
│   └── models/             # ONNX model files
│       ├── ndlocr/         # OCR models (detection, recognition, direction)
│       └── moonshine/      # STT models (encoder, decoder)
├── docs/                   # Documentation
├── .cargo/
│   └── config.toml         # Cargo build configuration (Android toolchain)
├── Cargo.toml              # Rust dependencies and features
├── Dioxus.toml             # Dioxus app configuration
└── .planning/              # Project planning documents
```

## Directory Purposes

**`src/core/`:**
- Purpose: Platform-agnostic business logic and data processing
- Contains: OCR engine, STT engine, database operations, vocabulary extraction, error types
- Key files: `db.rs` (314 lines), `error.rs` (104 lines), `vocab.rs` (231 lines)

**`src/core/ocr/`:**
- Purpose: OCR pipeline implementation
- Contains: Engine trait, preprocessing, postprocessing, markdown generation
- Key files: `engine.rs` (156 lines), `preprocess.rs`, `postprocess.rs`, `markdown.rs`

**`src/core/stt/`:**
- Purpose: Speech-to-text pipeline implementation
- Contains: Engine trait, decoder state, tokenizer
- Key files: `engine.rs` (165 lines), `decoder.rs`, `tokenizer.rs`

**`src/ui/`:**
- Purpose: Dioxus UI components and pages
- Contains: Page components, shared UI components, routing
- Key files: `camera.rs` (201 lines), `components.rs` (164 lines), `notes.rs`, `reader.rs`, `vocab.rs`

**`src/platform/`:**
- Purpose: Platform abstraction for native functionality
- Contains: `PlatformApi` trait, Android JNI implementation, iOS stub, desktop fallback
- Key files: `android.rs` (223 lines), `mod.rs` (135 lines), `ios.rs`

**`platform/android/`:**
- Purpose: Android native code (Java/Kotlin)
- Contains: MainActivity with Camera2 API integration
- Key files: `MainActivity.java` (15.6KB)

**`assets/models/`:**
- Purpose: ONNX model files for ML inference
- Contains: NDLOCR and Moonshine model directories
- Note: Models are referenced in `Dioxus.toml` for bundling

## Key File Locations

**Entry Points:**
- `src/main.rs`: Binary entry point - initializes logger, launches Dioxus app
- `src/lib.rs`: Library entry point - re-exports core types
- `src/app.rs`: Root Dioxus component with routing configuration

**Configuration:**
- `Cargo.toml`: Dependencies (dioxus, tract-onnx, rusqlite, tokio, etc.) and feature flags
- `Dioxus.toml`: Dioxus app settings (window size, asset dir, bundle config)
- `.cargo/config.toml`: Cross-compilation settings for Android targets

**Core Logic:**
- `src/core/db.rs`: Database schema, CRUD operations, FTS5 search
- `src/core/error.rs`: Error type hierarchy (`ShuseiError`, `OcrError`, `SttError`)
- `src/core/ocr/engine.rs`: `OcrEngine` trait and `NdlocrEngine` implementation
- `src/core/stt/engine.rs`: `SttEngine` trait and `MoonshineEngine` implementation

**UI Components:**
- `src/ui/camera.rs`: Camera capture page with JNI integration
- `src/ui/components.rs`: Reusable components (Button, Card, LoadingSpinner, etc.)
- `src/app.rs`: Route definitions and page wrappers

**Platform Bridge:**
- `src/platform/mod.rs`: `PlatformApi` trait definition and platform selection
- `src/platform/android.rs`: JNI implementation for camera, permissions, vibration
- `platform/android/app/src/main/java/com/shusei/app/MainActivity.java`: Android Camera2 API

## Naming Conventions

**Files:**
- Snake case for Rust modules: `word_extractor.rs`, `sticky_note.rs`
- Pascal case for components: `CameraPage`, `NotesPage`
- Module entry: `mod.rs` for each directory

**Types:**
- Pascal case for structs/enums: `OcrResult`, `SttEngine`, `ShuseiError`
- Snake case for functions/methods: `process_image()`, `create_sticky_note()`
- Pascal case for traits: `OcrEngine`, `SttEngine`, `PlatformApi`

**Directories:**
- Lowercase with underscores: `src/core/`, `src/ui/`, `assets/models/`

## Where to Add New Code

**New Feature (e.g., new page):**
- UI component: `src/ui/[feature].rs`
- Export in: `src/ui/mod.rs`
- Route in: `src/app.rs` (add to `Route` enum)

**New Core Module:**
- Implementation: `src/core/[module].rs` or `src/core/[module]/mod.rs`
- Export in: `src/core/mod.rs`
- Re-export in: `src/lib.rs` if public API needed

**New Engine Implementation:**
- OCR: `src/core/ocr/[engine_name].rs` implementing `OcrEngine` trait
- STT: `src/core/stt/[engine_name].rs` implementing `SttEngine` trait

**New Platform Support:**
- Add module: `src/platform/[platform].rs` implementing `PlatformApi` trait
- Update: `src/platform/mod.rs` with `#[cfg]` dispatch
- Add native code in: `platform/[platform]/`

**Utilities:**
- Shared helpers: `src/core/[domain]/` for domain-specific utilities
- UI helpers: `src/ui/components.rs` for reusable components

**Tests:**
- Unit tests: Inline `#[cfg(test)] mod tests` in source files
- Integration tests: `tests/` directory (not yet present)

## Special Directories

**`target/`:**
- Purpose: Build artifacts
- Generated: Yes (by Cargo)
- Committed: No (in .gitignore)

**`assets/models/`:**
- Purpose: ONNX model files for OCR/STT inference
- Generated: No (downloaded separately)
- Committed: Partially (directory structure, not model files)

**`platform/android/`:**
- Purpose: Android project structure
- Generated: Partially (Android Gradle project)
- Committed: Yes

**`.planning/`:**
- Purpose: Project planning and documentation
- Generated: Manually
- Committed: Yes

---

*Structure analysis: 2026-03-11*
