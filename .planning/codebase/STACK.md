# Technology Stack

**Analysis Date:** 2026-03-13

## Languages

**Primary:**
- Rust 1.93.0 (Edition 2021) - Full-stack application development
  - Location: `src/` directory
  - Used for: UI, business logic, ML inference, database operations, PDF rendering

**Secondary:**
- Java/Kotlin (Android) - Platform-specific integrations via JNI
  - Package: `com.shusei.app`
  - Main Activity: `MainActivity`
  - Used in: `src/platform/android.rs`

## Runtime

**Environment:**
- Native Rust runtime (no VM)
- Tokio async runtime with multi-threaded executor
- Features: `rt-multi-thread`, `sync`, `time`, `fs`, `macros`

**Package Manager:**
- Cargo 1.93.0
- Lockfile: `Cargo.lock` present (version 4, 147,617 bytes)

**Build System:**
- Cargo with custom profiles
- Dioxus CLI for cross-platform builds (`Dioxus.toml`)
- Android NDK toolchains for mobile builds

## Frameworks

**UI Framework:**
- Dioxus 0.7 - Cross-platform Rust UI framework
  - Features: `router`, `macro`
  - Router: dioxus-router 0.7
  - Configuration: `Dioxus.toml`
  - Platform config: Desktop (800x600), Android, Web

**Async Runtime:**
- Tokio 1.x - Multi-threaded async runtime
  - `rt-multi-thread` - Multi-threaded executor
  - `sync` - Channels and synchronization
  - `time` - Timers and delays
  - `fs` - Async filesystem
  - `macros` - `#[tokio::main]`, `#[tokio::test]`

**ML Inference:**
- **Desktop**: `ort` 2.0.0-rc.12 - ONNX Runtime bindings
- **Android**: `ort` with `alternative-backend` + `ort-tract` 0.3.0+0.22
- **Fallback**: `tract-onnx` 0.21 - Pure Rust ONNX runtime
- Used for: OCR (NDLOCR-Lite) and STT (Moonshine) models

**Image Processing:**
- `image` 0.25 - Image loading, format handling, resizing
- `ndarray` 0.16 - N-dimensional arrays for tensor operations

**PDF Processing:**
- `hayro` 0.4 - Pure Rust PDF renderer
  - No external dependencies (pdfium.dll in root is legacy)
  - Implementation: `src/core/pdf.rs`
  - Features: Page rendering to RGBA images, parallel batch processing

**Database:**
- `rusqlite` 0.32 with `bundled` feature
  - SQLite with FTS5 full-text search
  - WAL mode for concurrent reads
  - Location: `src/core/db.rs`

**Text Processing:**
- `pulldown-cmark` 0.12 - Markdown rendering for OCR output
- `tokenizers` 0.20 - Text tokenization for STT
- `lindera` 0.34 (optional) - Japanese morphological analysis with ipadic

**Audio Processing:**
- `hound` 3.5 - WAV file I/O for speech recording

**Parallel Processing:**
- `rayon` 1.8 - Data parallelism for batch PDF/OCR processing
  - Used in: `src/core/pdf.rs` for parallel page rendering

**Mobile Platform:**
- `jni` 0.21 (optional, `android` feature) - JNI bindings for Android

## Key Dependencies

**Critical Infrastructure:**
| Package | Version | Purpose |
|---------|---------|---------|
| dioxus | 0.7 | UI framework with router and macros |
| ort | 2.0.0-rc.12 | ONNX Runtime for ML inference |
| tract-onnx | 0.21 | Pure Rust ONNX runtime |
| tokio | 1.x | Async runtime |
| rusqlite | 0.32 | SQLite database with FTS5 |
| hayro | 0.4 | Pure Rust PDF rendering |
| image | 0.25 | Image processing |
| ndarray | 0.16 | Tensor operations |
| rayon | 1.8 | Parallel processing |

**Error Handling:**
| Package | Version | Purpose |
|---------|---------|---------|
| anyhow | 1.x | Error context and propagation |
| thiserror | 2.x | Custom error derive macros |
| Location: `src/core/error.rs` |

