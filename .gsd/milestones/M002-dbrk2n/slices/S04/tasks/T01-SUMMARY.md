---
id: T01
parent: S04
milestone: M002-dbrk2n
provides:
  - Database-connected vocabulary page with Word struct integration
  - get_all_words() method for fetching all vocabulary entries
  - WordCard component displaying word, context sentence, and source reference
key_files:
  - src/core/db.rs
  - src/ui/vocab.rs
key_decisions:
  - Use Word struct directly from db.rs instead of VocabularyEntry to match actual database schema
  - Open database with Database::open("shusei.db") in spawn_blocking for async UI compatibility
  - Show "Definition coming soon" placeholder per D007 design decision
patterns_established:
  - Async database loading pattern with spawn_blocking + tokio::task
  - Word struct as canonical vocabulary representation across UI and database layers
observability_surfaces:
  - Runtime logs: "Loaded N words from database" on VocabPage mount
  - Database inspection: sqlite3 shusei.db "SELECT * FROM words;"
  - Test coverage: ui::vocab::tests::test_vocab_loads_from_database
duration: 45m
verification_result: passed
completed_at: 2026-03-16
# Set blocker_discovered: true only if execution revealed the remaining slice plan
# is fundamentally invalid (wrong API, missing capability, architectural mismatch).
# Do NOT set true for ordinary bugs, minor deviations, or fixable issues.
blocker_discovered: false
---

# T01: Connect Vocabulary Page to Database

**VocabPage now loads actual words from SQLite database with proper Word struct integration.**

## What Happened

1. **Added `get_all_words()` method to Database** (`src/core/db.rs`):
   - Returns all words from `words` table ordered by `created_at DESC`
   - Follows same pattern as existing `get_ai_generated_words()` method

2. **Updated VocabPage component** (`src/ui/vocab.rs`):
   - Replaced `VocabularyEntry` import with `Word` struct from `db.rs`
   - Changed state from `Vec::<VocabularyEntry>` to `Vec::<Word>`
   - Implemented async database loading using `tokio::task::spawn_blocking`
   - Opens database with `Database::open("shusei.db")` following reader.rs pattern
   - Added logging: "Loaded N words from database" on successful fetch
   - Error handling for both database and task failures

3. **Updated WordCard component** (`src/ui/vocab.rs`):
   - Changed parameter from `VocabularyEntry` to `Word`
   - Displays `word.word` as the word text
   - Shows "Definition coming soon" placeholder (per D007)
   - Displays `context_text` as example sentence in italics
   - Shows source reference using `source_book_id` and `source_page`
   - Empty state message improved: "Tap on words while reading to add them to your vocabulary!"

4. **Added integration tests** (`src/ui/vocab.rs` test module):
   - `test_vocab_loads_from_database`: Verifies words are loaded correctly with all fields
   - `test_vocab_empty_database`: Verifies empty database returns empty vector

## Verification

- ✅ `cargo test --lib vocab::` — All 4 tests pass (2 new vocab tests + 2 existing core vocab tests)
- ✅ `cargo test --lib db::tests` — All 33 database tests pass
- ✅ `cargo check` — Compiles without errors (169 warnings, all pre-existing)
- ✅ Verified Word struct fields match database schema (word, context_text, source_book_id, source_page)

## Diagnostics

**How to inspect what this task built:**

1. **Runtime logs**: Check for "Loaded N words from database" message in console output
2. **Database inspection**: `sqlite3 shusei.db "SELECT COUNT(*) FROM words;"` to verify word count
3. **Test verification**: Run `cargo test --lib ui::vocab::tests::test_vocab_loads_from_database -- --nocapture` to see detailed logs
4. **Failure visibility**: Check logs for "Failed to load vocabulary: {error}" if database load fails

**Error shapes:**
- Database open failure: `ShuseiError::Database` with rusqlite error details
- Query failure: `ShuseiError::Database` with SQL error details
- Task failure: JoinError from tokio task spawn

## Deviations

None. All steps from the task plan were implemented as specified.

## Known Issues

None. The implementation is complete and all verification checks pass.

## Files Created/Modified

- `src/core/db.rs` — Added `get_all_words()` method to Database struct
- `src/ui/vocab.rs` — Complete rewrite to use Word struct, async database loading, and updated WordCard display
