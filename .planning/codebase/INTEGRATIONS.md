# External Integrations

**Analysis Date:** 2026-03-11

## APIs & External Services

**None detected** - This is a fully offline application with no external API dependencies.

**Model Sources (downloaded separately, not integrated via SDK):**
- Moonshine STT models: https://github.com/usefulsensors/moonshine
- Moonshine models on Hugging Face: https://huggingface.co/collections/UsefulSensors/moonshine
- NDLOCR models: Manual download required

## Data Storage

**Databases:**
- SQLite (embedded)
  - Client: `rusqlite 0.32` with bundled backend
  - Location: Filesystem path provided at runtime (`Database::open(path)`)
  - In-memory support for testing (`Database::in_memory()`)
  - Schema defined in `src/core/db.rs`

**Tables:**
- `sticky_notes` - User notes with OCR output and metadata
- `sticky_notes_fts` - FTS5 full-text search index
- `books` - PDF book library metadata
- `book_pages` - Converted book pages with markdown content
- `vocabulary` - Vocabulary/flashcard entries

**File Storage:**
- Local filesystem only
- Images stored in app-managed directories
- Models stored in `assets/models/` (excluded from git, downloaded separately)
- PDFs stored locally

**Caching:**
- None detected - all data persisted to SQLite or filesystem

## Authentication & Identity

**Auth Provider:**
- None - Offline-only application with no user accounts or authentication

## Monitoring & Observability

**Error Tracking:**
- None - No external error tracking service integrated

**Logs:**
- `env_logger` + `log` crate
- Output to stdout/stderr
- Configured via `RUST_LOG` environment variable
- Default level: "info"
- Initialized in `src/main.rs`

## CI/CD & Deployment

**Hosting:**
- Not applicable - Desktop/mobile application

**CI Pipeline:**
- None detected in repository

**Build Artifacts:**
- Desktop: Native binary via `cargo build --release`
- Android: APK/AAB via cross-compilation with Android NDK
- Web: Static assets via Dioxus web renderer

## Environment Configuration

**Required env vars:**
- `RUST_LOG` - Logging level (optional, default: "info")

**Secrets location:**
- None required - offline application with no API keys or credentials

## Webhooks & Callbacks

**Incoming:**
- None

**Outgoing:**
- None

## Platform Integrations

**Android:**
- JNI bindings via `jni 0.21` (optional feature)
- Camera permission handling via `src/platform/android.rs`
- Microphone permission handling via `src/platform/android.rs`
- Java interop through `extern "system"` functions
- AndroidManifest: `platform/android/AndroidManifest.xml`

**iOS:**
- Placeholder support in `src/platform/ios.rs`
- No active integration detected

**Desktop:**
- Dioxus desktop renderer
- Native file system access

**Web:**
- Dioxus web renderer
- Base path configured in `Dioxus.toml`: `/shusei`

## Model Inference

**ONNX Runtime:**
- `tract-onnx 0.21` - Pure Rust ONNX inference engine
- No external service calls - all inference runs locally

**OCR Pipeline (NDLOCR-Lite):**
- Model files: `assets/models/ndlocr/`
  - `text_detection.onnx`
  - `text_recognition.onnx`
  - `direction_classifier.onnx` (optional)
- Engine: `src/core/ocr/engine.rs`

**STT Pipeline (Moonshine Tiny):**
- Model files: `assets/models/moonshine/`
  - `encoder.onnx`
  - `decoder.onnx`
- Engine: `src/core/stt/engine.rs`
- Supported languages: Japanese (primary), English (optional)

---

*Integration audit: 2026-03-11*
