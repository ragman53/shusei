# External Integrations

**Analysis Date:** 2026-03-13

## APIs & External Services

**None - Fully Offline Application**

Shusei is designed as a completely offline-first application with no external API dependencies:
- No cloud services
- No external HTTP APIs
- No third-party SDK integrations
- All ML inference runs locally via ONNX Runtime

## Platform-Specific APIs

### Android (JNI Integration)
- **File**: `src/platform/android.rs`
- **JNI Bridge**: Uses `jni` crate (version 0.21)
- **Java Package**: `com.shusei.app`
- **Main Activity**: `MainActivity`

**JNI Native Methods (Rust → Java callbacks):**
```rust
// Native initialization - stores JavaVM reference
Java_com_shusei_app_MainActivity_nativeInit(JNIEnv, JClass)

// Image capture callback - receives camera data from Java
Java_com_shusei_app_MainActivity_onImageCaptured(
    JNIEnv, JClass, jbyteArray imageData, jint width, jint height
)
```

**Java Methods Called from Rust:**
| Method | Purpose |
|--------|---------|
| `startCameraCapture()` | Initiates camera capture via CameraX |
| `vibrate(long ms)` | Haptic feedback |
| `hasCameraPermission()` | Check camera permission status |
| `requestCameraPermission()` | Request camera permission |

**Static State:**
- `CAMERA_STATE`: Mutex<Option<CameraState>> - Pending capture result channel
- `JAVA_VM`: Lazy<Mutex<Option<JavaVM>>> - Stored JavaVM reference

### iOS
- **File**: `src/platform/ios.rs`
- **Status**: Stub implementation only
- **Planned APIs**: AVCaptureSession, AVAudioRecorder, UIDocumentPickerViewController

### Desktop
- **File**: `src/platform/mod.rs` (DesktopPlatform struct)
- **File Picker**: `rfd` 0.15 crate for native dialogs
- **Camera/Microphone**: Returns error (not implemented)

## Data Storage

### Local Database (Primary)
**SQLite with rusqlite**
- **File**: `src/core/db.rs`
- **Library**: rusqlite 0.32 with `bundled` feature
- **Features**: FTS5 full-text search, WAL mode for concurrent reads

**Database Schema:**
| Table | Purpose |
|-------|---------|
| `sticky_notes` | OCR results and voice transcripts |
| `sticky_notes_fts` | FTS5 virtual table for full-text search |
| `books` | PDF/book metadata with cover images |
| `book_pages` | Converted page content with OCR results |
| `vocabulary` | User vocabulary with review tracking |
| `processing_progress` | PDF conversion progress tracking |

**Database Path:**
- Desktop: `{current_dir}/.shusei/shusei.db`
- Android: `/data/data/com.shusei.app/files/shusei.db`

### File Storage
**Local Filesystem Only** - `src/core/storage.rs`
| Path | Purpose |
|------|---------|
| `{assets_dir}/images/` | General image storage |
| `{assets_dir}/pages/{book_id}/` | PDF page images |
| `{assets_dir}/pdfs/` | Imported PDF files |
| `{data_dir}/.shusei/app_state.json` | App state persistence |

**No Cloud Storage** - All data remains on device

### Caching
- In-memory caching via `parking_lot` Mutex/RwLock
- ONNX model sessions cached in `NdlocrEngine` and `MoonshineEngine`
- No external caching service

## Machine Learning Models

### OCR Engine (NDLOCR-Lite)
**ONNX Runtime / tract**
- **File**: `src/core/ocr/engine.rs`
- **Engine**: `NdlocrEngine` struct
- **Models Required**:
  - `deim-s-1024x1024.onnx` - Text detection (1024x1024 input)
  - `parseq-ndl-16x384-50-tiny-146epoch-tegaki2.onnx` - Text recognition
  - `direction_classifier.onnx` - Text direction (optional)
  - `dict.txt` - Character dictionary

**OCR Pipeline:**
1. Preprocess image → RGB tensor [1, 3, 1024, 1024]
2. Detect text regions via DEIM model
3. Extract and resize each region to 32px height
4. Recognize text via Parseq model with CTC decoding
5. Aggregate results with confidence scores

