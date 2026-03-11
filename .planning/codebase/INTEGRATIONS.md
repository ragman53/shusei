# External Integrations

**Analysis Date:** 2026-03-11

## APIs & External Services

**No External Cloud APIs** - This is an offline-first application with no external API dependencies. All processing is done locally.

**Platform-Specific APIs:**

### Android (JNI Integration)
- **File**: `src/platform/android.rs`
- **JNI Bridge**: Uses `jni` crate (version 0.21)
- **Java Package**: `com.shusei.app`
- **Main Activity**: `MainActivity`

**JNI Methods Exposed:**
```rust
// Native initialization
Java_com_shusei_app_MainActivity_nativeInit(JNIEnv, JClass)

// Image capture callback
Java_com_shusei_app_MainActivity_onImageCaptured(
    JNIEnv, JClass, jbyteArray, jint width, jint height
)
```

**Java Methods Called from Rust:**
- `startCameraCapture()` - Initiates camera capture
- `vibrate(long)` - Haptic feedback
- `hasCameraPermission()` - Check camera permission
- `requestCameraPermission()` - Request camera permission

### iOS (Planned)
- **File**: `src/platform/ios.rs`
- **Status**: Stub implementation, post-MVP
- **Planned APIs**:
  - `AVCaptureSession` - Camera access
  - `AVAudioRecorder` - Audio recording
  - `UIDocumentPickerViewController` - File picking
  - `UIImpactFeedbackGenerator` - Haptic feedback

## Data Storage

### Local Database (Primary)
**SQLite with rusqlite**
- **File**: `src/core/db.rs`
- **Library**: rusqlite 0.32 with `bundled` feature
- **Features**: FTS5 full-text search enabled

**Database Schema:**
- `sticky_notes` - OCR results and voice transcripts
- `sticky_notes_fts` - FTS5 virtual table for search
- `books` - PDF book metadata
- `book_pages` - Converted page content
- `vocabulary` - User vocabulary with review tracking

**Indexes:**
- `idx_sticky_notes_book` - Book title lookup
- `idx_sticky_notes_created` - Chronological ordering
- `idx_books_updated` - Recent books
- `idx_vocab_word` - Word lookup

### File Storage
**Local Filesystem Only**
- Image captures: Temporary storage during processing
- PDF files: User-selected documents
- ONNX models: `assets/models/` directory
  - `moonshine/` - STT models (not committed, ~50-60MB per language)
  - `ndlocr/` - OCR models (not committed, ~8-17MB total)

**No Cloud Storage** - All data remains on device

### Caching
- In-memory caching via `once_cell` and `parking_lot`
- No external caching service (Redis, etc.)

## Authentication & Identity

**No Authentication Provider**
- Fully offline application
- No user accounts
- No identity management
- No OAuth, JWT, or session management

## ML Model Integration

### OCR Engine (NDLOCR-Lite)
**tract-onnx Runtime**
- **File**: `src/core/ocr/engine.rs`
- **Models Required**:
  - `text_detection.onnx` (~2-5MB)
  - `text_recognition.onnx` (~5-10MB)
  - `direction_classifier.onnx` (~1-2MB)
- **Model Source**: https://github.com/ndl-lab/ndlocr_ocr
- **Input**: Image tensor [1, 3, H, W]
- **Output**: Text regions with bounding boxes and confidence

**OCR Pipeline:**
1. Preprocess image (grayscale, normalization)
2. Detect text regions
3. Classify text direction (0°, 90°, 180°, 270°)
4. Recognize text in each region
5. Sort by reading order
6. Generate markdown output

### STT Engine (Moonshine Tiny)
**tract-onnx Runtime**
- **File**: `src/core/stt/engine.rs`
- **Models Required** (per language):
  - `moonshine-tiny-{lang}-encoder.onnx` (~15-20MB)
  - `moonshine-tiny-{lang}-decoder.onnx` (~30-40MB)
- **Model Source**: https://github.com/usefulsensors/moonshine
- **Supported Languages**: English, Japanese (planned)
- **Input**: Log-mel spectrogram [batch, 80, time_frames]
- **Output**: Transcribed text with confidence

**STT Pipeline:**
1. Load audio (WAV via hound)
2. Resample to 16kHz if needed
3. Compute mel-spectrogram (80 bins, 25ms window, 10ms hop)
4. Log compression
5. Encoder forward pass
6. Autoregressive decoder generation
7. Token decoding

## Monitoring & Observability

**Error Tracking:**
- None (no Sentry, Rollbar, etc.)
- Errors logged to console via `env_logger`

**Logging:**
- Framework: `log` crate with `env_logger`
- Configuration: Environment variable `RUST_LOG`
- Default level: `info`
- Usage in code:
```rust
log::info!("Starting Shusei...");
log::debug!("Processing image: {} bytes", data.len());
log::error!("Failed to load model: {}", e);
```

**Metrics:**
- None (no Prometheus, StatsD, etc.)
- Manual timing for ML inference operations

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
cargo build --target wasm32-unknown-unknown --features web

# Tests
cargo test
cargo test --features ndlocr-test  # Requires ONNX models
cargo test --features moonshine-test  # Requires ONNX models
```

**Deployment:**
- Desktop: Native binaries
- Android: APK via Dioxus bundling
- iOS: Not yet implemented
- Web: WASM bundle

## Environment Configuration

**No Environment Variables Required**
- No API keys
- No database connection strings
- No external service configuration

**Optional Configuration:**
- `RUST_LOG` - Log level (default: info)
- `RUST_BACKTRACE` - Stack trace on panic

## Webhooks & Callbacks

**No Incoming Webhooks**
- No HTTP server
- No webhook endpoints

**No Outgoing Webhooks**
- No external callbacks
- No event streaming

**Internal Callbacks (JNI):**
- `onImageCaptured` - Called from Java when camera capture completes
- Triggered by Android Camera API, dispatched to Rust via JNI

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
| Monitoring | None | N/A |
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
- ONNX models expected in `assets/models/`
- Models not committed to repository
- Download instructions in test files:
  - `tests/moonshine_tract_test.rs`
  - `tests/ndlocr_tract_test.rs`

---

*Integration audit: 2026-03-11*
