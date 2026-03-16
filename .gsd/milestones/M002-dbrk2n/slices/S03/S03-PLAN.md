---
id: S03
parent: M002-dbrk2n
milestone: M002-dbrk2n
provides:
  - PDF reflow reader with proper markdown rendering (pulldown-cmark)
  - Word tap detection with sentence extraction and database persistence
  - Last-read position auto-save and restore
  - Font size preference persistence
requires:
  - S01: Android Build + Deploy (APK, SQLite, file picker)
affects:
  - S04: Word Collection (consumes word tap interaction)
  - S05: Model Bundling + Integration (consumes PDF conversion pipeline)
key_files:
  - src/ui/reader.rs
  - src/core/pdf.rs
  - src/core/vocab.rs
  - src/core/db.rs
  - Cargo.toml
key_decisions:
  - Use pulldown-cmark for proper markdown parsing (replaces fragile string replacement)
  - Word tap saves to `words` table with definition: None (placeholder per D007)
  - Progress stored in processing_progress table using last_processed_page field
  - Debounced scroll save (500ms) to prevent database thrashing
patterns_established:
  - Component-based word rendering with clickable spans
  - Debounced auto-save pattern for scroll position
  - Toast notification for user feedback on word save
observability_surfaces:
  - Reader component logs: word tap events, progress save events
  - Database: words table for saved vocabulary, processing_progress for reading position
  - cargo test --lib reader:: for automated verification
drill_down_paths:
  - .gsd/milestones/M002-dbrk2n/slices/S03/tasks/T01-PLAN.md
  - .gsd/milestones/M002-dbrk2n/slices/S03/tasks/T02-PLAN.md
  - .gsd/milestones/M002-dbrk2n/slices/S03/tasks/T03-PLAN.md
  - .gsd/milestones/M002-dbrk2n/slices/S03/tasks/T04-PLAN.md
duration: 4h
verification_result: pending
completed_at: null
---

# S03: PDF Reflow Reader — Plan

**Goal:** Complete the PDF reflow reader with word tap detection, progress persistence, and proper markdown rendering.

**Demo:** User can import PDF → convert to markdown → read with continuous scroll, font control (12-32px), tap words to save with example sentences, close and reopen book → last-read position restored.

## Must-Haves

- Replace `render_markdown()` with `pulldown-cmark` parser for proper HTML generation
- Word tap handler: extract word + sentence, save to `words` table with `definition: None`
- Persist last-read page to `processing_progress` table (auto-save on scroll, debounced)
- Restore last-read position on mount using `scroll_into_view()`
- Show "Word saved!" toast notification on tap
- Persist font size preference to database or localStorage

## Proof Level

- This slice proves: **Contract verification** (component-level tests) + **Integration verification** (database persistence, scroll restore)
- Real runtime required: **Yes** (SQLite database, actual PDF conversion)
- Human/UAT required: **No** (automated tests sufficient for prototype; device testing in S05)

## Verification

- `cargo test --lib reader::` — Run reader component tests (word extraction, progress save/restore)
- `cargo test --lib db::test_word_persistence` — Verify words table operations
- `cargo check` — Verify compilation with pulldown-cmark integration
- Manual verification: Import test PDF → convert → tap 3 words → close → reopen → verify position restored + words persist
- **Failure path verification:** Inspect browser dev tools to verify `data-word` attributes on rendered spans; check HTML structure for proper tag nesting (no unclosed tags)

## Observability / Diagnostics

- **Runtime signals:** 
  - Word tap events logged with word text and book_id
  - Progress save events logged with page number and timestamp
  - Conversion errors logged with full error context
- **Inspection surfaces:**
  - `adb logcat | grep -i shusei` for runtime logs
  - `sqlite3 shusei.db "SELECT * FROM words;"` for saved vocabulary
  - `sqlite3 shusei.db "SELECT * FROM processing_progress;"` for reading positions
