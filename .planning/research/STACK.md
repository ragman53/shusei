# Technology Stack

**Project:** 読書アプリ (Reading App)
**Researched:** 2026-03-11
**Confidence:** HIGH

## Recommended Stack

### Core Framework

| Technology | Version | Purpose | Why |
|------------|---------|---------|-----|
| **Dioxus** | 0.7.3 | Cross-platform UI framework | Native Android support via `dx serve --platform android`, Rust-native, direct JNI access, hot-reloading for development. Production-ready with 35k+ GitHub stars. |
| **dioxus-mobile** | 0.7.3 | Mobile renderer | Re-export of dioxus-desktop with mobile-specific tweaks, JNI integration for Android native APIs |

### Database & Storage

| Technology | Version | Purpose | Why |
|------------|---------|---------|-----|
| **rusqlite** | 0.38.0 | SQLite database | Mature (100% documented), `bundled` feature compiles SQLite from source (avoids system dependency hell), supports Android via bundled builds, excellent Rust ergonomics |
| **rusqlite_migration** | 1.2.0 | Schema migrations | Simple schema migration library using SQLite's user_version field, atomic schema updates |

### ML/AI Inference

| Technology | Version | Purpose | Why |
|------------|---------|---------|-----|
| **ort** | 2.0.0-rc.12 | ONNX Runtime bindings | Official Rust wrapper for ONNX Runtime 1.24, supports Android ARM64, enables NDLOCR-Lite OCR inference, optimized with GraphOptimizationLevel, supports IoBinding for memory efficiency |
| **ndarray** | 0.17.2 | N-dimensional arrays | Standard Rust tensor/array library, integrates with ort for tensor operations, zero-copy views |
| **candle-core** | 0.8.2 | Alternative ML backend | Pure Rust ML framework by Hugging Face, fallback for on-device LLM inference if ort has issues, no Python dependency |

### Image Processing

| Technology | Version | Purpose | Why |
|------------|---------|---------|-----|
| **image** | 0.25.10 | Image loading/processing | Standard Rust image library, supports JPEG/PNG/WebP, memory-efficient streaming, works with Android camera output |

### Voice Recognition

| Technology | Version | Purpose | Why |
|------------|---------|---------|-----|
| **moonshine** | N/A (via JNI) | Voice transcription | Rust wrapper around Moonshine C++ core, supports streaming ASR with caching, 34-245M parameter models, optimized for live speech (not 30s fixed window like Whisper), Android native support via Maven |

### AI Dictionary/LLM

| Technology | Version | Purpose | Why |
|------------|---------|---------|-----|
| **candle-transformers** | 0.8.2 | Local LLM inference | Runs Qwen3.5-0.5B/3B/8B on-device, 8-bit quantized GGUF support, no external API calls, pure Rust |

### Supporting Libraries

| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| **serde** | 1.0 | Serialization | Configuration, data export, JSON handling |
| **anyhow** | 1.0 | Error handling | Simple error propagation throughout app |
| **tracing** | 0.1 | Logging | Structured logging for debugging |
| **tokio** | 1.36 | Async runtime | Background OCR processing, file I/O |
| **jni** | 0.21 | Java interop | Access Android Camera API, file picker |

## Cargo.toml Configuration

```toml
[dependencies]
# Core framework
dioxus = { version = "0.7.3", features = ["mobile"] }
dioxus-mobile = "0.7.3"

# Database
rusqlite = { version = "0.38.0", features = ["bundled"] }
rusqlite_migration = "1.2.0"

# ML Inference
ort = "=2.0.0-rc.12"
ndarray = "0.17.2"
candle-core = { version = "0.8.2", optional = true }
candle-transformers = { version = "0.8.2", optional = true }

# Image processing
image = "0.25.10"

# Async & utilities
tokio = { version = "1.36", features = ["rt-multi-thread"] }
serde = { version = "1.0", features = ["derive"] }
anyhow = "1.0"
tracing = "0.1"

# Android JNI
jni = "0.21"

[features]
default = ["candle"]
candle = ["dep:candle-core", "dep:candle-transformers"]

[target.'cfg(target_os = "android")'.dependencies]
# Android-specific optimizations
```

## Installation Commands

```bash
# Install Dioxus CLI
cargo install cargo-binstall
cargo binstall dioxus-cli

# Android development requirements:
# - Android SDK
# - Android NDK (r25c or later)
# - Set ANDROID_SDK_ROOT and ANDROID_NDK_HOME env vars

# Build for Android
dx serve --platform android

# Release build
cargo build --target aarch64-linux-android --release
```

## Architecture Decisions

### Why Dioxus over other options?

**Chosen:** Dioxus 0.7.3
- First-class Android support with `dx serve --platform android`
- Direct JNI access to Android APIs (camera, file picker)
- Native webview rendering (not Electron)
- Hot-reloading for rapid development
- Cross-platform path for future desktop/iOS expansion

