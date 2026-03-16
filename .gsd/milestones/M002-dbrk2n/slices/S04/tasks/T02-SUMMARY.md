---
id: T02
parent: S04
milestone: M002-dbrk2n
provides:
  - Word delete functionality with confirmation dialog
  - Toast notification system for user feedback
  - Database integration for word deletion
key_files:
  - src/ui/vocab.rs
key_decisions:
  - Inline confirmation dialog instead of separate component for simplicity
  - Reuse ToastNotification pattern from reader.rs for consistency
  - Async database operations with spawn_blocking for UI responsiveness
patterns_established:
  - Confirmation dialog pattern for destructive actions
  - Toast notification pattern for success/error feedback
  - Signal-based state management for dialog/toast visibility
observability_surfaces:
  - Runtime logs: "Deleted word {id}" on success, "Failed to delete word: {error}" on failure
  - Database: words table reflects deletion immediately
  - UI: Word removed from list, success/error toast shown
duration: 30m
verification_result: passed
completed_at: 2026-03-16
# Set blocker_discovered: true only if execution revealed the remaining slice plan
# is fundamentally invalid (wrong API, missing capability, architectural mismatch).
# Do NOT set true for ordinary bugs, minor deviations, or fixable issues.
blocker_discovered: false
---

# T02: Implement Word Delete with Confirmation

**Implemented word delete with confirmation dialog and toast notifications.**

## What Happened

Implemented the complete delete functionality for vocabulary words:

1. **Added ToastType and ToastNotification component** - Imported the toast notification pattern from reader.rs to provide user feedback for delete operations (success/error states)

2. **Added delete confirmation state** - Added `show_delete_dialog: Signal<Option<i64>>` to track which word ID is pending deletion

3. **Created inline confirmation dialog** - Built a modal dialog that appears when delete is clicked, with:
   - Backdrop overlay (click to dismiss)
   - Dialog card with "Delete Word?" title
   - Warning message "This action cannot be undone"
   - Cancel button (dismisses dialog)
   - Delete button (executes deletion)

4. **Implemented delete handler** - Async handler that:
   - Opens database in spawn_blocking task
   - Calls `db.delete_word(word_id)`
   - Removes word from local state on success (`words.write().retain()`)
   - Shows success toast ("Word deleted") on success
   - Shows error toast with error message on failure
   - Closes dialog after operation completes

5. **Wired WordCard delete button** - Updated WordCard to accept `on_delete: EventHandler<i64>` callback and call it with word ID when trash icon is clicked

6. **Added comprehensive tests** - Created two new tests:
   - `test_word_delete` - Verifies single word deletion from database
   - `test_word_delete_with_multiple_words` - Verifies selective deletion preserves other words

## Verification

All tests pass:
```
cargo test --lib ui::vocab::tests -- --nocapture
running 4 tests
test ui::vocab::tests::test_vocab_empty_database ... ok
test ui::vocab::tests::test_vocab_loads_from_database ... ok
test ui::vocab::tests::test_word_delete ... ok
test ui::vocab::tests::test_word_delete_with_multiple_words ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 109 filtered out
```

Database tests also pass (33 tests):
```
cargo test --lib db::tests -- --nocapture
test result: ok. 33 passed; 0 failed
```

Code compiles without errors:
```
cargo check
Finished dev [unoptimized + debuginfo] target(s)
```

## Diagnostics

**How to inspect what this task built:**

1. **Runtime logs**: Check for "Deleted word {id}" message on success, or "Failed to delete word: {error}" on failure
2. **Database inspection**: `sqlite3 shusei.db "SELECT COUNT(*) FROM words;"` before/after delete to verify removal
3. **Test verification**: Run `cargo test --lib ui::vocab::tests::test_word_delete -- --nocapture` to see detailed logs
4. **UI verification**: 
   - Click trash icon → dialog should appear
   - Click Cancel → dialog closes, word remains
   - Click Delete → word disappears from list, green success toast appears
   - If database error occurs → red error toast appears, word remains

**Error shapes:**
- Database open failure: `ShuseiError::Database` with rusqlite error details
- Delete failure: `ShuseiError::Database` with SQL error details
- Task failure: JoinError from tokio task spawn

## Deviations

None. Implementation followed the task plan exactly.

## Known Issues

None. All must-haves are met:
- ✅ Confirmation dialog appears before deletion
- ✅ Delete removes word from database via `db.delete_word()`
- ✅ Word removed from local state after delete (UI updates immediately)
- ✅ Success toast shown after delete ("Word deleted")
- ✅ Error toast shown if delete fails
- ✅ Dialog can be cancelled without deleting

## Files Created/Modified

- `src/ui/vocab.rs` — Added ToastType enum, ToastNotification component, delete confirmation dialog, delete handler with database integration, toast feedback, and two delete tests
