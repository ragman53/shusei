# Phase 1: Core Infrastructure - Context

**Gathered:** 2026-03-11
**Status:** Ready for planning

<domain>
## Phase Boundary

Android app shell with book library screen, SQLite database for book metadata, and filesystem storage for cover photos. This phase delivers the foundation that all subsequent phases build upon.

**In scope:**
- Book library list view (title + author + progress)
- Add book modal form (title, author, optional cover photo)
- SQLite database with books table
- File storage for cover images
- Android lifecycle handling (background/foreground transitions)
- State persistence and recovery

**Out of scope:**
- OCR processing (Phase 2)
- PDF import (Phase 3)
- Voice memos (Phase 5)
- AI dictionary (Phase 6)

</domain>

<decisions>
## Implementation Decisions

### Library UI Layout
- **List view (1 column)** — More books visible, cleaner display
- **Metadata per book:** Title + author + progress (% or pages captured)
- **Empty state:** Simple message ("No books yet") + prominent "Add Book" button
- **Default sorting:** Alphabetical by title (A-Z)
- **Progress field:** Generic approach (`pages_captured`, `total_pages`) works for both physical books and PDFs

### Book Add Flow
- **Modal form** — Inline modal overlay on library screen
- **Cover photo:** Optional field with "Add cover photo" button that opens camera, shows preview
- **Validation:** Title + author both required
- **After add:** Navigate to newly added book's detail page (not back to library)

### Database Schema
- **Books table fields:** `id`, `title`, `author`, `cover_path`, `pages_captured`, `total_pages`, `last_opened_at`, `created_at`
- **Indexes:** Title only (for alphabetical sorting)
- **WAL mode:** Enabled for concurrent reads (background operations + UI)
- **File storage:** Filesystem for images (paths stored in SQLite) — critical for memory optimization on low-RAM devices

### Android Lifecycle
- **Background behavior:** Save state, pause UI — database operations complete, UI freezes, resume same screen on return
- **JNI cleanup:** PushLocalFrame/PopLocalFrame pattern for batches (existing android.rs uses this)
- **State persistence:** Current route + scroll position saved
- **Recovery from kill:** Restore last route from saved state (standard Android behavior)

### OpenCode's Discretion
- Exact color scheme and typography (use existing Dioxus/Tailwind patterns from existing code)
- Specific animation durations for modal transitions
- Database connection pooling strategy (existing db.rs patterns)

</decisions>

<code_context>
## Existing Code Insights

### Reusable Assets
- **Dioxus routing** (`src/app.rs`) — Route enum already defines Home, Reader, Vocab, Camera, Notes, Settings
- **SQLite database layer** (`src/core/db.rs` - 10KB) — Existing schema and connection management
- **UI components** (`src/ui/components.rs`) — Reusable UI primitives
- **Android platform layer** (`src/platform/android.rs` - 6KB) — JNI setup, camera integration patterns
- **Cargo.toml dependencies:** All Phase 1 deps present (dioxus 0.7, rusqlite 0.32 bundled, tokio, serde, etc.)

### Established Patterns
- **Dioxus 0.7** with router — Component-based UI, signal-based state management
- **Rust error handling** — anyhow/thiserror throughout codebase
- **Async runtime** — tokio with rt-multi-thread, sync, time, fs features
- **Logging** — log + env_logger pattern established

### Integration Points
- **Navigation:** Extend existing Route enum with new book library routes
- **Database:** Extend existing db.rs schema (books table doesn't exist yet)
- **File storage:** Use existing assets directory pattern for cover photo storage
- **Android lifecycle:** Extend existing android.rs with proper onPause/onResume handlers

</code_context>

<specifics>
## Specific Ideas

- Progress tracking uses generic `pages_captured` / `total_pages` fields to support both physical books (pages photographed/OCR'd) and PDFs (actual page numbers)
- Cover photos stored as files with paths in database — follows research recommendation to avoid SQLite BLOB memory issues
- WAL mode explicitly requested for concurrent read scenarios (background processing + UI reads)
- Modal form for adding books keeps users in context (don't navigate away from library)
- Navigate to book detail after add — lets users immediately start using the book they just added

</specifics>

<deferred>
## Deferred Ideas

- **Search/filter functionality** — Mentioned as potential future enhancement, belongs in separate phase after library is stable
- **Book collections/folders** — Organizational feature for later
- **Reading statistics** — Analytics about reading habits (v2 requirement)
- **ISBN barcode scanning** — Automatic metadata lookup (requires external API or offline database — out of scope for offline-first design)
- **Cloud backup/sync** — Explicitly out of scope (100% offline requirement)

</deferred>

---

*Phase: 01-core-infrastructure*
*Context gathered: 2026-03-11*
