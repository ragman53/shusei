---
id: S04
parent: M002-dbrk2n
milestone: M002-dbrk2n
title: Word Collection
goal: Build vocabulary list UI that displays words saved via word tap, with search, delete, and export functionality
demo: User can view saved words with example sentences, search by word text, delete words with confirmation, and export vocabulary as Markdown/CSV
provides:
  - Vocabulary list UI connected to SQLite words table
  - Word detail view with delete confirmation
  - Search/filter by word text (client-side)
  - Export to Markdown/CSV/JSON using existing export functions
requires:
  - S03: PDF Reflow Reader (word tap saves words to database)
  - S01: Android Build + Deploy (SQLite persistence)
affects:
  - S05: Model Bundling + Integration (consumes vocabulary list for end-to-end verification)
key_files:
  - src/ui/vocab.rs
  - src/core/db.rs
  - src/core/vocab.rs
  - src/app.rs
key_decisions:
  - Use Word struct directly instead of VocabularyEntry to avoid schema mismatch
  - Client-side search sufficient for <1000 words (defer FTS to M003)
  - Reuse ToastNotification from S03 for user feedback
  - Delete requires confirmation dialog to prevent accidental loss
  - Export uses existing export_vocabulary() functions from src/core/vocab.rs
patterns_established:
  - Database-first vocabulary loading with async effect
  - Toast notification pattern for delete/export feedback
  - Confirmation dialog for destructive actions
observability_surfaces:
  - Database: words table with word, context_text, source_book_id, source_page
  - Runtime logs: adb logcat | grep -i "vocab\|word" for delete/export events
  - cargo test --lib vocab:: for automated verification
duration: 3h
verification_result: pending
completed_at: null
blocker_discovered: false
---

# S04: Word Collection — Plan

**Goal:** Build vocabulary list UI that displays words saved via word tap, with search, delete, and export functionality.

**Demo:** User can view saved words with example sentences, search by word text, delete words with confirmation, and export vocabulary as Markdown/CSV.

## Must-Haves

- Vocabulary list loads from actual database (words table)
- WordCard displays word, context sentence, source book/page reference
- "Definition coming soon" placeholder shown (per D007)
- Search filters words by text (client-side)
- Delete word with confirmation dialog
- Export buttons wire to existing export_vocabulary() functions
- Toast notifications for feedback (reuse S03 component)

## Proof Level

- This slice proves: **Contract verification** (UI components, database integration) + **Integration verification** (end-to-end word save → display → delete → export flow)
- Real runtime required: **Yes** (SQLite database, UI rendering)
- Human/UAT required: **No** (automated tests sufficient for prototype)

## Verification

- `cargo test --lib vocab::` — Integration tests for vocabulary list loading, search, delete, export
- `cargo test --lib db::tests::word_operations` — Regression check for word CRUD operations
- `cargo check` — Compiles without errors

## Observability / Diagnostics

- Runtime signals: Word load events, delete events, export events logged to console
- Inspection surfaces: SQLite words table (`SELECT * FROM words;`), runtime logs (`adb logcat | grep -i vocab`)
- Failure visibility: Empty list when database has words (load failure), delete without confirmation (UX bug)
- Redaction constraints: None (user's own vocabulary data)

## Integration Closure

- Upstream surfaces consumed: S03 word tap saves to `words` table; S01 SQLite persistence
- New wiring introduced: VocabPage connects to Database via `get_all_words()` query (new method needed)
- What remains before milestone is truly usable end-to-end: S05 device testing on Moto G66j 5G

## Tasks

- [x] **T01: Connect Vocabulary Page to Database** `est:1h`
  - Why: VocabPage currently loads empty data; must connect to actual words table
  - Files: `src/ui/vocab.rs`, `src/core/db.rs`
  - Do: Add `get_all_words()` method to Database; update VocabPage to load words on mount; replace VocabularyEntry with Word struct or create adapter
  - Verify: `cargo test --lib vocab::test_vocab_loads_from_database`
  - Done when: VocabPage displays words from database with word text, context sentence, source reference

- [x] **T02: Implement Word Delete with Confirmation** `est:45m`
  - Why: WordCard has delete button placeholder; must add confirmation to prevent accidental deletion
  - Files: `src/ui/vocab.rs`
  - Do: Add confirmation dialog component; wire delete button to show dialog; call `db.delete_word()` on confirm; show toast on success
  - Verify: `cargo test --lib vocab::test_word_delete_with_confirmation`
  - Done when: Delete shows confirmation dialog, removes word from database, shows success toast

- [x] **T03: Wire Export Functionality** `est:30m`
  - Why: Export buttons have TODO handlers; must connect to existing export_vocabulary() functions
  - Files: `src/ui/vocab.rs`, `src/core/vocab.rs`
  - Do: Import export_vocabulary(), ExportFormat; wire Markdown/CSV buttons to export functions; show toast with export confirmation
  - Verify: `cargo test --lib vocab::test_export_functions`
  - Done when: Export buttons generate Markdown/CSV output and show success toast

- [ ] **T04: Write Integration Tests** `est:45m`
  - Why: Verify vocabulary list loading, search, delete, export work correctly with in-memory SQLite
  - Files: `src/ui/vocab.rs` (test module)
  - Do: Add tests for vocab loading, search filtering, delete operation, export generation; use in-memory database
  - Verify: `cargo test --lib vocab::` — all tests pass
  - Done when: 4+ integration tests pass covering load, search, delete, export

## Files Likely Touched

- `src/ui/vocab.rs`
- `src/core/db.rs`
- `src/core/vocab.rs`

---
estimated_steps: 12
estimated_files: 3
---
