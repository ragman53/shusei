# T02: Annotation Models and CRUD Operations

**Goal:** Implement Annotation models and full CRUD operations

## Implementation Plan

1. Create `Annotation`, `NewAnnotation`, `UpdateAnnotation` structs
2. Create `AnnotationType` enum with Highlight, Bookmark, Note variants
3. Implement `FromStr` and `Display` for AnnotationType
4. Implement CRUD methods:
   - `create_annotation()` - Insert new annotation
   - `get_annotation()` - Retrieve by ID
   - `get_annotations_by_book()` - Get all annotations for a book
   - `get_annotations_by_type()` - Filter by type (highlight/bookmark/note)
   - `get_bookmarks()`, `get_highlights()`, `get_notes()` - Convenience methods
   - `update_annotation()` - Update existing annotation
   - `delete_annotation()` - Delete by ID
   - `delete_annotations_by_book()` - Bulk delete for a book
5. Add helper methods on `NewAnnotation` for easy construction

## Files to Modify

- `src/core/db.rs` - Models and CRUD methods

## Acceptance Criteria

- [x] AnnotationType enum with string conversion
- [x] Annotation struct with all fields
- [x] All CRUD operations implemented
- [x] Convenience methods for each annotation type
- [x] Code compiles without errors
