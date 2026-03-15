# Architectural and Pattern Decisions

## S01: Core Infrastructure (2026-03-11)

### Storage Architecture
- **Filesystem storage over SQLite BLOBs** - Avoids memory issues on low-RAM Android devices. Store relative file paths in database (e.g., `images/cover_abc123.bin`), not absolute paths.
- **`.bin` extension for all images** - Format-agnostic approach; storage doesn't need to know image format.
- **`{assets_dir}/images/` subdirectory** - Organized storage structure, auto-created on first save.

### Database Design
- **WAL mode enabled** - `PRAGMA journal_mode=WAL` for concurrent read support.
- **`updated_at` column in books table** - Required for tracking modifications, NOT NULL constraint.
- **Parameterized queries only** - No SQL injection risk, all CRUD operations use `params![]` macro.

### State Persistence
- **JSON file over SharedPreferences** - Cross-platform compatibility, easier debugging, human-readable state.
- **`.shusei` subdirectory** - Dedicated directory for app state files, organized and discoverable.
- **AppState fields: route, scroll_position, timestamp** - Minimal viable state for lifecycle restoration.

### Android JNI Patterns
- **PushLocalFrame/PopLocalFrame** - Prevents native memory leaks during lifecycle transitions. Capacity of 16 local references sufficient for state operations.
- **Graceful JavaVM fallback** - When JavaVM not initialized, fallback to `std::env::current_dir()` for desktop development.

### UI Architecture
- **Modal overlay for AddBookForm** - Keeps users in library context, better UX than separate page.
- **Validation from signal state** - Computed `is_valid` from title/author signals, not on submit.
- **Explicit `()` in onclick handlers** - Required by Dioxus 0.7 event handler type system.
- **Placeholder components for router** - Satisfy Dioxus router compilation before full UI implementation.

---

- "Code was pre-existing - verified and fixed test bugs instead of implementing from scratch"
- "Used placeholder components for routes to enable compilation before UI implementation"
- "Implemented validation logic in AddBookForm with is_valid signal"
- "Modal overlay pattern for AddBookForm to maintain context"
