---
id: T04
slice: S04
milestone: M002-dbrk2n
title: Write Integration Tests
estimated_steps: 5
estimated_files: 1
---

# T04: Write Integration Tests

**Slice:** S04 — Word Collection
**Milestone:** M002-dbrk2n

## Description

Write integration tests to verify vocabulary list loading, search filtering, delete operations, and export generation work correctly with in-memory SQLite. These tests mirror the S03 integration test pattern.

## Steps

1. Add test module to `src/ui/vocab.rs` with `#[cfg(test)]`
2. Create `test_vocab_loads_from_database()` — Insert words via `db.create_word()`, verify VocabPage loads them
3. Create `test_word_search_filter()` — Insert multiple words, apply search query, verify filtered results
4. Create `test_word_delete_with_confirmation()` — Insert word, delete via UI, verify removed from database
5. Create `test_export_functions()` — Insert words, call export functions, verify output format
6. Run all tests with `cargo test --lib vocab::`

## Must-Haves

- [ ] Test for vocabulary loading from database
- [ ] Test for search/filter functionality
- [ ] Test for delete operation (verifies database deletion)
- [ ] Test for export generation (verifies output format)
- [ ] All tests use in-memory SQLite for isolation
- [ ] All tests pass with `cargo test --lib vocab::`

## Verification

- `cargo test --lib vocab::` — 4+ tests pass
- `cargo test --lib db::tests::word_operations` — Regression check passes
- `cargo check` — Compiles without errors

## Observability Impact

- Signals added/changed: Test assertions verify database state and UI behavior
- How a future agent inspects this: Run `cargo test --lib vocab::` to verify vocab functionality
- Failure state exposed: Test failures indicate which operation (load/search/delete/export) is broken

## Inputs

- `src/ui/vocab.rs` — VocabPage, WordCard, search logic, delete handler, export handlers
- `src/core/db.rs` — Database methods for word operations
- S03 integration tests — Pattern to follow (test_word_tap_saves_to_database, etc.)

## Expected Output

- `src/ui/vocab.rs` — Test module with 4+ integration tests
- Test coverage for all major vocab operations (load, search, delete, export)
