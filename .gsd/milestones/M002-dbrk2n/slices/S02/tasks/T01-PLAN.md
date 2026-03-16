# T01: Book Creation Flow

**Slice:** S02 — Camera Book Capture
**Milestone:** M002-dbrk2n

## Description

Implement actual book creation in `AddBookForm` by wiring to `Database::create_book()`. Currently the form navigates without saving — this task makes book creation persistent so the camera page has a valid `book_id` to save pages to.

## Steps

1. Add `Database` signal to `AddBookForm` component (initialize with app data directory)
2. Modify `handle_submit` closure to:
   - Call `Database::create_book(title, author)` asynchronously
   - Handle errors (show user-friendly message in UI)
   - Navigate to `/camera/:book_id` on success (use `Route::Camera` with book_id)
3. Add error state signal (`error_message: Signal<Option<String>>`)
4. Add loading state signal (`is_saving: Signal<bool>`) for submit button feedback
5. Disable submit button while saving
6. Update route navigation to pass `book_id` parameter

## Must-Haves

- [ ] Database initialized with correct path (`shusei.db` in app data directory)
- [ ] `create_book()` called asynchronously in `spawn(async move { ... })`
- [ ] Error message displayed in UI on database failure
- [ ] Loading indicator shown while saving
- [ ] Navigation to `/camera/:book_id` on success (not `/camera` without ID)
- [ ] Form validation (title and author required) preserved

## Verification

- `cargo test --lib db::tests::test_create_book_and_save_page` — New test verifies book + page save flow
- Manual desktop test: Run `dx serve`, create book, verify in database file
- Code inspection: `AddBookForm` calls `Database::create_book()` before navigation

## Observability Impact

- **Signals added/changed:**
  - `log::info!("Book created: id={}, title={}", book.id, book.title)` — Book creation success
  - `log::error!("Book creation failed: {}", e)` — Database error
- **How a future agent inspects this:**
  - Check database file: `sqlite3 shusei.db "SELECT * FROM books;"`
  - Run `cargo test --lib db::` to verify persistence logic
- **Failure state exposed:**
  - `error_message` signal displayed in red banner at top of form
  - Submit button re-enabled after error (user can retry)

## Inputs

- `src/core/db.rs` — `Database::create_book()` method (already implemented, tested in S01)
- `src/app.rs` — Route definitions (need to add book_id parameter to Camera route)
- S01 database persistence tests — Pattern to follow for error handling

## Expected Output

- `src/ui/add_book.rs` — Modified to actually create books via database
- `src/core/db.rs` — New test `test_create_book_and_save_page` (integration test for book + page flow)
- Working book creation flow: user enters title/author → book saved to SQLite → camera opens with book_id
