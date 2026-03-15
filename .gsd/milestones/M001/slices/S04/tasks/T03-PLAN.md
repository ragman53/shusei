# T03: Annotation Unit Tests

**Goal:** Write comprehensive unit tests proving annotation foundation works

## Implementation Plan

Write unit tests covering:

1. **Creation tests:**
   - `test_create_highlight_annotation` - Create highlight with color
   - `test_create_bookmark_annotation` - Create bookmark
   - `test_create_note_annotation` - Create note with user memo
   - `test_annotation_with_position` - Create with position range

2. **Query tests:**
   - `test_get_annotations_by_book` - Retrieve all annotations for a book
   - `test_get_annotations_by_type` - Filter by type
   - `test_get_annotation_returns_none_for_non_existent` - Handle missing data

3. **Type-specific tests:**
   - `test_get_bookmarks` - Get only bookmarks
   - `test_get_highlights` - Get only highlights
   - `test_get_notes` - Get only notes

4. **Update/delete tests:**
   - `test_update_annotation` - Update fields
   - `test_delete_annotation` - Delete single annotation
   - `test_delete_annotations_by_book` - Bulk delete

5. **Edge cases:**
   - `test_annotation_type_enum_conversion` - String ↔ enum conversion
   - `test_annotation_default_color_for_highlights` - Null color handling
   - `test_multiple_bookmarks_same_page` - Multiple annotations on same page

## Files to Modify

- `src/core/db.rs` - Unit tests in tests module

## Acceptance Criteria

- [x] All 15 test cases implemented
- [x] Tests cover CRUD operations
- [x] Tests cover edge cases
- [x] Tests follow existing patterns from books/book_pages tests