- **Failure visibility:**
  - Word tap failures: toast notification with error message
  - Progress save failures: logged to console, non-fatal (user won't lose place permanently)
  - Markdown parse failures: fallback to plain text rendering
- **Redaction constraints:** None (no PII or secrets in reader flow)

## Integration Closure

- **Upstream surfaces consumed:**
  - `src/core/pdf.rs` — `PdfConversionService::convert_pdf()` for PDF → markdown pipeline
  - `src/core/db.rs` — `create_word()`, `get_progress()`, `update_progress()` for persistence
  - `src/core/vocab.rs` — `WordExtractor::extract_sentence()` for sentence extraction
- **New wiring introduced in this slice:**
  - `ReaderBookView` component extended with word tap handlers and progress auto-save
  - `processing_progress` table used for last-read position (uses `last_processed_page` field)
  - `words` table populated with word + sentence context from PDF pages
- **What remains before the milestone is truly usable end-to-end:**
  - S04: Word collection UI (vocabulary list view, word detail with definition placeholder)
  - S05: Device testing on Moto G66j 5G, model bundling verification

## Tasks

- [x] **T01: Replace Markdown Renderer with pulldown-cmark** `est:1h`
  - Why: Current `render_markdown()` uses fragile string replacement; word tap requires proper HTML structure with semantic elements
  - Files: `src/ui/reader.rs`, `Cargo.toml`
  - Do: Add `pulldown-cmark` dependency; create `render_markdown_to_html()` function with proper CommonMark parsing; support headers, paragraphs, bold, italic, lists; wrap each word in `<span>` with data attributes for word tap
  - Verify: `cargo test --lib reader::test_markdown_rendering` — renders headers, paragraphs, bold text correctly
  - Done when: Markdown renders with proper HTML structure; each word wrapped in span with `data-word` attribute

- [x] **T02: Implement Word Tap Detection** `est:1.5h`
  - Why: R003 requires word + example sentence collection with placeholder definition
  - Files: `src/ui/reader.rs`, `src/core/vocab.rs`, `src/core/db.rs`
  - Do: Add onclick handler to word spans; on tap: extract word text, call `WordExtractor::extract_sentence()` to get context, save to database via `db.create_word()` with `definition: None`, show "Word saved!" toast; handle duplicate words (skip or update)
  - Verify: `cargo test --lib reader::test_word_tap` — word saved to database with sentence context; `cargo test --lib vocab::test_extract_sentence` — sentence extraction works
  - Done when: Tapping a word saves it to `words` table with full sentence; toast notification appears; duplicate words handled gracefully

- [x] **T03: Implement Progress Persistence** `est:1h`
  - Why: R002 requires last-read position sync and auto-scroll progress detection
  - Files: `src/ui/reader.rs`, `src/core/db.rs`
  - Do: Add debounced scroll handler (500ms timeout); on scroll end: calculate current page, call `db.update_progress()` with `last_processed_page`; on mount: load progress via `db.get_progress()`, scroll to page using `element.scroll_into_view()`; persist font size preference similarly
  - Verify: `cargo test --lib reader::test_progress_persistence` — progress saves and restores correctly; manual test: scroll to page 5, close, reopen, verify page 5 visible
  - Done when: Last-read position persists across app restarts; font size preference persists; progress auto-saves on scroll (debounced)

- [x] **T04: Write Integration Tests** `est:30m`
  - Why: Verify all features work together; provide regression protection
  - Files: `src/ui/reader.rs` (test module), `src/core/db.rs` (test module)
  - Do: Add test functions: `test_word_tap_saves_to_database()`, `test_progress_auto_save()`, `test_last_position_restore()`; use in-memory database for isolation; verify database state after operations
  - Verify: `cargo test --lib reader::` — all tests pass; `cargo test --lib db::` — all database tests pass
  - Done when: 3+ integration tests pass covering word tap, progress save, progress restore

## Files Likely Touched

- `src/ui/reader.rs` — Primary component (markdown rendering, word tap, progress persistence)
- `Cargo.toml` — Add `pulldown-cmark` dependency
- `src/core/db.rs` — Verify `create_word()` and `update_progress()` signatures
- `src/core/vocab.rs` — Use existing `extract_sentence()` method

---

estimated_steps: 12
estimated_files: 4
