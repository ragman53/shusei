# Coding Conventions

**Analysis Date:** 2026-03-11

## Naming Patterns

**Files:**
- Snake_case for all Rust files: `engine.rs`, `mod.rs`, `postprocess.rs`
- Module entry points: `mod.rs` in each directory

**Modules:**
- `src/core/mod.rs` - Core business logic
- `src/ui/mod.rs` - UI components
- `src/platform/mod.rs` - Platform abstraction

**Structs:**
- PascalCase: `OcrEngine`, `SttResult`, `NdlocrEngine`, `MoonshineEngine`
- Descriptive names that describe the concept

**Traits:**
- PascalCase with descriptive names: `OcrEngine`, `SttEngine`, `PlatformApi`
- Async traits use `#[async_trait]` attribute

**Functions:**
- Snake_case: `process_image`, `transcribe`, `capture_image`
- Async functions prefix not required (trait-based)

**Variables:**
- Snake_case: `image_data`, `max_duration_seconds`, `model_dir`
- Boolean flags: `is_ready`, `is_capturing`, `is_processing`

**Enums:**
- PascalCase variants: `Language::English`, `Language::Japanese`
- Error enums: `OcrError`, `SttError`, `ShuseiError`

**Constants:**
- UPPER_SNAKE_CASE: `ENGLISH_MODELS`, `JAPANESE_MODELS`

## Code Style

**Formatting:**
- Standard `rustfmt` formatting (no custom config file found)
- 4 spaces for indentation
- Max line length: standard Rust convention (~100 chars)

**Linting:**
- Clippy used (implied by Rust best practices)
- No custom clippy configuration

## Import Organization

**Order:**
1. Standard library imports (`std::path::Path`, `std::collections::HashMap`)
2. Third-party crate imports (`serde`, `async_trait`, `tract_onnx`)
3. Internal module imports (`crate::core::error`)

**Patterns:**
```rust
use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use async_trait::async_trait;

use crate::core::error::{OcrError, Result};
```

**Re-exports:**
- `pub use` pattern for exposing public API
- Example from `src/core/mod.rs`:
```rust
pub use error::{ShuseiError, OcrError, SttError};
pub use ocr::OcrEngine;
pub use stt::SttEngine;
pub use db::Database;
```

## Documentation

**Module Documentation:**
- Every module starts with a doc comment (`//!`)
- Describes the module's purpose

```rust
//! OCR (Optical Character Recognition) pipeline
//!
//! This module implements the OCR pipeline using NDLOCR-Lite ONNX models
//! with the tract inference runtime.
```

**Item Documentation:**
- Public structs and functions have doc comments
- Include usage examples for complex items

**Doc Comments:**
- `//!` for module-level documentation
- `///` for item-level documentation

## Error Handling

**Strategy:** `thiserror` for custom error types

**Pattern:**
```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum OcrError {
    #[error("Image preprocessing failed: {0}")]
    Preprocessing(String),

    #[error("Model loading failed: {0}")]
    ModelLoading(String),
    // ...
}
```

**Result Type:**
- Custom `Result<T>` alias for `std::result::Result<T, ShuseiError>`
- Located in `src/core/error.rs`

**Error Propagation:**
- Use `?` operator for automatic conversion
- `#[from]` attribute for automatic error conversion

## Logging

**Framework:** `log` crate with `env_logger`

**Patterns:**
```rust
log::info!("Initializing NDLOCR engine from {:?}", self.model_dir);
log::warn!("Direction classifier model not found, direction classification will be disabled");
log::error!("Capture failed: {}", e);
log::debug!("Preprocessing audio: {} samples", audio.len());
```

**Levels:**
- `info!` - Initialization, completion events
- `warn!` - Non-critical issues
- `error!` - Errors that impact functionality
- `debug!` - Detailed debugging information

## Function Design

**Size:** Functions tend to be focused and under 50 lines

**Parameters:**
- Use `impl Into<PathBuf>` for flexible path arguments
- Use `Option<T>` for optional parameters in structs

**Async Functions:**
- Marked with `async fn` or `#[async_trait]` for traits
- Return `Result<T>` for fallible operations

**Builder Pattern:**
- `Default` trait for configuration structs
- Example: `OcrConfig`, `SttConfig`

## Module Design

**Structure:**
```
src/
├── core/          # Business logic
│   ├── error.rs   # Error types
│   ├── db.rs      # Database operations
│   ├── ocr/       # OCR module
│   ├── stt/       # STT module
│   └── vocab.rs   # Vocabulary management
├── ui/            # Dioxus UI components
├── platform/      # Platform abstraction
├── app.rs         # Main app component
├── lib.rs         # Library entry
└── main.rs        # Binary entry
```

**Visibility:**
- Public API: `pub`
- Module-private: default (no keyword)
- Re-export from parent modules for convenience

## Traits and Generics

**Async Trait Pattern:**
```rust
use async_trait::async_trait;

#[async_trait]
pub trait OcrEngine: Send + Sync {
    async fn process_image(&self, image_data: &[u8]) -> Result<OcrResult>;
    fn is_ready(&self) -> bool;
    fn name(&self) -> &'static str;
}
```

**Generic Bounds:**
- `Send + Sync` for thread safety
- `impl AsRef<Path>` for path parameters

## Serde Serialization

**Pattern:**
```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SttResult {
    pub text: String,
    pub confidence: Option<f32>,
    // ...
}
```

**Feature Gates:**
- Some modules conditionally compiled: `#[cfg(feature = "pdf")]`

## Testing Conventions

**Inline Tests:**
- Located in `#[cfg(test)]` modules at bottom of file
- Example in `src/core/db.rs` (lines 287-313)

**Integration Tests:**
- Located in `tests/` directory
- Named: `*_test.rs` pattern

---

*Convention analysis: 2026-03-11*
