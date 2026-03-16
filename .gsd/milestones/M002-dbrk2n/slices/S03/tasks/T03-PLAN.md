# T03: Implement Progress Persistence

**Slice:** S03 — PDF Reflow Reader
**Milestone:** M002-dbrk2n

## Description

Implement last-read position persistence: auto-save scroll position to `processing_progress` table (debounced 500ms), restore position on mount using `scroll_into_view()`, and persist font size preference. This delivers R002 (PDF reflow reader with progress tracking).

## Steps

1. Add debounced scroll handler to continuous scroll container:
   - Use `use_effect` with 500ms timeout
   - On scroll end: calculate current page from scroll position
   - Call `db.update_progress(book_id, current_page, "reading")` to save `last_processed_page`
2. Add position restore on mount:
   - In existing `use_effect` that loads book/pages, also call `db.get_progress(book_id)`
   - If progress exists with `last_processed_page > 0`, scroll to that page after render
   - Use `document.get_element_by_id(&format!("page-{}", page_num)).scroll_into_view()`
3. Persist font size preference:
   - Store in `localStorage` or add `font_size_preference` field to user settings (if exists)
   - For simplicity: use `localStorage` via web-sys or dioxus localStorage API
   - Load on mount, apply to container style
4. Add "Page X of Y" display in header (already partially implemented, verify it updates correctly)
5. Test: Scroll to page 5, close app, reopen → verify page 5 visible

## Must-Haves

- [ ] Debounced scroll save (500ms timeout, saves only on scroll end)
- [ ] Progress saved to `processing_progress.last_processed_page`
- [ ] Position restored on mount using `scroll_into_view()`
- [ ] Font size preference persists (localStorage or database)
- [ ] "Page X of Y" display updates as user scrolls

## Verification

- `cargo test --lib reader::test_progress_persistence` — Progress saves and restores correctly
- Manual test: Scroll to page 5 → close app → reopen → verify page 5 at top of viewport
- Manual test: Change font size to 24px → navigate away → return → verify 24px applied
- Database check: `sqlite3 shusei.db "SELECT * FROM processing_progress;"` — verify `last_processed_page` updated

## Observability Impact

- **Signals added/changed:**
  - Progress save event logged with book_id, page_number, timestamp
  - Position restore event logged on mount
  - Font size change logged for preference tracking
- **How a future agent inspects this:**
  - `sqlite3 shusei.db "SELECT book_id, last_processed_page, status FROM processing_progress;"` — verify progress
  - `adb logcat | grep -i "progress saved"` — save events in logs
  - Browser localStorage inspection for font size preference
- **Failure state exposed:**
  - Progress save failures logged (non-fatal, user won't lose place permanently)
  - Position restore failures logged (fallback to page 1)
  - No user-visible error (graceful degradation)

## Inputs

- T01 output: Continuous scroll container with page IDs (`id="page-{page_number}"`)
- `src/core/db.rs` — `update_progress()` and `get_progress()` methods
- `S03-PLAN.md` — Slice requirements for progress tracking

## Expected Output

- `src/ui/reader.rs` — Debounced scroll handler with database save
- `src/ui/reader.rs` — Position restore logic in mount effect
- `src/ui/reader.rs` — Font size persistence using localStorage
- Test function `test_progress_auto_save()` and `test_last_position_restore()`
