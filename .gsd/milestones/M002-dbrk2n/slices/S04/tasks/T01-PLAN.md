---
id: T01
slice: S04
milestone: M002-dbrk2n
title: Connect Vocabulary Page to Database
estimated_steps: 6
estimated_files: 2
---

# T01: Connect Vocabulary Page to Database

**Slice:** S04 — Word Collection
**Milestone:** M002-dbrk2n

## Description

Connect the vocabulary page to the actual SQLite database to display saved words. Currently, VocabPage loads empty data with a TODO comment. This task wires up the database connection and replaces the placeholder VocabularyEntry struct with the actual Word struct from db.rs.

## Steps

1. Add `get_all_words()` method to Database in `src/core/db.rs` (currently only has `get_ai_generated_words()`)
2. Update `src/ui/vocab.rs` to import `Word` struct from `src/core/db.rs` instead of `VocabularyEntry`
3. Replace `VocabularyEntry` usage in VocabPage with `Word` struct
4. Update `use_effect` to call `db.get_all_words()` instead of returning empty vector
5. Update WordCard component to display `definition` as "Definition coming soon" placeholder (per D007)
6. Update WordCard to display `context_text` as example sentence and `source_book_id`/`source_page` as source reference

## Must-Haves

- [ ] `get_all_words()` method added to Database (returns all words ordered by created_at DESC)
- [ ] VocabPage loads words from database on mount
- [ ] WordCard displays word text, context sentence, source book/page
- [ ] Definition shows "Definition coming soon" placeholder (not empty)
- [ ] Loading state handled gracefully
- [ ] Empty state shows helpful message ("Tap on words while reading...")

## Verification

- `cargo test --lib vocab::test_vocab_loads_from_database` — Test passes
- `cargo check` — Compiles without errors
- Verify WordCard displays all required fields (word, context, source)

## Observability Impact

- Signals added/changed: VocabPage load event logged when words are fetched
- How a future agent inspects this: Check `words` table with `sqlite3 shusei.db "SELECT COUNT(*) FROM words;"`; runtime logs show word count loaded
- Failure state exposed: Empty list displayed when database has words (check logs for database error)

## Inputs

- `src/core/db.rs` — Existing Word struct and CRUD operations
- `src/ui/vocab.rs` — Current VocabPage scaffold with TODO placeholders
- S03 word tap implementation — Saves words to database with context_text

## Expected Output

- `src/core/db.rs` — New `get_all_words()` method
- `src/ui/vocab.rs` — VocabPage connected to database, WordCard updated to use Word struct fields
