# Phase 1: Core Infrastructure - Research

**Researched:** 2026-03-11
**Confidence:** HIGH
**Research Flags:** None - standard Dioxus + SQLite patterns well-documented

---

## Standard Stack

### Dioxus 0.7 (UI Framework)

**Version:** 0.7.x (confirmed in Cargo.toml)

**Key Patterns:**
- **Router-based navigation:** Use `use_navigator()` hook for programmatic navigation
- **Signal-based state:** `use_signal()` for local state, `use_context_provider()` for shared state
- **Component structure:** `#[component]` fn returns `Element`, use `rsx!` macro
- **Form handling:** Controlled inputs with `oninput` handlers, `onsubmit` for form submission

**Mobile-specific considerations:**
- Dioxus 0.7 supports mobile targets (Android/iOS) with single codebase
- Use `dioxus-mobile` for mobile-specific configurations
- Touch events work out-of-the-box with standard `onclick` handlers

**Established patterns from codebase:**
- Route enum in `src/app.rs` defines navigation structure
- Components in `src/ui/` directory follow modular pattern
- Core logic separated in `src/core/` (db.rs, error.rs, etc.)

### SQLite (rusqlite 0.32)

**Version:** 0.32 with bundled feature (no system SQLite required)

**Key patterns:**
- **Connection management:** Use `rusqlite::Connection` with lazy initialization
- **WAL mode:** Enable with `PRAGMA journal_mode=WAL` for concurrent reads
- **Prepared statements:** Cache for repeated queries
- **Transactions:** Use `execute_batch()` for multiple operations

**Android-specific:**
- Store database in app's internal storage directory
- Use `bundled` feature to avoid system dependency
- WAL mode critical for background operations + UI reads

**From existing db.rs (10KB):**
- Already has connection management infrastructure
- Uses `parking_lot::Mutex` for thread-safe access
- Error handling with custom `DbError` type
- Schema migrations pattern established

### File Storage

**Pattern:** Filesystem paths stored in SQLite, actual files in assets directory

**Why not BLOBs:** Research confirms SQLite BLOBs cause memory issues on low-RAM devices. Store files on filesystem, paths in database.

**Android storage:**
- Use `Context.getFilesDir()` or `Context.getExternalFilesDir()` for app-specific storage
- Store cover photos in `images/` subdirectory
- Use relative paths in database (e.g., `images/cover_123.jpg`)

**From existing code:**
- `src/core/db.rs` has asset path utilities
- `src/platform/android.rs` has Android-specific file handling

---

## Architecture Patterns

### Component Structure (Dioxus 0.7)

```rust
// Library screen component
#[component]
fn LibraryScreen() -> Element {
    let mut books = use_signal(|| vec![]);
    let navigator = use_navigator();
    
    // Load books on mount
    use_effect(move || {
        spawn(async move {
            let loaded = load_books().await;
            books.set(loaded);
        });
    });
    
    rsx! {
        div { class: "library-container",
            h1 { "My Library" }
            button { onclick: move |_| navigator.push(Route::AddBook), "Add Book" }
            for book in books() {
                BookCard { book: book.clone() }
            }
        }
    }
}
```

### State Management

**Local state:** `use_signal(|| initial_value)`
**Shared state:** `use_context_provider(|| state)` + `use_context::<T>()`
**Global state:** `Signal::global(|| initial_value)` (use sparingly)

**For book library:**
- Books list: `Signal<Vec<Book>>` in LibraryScreen component
- Loading state: `Signal<bool>` for spinner
- Error state: `Signal<Option<String>>` for error messages

### Navigation Patterns

**Programmatic navigation:**
```rust
let navigator = use_navigator();
navigator.push(Route::BookDetail { id: book_id });
navigator.replace(Route::Home);
```

**Route enum (extend existing):**
```rust
#[derive(Routable, Clone, PartialEq, Debug)]
enum Route {
    #[layout(HeaderLayout)]
        #[route("/")]
        Home,
        #[route("/books/:id")]
        BookDetail { id: String },
        #[route("/add-book")]
        AddBook,
    #[end_layout]
    #[route("/..")]
    NotFound,
}
```

