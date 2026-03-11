# Codebase Structure

**Analysis Date:** 2026-03-11

## Directory Layout

```
shusei/
├── .cargo/                     # Cargo configuration
│   └── config.toml             # Android cross-compilation settings
├── .planning/                  # Planning documents (this codebase mapping)
│   ├── backups/                # File backups
│   └── codebase/               # Architecture documentation
├── assets/                     # Static assets
│   └── models/                 # ONNX model files
│       ├── moonshine/          # STT models
│       └── ndlocr/             # OCR models
├── docs/                       # Documentation
├── platform/                   # Platform-specific code
│   └── android/                # Android project
│       └── app/                # Android app source
├── src/                        # Rust source code
│   ├── core/                   # Business logic
│   │   ├── ocr/                # OCR pipeline
│   │   ├── stt/                # STT pipeline
│   │   ├── db.rs               # Database operations
│   │   ├── error.rs            # Error types
│   │   ├── mod.rs              # Core module exports
│   │   ├── pdf.rs              # PDF processing (optional)
│   │   └── vocab.rs            # Vocabulary management
│   ├── platform/               # Platform abstraction
│   │   ├── android.rs          # Android JNI implementation
│   │   ├── ios.rs              # iOS implementation stub
│   │   └── mod.rs              # PlatformApi trait
│   ├── ui/                     # UI components
│   │   ├── camera.rs           # Camera page
│   │   ├── components.rs       # Shared UI components
│   │   ├── mod.rs              # UI module exports
│   │   ├── notes.rs            # Notes page
│   │   ├── reader.rs           # PDF reader page
│   │   └── vocab.rs            # Vocabulary page
│   ├── app.rs                  # Routing and main app
│   ├── lib.rs                  # Library entry point
│   └── main.rs                 # Desktop entry point
├── tests/                      # Integration tests
│   ├── moonshine_tract_test.rs # STT model tests
│   └── ndlocr_tract_test.rs    # OCR model tests
├── Cargo.toml                  # Rust dependencies
├── Cargo.lock                  # Dependency lock
└── Dioxus.toml                 # UI framework config
```

## Directory Purposes

**`.cargo/`:**
- Contains Android linker configuration for cross-compilation
- Key file: `config.toml` with NDK toolchain settings

**`assets/models/`:**
- ONNX model files for ML inference
- `moonshine/` - Encoder and decoder models for STT
- `ndlocr/` - Detection, recognition, direction classifier for OCR
- Referenced in `Dioxus.toml` as bundled resources

**`docs/`:**
- Project documentation
- Not actively used in current codebase

**`platform/android/`:**
- Full Android project structure
- Java/Kotlin source in `app/src/main/java/`
- MainActivity with JNI bridge to Rust

**`src/core/`:**
- Business logic with no UI or platform dependencies
- Each submodule has its own directory for complex features
- `mod.rs` re-exports public types

**`src/platform/`:**
- Platform abstraction layer
- Conditional compilation via `#[cfg]` attributes
- Desktop fallback implementation for development

**`src/ui/`:**
- Dioxus UI components
- One file per major page/screen
- Shared components in `components.rs`

**`tests/`:**
- Integration tests for ML model compatibility
- Tests tract runtime with actual ONNX models

## Key File Locations

**Entry Points:**
- `src/main.rs`: Desktop application entry
- `src/lib.rs`: Library entry for Android/iOS integration
- `src/platform/android.rs`: JNI entry points (`Java_com_shusei_app_MainActivity_*`)

**Configuration:**
- `Cargo.toml`: Dependencies, features, profiles
- `Dioxus.toml`: UI configuration, platform settings
- `.cargo/config.toml`: Cross-compilation toolchain

**Core Logic:**
- `src/core/db.rs`: SQLite operations, 314 lines
- `src/core/ocr/engine.rs`: OCR trait and NDLOCR implementation, 156 lines
- `src/core/stt/engine.rs`: STT trait and Moonshine implementation, 165 lines
- `src/core/vocab.rs`: Vocabulary extraction and export, 231 lines

