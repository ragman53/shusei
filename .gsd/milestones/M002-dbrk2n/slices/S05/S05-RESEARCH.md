# S05: Model Bundling + Integration — Research

**Date:** 2026-03-16

## Summary

S05 is the final integration slice for M002, focusing on bundling NDLOCR and Moonshine models into the APK and verifying end-to-end flows on the Moto G66j 5G device. The slice has two main objectives: (1) ensure all required model files are present in `assets/models/` and bundled into the APK via Dioxus.toml configuration, and (2) perform device-level verification that camera→OCR, PDF→read, and word→save flows work correctly on real hardware.

**Key findings:**
- NDLOCR models exist (147MB total, 4 ONNX files in `assets/models/ndlocr/`) but worktree only has 2 models in `assets/ocr/models/`
- Moonshine models are **missing** — only README.md exists in `assets/models/moonshine/` with download instructions
- Dioxus.toml already has correct bundle config: `resources = ["assets/models/*", "assets/test/*"]`
- Camera page loads models from `app_data_dir.join("models")` at runtime — expects models to be copied alongside executable
- Android APK bundles assets automatically via Dioxus, but runtime path resolution differs from desktop

**Primary recommendation:** Acquire Moonshine models first (blocker), then verify NDLOCR model paths match between worktree and main project, build APK, and run device verification scripts. This slice is straightforward integration work — no novel architecture or complex patterns needed.

## Recommendation

**Approach:** Sequential verification and integration

1. **Acquire Moonshine models** — Download encoder/decoder ONNX files from Hugging Face (per README.md instructions). This is a hard blocker — cannot bundle what doesn't exist.

2. **Consolidate model paths** — Worktree uses `assets/ocr/models/` but main project uses `assets/models/ndlocr/`. Standardize on `assets/models/ndlocr/` and `assets/models/moonshine/` to match Dioxus.toml bundle config.

3. **Build APK with models** — Run `scripts/android-build.sh` to generate Gradle files, apply patch, and build debug APK.

4. **Verify model bundling** — Run `scripts/verify-apk-models.sh` to confirm all 4 NDLOCR + 4 Moonshine models are present in APK.

5. **Device testing** — Install APK on Moto G66j 5G via `adb install`, run `scripts/verify-s02-camera.sh` and manual flows for PDF reading and word collection.

**Why this approach:** Models must exist before bundling can be verified. Path consolidation ensures worktree matches main project structure. Verification scripts already exist and are comprehensive. Device testing is the ultimate proof — all flows must work on real hardware.

## Implementation Landscape

### Key Files

#### Model Files (must exist before build)
- `assets/models/ndlocr/deim-s-1024x1024.onnx` (39MB) — Text detection model
- `assets/models/ndlocr/parseq-ndl-16x256-30-tiny-192epoch-tegaki3.onnx` (35MB) — Recognition model (small)
- `assets/models/ndlocr/parseq-ndl-16x384-50-tiny-146epoch-tegaki2.onnx` (36MB) — Recognition model (medium) ← **Currently used by camera.rs**
- `assets/models/ndlocr/parseq-ndl-16x768-100-tiny-165epoch-tegaki2.onnx` (40MB) — Recognition model (large)
- `assets/models/moonshine/moonshine-tiny-en-encoder.onnx` (15-20MB) — English STT encoder ← **MISSING**
- `assets/models/moonshine/moonshine-tiny-en-decoder.onnx` (30-40MB) — English STT decoder ← **MISSING**
- `assets/models/moonshine/moonshine-tiny-ja-encoder.onnx` (15-20MB) — Japanese STT encoder ← **MISSING**
- `assets/models/moonshine/moonshine-tiny-ja-decoder.onnx` (30-40MB) — Japanese STT decoder ← **MISSING**

#### Build Configuration
- `Dioxus.toml` — Bundle config already correct: `resources = ["assets/models/*", "assets/test/*"]`
- `scripts/android-build.sh` — Wrapper script that runs `dx build`, applies patch, builds APK via gradlew
- `scripts/android-patch.sh` — Fixes Gradle Java version (8→17), removes deprecated manifest attributes, disables broken lint tasks
- `scripts/verify-apk-models.sh` — Verifies APK contains all required ONNX files, reports counts and sizes

#### Runtime Model Loading
- `src/ui/camera.rs:54-70` — Initializes `NdlocrEngine` on mount, loads from `app_data_dir.join("models")`
- `src/core/ocr/engine_tract.rs:66-113` — `NdlocrEngineTract::initialize()` loads detection, recognition, direction models + dictionary
- `src/core/stt/engine_tract.rs:40-74` — `MoonshineEngineTract::initialize()` loads encoder/decoder models
- `src/core/ocr/mod.rs:21-31` — Defines `MODEL_DETECTION_PATH` and `MODEL_RECOGNITION_PATH` constants

#### Verification Scripts
- `scripts/verify-apk-models.sh` — Checks APK for ONNX files, counts models, verifies sizes
- `scripts/verify-s02-camera.sh` — Device testing for camera capture flow (already exists from S02)
- `scripts/verify-app-launch.sh` — General app launch verification on device

### Build Order

1. **Moonshine model acquisition** (BLOCKER) — Download 4 ONNX files to `assets/models/moonshine/`. Cannot proceed without these.

2. **Model path consolidation** — Copy/symlink NDLOCR models from `assets/ocr/models/` to `assets/models/ndlocr/` in worktree to match main project structure. Update camera.rs if needed to use correct path.