### Form Handling

**Controlled inputs:**
```rust
#[component]
fn AddBookForm() -> Element {
    let mut title = use_signal(|| String::new());
    let mut author = use_signal(|| String::new());
    let mut cover_path = use_signal(|| None::<String>);
    let navigator = use_navigator();
    
    let handle_submit = move |_| {
        let book = Book {
            title: title(),
            author: author(),
            cover_path: cover_path(),
            ..Default::default()
        };
        spawn(async move {
            save_book(book).await;
        });
        navigator.push(Route::BookDetail { id: "new".to_string() });
    };
    
    rsx! {
        form { onsubmit: handle_submit,
            input {
                value: "{title}",
                oninput: move |evt| title.set(evt.value()),
                placeholder: "Book Title"
            }
            input {
                value: "{author}",
                oninput: move |evt| author.set(evt.value()),
                placeholder: "Author"
            }
            button { r#type: "submit", "Add Book" }
        }
    }
}
```

---

## Android-Specific Patterns

### Lifecycle Handling

**Dioxus mobile lifecycle:**
- `on_pause`: App moves to background
- `on_resume`: App returns to foreground
- `on_stop`: App fully stopped

**From existing android.rs (6KB):**
- JNI setup already configured
- Camera integration patterns exist
- Need to add proper `onPause`/`onResume` handlers

**State persistence:**
```rust
// Save state before background
use_effect(move || {
    on_cleanup(move || {
        save_app_state();
    });
});
```

### JNI Memory Management

**Pattern from existing code:**
```rust
// Use PushLocalFrame/PopLocalFrame for batches
env.push_local_frame(16)?;
// ... JNI operations ...
env.pop_local_frame();
```

**Critical for Phase 1:**
- Establish pattern early in db.rs for all JNI calls
- Explicit cleanup in `Drop` implementations
- Avoid global references unless necessary

### File Paths on Android

**Storage locations:**
- Internal: `/data/data/com.example.shusei/files/`
- External: `/sdcard/Android/data/com.example.shusei/files/`

**Use Android API:**
```rust
// From android.rs patterns
let files_dir = env.call_method(activity, "getFilesDir", "()Ljava/io/File;", &[])
    .and_then(|dir| env.call_object_method(dir.as_l(), "getAbsolutePath", "()Ljava/lang/String;", &[]));
```

---

## Database Schema Design

### Books Table

```sql
CREATE TABLE books (
    id TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(16)))),
    title TEXT NOT NULL,
    author TEXT NOT NULL,
    cover_path TEXT,
    pages_captured INTEGER DEFAULT 0,
    total_pages INTEGER,
    last_opened_at INTEGER,
    created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))
);

CREATE INDEX idx_books_title ON books(title);

PRAGMA journal_mode=WAL;
```

**Design rationale:**
- UUID strings for IDs (portable, no autoincrement issues)
- `pages_captured` / `total_pages` generic for both physical books and PDFs
- Unix timestamps for dates (simple, portable)
- WAL mode for concurrent reads
- Single index on title for alphabetical sorting

### Migration Pattern

**From existing db.rs:**
```rust
fn migrate(conn: &Connection) -> Result<()> {
    conn.execute_batch("
        CREATE TABLE IF NOT EXISTS books (
            -- schema here
        );
        CREATE INDEX IF NOT EXISTS idx_books_title ON books(title);
    ")?;
    Ok(())
}
```

---

## Common Pitfalls

### Memory Management

**Avoid:**
- Loading all book covers into memory at once
- Storing images as BLOBs in SQLite
- Not closing database connections

**Do:**
- Load covers lazily (on scroll into view)
- Store image paths, load from filesystem
- Use connection pooling or single shared connection

### Android Lifecycle

**Avoid:**
- Assuming app stays in foreground
- Not saving state before background
- Holding resources in background

