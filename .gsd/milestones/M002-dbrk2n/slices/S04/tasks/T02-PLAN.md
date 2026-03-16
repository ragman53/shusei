---
id: T02
slice: S04
milestone: M002-dbrk2n
title: Implement Word Delete with Confirmation
estimated_steps: 4
estimated_files: 1
---

# T02: Implement Word Delete with Confirmation

**Slice:** S04 — Word Collection
**Milestone:** M002-dbrk2n

## Description

Implement the delete functionality for vocabulary words with a confirmation dialog to prevent accidental deletion. The WordCard component already has a delete button with a TODO handler - this task wires it up to actually delete from the database with proper UX.

## Steps

1. Add confirmation dialog state to VocabPage (show_dialog: Signal<Option<i64>> for word ID to delete)
2. Create ConfirmationDialog component (or reuse existing pattern from S02/S03)
3. Wire delete button in WordCard to set show_dialog with word ID
4. On confirm: call `db.delete_word(id)`, remove word from local state, show success toast
5. On cancel: clear show_dialog, no action taken
6. Handle delete errors gracefully (show error toast if delete fails)

## Must-Haves

- [ ] Confirmation dialog appears before deletion
- [ ] Delete removes word from database via `db.delete_word()`
- [ ] Word removed from local state after delete (UI updates immediately)
- [ ] Success toast shown after delete ("Word deleted")
- [ ] Error toast shown if delete fails
- [ ] Dialog can be cancelled without deleting

## Verification

- `cargo test --lib vocab::test_word_delete_with_confirmation` — Test passes
- Manual test: Click delete → dialog appears → confirm → word disappears → toast shown
- Manual test: Click delete → dialog appears → cancel → word remains

## Observability Impact

- Signals added/changed: Delete event logged with word ID; success/error toast events
- How a future agent inspects this: Check `words` table before/after delete; runtime logs show "Deleted word {id}"
- Failure state exposed: Error toast with message if delete fails; word remains in list

## Inputs

- `src/ui/vocab.rs` — Current WordCard with delete button placeholder
- `src/core/db.rs` — `delete_word()` method (already exists)
- S03 ToastNotification component — Reusable for success/error feedback

## Expected Output

- `src/ui/vocab.rs` — Confirmation dialog component, delete handler with database integration, toast feedback