3. **APK build** — Run `bash scripts/android-build.sh` to generate Gradle files, apply patch, build debug APK (~139MB expected).

4. **APK verification** — Run `bash scripts/verify-apk-models.sh` to confirm all 8 models are bundled.

5. **Device install** — `adb install -r target/dx/shusei/debug/android/app/app/build/outputs/apk/debug/app-debug.apk`

6. **End-to-end testing** — Manual flows: camera→OCR→save, PDF→read→progress, word→save→vocab list

### Verification Approach

**Automated:**
```bash
# Build APK
bash scripts/android-build.sh

# Verify models in APK
bash scripts/verify-apk-models.sh

# Run desktop integration tests (regression check)
cargo test --test camera_ocr_integration
cargo test --lib reader::
cargo test --lib vocab::
```

**Device Testing (Moto G66j 5G):**
```bash
# Install APK
adb install -r target/dx/shusei/debug/android/app/app/build/outputs/apk/debug/app-debug.apk

# Run camera verification
bash scripts/verify-s02-camera.sh

# Manual flows:
# 1. Create book → capture 2 pages → OCR → save → verify in library
# 2. Import PDF → convert → scroll → verify progress saves
# 3. Tap 3 words → save → open vocabulary → verify words persist

# Check logs
adb logcat | grep -i shusei | grep -i "OCR\|model\|engine"
```

**Database Verification (on device):**
```bash
# After camera flow
adb shell "sqlite3 /data/data/com.shusei.app/files/shusei.db 'SELECT * FROM book_pages;'"

# After word collection
adb shell "sqlite3 /data/data/com.shusei.app/files/shusei.db 'SELECT word, context_text FROM words;'"
```

## Don't Hand-Roll

| Problem | Existing Solution | Why Use It |
|---------|------------------|------------|
| Model download | Hugging Face curl commands (README.md) | Official source, verified URLs, no need to host elsewhere |
| APK inspection | `unzip -l apk_file` + grep | Standard tool, no special setup required |
| Gradle patching | `scripts/android-patch.sh` | Already tested and working from S01 |
| Device verification | `scripts/verify-s02-camera.sh` | Comprehensive UAT script from S02, reusable |

## Constraints

- **Moonshine models missing** — Hard blocker. Cannot bundle or test STT functionality without encoder/decoder ONNX files.
- **Model size** — NDLOCR (147MB) + Moonshine (~90MB estimated) = ~240MB total. APK will be 250-300MB. Moto G66j 5G storage capacity unknown but should handle this.
- **Worktree path mismatch** — Worktree uses `assets/ocr/models/` but main project uses `assets/models/ndlocr/`. Camera.rs expects models in `app_data_dir.join("models")` at runtime.
- **Dioxus asset bundling** — Dioxus 0.7 bundles assets specified in `Dioxus.toml` but runtime path resolution on Android differs from desktop (assets extracted to APK internal storage).
- **No Moonshine tokenizer integration** — `engine_tract.rs:173-180` returns placeholder tokens; full STT pipeline not yet implemented (deferred to S08 per comments).

## Common Pitfalls

- **Model path confusion** — Camera.rs loads from `app_data_dir.join("models")` but build system bundles to `assets/models/`. On Android, Dioxus extracts assets to app-internal storage; runtime code must use correct path (may need `include_str!` or Android AssetManager JNI).
- **Moonshine incomplete** — Even with models, `MoonshineEngineTract::transcribe()` returns empty tokens (TODO in code). S05 should verify model loading works but cannot prove full STT pipeline.
- **APK size limits** — Google Play has 150MB APK limit (with expansion files up to 2GB). For prototype, direct install via adb has no limit, but device storage may be constrained.
- **Model loading time** — OCR engine initialization takes 2-5s on desktop (per S02 summary). On mid-range Android device, may take 5-10s. Loading state UI already implemented but user experience may vary.

## Open Risks

- **Moonshine model availability** — Hugging Face links in README.md may be outdated or models may have moved. Alternative source may be needed.
- ** tract compatibility** — Moonshine models may have ONNX opset version incompatibilities with tract-onnx 0.21. Fallback to Whisper Tiny documented in README.md but not yet tested.
- **Device memory constraints** — Moto G66j 5G has moderate RAM. Loading 240MB of models + camera capture + OCR inference may cause OOM crashes. No graceful degradation implemented.
- **Android asset access** — Dioxus 0.7 Android asset bundling mechanism not fully documented. May require JNI calls to Android AssetManager if standard file I/O doesn't work on bundled assets.

## Skills Discovered

| Technology | Skill | Status |
|------------|-------|--------|
| Dioxus Android | None found | No specific Dioxus Android skill in available_skills |
| Android APK | None found | No Android build skill in available_skills |
| ONNX/tract | None found | No ML inference skill in available_skills |

## Sources

- **Dioxus 0.7 Asset Bundling** — `Dioxus.toml` bundle configuration and S01 build experience
- **NDLOCR Models** — `assets/models/ndlocr/` directory and `assets/ocr/README.md`
- **Moonshine Models** — `assets/models/moonshine/README.md` with download instructions
- **Model Loading Code** — `src/core/ocr/engine_tract.rs`, `src/core/stt/engine_tract.rs`, `src/ui/camera.rs`
- **Verification Scripts** — `scripts/verify-apk-models.sh`, `scripts/verify-s02-camera.sh`
- **S01 Build Experience** — Gradle patch script and android-build.sh wrapper
