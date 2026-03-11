# Testing Patterns

**Analysis Date:** 2026-03-11

## Test Framework

**Runner:**
- Rust built-in test framework (`cargo test`)
- No external test framework (no `test-group`, `rstest`, etc.)
- Config: `Cargo.toml` with feature flags for test variants

**Assertion Library:**
- Standard `assert!()`, `assert_eq!()`, `assert!(...is_none())`
- `panic!()` for explicit failures in integration tests
- No external assertion library (no `assert_cmd`, `pretty_assertions`)

**Run Commands:**
```bash
cargo test                    # Run all tests
cargo test --lib              # Run library tests only
cargo test --test <name>      # Run specific integration test
cargo test --features ndlocr-test  # Run with feature flag
cargo test -- --nocapture     # Show println! output
cargo test <pattern>          # Run tests matching pattern
```

## Test File Organization

**Location:**
- Integration tests: `tests/` directory at project root
- Unit tests: `#[cfg(test)] mod tests` within source files
- Co-located pattern for unit tests

**Naming:**
- Integration tests: `{feature}_tract_test.rs`
  - `moonshine_tract_test.rs`
  - `ndlocr_tract_test.rs`
- Unit test modules: `mod tests` inside source files

**Structure:**
```
shusei/
├── src/
│   ├── core/
│   │   ├── db.rs           # Contains #[cfg(test)] mod tests
│   │   └── ...
│   └── ...
└── tests/
    ├── moonshine_tract_test.rs
    └── ndlocr_tract_test.rs
```

## Test Structure

**Unit Test Organization (from `src/core/db.rs`):**
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

**Integration Test Organization (from `tests/moonshine_tract_test.rs`):**
```rust
#[cfg(feature = "moonshine-test")]
mod tract_tests {
    use super::*;
    use tract_onnx::prelude::*;

    #[test]
    fn test_english_encoder() {
        test_model("moonshine-tiny-en-encoder.onnx");
    }

    #[test]
    fn test_english_decoder() {
        test_model("moonshine-tiny-en-decoder.onnx");
    }
}

#[test]
fn test_models_existence_check() {
    // Always runs, checks model availability
}
```

**Patterns:**
- Setup: Direct instantiation or helper functions
- Teardown: Not used (Rust drops automatically)
- Assertions: Standard library `assert!` macros
- Test helpers: Private functions within test modules

## Mocking

**Framework:** None (Rust standard library only)

**Patterns:**
- In-memory databases for isolation: `Database::in_memory()`
- Trait-based abstraction for testability: `OcrEngine`, `SttEngine`, `PlatformApi`
- Desktop stub implementations for platform APIs

**Example from `src/platform/mod.rs`:**
```rust
/// Desktop platform implementation (for testing/development)
pub struct DesktopPlatform;

#[async_trait]
impl PlatformApi for DesktopPlatform {
    async fn capture_image(&self) -> Result<CameraResult> {
        Err(ShuseiError::Platform("Camera not available on desktop".into()).into())
    }
    // ... stub implementations
}
```

**What to Mock:**
- External hardware (camera, microphone)
- Platform-specific APIs
- File system access (use in-memory or temp files)

**What NOT to Mock:**
- Business logic (test directly)
- Database operations (use in-memory SQLite)
- Error handling paths (test with actual errors)

## Fixtures and Factories

**Test Data:**
```rust
// From src/core/db.rs
let new_note = NewStickyNote {
    ocr_markdown: Some("# Test\nHello world".to_string()),
    ocr_text_plain: Some("Test Hello world".to_string()),
    ..Default::default()
};
```

**Helper Functions:**
```rust
// From tests/moonshine_tract_test.rs
fn get_model_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("assets")
        .join("models")
        .join("moonshine")
}

fn load_onnx_model(model_path: &PathBuf) -> Result<TypedModel, String> {
    tract_onnx::onnx()
        .model_for_path(model_path)
        .map_err(|e| format!("Failed to load model: {:?}", e))?
        // ...
}
```

**Location:**
- Helper functions defined at top of test files
- No separate `fixtures/` or `factories/` directory
- `Default` trait used for creating base instances

## Coverage

**Requirements:** None enforced (no `cargo-llvm-cov` or similar configured)

**View Coverage:**
```bash
# Install cargo-llvm-cov (if needed)
cargo install cargo-llvm-cov

# Generate coverage report
cargo llvm-cov
cargo llvm-cov --html
```

**Current Test Count:**
- 7 test functions identified with `#[test]`
- Unit tests in `src/core/db.rs`
- Integration tests in `tests/` directory

## Test Types

**Unit Tests:**
- Scope: Individual functions and methods
- Location: `#[cfg(test)] mod tests` within source files
- Approach: Test with in-memory data, verify outputs
- Example: Database CRUD operations

**Integration Tests:**
- Scope: Model loading, ONNX compatibility
- Location: `tests/` directory
- Approach: Feature-flagged, requires external models
- Example: `moonshine_tract_test.rs`, `ndlocr_tract_test.rs`

**E2E Tests:**
- Not used
- UI testing would require Dioxus testing utilities

## Common Patterns

**Async Testing:**
```rust
// Note: Tests are synchronous, async functions called with block_on if needed
// Current pattern: Test async-capable code synchronously where possible
#[test]
fn test_database_in_memory() {
    let db = Database::in_memory().unwrap();
    // Database methods may be async in real usage
}
```

**Error Testing:**
```rust
// Test error conditions with Result expectations
#[test]
fn test_model_not_found() {
    let result = engine.process_image(&data).await;
    assert!(result.is_err());
    match result.unwrap_err() {
        ShuseiError::Ocr(OcrError::ModelLoading(msg)) => {
            assert!(msg.contains("not found"));
        }
        _ => panic!("Expected ModelLoading error"),
    }
}
```

**Feature-Flagged Tests:**
```rust
// Tests that require external dependencies
#[cfg(feature = "moonshine-test")]
mod tract_tests {
    #[test]
    fn test_english_encoder() {
        // Only runs with --features moonshine-test
    }
}
```

**Skip Pattern:**
```rust
// Gracefully skip tests when prerequisites not met
#[test]
fn test_models_existence_check() {
    if !models_exist() {
        eprintln!("SKIP: Models not found");
        return;
    }
    // ... test logic
}
```

## Test Documentation

**Inline Documentation:**
- Module-level doc comments explain test purpose
- Comments document expected model files and locations
- README-style headers in test files

**Example from `tests/moonshine_tract_test.rs`:**
```rust
//! Moonshine Tiny ONNX tract compatibility test
//!
//! ## Models Required
//! Moonshine is a family of speech-to-text models...
//!
//! ## Running the Test
//! 1. Download the Moonshine Tiny ONNX models
//! 2. Place them in `assets/models/moonshine/`
//! 3. Run: `cargo test moonshine_tract_test --features moonshine-test`
```

---

*Testing analysis: 2026-03-11*
