# S04: Word Collection — Research

**Date:** 2026-03-16

## Summary

S04 builds the vocabulary list UI that displays words saved via the word tap feature implemented in S03. The core infrastructure is already in place: S03's reader implements word tap detection that saves words to the `words` table with sentence context, and the database schema supports all required fields (word, definition, context_text, source_book_id, source_page). The existing `src/ui/vocab.rs` provides a scaffold that needs to be connected to the actual database and enhanced with word detail views showing the "definition coming soon" placeholder per D007.

Key findings:
- **Word tap already works**: S03 implemented `handle_word_save()` in `ReaderBookView` that saves words with sentence extraction via `WordExtractor::extract_sentence()`
- **Database ready**: `words` table schema matches requirements with `definition TEXT NULL` and `ai_generated BOOLEAN DEFAULT FALSE`
- **Vocab scaffold exists**: `src/ui/vocab.rs` has UI structure but loads empty data (TODO comment: "Load from actual database")
- **Definition deferred**: Per D007, M002 shows placeholder - no dictionary or AI definitions needed yet
- **Toast component reusable**: S03's `ToastNotification` component can be reused for feedback in vocab UI

The main work is connecting the vocabulary page to the database, implementing word list rendering with proper filtering/search, adding word detail view with delete functionality, and ensuring the UI works on Android (localStorage may need native alternative for preferences).

## Recommendation

**Build on existing S03 foundation** by:
1. **Connect vocab page to database** - Replace TODO placeholder with actual `db.get_ai_generated_words()` or new `db.get_all_words()` query
2. **Enhance WordCard component** - Display saved words with sentence context, source book/page reference, and "definition coming soon" placeholder
3. **Add word detail view** - Create route `/vocab/:word_id` showing full word details with delete confirmation
4. **Implement search/filter** - Client-side filtering by word text (already scaffolded in vocab.rs)
5. **Add export functionality** - Use existing `vocab::export_vocabulary()` functions (Markdown, CSV, JSON already implemented in `src/core/vocab.rs`)
6. **Write integration tests** - Verify word list loading, search, delete operations with in-memory SQLite

**Why this approach:**
- Leverages S03's word tap implementation - no changes needed to reader
- Uses existing database queries (`get_word_by_text`, `create_word`) - just need `get_all_words` and `delete_word` (already exists)
- Reuses ToastNotification component for consistent UX
- Minimal new code - mostly wiring existing pieces together
- Matches S03 patterns (localStorage for preferences, debounced operations, toast feedback)

## Don't Hand-Roll

| Problem | Existing Solution | Why Use It |
|---------|------------------|------------|
| Word list display | `src/ui/vocab.rs` WordCard component | Already styled, just needs database connection |
| Word search | `src/ui/vocab.rs` search bar with `search_query` signal | Client-side filtering is sufficient for prototype (<1000 words expected) |
| Toast notifications | `src/ui/reader.rs` ToastNotification component | Reusable, already tested in S03 |
| Export functionality | `src/core/vocab.rs` export_vocabulary(), export_markdown(), export_csv(), export_json() | Already implemented with proper formatting |
| Database operations | `src/core/db.rs` create_word(), get_word(), get_word_by_text(), delete_word() | All CRUD operations exist |
| Sentence extraction | `src/core/vocab.rs` WordExtractor::extract_sentence() | Already integrated in S03 word tap |

## Existing Code and Patterns

- `src/ui/reader.rs` — **WordSpan** and **TapParagraph** components with `cursor-pointer hover:bg-yellow-200` for word tap feedback; **ToastNotification** component for success/info/error messages (3-second auto-dismiss); `handle_word_save()` callback pattern with duplicate detection
- `src/ui/vocab.rs` — **VocabPage** scaffold with search bar, filter state, empty state handling, **WordCard** component with delete button placeholder; export buttons (Markdown/CSV) with TODO handlers
- `src/core/vocab.rs` — **WordExtractor** with `extract_sentence()` method; **export_vocabulary()** functions for Markdown/CSV/JSON export; **VocabularyEntry** struct (note: different schema than `words` table - may need migration or adapter)
- `src/core/db.rs` — **words** table schema with `id, word, definition, ai_generated, source_book_id, source_page, context_text, created_at, updated_at`; **Word** and **NewWord** structs; CRUD methods: `create_word()`, `get_word()`, `get_word_by_text()`, `get_words_by_book()`, `update_word_definition()`, `delete_word()`, `get_ai_generated_words()`
- `src/app.rs` — Route enum with `/vocab` route; **Vocab()** wrapper component

