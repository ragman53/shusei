# Technology Stack

**Analysis Date:** 2026-03-11

## Languages

**Primary:**
- Rust (Edition 2021) - Full-stack application development
  - Version: 1.93.0
  - Location: `src/` directory
  - Used for: UI, business logic, ML inference, database operations

**Secondary:**
- Java/Kotlin (Android) - Platform-specific integrations via JNI
  - Used in: `src/platform/android.rs`
- JavaScript (WebAssembly bindings) - Web platform support
  - Used via Dioxus web bindings

## Runtime

**Environment:**
- Rust native runtime with Tokio async runtime
- Multi-threaded executor (`rt-multi-thread` feature)

**Package Manager:**
- Cargo 1.93.0
- Lockfile: `Cargo.lock` present (version 4)

**Build System:**
- Cargo with custom profiles
- Dioxus CLI for cross-platform builds
- Android NDK toolchains for mobile builds

## Frameworks

**UI Framework:**
- Dioxus 0.7 - Cross-platform Rust UI framework
  - Features: `router` for navigation
  - Dioxus Router 0.7 for routing
  - Configuration: `Dioxus.toml`

**Async Runtime:**
- Tokio 1.x with features:
  - `rt-multi-thread` - Multi-threaded runtime
  - `sync` - Synchronization primitives
  - `time` - Timer utilities
  - `fs` - Async filesystem operations

**ML Inference:**
- tract-onnx 0.21 - Pure Rust ONNX runtime
  - Used for: OCR (NDLOCR-Lite) and STT (Moonshine) models
  - Location: `src/core/ocr/engine.rs`, `src/core/stt/engine.rs`

**Image Processing:**
- image 0.25 - Image manipulation
- ndarray 0.16 - N-dimensional arrays for ML preprocessing

**PDF Processing:**
- pdfium-render 0.8 (optional, `pdf` feature)
  - Features: `static` for static linking
  - Used in: `src/core/pdf.rs`

**Database:**
- rusqlite 0.32 with `bundled` feature
  - SQLite with FTS5 full-text search
  - Location: `src/core/db.rs`

**Text Processing:**
- pulldown-cmark 0.12 - Markdown rendering
- tokenizers 0.20 - Text tokenization for STT
- lindera 0.34 (optional, `lindera` feature) - Japanese morphological analysis

**Audio Processing:**
- hound 3.5 - WAV file I/O

**Mobile Platform:**
- jni 0.21 (optional, `android` feature) - JNI bindings for Android
- ndk, ndk-sys, ndk-context - Android NDK integration

## Key Dependencies

**Critical Infrastructure:**
| Package | Version | Purpose |
|---------|---------|---------|
| dioxus | 0.7 | UI framework |
| tract-onnx | 0.21 | ML inference engine |
| tokio | 1.x | Async runtime |
| rusqlite | 0.32 | SQLite database |
| image | 0.25 | Image processing |
| ndarray | 0.16 | Array operations |

**Error Handling:**
- anyhow 1.x - Flexible error handling
- thiserror 2.x - Derive macro for custom errors
- Location: `src/core/error.rs`

**Serialization:**
- serde 1.x - Serialization framework
- serde_json 1.x - JSON support
- base64 0.22 - Base64 encoding

**Concurrency:**
- parking_lot 0.12 - Fast synchronization primitives
- once_cell 1.x - Lazy static initialization
- async-trait 0.1 - Async trait support

**Logging:**
- log 0.4 - Logging facade
- env_logger 0.11 - Environment-based logger configuration

## Configuration

**Cargo Configuration:**
- File: `Cargo.toml`
- Edition: 2021
- Crate type: Library + Binary (`src/lib.rs`, `src/main.rs`)

**Build Profiles:**
```toml
[profile.release]
opt-level = 3
lto = true
codegen-units = 1
panic = "abort"
strip = true
```

**Feature Flags:**
- `default` - No features enabled by default
- `android` - JNI support (`["jni"]`)
- `ios` - iOS platform support
- `desktop` - Desktop platform support
- `web` - Web platform support
- `lindera` - Japanese morphological analysis
- `pdf` - PDF processing support
- `ndlocr-test` - NDLOCR tract compatibility tests
- `moonshine-test` - Moonshine tract compatibility tests

**Dioxus Configuration:**
- File: `Dioxus.toml`
- Default platform: Desktop
- Asset directory: `assets/`
- Web app title: "Shusei"
- Desktop window: 800x600, resizable
- Bundle identifier: `com.shusei.app`

**Android Toolchain:**
- File: `.cargo/config.toml`
- Targets configured:
  - `aarch64-linux-android`
  - `armv7-linux-androideabi`
  - `i686-linux-android`
  - `x86_64-linux-android`
- Linker: Android NDK clang

## Platform Requirements

**Development:**
- Rust 1.93.0+ (stable toolchain)
- Cargo
- Dioxus CLI (for cross-platform builds)
- Android NDK (for Android builds)

**Installed Rust Targets:**
- `aarch64-linux-android` - Android ARM64
- `wasm32-unknown-unknown` - WebAssembly
- `x86_64-linux-android` - Android x86_64
- `x86_64-pc-windows-msvc` - Windows native

**Production:**
- Desktop: Windows, macOS, Linux
- Mobile: Android (primary), iOS (planned)
- Web: WebAssembly via wasm32 target

**Model Assets:**
- Location: `assets/models/`
- Subdirectories:
  - `moonshine/` - STT models (encoder/decoder ONNX files)
  - `ndlocr/` - OCR models (detection/recognition/direction ONNX files)

---

*Stack analysis: 2026-03-11*