**Serialization:**
| Package | Version | Purpose |
|---------|---------|---------|
| serde | 1.x | Serialization framework (derive feature) |
| serde_json | 1.x | JSON serialization |
| base64 | 0.22 | Base64 encoding |

**Concurrency:**
| Package | Version | Purpose |
|---------|---------|---------|
| parking_lot | 0.12 | Fast mutex/RwLock |
| once_cell | 1.x | Lazy static initialization |
| async-trait | 0.1 | Async trait support |
| futures | 0.3 | Async utilities |

**Utilities:**
| Package | Version | Purpose |
|---------|---------|---------|
| uuid | 1.x (v4) | UUID generation for file naming |
| tempfile | 3.x | Temporary file handling |

**Logging:**
| Package | Version | Purpose |
|---------|---------|---------|
| log | 0.4 | Logging facade |
| env_logger | 0.11 | Environment-based logger |

## Configuration

**Cargo Configuration:**
- File: `Cargo.toml`
- Edition: 2021
- Crate type: Library + Binary
  - Library: `src/lib.rs`
  - Binary: `src/main.rs`

**Build Profiles:**
```toml
[profile.release]
opt-level = 3
lto = true
codegen-units = 1
panic = "abort"
strip = true

[profile.dev]
opt-level = 0
debug = true
```

**Feature Flags:**
| Flag | Purpose |
|------|---------|
| `default` | No features enabled by default |
| `android` | JNI support for Android |
| `ios` | iOS platform support (planned) |
| `desktop` | Desktop platform support |
| `web` | Web platform support |
| `lindera` | Japanese morphological analysis |
| `ndlocr-test` | NDLOCR tract compatibility tests |
| `moonshine-test` | Moonshine tract compatibility tests |

**Dioxus Configuration (`Dioxus.toml`):**
- Default platform: Desktop
- Asset directory: `assets/`
- Web app title: "Shusei"
- Desktop window: 800x600, resizable
- Bundle identifier: `com.shusei.app`
- Resources: `assets/models/*`

**Android Toolchain (`.cargo/config.toml`):**
- Targets: `aarch64-linux-android`, `armv7-linux-androideabi`, `i686-linux-android`, `x86_64-linux-android`
- Linker: Android NDK clang

## Platform Requirements

**Development:**
- Rust 1.93.0+ (stable toolchain)
- Cargo package manager
- Dioxus CLI (for cross-platform builds)
- Android NDK r25+ (for Android builds)

**Installed Rust Targets:**
- `aarch64-linux-android` - Android ARM64
- `wasm32-unknown-unknown` - WebAssembly
- `x86_64-linux-android` - Android x86_64
- `x86_64-pc-windows-msvc` - Windows native

**Production Targets:**

| Platform | Status | Notes |
|----------|--------|-------|
| Desktop (Windows/macOS/Linux) | Primary | Native builds, rfd for file picker |
| Android | Supported | JNI integration, alternative ONNX backend |
| iOS | Planned | Feature flag exists, no implementation |
| Web | Planned | Feature flag exists |

## Model Assets

**OCR Models (NDLOCR-Lite):**
- Location: `assets/ocr/models/` and `assets/models/ndlocr/`
- Detection: `deim-s-1024x1024.onnx`
- Recognition: `parseq-ndl-16x384-50-tiny-146epoch-tegaki2.onnx` (multiple sizes available)
- Dictionary: `dict.txt`
- Size: ~165MB total
- License: Apache 2.0
- Source: HuggingFace `monkt/paddleocr-onnx`

**STT Models (Moonshine):**
- Location: `assets/models/moonshine/`
- Files: `encoder.onnx`, `decoder.onnx` (not yet present)
- Size: ~50-60MB per language
- License: Moonshine Community License
- Source: GitHub `usefulsensors/moonshine`, HuggingFace `UsefulSensors`

---

*Stack analysis: 2026-03-13*