**UI Components:**
- `src/ui/camera.rs`: Camera capture page, 201 lines
- `src/ui/notes.rs`: Notes list with search, 121 lines
- `src/ui/components.rs`: Reusable UI primitives, 164 lines

**Platform:**
- `src/platform/mod.rs`: PlatformApi trait definition, 135 lines
- `src/platform/android.rs`: JNI implementation, 223 lines

**Error Handling:**
- `src/core/error.rs`: All error types, 104 lines

## Naming Conventions

**Files:**
- Rust modules: `snake_case.rs` (e.g., `engine.rs`, `preprocess.rs`)
- Module directories: `snake_case/` (e.g., `ocr/`, `stt/`)
- Module entry: `mod.rs` in each directory
- Tests: Co-located in `#[cfg(test)]` or separate `tests/*.rs`

**Directories:**
- Top-level: descriptive (e.g., `core/`, `ui/`, `platform/`)
- Feature subdirectories: plural nouns (e.g., `models/`, `docs/`)

**Rust Naming:**
- Types (structs, enums, traits): PascalCase (e.g., `OcrEngine`, `ShuseiError`)
- Functions/variables: snake_case (e.g., `process_image`, `captured_image`)
- Constants: UPPER_SNAKE_CASE or SCREAMING_SNAKE_CASE
- Modules: snake_case (e.g., `mod camera`, `mod vocab`)

**Examples from codebase:**
```rust
// Traits: PascalCase
pub trait OcrEngine: Send + Sync { }
pub trait SttEngine: Send + Sync { }
pub trait PlatformApi: Send + Sync { }

// Structs: PascalCase
pub struct NdlocrEngine { }
pub struct MoonshineEngine { }
pub struct StickyNote { }

// Functions: snake_case
fn preprocess_image(image_data: &[u8], config: &PreprocessConfig) -> Result<Array3<f32>>
fn capture_image(&self) -> Result<CameraResult>

// Enums: PascalCase variants
pub enum ShuseiError {
    Ocr(#[from] OcrError),
    Platform(String),
    // ...
}
```

## Where to Add New Code

**New UI Page:**
- Create `src/ui/{page_name}.rs`
- Add to `src/ui/mod.rs` exports
- Add route in `src/app.rs` Route enum
- Add page wrapper component in `src/app.rs`

**New OCR Feature:**
- Add to `src/core/ocr/` directory
- Export from `src/core/ocr/mod.rs`
- Update `src/core/mod.rs` if needed

**New Platform Implementation:**
- Create `src/platform/{platform}.rs`
- Implement `PlatformApi` trait
- Add `#[cfg]` conditional in `src/platform/mod.rs` `get_platform_api()`

**New Database Table:**
- Add schema to `src/core/db.rs` `initialize_schema()`
- Add model struct (e.g., `NewBook`, `Book`)
- Add CRUD methods to `Database` impl

**New Shared Component:**
- Add to `src/ui/components.rs`
- Export from `src/ui/mod.rs`
- Use in page components via `crate::ui::ComponentName`

**New Test:**
- Unit tests: Co-located in source files under `#[cfg(test)]`
- Integration tests: Add to `tests/` directory
- Name: `{feature}_test.rs` or `{feature}_tests.rs`

## Special Directories

**`target/`:**
- Purpose: Cargo build output
- Generated: Yes (by cargo)
- Committed: No (in `.gitignore`)
- Contains: Compiled binaries, intermediate artifacts

**`assets/models/`:**
- Purpose: ML model storage
- Generated: No (manually placed)
- Committed: No (models tracked separately)
- Expected files: `*.onnx` model files

**`.planning/`:**
- Purpose: Project planning documents
- Generated: No (manually maintained)
- Committed: Yes
- Contains: Architecture docs, task planning

**`platform/android/app/`:**
- Purpose: Android project source
- Generated: Partially (by Android build tools)
- Committed: Yes
- Contains: Java source, resources, manifest

---

*Structure analysis: 2026-03-11*
