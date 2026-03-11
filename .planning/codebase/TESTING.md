# Testing Patterns

**Analysis Date:** 2026-03-11

## Test Framework

**Runner:** Built-in Rust test framework (`cargo test`)
- No external test runner (like nextest) detected
- Standard `#[test]` attribute for test functions

**Test Organization:**
- Unit tests: Inline in `#[cfg(test)]` modules within source files
- Integration tests: Separate files in `tests/` directory

## Test File Organization

**Location:**
- Unit tests: At end of source file in `#[cfg(test)] mod tests`
- Integration tests: `tests/` directory at project root

**Naming:**
- Unit tests: `mod tests` inside source file
- Integration tests: `*_test.rs` pattern (e.g., `moonshine_tract_test.rs`, `ndlocr_tract_test.rs`)

**Current Test Files:**
- `tests/moonshine_tract_test.rs` - Moonshine Tiny ONNX compatibility tests
- `tests/ndlocr_tract_test.rs` - NDLOCR-Lite ONNX compatibility tests
- `src/core/db.rs` (lines 287-313) - Inline database tests

## Test Structure

**Inline Unit Test Pattern:**
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_database_in_memory() {
        let db = Database::in_memory().unwrap();
        let note = db.get_sticky_note(1).unwrap();
        assert!(note.is_none());
    }
    
    #[test]
    fn test_create_and_get_sticky_note() {
        let db = Database::in_memory().unwrap();
        
        let new_note = NewStickyNote {
            ocr_markdown: Some("# Test\nHello world".to_string()),
            ocr_text_plain: Some("Test Hello world".to_string()),
            ..Default::default()
        };
        
        let id = db.create_sticky_note(&new_note).unwrap();
        assert!(id > 0);
        
        let note = db.get_sticky_note(id).unwrap().unwrap();
        assert_eq!(note.ocr_markdown, Some("# Test\nHello world".to_string()));
    }
}
```

**Integration Test Pattern:**
```rust
//! Module-level documentation describing test purpose

use std::path::PathBuf;

/// Helper function
fn get_model_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("assets")
        .join("models")
}

/// Conditional compilation for feature-gated tests
#[cfg(feature = "moonshine-test")]
mod tract_tests {
    use super::*;
    use tract_onnx::prelude::*;

    #[test]
    fn test_model_loading() {
        // Test implementation
    }
}

/// Standard test (always runs)
#[test]
fn test_models_existence_check() {
    println!("\n=== Model Existence Check ===");
    // Test implementation
}
```

## Feature-Gated Tests

**Pattern:** Tests conditionally compiled based on Cargo features

**Features in `Cargo.toml`:**
```toml
[features]
default = []
ndlocr-test = []  # Enable NDLOCR tract compatibility tests
moonshine-test = []  # Enable Moonshine tract compatibility tests
```

**Usage:**
```rust
#[cfg(feature = "moonshine-test")]
mod tract_tests {
    // Tests that require ONNX models
}

// Tests outside the module always run
#[test]
fn test_always_runs() {
    // Basic availability checks
}
```

**Run Commands:**
```bash
# Run all tests
cargo test

# Run feature-gated tests
cargo test --features moonshine-test
cargo test --features ndlocr-test

# Run specific test
cargo test test_model_loading
```

## Test Patterns

**Model Existence Checks:**
```rust
fn models_exist() -> bool {
    let model_dir = get_model_dir();
    let detection = model_dir.join("text_detection.onnx");
    let recognition = model_dir.join("text_recognition.onnx");
    
    detection.exists() && recognition.exists()
}

#[test]
fn test_models_existence_check() {
    if models_exist() {
        println!("✓ All required models are present");
    } else {
        let missing = get_missing_models();
        println!("✗ Missing models: {:?}", missing);
    }
}
```

**Skip Pattern (Conditional Test):**
```rust
#[test]
fn test_model() {
    let model_path = get_model_dir().join("model.onnx");
    
    if !model_path.exists() {
        eprintln!("SKIP: Model not found at {:?}", model_path);
        return;  // Early return, test passes
    }
    
    // Actual test logic...
}
```

**Helper Functions:**
```rust
/// Load and analyze an ONNX model file
fn load_onnx_model(model_path: &PathBuf) -> Result<TypedModel, String> {
    tract_onnx::onnx()
        .model_for_path(model_path)
        .map_err(|e| format!("Failed to load model: {:?}", e))?
        .into_optimized()
        .map_err(|e| format!("Failed to optimize model: {:?}", e))?
        .into_run_config()
        .map_err(|e| format!("Failed to configure model: {:?}", e))
}
```

## Test Data and Fixtures

**Model Paths:**
- Models expected at: `assets/models/{model_type}/`
- Path construction using `CARGO_MANIFEST_DIR`:
```rust
fn get_model_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("assets")
        .join("models")
        .join("moonshine")
}
```

**Constants for Model Files:**
```rust
const ENGLISH_MODELS: [&str; 2] = [
    "moonshine-tiny-en-encoder.onnx",
    "moonshine-tiny-en-decoder.onnx",
];

const JAPANESE_MODELS: [&str; 2] = [
    "moonshine-tiny-ja-encoder.onnx",
    "moonshine-tiny-ja-decoder.onnx",
];
```

## Mocking

**No Mocking Framework:**
- No mocking libraries detected in `Cargo.toml`
- No `mockall`, `mockiato`, or similar crates

**Manual Stub Pattern:**
```rust
// In DesktopPlatform implementation
async fn capture_image(&self) -> Result<CameraResult> {
    Err(ShuseiError::Platform("Camera not available on desktop".into()).into())
}
```

**Test Doubles via Traits:**
- Traits like `OcrEngine`, `SttEngine`, `PlatformApi` allow different implementations
- `DesktopPlatform` acts as a stub for testing

## Coverage

**Coverage Tool:** Not configured
- No `tarpaulin` or `grcov` configuration found
- No coverage CI integration detected

**View Coverage (if using tarpaulin):**
```bash
# Install tarpaulin
cargo install cargo-tarpaulin

# Run with coverage
cargo tarpaulin --out Html
```

## Test Types

**Unit Tests:**
- Database operations (`src/core/db.rs`)
- Model creation and retrieval
- In-memory database for isolation

**Integration Tests:**
- ONNX model compatibility tests
- Tract runtime integration tests
- End-to-end model loading tests

**Documentation Tests:**
- Doc comments may contain code examples
- Run with `cargo test --doc`

## Testing Best Practices Observed

1. **In-memory databases** for unit tests to avoid file system dependencies
2. **Feature gates** for tests requiring external resources (model files)
3. **Early return pattern** for graceful skipping when resources unavailable
4. **Comprehensive logging** in tests with `println!` and `eprintln!`
5. **Detailed documentation** in test files explaining model requirements

## Known Testing Gaps

**Not Yet Implemented:**
- OCR engine tests (placeholder implementation)
- STT engine tests (placeholder implementation)
- UI component tests (Dioxus testing)
- Platform API tests (requires mocking)
- Vocabulary extraction tests (lindera not yet integrated)

**TODO Comments Related to Testing:**
- `src/core/ocr/engine.rs:79` - Model loading implementation
- `src/core/stt/engine.rs:68` - Model loading implementation
- Various pipeline implementations pending

---

*Testing analysis: 2026-03-11*
