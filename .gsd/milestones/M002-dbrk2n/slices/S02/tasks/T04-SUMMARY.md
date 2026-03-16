---
id: T04
parent: S02
milestone: M002-dbrk2n
provides:
  - "Capture Pages" button on book cards
  - Page count badge showing number of captured pages
  - Navigation from library to camera with book_id
key_files:
  - src/ui/library.rs
  - src/core/db.rs
key_decisions:
  - "Used separate book_id clone for use_effect closure to avoid move errors"
  - "Page count fetched asynchronously on component mount to avoid blocking UI"
patterns_established:
  - "Async page count loading in BookCard component"
  - "Database query for aggregations (COUNT) in db.rs"
observability_surfaces:
  - "log::debug!(\"Navigating to camera for book_id={}\") — Navigation event"
  - "log::debug!(\"Book card rendered: {} pages\") — Card render logging"
duration: 2h
verification_result: passed
completed_at: 2026-03-16
blocker_discovered: false
---

# T04: Library Navigation

**Added "Capture Pages" button and page count badge to book cards in LibraryScreen.**

## What Happened

Implemented the library navigation flow by modifying `BookCard` component to display:
1. **Page count badge** - Shows "X pages" or "No pages yet" based on actual database count
2. **"Capture Pages" button** - Green button that navigates to `/camera/:book_id` route

Key implementation steps:
1. Added `get_page_count(&book_id)` method to `Database` struct in `src/core/db.rs`
2. Modified `BookCard` component to:
   - Load page count asynchronously on mount via `use_effect`
   - Display page count badge with appropriate styling
   - Add "Capture Pages" button with navigation logic
3. Fixed Rust ownership issues by cloning `book_id` for use in closures
4. Added comprehensive tests for `get_page_count` method

## Verification

**Unit tests passed:**
```bash
cargo test --lib ui::library  # 5 passed
cargo test --lib db::tests::book_pages  # 7 passed (including 2 new tests)
```

**New tests added:**
- `test_get_page_count_returns_correct_count` - Verifies count increments as pages are added
- `test_get_page_count_returns_zero_for_book_without_pages` - Verifies initial state

**Code inspection:**
- `LibraryScreen` uses `use_navigator()` for navigation ✓
- Button navigates to `Route::CameraBook { book_id: ... }` ✓
- Page count fetched via `Database::get_page_count()` ✓
- Debug logging present for navigation events ✓

**Compilation:** Code compiles successfully with `cargo check` and `cargo run --features desktop`

## Diagnostics

**How to inspect what this task built:**

1. **Check navigation logs (desktop):**
   ```bash
   cargo run --features desktop 2>&1 | grep "Navigating to camera"
   # Expected: DEBUG Navigating to camera for book_id=<book_id>
   ```

2. **Check page count logging:**
   ```bash
   cargo run --features desktop 2>&1 | grep "Book card rendered"
   # Expected: DEBUG Book card rendered: X pages
   ```

3. **UI verification:**
   - Navigate to library screen
   - Create a book via "Add Book" button
   - Verify "Capture Pages" button appears on book card
   - Verify page count badge shows "No pages yet"
   - Click "Capture Pages" → verify navigation to `/camera/{book_id}`
   - After capturing page, return to library → verify badge shows "1 pages"

4. **Database inspection:**
   ```bash
   sqlite3 shusei.db "SELECT id, title, pages_captured FROM books;"
   sqlite3 shusei.db "SELECT book_id, COUNT(*) as page_count FROM book_pages GROUP BY book_id;"
   ```

## Deviations

None - implementation followed the task plan exactly.

## Known Issues

None - all must-haves met:
- [x] "Capture Pages" button visible on each book card
- [x] Button navigates to `/camera/:book_id` with correct book_id
- [x] Page count badge shows current number of pages
- [x] Empty book list handled gracefully (existing functionality)
- [x] Navigation flow works end-to-end (verified via code inspection)

## Files Created/Modified

- `src/core/db.rs` — Added `get_page_count()` method and 2 new unit tests
- `src/ui/library.rs` — Modified `BookCard` component with page count badge and "Capture Pages" button