**Parallel Processing:**
- `process_pages_parallel()` - Batch OCR with concurrency limit of 3
- Retry logic: Up to 3 attempts per page
- Progress callback for UI updates

### Speech-to-Text (Moonshine Tiny)
**tract-onnx Runtime**
- **File**: `src/core/stt/engine.rs`
- **Engine**: `MoonshineEngine` struct
- **Models Required**:
  - `encoder.onnx` - Audio encoder (~15-20MB)
  - `decoder.onnx` - Text decoder (~30-40MB)

**STT Pipeline (Planned):**
1. Load audio (WAV via hound, 16kHz mono)
2. Compute mel-spectrogram (80 bins, 25ms window, 10ms hop)
3. Encoder forward pass
4. Autoregressive decoder generation with KV cache
5. Token decoding to text

**Current Status:**
- Engine structure implemented
- Model loading checks implemented
- Inference pipeline not yet implemented (TODO in source)

## Authentication & Identity

**No Authentication Provider**
- Fully offline application
- No user accounts
- No identity management
- No OAuth, JWT, or session management

## Monitoring & Observability

**Error Tracking:**
- None (no Sentry, Rollbar, etc.)
- Errors handled via `anyhow` and `thiserror`

**Logging:**
- Framework: `log` crate with `env_logger` backend
- Configuration: `src/main.rs` lines 14-15
  ```rust
  env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
      .init();
  ```
- Default level: `info`
- Environment variable: `RUST_LOG` for level control

**Metrics:**
- None (no Prometheus, StatsD, etc.)
- Manual timing via `std::time::Instant` for ML inference

## CI/CD & Deployment

**Not Configured**
- No CI pipeline detected (no `.github/workflows/`, `.gitlab-ci.yml`, etc.)
- No automated testing on PR
- No automated releases

**Build Commands:**
```bash
# Desktop
cargo run

# Android (requires NDK)
cargo build --target aarch64-linux-android --features android

# Web
dx serve  # Dioxus CLI

# Tests
cargo test
cargo test --features ndlocr-test  # Requires ONNX models
cargo test --features moonshine-test  # Requires ONNX models
```

## Environment Configuration

**No Environment Variables Required**
- No API keys
- No database connection strings
- No external service configuration

**Optional Configuration:**
| Variable | Purpose | Default |
|----------|---------|---------|
| `RUST_LOG` | Log level | `info` |
| `RUST_BACKTRACE` | Stack trace on panic | unset |

## Webhooks & Callbacks

**No Incoming Webhooks**
- No HTTP server
- No webhook endpoints

**No Outgoing Webhooks**
- No external callbacks
- No event streaming

**Internal Callbacks (JNI):**
| Callback | Trigger |
|----------|---------|
| `onImageCaptured` | Android CameraX capture complete |
| Progress callbacks | PDF rendering, OCR processing |

## External Dependencies Summary

| Category | Service | Status |
|----------|---------|--------|
| Cloud APIs | None | N/A |
| Auth | None | N/A |
| Database | SQLite (local) | ✓ Active |
| File Storage | Local filesystem | ✓ Active |
| Caching | In-memory | ✓ Active |
| Error Tracking | None | N/A |
| CI/CD | None | ⚠ Not configured |
| Monitoring | Console logging | ✓ Active |
| ML Models | NDLOCR, Moonshine | ✓ Local ONNX |
| Mobile Platform | Android JNI | ✓ Active |
| Web Platform | WASM | ✓ Supported |

## Security Considerations

**No Secrets in Code**
- No API keys to leak
- No credentials file
- `.env` files not used

**All Processing Local**
- No data leaves device
- No network requests
- Fully offline capable

**Model Assets**
- ONNX models expected in `assets/models/` and `assets/ocr/models/`
- Models gitignored in `.gitignore`
- Download instructions:
  - `assets/ocr/README.md` - NDLOCR models
  - `assets/models/moonshine/README.md` - Moonshine models

---

*Integration audit: 2026-03-13*