# S01: Core Infrastructure

**Goal:** Create database foundation with Book model and books table schema

Purpose: Establish the data layer that all book operations depend on
Output: Working database schema with tested CRUD operations
**Demo:** Create database foundation with Book model and books table schema

Purpose: Establish the data layer that all book operations depend on
Output: Working database schema with tested CRUD operations

## Must-Haves


## Tasks

- [x] **T01: 01-core-infrastructure 01** `est:15min`
  - Create database foundation with Book model and books table schema

Purpose: Establish the data layer that all book operations depend on
Output: Working database schema with tested CRUD operations
- [x] **T02: 01-core-infrastructure 02**
  - Implement filesystem storage for cover photos with database path references

Purpose: Enable image storage without SQLite BLOB memory issues
Output: Working file storage system integrated with book database
- [x] **T03: 01-core-infrastructure 03** `est:15min`
  - Build library UI with book list and add book modal form

Purpose: Provide user interface for viewing and adding books
Output: Working library screen and add book form
- [x] **T04: 01-core-infrastructure 04**
  - Implement Android lifecycle handling with state persistence and JNI memory management

Purpose: Ensure app handles background transitions gracefully without memory leaks
Output: Lifecycle-aware app with state restoration and clean JNI patterns

## Files Likely Touched

- `src/core/models.rs`
- `src/core/db.rs`
- `src/core/storage.rs`
- `src/core/db.rs`
- `src/ui/library.rs`
- `src/ui/add_book.rs`
- `src/app.rs`
- `src/platform/android.rs`
- `src/core/state.rs`
- `src/app.rs`