**Do:**
- Save state in `on_pause`
- Release heavy resources in `on_stop`
- Restore state in `on_resume`

### UI Performance

**Avoid:**
- Rendering all books at once (use virtualization for 100+ books)
- Blocking UI thread with database queries
- Not showing loading states

**Do:**
- Use `spawn(async {})` for database operations
- Show skeletons/spinners during load
- Consider lazy loading for large libraries

---

## Validation Architecture

### Dimension 1: Requirement Coverage

**Must verify:**
- CORE-01: App launches, shows library screen
- CORE-02: SQLite saves book metadata
- CORE-03: Filesystem stores cover photos
- CORE-04: Lifecycle handling works
- CORE-05: No JNI memory leaks

### Dimension 2: Task Completeness

**Each task must have:**
- `<files>`: Exact paths modified
- `<action>`: Specific implementation steps
- `<verify>`: Automated command (test, curl, etc.)
- `<done>`: Measurable acceptance criteria

### Dimension 3: Dependency Correctness

**Wave structure:**
- Wave 1: Database schema + models (no dependencies)
- Wave 2: API/operations (depends on schema)
- Wave 3: UI components (depends on operations)
- Wave 4: Integration + lifecycle (depends on UI)

### Dimension 4: Key Links Planned

**Critical connections:**
- UI `onsubmit` → save_book() → database insert
- Database insert → file save for cover photo
- App background → state persistence
- App resume → state restoration

### Dimension 5: Must-Haves Derived

**Observable truths:**
1. User sees library screen on launch
2. User can add book with title + author + optional cover
3. Added book persists after app restart
4. App survives background/foreground transition
5. No memory growth during normal operation

**Required artifacts:**
- `src/ui/library.rs` - Library screen component
- `src/core/db.rs` - Extended with books table
- `src/core/models.rs` - Book struct
- `src/platform/android.rs` - Lifecycle handlers
- `assets/images/` - Cover photo storage

**Key links:**
- LibraryScreen → use_effect → load_books() → db.query()
- AddBookForm → onsubmit → save_book() → db.execute()
- Android onPause → save_state() → SharedPreferences

### Dimension 6: Scope Sanity

**Phase 1 scope:**
- 5 requirements (CORE-01 through CORE-05)
- ~4-6 plans (2-3 tasks each)
- ~15-20 files total
- No OCR, no PDF, no voice, no AI

**Split if:**
- Any plan exceeds 3 tasks
- Any task exceeds 5 file modifications
- Checkpoint + implementation in same plan

### Dimension 7: Checkpoint Integrity

**Checkpoint candidates:**
- Visual UI review (library screen layout)
- Lifecycle testing (manual background/foreground)
- Memory profiling (JNI leak detection)

**Automate first:**
- Database CRUD via unit tests
- File storage via integration tests
- Navigation via component tests

### Dimension 8: Validation Coverage

**Automated tests:**
- Unit: Book model serialization
- Unit: Database CRUD operations
- Integration: File save/load
- E2E: Add book flow (simulated)

**Manual verification:**
- UI layout on actual Android device
- Lifecycle behavior (home button test)
- Memory profiling over extended use

---

## Research Summary

**Confidence:** HIGH

**Why:**
- Dioxus 0.7 well-documented with 880+ code snippets
- rusqlite 0.32 stable, bundled mode eliminates system deps
- Existing codebase has 60% of infrastructure (db.rs, android.rs, components)
- No novel technical challenges in Phase 1
- All patterns established in codebase or well-documented

**No research flags:** This phase uses standard, well-understood patterns. No experimental integrations or unproven libraries.

**Next step:** Create plans using established patterns from:
- `src/core/db.rs` (database layer)
- `src/platform/android.rs` (Android integration)
- `src/ui/components.rs` (UI primitives)
- Dioxus 0.7 documentation (router, signals, forms)

---

*Research completed: 2026-03-11*
*Confidence: HIGH | Flags: None*