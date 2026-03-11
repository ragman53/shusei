# Coding Conventions

**Analysis Date:** 2026-03-11

## Naming Patterns

**Files:**
- Snake case for module files: `engine.rs`, `decoder.rs`, `tokenizer.rs`
- Module root files named `mod.rs`
- Test files use `{module}_test.rs` pattern: `moonshine_tract_test.rs`, `ndlocr_tract_test.rs`

**Functions:**
- Snake case: `process_image()`, `transcribe()`, `initialize_schema()`
- Async functions prefixed with `async`: `async fn capture_image()`
- Builder/constructor pattern: `new()`, `open()`, `in_memory()`

**Variables:**
- Snake case: `model_dir`, `initialized`, `audio_data`
- Option types clearly named: `image_path: Option<String>`
- Boolean flags: `initialized`, `enable_direction_classification`

**Types:**
- PascalCase for structs and enums: `OcrResult`, `SttEngine`, `ShuseiError`
- Trait names end with `Engine` or `Api`: `OcrEngine`, `SttEngine`, `PlatformApi`
- Result aliases: `Result<T>` (module-specific error type)

## Code Style

**Formatting:**
- No `.rustfmt.toml` detected - uses Rust defaults
- 4-space indentation (Rust standard)
- Maximum line length follows Rust conventions (~100 chars)
- Blank lines between function definitions
- Trailing commas in multi-line structs/arrays

**Linting:**
- No ESLint/Biome config (Rust project)
- Uses Rust compiler warnings
- `#[derive(Debug)]` on most types for debugging

## Import Organization

**Order:**
1. External crates: `use dioxus::prelude::*;`
2. Standard library: `use std::path::Path;`
3. Internal modules: `use crate::core::error::Result;`
4. Super module imports: `use super::Language;`

**Example from `src/core/db.rs`:**
```rust
use std::path::Path;

use rusqlite::{Connection, params, Row, OptionalExtension};
use serde::{Deserialize, Serialize};

use crate::core::error::{ShuseiError, Result};
```

**Path Aliases:**
- `crate::` for root module access
- `super::` for parent module access
- No path aliases configured in `Cargo.toml`

## Error Handling

**Patterns:**
- `thiserror` for custom error types with `#[derive(Error)]`
- `anyhow` for application-level errors
- Result type alias per module: `pub type Result<T> = std::result::Result<T, ShuseiError>;`
- Error conversion with `#[from]` attribute for automatic `?` propagation

**Example from `src/core/error.rs`:**
```rust
#[derive(Error, Debug)]
pub enum ShuseiError {
    #[error("OCR error: {0}")]
    Ocr(#[from] OcrError),

    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}
```

**Error Propagation:**
- Use `?` operator for automatic conversion
- `.into()` for manual conversion when needed
- Context added with string interpolation: `OcrError::ModelLoading(format!(...))`

## Logging

**Framework:** `log` crate with `env_logger` for initialization

**Patterns:**
- `log::info!()` for startup/shutdown events
- `log::debug!()` for detailed operation info
- `log::warn!()` for non-critical issues
- `log::error!()` avoided - errors returned as `Result` instead

**Example from `src/core/ocr/engine.rs`:**
```rust
log::info!("Initializing NDLOCR engine from {:?}", self.model_dir);
log::warn!("Direction classifier model not found, direction classification will be disabled");
```

**Configuration:**
- Initialized in `main.rs`: `env_logger::Env::default().default_filter_or("info")`
- Runtime filtering via `RUST_LOG` environment variable

## Comments

**When to Comment:**
- Module-level doc comments (`//!`) for every module
- Function doc comments (`///`) for public APIs
- Inline comments (`//`) for TODOs and complex logic

**JSDoc/TSDoc equivalent (Rust doc comments):**
```rust
//! Shusei - Offline reading app with OCR and STT capabilities
//!
//! Library module exposing core functionality.

/// Top-level error type for the Shusei application
#[derive(Error, Debug)]
pub enum ShuseiError { ... }
```

**TODO Pattern:**
- Format: `// TODO: Description`
- Often includes context: `// TODO: Implement mel-spectrogram computation`
- Tracked in codebase: 28 TODOs found across source files

## Function Design

**Size:**
- Small to medium functions (10-50 lines typical)
- Single responsibility per function
- Helper functions extracted for complex operations

**Parameters:**
- Simple types passed by value: `id: i64`, `path: impl AsRef<Path>`
- Large types passed by reference: `note: &NewStickyNote`
- Configuration structs for many parameters: `OcrConfig`, `SttConfig`

**Return Values:**
- Always `Result<T>` for fallible operations
- `Option<T>` for optional values
- Specific result types: `OcrResult`, `SttResult`, `CameraResult`

## Module Design

**Exports:**
- Public re-exports in `mod.rs` files
- Clear separation between public API and internal implementation
- Use of `pub use` for convenience re-exports

**Example from `src/core/mod.rs`:**
```rust
pub mod error;
pub mod ocr;
pub mod stt;

pub use error::{ShuseiError, OcrError, SttError};
pub use ocr::OcrEngine;
pub use stt::SttEngine;
```

**Barrel Files:**
- `mod.rs` files serve as barrel files for each module
- `ui/components.rs` exports shared UI components
- `lib.rs` re-exports commonly used types at crate root

## Trait Design

**Patterns:**
- Async traits use `#[async_trait]` macro
- Trait names end with descriptive suffix: `Engine`, `Api`
- Common methods: `is_ready()`, `name()`, primary operation method

**Example from `src/core/ocr/engine.rs`:**
```rust
#[async_trait]
pub trait OcrEngine: Send + Sync {
    async fn process_image(&self, image_data: &[u8]) -> Result<OcrResult>;
    fn is_ready(&self) -> bool;
    fn name(&self) -> &'static str;
}
```

## Data Structures

**Structs:**
- Use `#[derive(Debug, Clone, Serialize, Deserialize)]` for data types
- Separate `New*` and `Update*` structs for database operations
- Builder pattern not used - direct struct initialization

**Example from `src/core/db.rs`:**
```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StickyNote {
    pub id: i64,
    pub created_at: String,
    pub ocr_markdown: Option<String>,
    // ...
}
```

## Async Patterns

**Runtime:** Tokio with `rt-multi-thread`, `sync`, `time`, `fs` features

**Patterns:**
- `async fn` for I/O operations
- `#[async_trait]` for trait methods
- `await` for async calls
- No explicit spawn in library code

---

*Convention analysis: 2026-03-11*