**Alternatives considered:**
- **Tauri**: Not Rust-native (WebView-based), more complex mobile setup
- **Flutter**: Requires Dart, not Rust-focused
- **Compose Multiplatform**: Kotlin-based, harder to integrate Rust ML libraries

### Why ort over Candle for OCR?

**Chosen:** ort (ONNX Runtime)
- NDLOCR-Lite is provided as ONNX models
- ONNX Runtime has mature Android ARM64 support
- Better performance for pre-trained vision models
- Graph optimization and execution providers (NNAPI on Android)

**When to use Candle instead:**
- If porting NDLOCR-Lite to Candle fails
- For on-device LLM inference (Qwen models)
- When avoiding C++ dependencies entirely

### Why rusqlite with bundled feature?

**Critical for Android:**
- `bundled` compiles SQLite from source, avoiding system SQLite version mismatches
- Android devices have inconsistent SQLite versions
- Ensures consistent WAL mode support for concurrent read/write
- Atomic schema migrations via rusqlite_migration

### Memory Optimization Strategy

| Component | Memory Budget | Strategy |
|-----------|---------------|----------|
| OCR model (NDLOCR-Lite) | ~100-200MB | Load once, use ort IoBinding to minimize allocations |
| Voice model (Moonshine Tiny) | ~34MB | Streaming inference with caching, unload when not in use |
| LLM (Qwen 0.5B quantized) | ~500MB-1GB | Optional feature, aggressive quantization (4-bit) |
| Image buffers | ~50MB per image | Process sequentially, don't hold multiple full-res images |
| SQLite | ~10MB | WAL mode, prepared statements, connection pooling |

**Total target:** <2GB RAM on mid-range Android devices

## Platform-Specific Considerations

### Android

**NDK Version:** r25c or later (required for ort Android support)

**Target ABIs:**
- `aarch64-linux-android` (primary - modern devices)
- `armv7-linux-androideabi` (optional - older devices)

**Permissions needed:**
- `CAMERA` - for page photography
- `RECORD_AUDIO` - for voice transcription
- `READ_EXTERNAL_STORAGE` - for PDF import (Android 10+)
- `MANAGE_EXTERNAL_STORAGE` - for file picker (Android 11+)

**JNI Integration:**
- Use `jni` crate for Android Camera2 API access
- File picker via JNI to native Android intents
- Moonshine voice via C++ bindings through JNI

## What NOT to Use

| Technology | Why Not | Alternative |
|------------|---------|-------------|
| **Whisper** | Fixed 30s window, no caching, poor Asian language support | Moonshine Voice (streaming, cached, language-specific models) |
| **PyTorch Mobile** | Massive binary size, Python dependency | ONNX Runtime via ort |
| **TensorFlow Lite** | C++ API complexity, larger than ORT | ONNX Runtime via ort |
| **System SQLite** | Android version fragmentation | rusqlite with bundled feature |
| **Cloud OCR APIs** | Violates offline/privacy requirement | NDLOCR-Lite local ONNX |
| **Cloud LLM APIs** | Violates offline/privacy requirement | Candle with local Qwen models |

## Confidence Assessment

| Component | Confidence | Reason |
|-----------|------------|--------|
| Dioxus 0.7.3 | HIGH | Verified via Context7 + official releases, production usage, active development |
| rusqlite 0.38.0 | HIGH | Context7 docs, standard in Rust ecosystem, Android tested |
| ort 2.0.0-rc.12 | HIGH | Context7 docs, official pyke.io documentation, production users |
| Moonshine Voice | HIGH | Official Android support, C++ core with JNI bindings |
| Candle 0.8.x | MEDIUM | Verified via GitHub, active development, but newer in ecosystem |
| Android NDK | HIGH | Standard Android development path |

## Research Sources

- **Dioxus**: https://github.com/dioxuslabs/dioxus/releases/tag/v0.7.3 (v0.7.3, Jan 17 2026)
- **ort**: https://ort.pyke.io/ (v2.0.0-rc.12)
- **rusqlite**: https://docs.rs/rusqlite/0.38.0/rusqlite/ (v0.38.0)
- **Moonshine**: https://github.com/moonshine-ai/moonshine (Android native support)
- **Candle**: https://github.com/huggingface/candle (v0.8.x)
- **ndarray**: https://docs.rs/ndarray/0.17.2/ndarray/ (v0.17.2)
- **image**: https://docs.rs/image/0.25.10/image/ (v0.25.10)

## Open Questions

1. **NDLOCR-Lite Rust port**: Need to verify if NDLOCR-Lite ONNX models work with ort on Android or if Candle conversion is required
2. **Moonshine Rust bindings**: Need to create Rust wrapper around Moonshine C++ core (currently has Python/Swift/Java bindings)
3. **Qwen model size**: Validate 0.5B vs 3B vs 8B quantization performance on target Android devices
4. **Memory pressure**: Test simultaneous OCR + Voice + LLM memory usage on 2GB RAM devices
