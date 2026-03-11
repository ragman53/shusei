# Technology Stack

**Analysis Date:** 2026-03-11

## Languages

**Primary:**
- Rust 2021 Edition (rustc 1.93.0) - All application code in `src/`

**Secondary:**
- None detected

## Runtime

**Environment:**
- Rust native runtime (no external runtime required)
- Compiled to native code for target platforms

**Package Manager:**
- Cargo 1.93.0
- Lockfile: `Cargo.lock` present (120KB)

## Frameworks

**Core:**
- Dioxus 0.7 - Cross-platform UI framework (desktop, web, mobile)
- Dioxus Router 0.7 - Client-side routing for navigation

**Testing:**
- Built-in Rust test framework (`#[cfg(test)]` modules)
- Test files in `tests/` directory

**Build/Dev:**
- Cargo build system
- LTO (Link Time Optimization) enabled for release builds
- Platform-specific toolchains for Android (via `.cargo/config.toml`)

## Key Dependencies

**Critical:**
- `dioxus 0.7` - UI framework with router support
- `tract-onnx 0.21` - ONNX runtime for OCR and STT model inference (pure Rust)
- `rusqlite 0.32` - SQLite database with bundled backend
- `tokio 1` - Async runtime with multi-threading, sync primitives, time, and fs
- `async-trait 0.1` - Async trait support for engine interfaces

**Infrastructure:**
- `image 0.25` - Image processing for OCR preprocessing
- `ndarray 0.16` - Numerical computing for model input/output
- `hound 3.5` - Audio file processing for STT
- `tokenizers 0.20` - Tokenization for STT output processing
- `pulldown-cmark 0.12` - Markdown rendering
- `pdfium-render 0.8` (optional) - PDF processing via pdfium
- `lindera 0.34` (optional) - Japanese morphological analysis

**Serialization:**
- `serde 1` with derive - Serialization/deserialization framework
- `serde_json 1` - JSON support
- `base64 0.22` - Base64 encoding/decoding

**Error Handling:**
- `anyhow 1` - Application-level error handling
- `thiserror 2` - Library-level error types

**Logging:**
- `log 0.4` - Logging facade
- `env_logger 0.11` - Environment-configured logger

**Utilities:**
- `parking_lot 0.12` - Synchronization primitives
- `once_cell 1` - Lazy initialization

## Configuration

**Environment:**
- No `.env` files used - configuration via compile-time features
- Logging configured via `RUST_LOG` environment variable (default: "info")
- Platform-specific configuration in `.cargo/config.toml`

**Build:**
- `Cargo.toml` - Main package and dependency configuration
- `Dioxus.toml` - Dioxus platform configuration (desktop, web, Android)
- `.cargo/config.toml` - Android cross-compilation toolchain configuration

**Feature Flags:**
- `default` - Empty (no default features)
- `android` - Enables JNI bindings
- `ios` - iOS-specific features (placeholder)
- `desktop` - Desktop-specific features
- `web` - Web platform features
- `lindera` - Japanese language processing
- `pdf` - PDF processing support
- `ndlocr-test` - NDLOCR tract compatibility tests
- `moonshine-test` - Moonshine STT tract compatibility tests

## Platform Requirements

**Development:**
- Rust 1.93.0+ (via rustup)
- Cargo 1.93.0+
- Android Studio + SDK + NDK (for Android builds)
- Platform toolchains configured in `.cargo/config.toml`:
  - `aarch64-linux-android30-clang` (Android ARM64)
  - `armv7a-linux-androideabi21-clang` (Android ARM)
  - `i686-linux-android21-clang` (Android x86)
  - `x86_64-linux-android21-clang` (Android x64)

**Production:**
- Native binary for target platform
- Model files in `assets/models/` (NDLOCR, Moonshine)
- SQLite database (created at runtime)
- Cross-platform targets:
  - Desktop (Windows, macOS, Linux)
  - Android (API 35 primary, API 34 fallback)
  - Web (via Dioxus web renderer)
  - iOS (placeholder support)

---

*Stack analysis: 2026-03-11*
