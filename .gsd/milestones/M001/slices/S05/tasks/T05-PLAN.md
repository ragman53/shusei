# T05: Reader Integration

**Goal:** Integrate voice memo button into note creation dialog in reader view

## Plan

1. **Review existing reader note UI**
   - Examine `src/ui/reader.rs` for note creation flow
   - Understand how annotations are created (uses S04's `NewAnnotation::note()`)
   - Find where `user_note` field is populated in current implementation

2. **Add voice memo button to note dialog**
   - Add microphone icon button next to text input
   - Button opens `VoiceMemoInput` component (from T04)
   - Show transcript result in note text area
   - Allow user to edit before saving

3. **Integrate with annotation save flow**
   - Voice memo transcript → `NewAnnotation::note().with_user_note(transcript)`
   - Save to `annotations` table via `db.create_annotation()`
   - Associate with current book_id and page_number
   - Update UI to show saved state

4. **Handle page-level attachment**
   - Voice memos attach to page number (not text selection)
   - Store page_number in annotation
   - Display voice memo indicator on pages with memos

5. **Add lazy model loading**
   - Load Moonshine model on first voice memo use
   - Keep model loaded until app backgrounded
   - Handle `onPause()` lifecycle (Android) to unload model

6. **Write integration tests**
   - Test voice memo button appears in reader
   - Test transcript saves to annotation correctly
   - Test page association works

## Files to Create/Modify

- `src/ui/reader.rs` — Add voice memo button to note dialog
- `src/core/db.rs` — May need to add voice memo-specific query methods
- `.gsd/milestones/M001/slices/S05/tasks/T05-PLAN.md` — This file

## Verification

- [ ] `cargo check --lib` passes
- [ ] Voice memo button visible in reader note dialog
- [ ] Recording flow integrates smoothly
- [ ] Transcript saves to `annotations.user_note` field
- [ ] Page association works correctly
- [ ] Lazy model loading works
- [ ] 3+ integration tests pass

## Dependencies

- Consumes: VoiceMemoInput component from T04, annotation CRUD from S04
- Produces: Complete voice memo feature integrated with annotations
