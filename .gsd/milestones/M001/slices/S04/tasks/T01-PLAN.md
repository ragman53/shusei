# T01: Annotation Database Schema

**Goal:** Create database schema for annotations (highlights, bookmarks, notes)

## Implementation Plan

1. Add `annotations` table to database schema with fields:
   - id, book_id, page_number, annotation_type, content
   - color, position_start, position_end, user_note
   - created_at, updated_at
2. Add indexes for efficient querying by book, page, and type
3. Add CHECK constraint for annotation_type enum

## Files to Modify

- `src/core/db.rs` - Database schema initialization

## Acceptance Criteria

- [x] Annotations table created with all required fields
- [x] Indexes created for book_id, page_number, annotation_type
- [x] CHECK constraint ensures valid annotation types
- [x] Schema compiles without errors
