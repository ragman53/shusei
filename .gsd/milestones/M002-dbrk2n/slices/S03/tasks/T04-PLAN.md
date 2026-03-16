# T04: Write Integration Tests

**Slice:** S03 — PDF Reflow Reader
**Milestone:** M002-dbrk2n

## Description

Write integration tests that verify all S03 features work together: word tap saves to database, progress auto-saves on scroll, last position restores on mount. These tests provide regression protection and executable verification that the slice is complete.

## Steps

1. Add test module to `src/ui/reader.rs` with `#[cfg(test)]`
2. Create `test_word_tap_saves_to_database()`:
   - Create in-memory database with test book and page
   - Simulate word tap by calling `handle_word_tap()` logic
   - Verify word saved to `words` table with correct fields
   - Assert sentence context extracted correctly
3. Create `test_progress_auto_save()`:
   - Create in-memory database with test book
   - Simulate scroll event, trigger debounced save
   - Verify `processing_progress` table updated with `last_processed_page`
4. Create `test_last_position_restore()`:
   - Create book with progress (last_processed_page = 5)
   - Simulate mount, verify position restore logic scrolls to page 5
   - Assert correct page ID targeted
5. Run all tests with `cargo test --lib reader::`

## Must-Haves

- [ ] Test module added to `src/ui/reader.rs`
- [ ] `test_word_tap_saves_to_database()` passes
- [ ] `test_progress_auto_save()` passes
- [ ] `test_last_position_restore()` passes
- [ ] All tests use in-memory database for isolation
- [ ] Tests verify database state after operations

## Verification

- `cargo test --lib reader::` — All 3+ tests pass
- `cargo test --lib db::` — All existing database tests still pass (no regressions)
- CI/CD: Tests run on every build (future enhancement)

## Observability Impact

- **Signals added/changed:** None (tests are internal verification)
- **How a future agent inspects this:**
  - Run `cargo test --lib reader::` to verify slice functionality
  - Read test code to understand expected behavior
  - Use tests as documentation for how features should work
- **Failure state exposed:**
  - Test failures show exact assertion that failed
  - Database state visible in test output for debugging

## Inputs

- T01 output: `render_markdown_to_html()` function
- T02 output: `handle_word_tap()` function with database integration
- T03 output: Progress save/restore logic
- `src/core/db.rs` — Database operations for test setup and verification

## Expected Output

- `src/ui/reader.rs` — Test module with 3+ integration tests
- All tests passing with `cargo test --lib reader::`
- Test coverage for word tap, progress save, progress restore
