# Coding Conventions

**Analysis Date:** 2026-03-13

## Naming Patterns

**Files:**
- Module files: `snake_case.rs` (e.g., `engine.rs`, `mod.rs`)
- Test files: `*_test.rs` in `tests/` directory (e.g., `ndlocr_tract_test.rs`)
- Module subdirectories: `snake_case/` (e.g., `src/core/ocr/`)

**Functions:**
- Public functions: `snake_case` (e.g., `process_image`, `save_to_prefs`)
- Private helper functions: `snake_case` (e.g., `preprocess_image_for_inference`)
- Async functions: Named same as sync, distinguished by `async` keyword
- Test functions: `test_<description>` or `test_<feature>_<behavior>` pattern

**Variables:**
- Local variables: `snake_case` (e.g., `image_data`, `book_id`)
- Constants: `SCREAMING_SNAKE_CASE` (e.g., `MODEL_DETECTION_PATH`, `ENGLISH_MODELS`)
- Static variables: `SCREAMING_SNAKE_CASE`

**Types:**
- Structs: `PascalCase` (e.g., `OcrEngine`, `SttResult`, `AppState`)
- Enums: `PascalCase` for type, `PascalCase` for variants (e.g., `Language::English`, `Route::Home`)
- Traits: `PascalCase` with descriptive names (e.g., `OcrEngine`, `SttEngine`)
- Type aliases: `PascalCase` (e.g., `Result<T>` in `src/core/error.rs`)

## Code Style

**Formatting:**
- Default Rust formatting (no custom rustfmt.toml detected)
- Indentation: 4 spaces
- Max line length: Default (100 chars)

**Linting:**
- Standard Clippy rules (no custom clippy.toml detected)
- Warnings treated as errors in CI recommended

## Import Organization

**Order:**
1. External crates (e.g., `use dioxus::prelude::*`)
2. Standard library (e.g., `use std::path::PathBuf`)
3. Local modules (e.g., `use crate::core::error::Result`)

**Pattern observed in `src/core/ocr/engine.rs`:**
```rust
use async_trait::async_trait;
use ort::session::Session;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::path::PathBuf;
use ndarray::Array4;
use image::DynamicImage;
// ... more external imports

use crate::core::error::{OcrError, Result};
use crate::core::db::{Database, NewBookPage};
```

**Path Aliases:**
- `crate::` prefix for internal module access
- Re-exports in `mod.rs` files for clean public API

## Error Handling

**Patterns:**
- Use `thiserror` for library error types (`src/core/error.rs`)
- Use `anyhow` for application-level error handling (`src/core/storage.rs`)
- Result type alias: `pub type Result<T> = std::result::Result<T, ShuseiError>`

**Error Type Structure (`src/core/error.rs`):**
```rust
#[derive(Error, Debug)]
pub enum ShuseiError {
    #[error("OCR error: {0}")]
    Ocr(#[from] OcrError),

    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),
    // ... with #[from] for automatic conversion
}

#[derive(Error, Debug)]
pub enum OcrError {
    #[error("Image preprocessing failed: {0}")]
    Preprocessing(String),

    #[error("Model loading failed: {0}")]
    ModelLoading(String),
    // ... descriptive error messages with context
}
```

**Error Context with anyhow (`src/core/storage.rs`):**
```rust
fs::create_dir_all(&images_dir)
    .with_context(|| format!("Failed to create images directory: {:?}", images_dir))?;
```

**Error Propagation:**
- Use `?` operator for automatic conversion
- Use `.map_err(|e| ...)` for custom error wrapping

## Logging

**Framework:** `log` crate with `env_logger`

**Initialization (`src/main.rs`):**
```rust
env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
    .init();
```

**Patterns:**
```rust
log::info!("Starting Shusei...");
log::info!("Initializing NDLOCR engine from {:?}", self.model_dir);
log::debug!("App initialized with default state (no saved state found)");
log::warn!("Detection model not found: {:?}", detection_model);
log::warn!("Direction classifier model not found, direction classification will be disabled");
log::error!("Failed to save page {}: {}", page_num, e);
```

**Log Level Usage:**
- `info!`: Application lifecycle events, major operations, initialization
- `debug!`: Detailed execution flow, state changes
- `warn!`: Non-critical issues, missing optional resources
- `error!`: Failures that affect functionality

## Comments

**When to Comment:**
- Module-level doc comments: `//!` for crate and module overview
- Public items: `///` for documentation with examples
- Complex algorithms: Inline comments explaining why

**Module Documentation Pattern (`src/core/error.rs`):**
```rust
//! Error types for the Shusei application
//!
//! This module defines all error types used throughout the application.
```

**Function Documentation (`src/core/ocr/engine.rs`):**
```rust
/// Process multiple pages in parallel with concurrency control
///
/// # Arguments
/// * `pages` - Vec of (page_number, image_bytes) to process
/// * `book_id` - Book identifier for database storage
/// * `db` - Database connection for saving results
/// * `progress_cb` - Callback called after each page completes: (page_num, total)
///
/// # Returns
/// Ok(()) on success, or error if critical failure occurs
pub async fn process_pages_parallel(
    &self,
    pages: Vec<(u32, Vec<u8>)>,
    book_id: &str,
    db: &Database,
    progress_cb: impl Fn(u32, u32),
) -> Result<()>
```

