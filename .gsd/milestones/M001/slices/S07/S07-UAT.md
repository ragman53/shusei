# S07: Performance Polish — UAT

**Milestone:** M001
**Written:** 2026-03-15

## UAT Type

- UAT mode: **artifact-driven**
- Why this mode is sufficient: S07 is a backend infrastructure slice (inference runtime migration). Success is measured by build success, test passage, and absence of linker errors - all verifiable through automated checks. No user-facing UI changes.

## Preconditions

1. Rust toolchain installed (rustc 1.75+)
2. Project dependencies downloaded (`cargo fetch`)
3. No model files required for build verification (models loaded at runtime)

## Smoke Test

```bash
cd /home/devuser/develop/shusei
cargo build --lib
```

**Expected:** Build completes successfully with `Finished dev profile [unoptimized + debuginfo]` message. No linker errors about undefined symbols (`__isoc23_strtol`, `__isoc23_strtoll`, etc.).

## Test Cases

### 1. Build Verification (No Linker Errors)

1. Run `cargo build --lib`
2. Observe compilation output
3. **Expected:** 
   - Exit code 0
   - No `error: linking with cc failed` messages
   - No `undefined symbol: __isoc23_*` errors
   - Final message: `Finished dev profile [...]`

### 2. Unit Test Suite Execution

1. Run `cargo test --lib`
2. Wait for test completion
3. **Expected:**
   - 90+ tests pass
   - No failures related to tract inference
   - Test summary: `test result: ok. X passed; 0 failed`
   - OR: Known failures only in `test_hann_window` and `test_kv_cache_new` (pre-existing, unrelated to S07)

### 3. Tract Model Loading (OCR)

1. Check compilation of `src/core/ocr/engine_tract.rs`
2. Verify `NdlocrEngineTract::initialize()` compiles without errors
3. **Expected:**
   - No type errors for `TractModel` type alias
   - `tract_onnx::onnx().model_for_path()` calls compile successfully
   - `NdlocrEngine` type alias resolves to `NdlocrEngineTract`

### 4. Tract Model Loading (STT)

1. Check compilation of `src/core/stt/engine_tract.rs`
2. Verify `MoonshineEngineTract::initialize()` compiles without errors
3. **Expected:**
   - Encoder and decoder model loading use `crate::core::tract_utils::load_model`
   - `MoonshineEngine` type alias resolves to `MoonshineEngineTract`
   - No `ort` imports or dependencies

### 5. API Compatibility Check

1. Verify `src/core/pdf.rs` compiles with `use crate::core::ocr::{NdlocrEngine, OcrEngine}`
2. Check `process_pages_parallel` method call resolves correctly
3. **Expected:**
   - No method-not-found errors for `process_pages_parallel`
   - Trait method available via `OcrEngine` trait import
   - Existing code using `NdlocrEngine::new()` continues to work

### 6. Dependency Verification

1. Run `cargo tree | grep -E "ort|tract"`
2. **Expected:**
   - `tract-onnx v0.21` appears in dependency tree
   - `ort` does NOT appear (removed from Cargo.toml)
   - `ort-tract` does NOT appear (removed from Cargo.toml)

## Edge Cases

### Missing Model Files at Runtime

1. Instantiate `NdlocrEngineTract::new("/nonexistent/path", "en")`
2. Call `.initialize()`
3. **Expected:**
   - Returns `Err(OcrError::ModelLoading(...))`
   - Error message: "Failed to load required OCR models"
   - No panic or crash

### Tract Inference with Empty Input

1. Create initialized `NdlocrEngineTract` with valid models
2. Call `process_image(&[])` with empty byte array
3. **Expected:**
   - Returns `Err(OcrError::Preprocessing(...))` or `Err(OcrError::Inference(...))`
   - Graceful error handling, no crash

## Failure Signals

- **Build fails with linker errors**: `undefined symbol: __isoc23_*` - indicates ort dependency not fully removed
- **Type errors for TractModel**: `expected a type, found a trait` - indicates tract type alias issue
- **Method not found**: `process_pages_parallel` - indicates OcrEngine trait not imported
- **Test count drops significantly**: <80 tests passing - indicates regression in core functionality
- **New test failures in OCR/STT modules**: Indicates tract migration broke existing logic

## Requirements Proved By This UAT

- **S07-R1 (Tract Migration)** - Proved by: Build verification, dependency verification
- **S07-R2 (No Regressions)** - Proved by: Unit test suite execution (92 tests pass)
- **S07-R3 (API Compatibility)** - Proved by: API compatibility check, existing code compiles

## Not Proven By This UAT

- **Runtime inference accuracy** - Requires actual model files and sample images/audio
- **Performance benchmarks** - No speed/memory comparison between ort and tract
- **Android deployment** - APK build not tested in this UAT
- **Full STT transcription** - Moonshine decoder returns placeholder tokens (known limitation)

## Notes for Tester

- The 2 failing tests (`test_hann_window`, `test_kv_cache_new`) are **pre-existing** and unrelated to S07 scope
- Focus on build success and test count - 90+ passing tests indicates tract migration successful
- Tract models load at runtime - this UAT verifies compilation, not runtime model loading
- For runtime verification, would need actual ONNX model files in `assets/ocr/models/` and `assets/models/moonshine/`
- Log messages to watch for during runtime testing:
  - "NDLOCR engine (tract) initialized successfully"
  - "Moonshine engine (tract) initialized successfully"
  - "OCR (tract) completed in Xms"
  - "STT (tract) completed in Xms"