**Important schema mismatch noted:**
- `src/core/vocab.rs` defines `VocabularyEntry` with fields: `id, word, meaning, example_sentence, source_book, source_page, tags, created_at, review_count, last_reviewed_at`
- `src/core/db.rs` defines `Word` with fields: `id, word, definition, ai_generated, source_book_id, source_page, context_text, created_at, updated_at`
- **Action needed**: Either update `VocabularyEntry` to match `Word` schema, or create adapter function for S04

## Constraints

- **Definition deferred to M003** (D007) — Must show "coming soon" placeholder, not attempt to fetch definitions from dictionary or AI
- **localStorage for preferences** — S03 used localStorage for font size; Android native may need alternative (SharedPreferences or database)
- **Dioxus rsx! macro limitations** — S03 discovered dynamic element generation issues; keep component structure simple
- **Optional lindera dependency** — Japanese word extraction requires `--features lindera` and dictionary download; English word splitting works without it
- **Android JNI stability** — Mid-range device (Moto G66j 5G) has moderate RAM; avoid loading entire vocabulary into memory if >1000 words

## Common Pitfalls

- **Schema mismatch between VocabularyEntry and Word** — `VocabularyEntry` uses `meaning` and `example_sentence` while `Word` uses `definition` and `context_text`; must align these before S04 implementation
- **localStorage not available on native Android** — S03 used localStorage for font size preference; vocab page should use database or SharedPreferences for native compatibility
- **Duplicate word detection** — S03 checks `get_word_by_text()` before save; vocab page should handle display of duplicates gracefully
- **Search performance** — Client-side filtering is fine for <1000 words, but database FTS query needed for larger vocabularies (defer to M003)
- **Delete without confirmation** — WordCard has delete button; must add confirmation dialog to prevent accidental deletion

## Open Risks

- **VocabularyEntry vs Word schema mismatch** — May require database migration or adapter layer; could block S04 if not resolved early
- **Android localStorage compatibility** — web-sys localStorage API may not work in Dioxus Android webview; need to verify or implement native alternative
- **Word count at scale** — Unknown how many words users will save; if >1000, client-side search may cause performance issues on mid-range device
- **Sentence extraction quality** — `WordExtractor::extract_sentence()` uses simple `.`/`!`/`。` splitting; may produce incomplete sentences for complex text

## Skills Discovered

| Technology | Skill | Status |
|------------|-------|--------|
| Dioxus | dioxuslabs/dioxus | Installed (v0.7) |
| Lindera | lindera-morphology/lindera | Optional feature (v0.34, requires dictionary download) |
| rusqlite | rusqlite/rusqlite | Installed (v0.32 with bundled feature) |
| web-sys | rustwasm/wasm-bindgen | Installed (v0.3 for localStorage/DOM APIs) |

## Sources

- **S03 Summary** (`.gsd/milestones/M002-dbrk2n/slices/S03/S03-SUMMARY.md`) — Word tap implementation details, ToastNotification component, localStorage usage patterns, database schema for `words` table
- **S03 Reader Code** (`src/ui/reader.rs`) — Working word tap handler with duplicate detection, sentence extraction, toast feedback
- **Vocab Scaffold** (`src/ui/vocab.rs`) — Existing vocabulary list UI structure with search and export buttons
- **Vocab Core** (`src/core/vocab.rs`) — WordExtractor, export functions, VocabularyEntry struct (schema mismatch noted)
- **Database Layer** (`src/core/db.rs`) — Word CRUD operations, words table schema
- **Decision D007** (`.gsd/DECISIONS.md`) — Word definition deferred to M003, placeholder in M002