## Function Design

**Size:** Functions typically 10-50 lines; larger functions split into helpers

**Parameters:**
- Use `impl Into<PathBuf>` for flexible path arguments
- Use `&str` for string references, `String` for ownership transfer
- Group related parameters into config structs (e.g., `OcrConfig`, `SttConfig`)

**Return Values:**
- Return `Result<T>` for fallible operations
- Return `Option<T>` for optional results
- Use `impl Trait` for complex return types

**Async Functions:**
- Mark with `async` keyword
- Use `#[async_trait]` for trait definitions
- Spawn blocking operations with `tokio::task::spawn_blocking`

**Example from `src/ui/reader.rs`:**
```rust
use_effect(move || {
    spawn(async move {
        let result = tokio::task::spawn_blocking(move || {
            match Database::open("shusei.db") {
                Ok(db) => {
                    let book_result = db.get_book(&book_id.to_string());
                    let pages_result = db.get_pages_by_book(&book_id.to_string());
                    match (book_result, pages_result) {
                        (Ok(Some(b)), Ok(p)) => Some((b, p)),
                        _ => None,
                    }
                }
                Err(_) => None,
            }
        }).await;
        // ...
    });
});
```

## Module Design

**Exports:**
- Public API defined in `mod.rs` with `pub use`
- Private implementation details in submodules

**Barrel Files (`src/core/mod.rs`):**
```rust
pub mod error;
pub mod ocr;
pub mod stt;
pub mod db;
pub mod vocab;
pub mod storage;
pub mod models;
pub mod state;
pub mod pdf;

pub use error::{ShuseiError, OcrError, SttError};
pub use ocr::OcrEngine;
pub use stt::SttEngine;
pub use db::Database;
pub use storage::StorageService;
pub use state::AppState;
```

**Trait-Based Abstraction:**
- Define traits for major components (e.g., `OcrEngine`, `SttEngine`)
- Implement for concrete types (e.g., `NdlocrEngine`, `MoonshineEngine`)
- Use `Arc<Mutex<>>` for shared mutable state in async contexts

**Trait Pattern (`src/core/ocr/engine.rs`):**
```rust
#[async_trait]
pub trait OcrEngine: Send + Sync {
    async fn process_image(&self, image_data: &[u8]) -> Result<OcrResult>;
    fn is_ready(&self) -> bool;
    fn name(&self) -> &'static str;
}
```

## Dioxus UI Conventions

**Component Structure (`src/ui/components.rs`):**
```rust
#[component]
pub fn Button(
    text: String,
    onclick: EventHandler<MouseEvent>,
    variant: Option<String>,
    disabled: Option<bool>,
) -> Element {
    let variant_class = match variant.as_deref() {
        Some("primary") => "bg-blue-600 text-white",
        Some("secondary") => "bg-gray-200 text-gray-800",
        Some("danger") => "bg-red-600 text-white",
        _ => "bg-blue-600 text-white",
    };

    rsx! {
        button {
            class: "px-4 py-2 rounded-lg {variant_class}",
            onclick: move |e| onclick.call(e),
            disabled: disabled.unwrap_or(false),
            "{text}"
        }
    }
}
```

**State Management:**
- Use `use_signal` for reactive state: `let mut font_size = use_signal(|| 18);`
- Use `use_effect` for side effects
- Use `spawn` for async operations in event handlers

**Routing (`src/app.rs`):**
```rust
#[derive(Routable, Clone, PartialEq, Debug)]
pub enum Route {
    #[route("/")]
    Home,

    #[route("/camera")]
    Camera,

    #[route("/notes")]
    Notes,

    #[route("/reader/:book_id")]
    ReaderBook { book_id: i64 },
    // ...
}
```

## Serde Patterns

**Serialization:**
- Derive `Serialize` and `Deserialize` for data models
- Use `#[serde(default)]` for optional fields

**Example (`src/core/state.rs`):**
```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AppState {
    pub current_route: String,
    pub scroll_position: f32,
    pub timestamp: i64,
}
```

## Platform Conditional Compilation

**Pattern (`src/core/state.rs`):**
```rust
#[cfg(target_os = "android")]
let base_dir = match crate::platform::android::get_assets_directory() {
    Ok(dir) => dir,
    Err(_) => std::env::current_dir()?,
};

#[cfg(not(target_os = "android"))]
let base_dir = std::env::current_dir()?;
```

**Feature Flags (`Cargo.toml`):**
```toml
[features]
default = []
android = ["jni"]
ios = []
desktop = []
web = []
lindera = ["dep:lindera"]
ndlocr-test = []
moonshine-test = []
```

## Default Trait Pattern

**Use Default for Configuration (`src/core/ocr/mod.rs`):**
```rust
#[derive(Debug, Clone)]
pub struct OcrConfig {
    pub max_image_size: u32,
    pub detection_threshold: f32,
    pub recognition_threshold: f32,
    pub enable_direction_classification: bool,
}

impl Default for OcrConfig {
    fn default() -> Self {
        Self {
            max_image_size: 1024,
            detection_threshold: 0.5,
            recognition_threshold: 0.5,
            enable_direction_classification: true,
        }
    }
}
```

---

*Convention analysis: 2026-03-13*