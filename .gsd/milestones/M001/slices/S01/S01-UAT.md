# S01: Core Infrastructure — UAT

**Milestone:** M001
**Written:** 2026-03-11

## UAT Type

- UAT mode: artifact-driven
- Why this mode is sufficient: This slice establishes foundational infrastructure (database schema, storage layer, UI components, lifecycle handling) that is fully verifiable through automated unit tests. No live runtime or human experience testing is required because:
  - Database operations are verified through in-memory SQLite tests
  - Storage operations use temporary directories in tests
  - UI components are static Dioxus components without backend integration yet
  - Android lifecycle handlers are platform-specific and will be verified during S02+ when the app runs on actual devices

## Preconditions

- Rust toolchain installed with Android target (for cross-compilation verification)
- Project dependencies installed via `cargo fetch`
- No external services or API keys required (fully offline)

## Smoke Test

```bash
# Build the project to verify all components compile together
cargo build --lib

# Run all slice tests
cargo test --lib
```

**Expected:** Build succeeds with no errors. All 48 tests pass (4 model + 18 database + 10 storage + 11 state + 5 UI tests).

## Test Cases

### 1. Database Schema Verification

1. Run: `cargo test --lib db::tests::books_schema -- --nocapture`
2. Check output for all 6 tests passing

**Expected:**
- ✓ table_exists - books table exists in SQLite
- ✓ index_exists - idx_books_title index created
- ✓ wal_mode_supported - WAL journal mode enabled
- ✓ insert_valid_book_succeeds - Can insert book with required fields
- ✓ reject_missing_title - NOT NULL constraint enforced
- ✓ reject_missing_author - NOT NULL constraint enforced

### 2. Book CRUD Operations

1. Run: `cargo test --lib db::tests::books_crud -- --nocapture`
2. Verify all 6 CRUD operations work

**Expected:**
- ✓ create_book_inserts_and_returns_id - Returns non-empty UUID
- ✓ get_book_retrieves_by_id - Returns book with matching ID
- ✓ get_book_returns_none_for_non_existent - Returns None for invalid ID
- ✓ get_all_books_returns_all_sorted_by_title - Returns alphabetically sorted list
- ✓ update_book_modifies_existing - Changes persist after update
- ✓ delete_book_removes_book - Book no longer retrievable after deletion

### 3. Cover Photo Storage Integration

1. Run: `cargo test --lib db::tests::cover_photo -- --nocapture`
2. Verify filesystem storage works with database references

**Expected:**
- ✓ test_save_cover_photo_saves_file_and_updates_database - File created, database updated
- ✓ test_save_cover_photo_returns_stored_path - Returns relative path (e.g., `images/cover_abc123.bin`)
- ✓ test_remove_cover_photo_deletes_file_and_clears_database - File deleted, database field cleared
- ✓ test_get_book_returns_book_with_cover_path_after_save - Book includes cover_path after save

### 4. Filesystem Storage Operations

1. Run: `cargo test --lib storage::tests -- --nocapture`
2. Verify StorageService handles all image operations

**Expected:**
- ✓ test_save_image_writes_file_to_correct_directory - File created in images/ subdirectory
- ✓ test_save_image_returns_relative_path - Returns relative path, not absolute
- ✓ test_get_image_reads_file_content_back - Loaded bytes match saved bytes
- ✓ test_delete_image_removes_file - File no longer exists after deletion
- ✓ test_get_image_returns_error_for_non_existent_file - Returns error for missing files
- ✓ test_images_directory_created_if_not_exists - Directory auto-created on first save
- ✓ test_save_page_image_creates_book_directory - Pages/{book_id}/ structure created
- ✓ test_save_page_image_returns_relative_path - Returns relative path for page images
- ✓ test_save_page_image_preserves_content - Page image content preserved

### 5. State Persistence and Serialization

1. Run: `cargo test --lib state::tests -- --nocapture`
2. Verify AppState serialization and file I/O

**Expected:**
- ✓ test_appstate_serializes_to_json - JSON contains all fields
- ✓ test_appstate_deserializes_from_json - Correctly parses JSON back to struct
- ✓ test_appstate_default_values - Default state has route="/", scroll=0.0, timestamp=0
- ✓ test_save_to_prefs_writes_to_file - File created in .shusei/ subdirectory
- ✓ test_load_from_prefs_reads_from_file - Correctly reads and parses saved state
- ✓ test_load_from_prefs_returns_none_if_file_not_exists - Graceful handling of missing file
- ✓ test_roundtrip_serialization - Serialize → deserialize produces identical struct

### 6. Android Lifecycle Handlers

1. Run: `cargo test --lib android::tests::lifecycle -- --nocapture`
2. Verify lifecycle logic and JNI patterns

**Expected:**
- ✓ test_on_pause_saves_state - State serialization logic verified
- ✓ test_on_resume_loads_state - State deserialization logic verified
- ✓ test_jni_frame_management_pattern - JNI frame pattern documented
- ✓ test_lifecycle_error_handling - Graceful error handling when file missing

### 7. UI Component Compilation

1. Run: `cargo test --lib ui::library::tests -- --nocapture`
2. Run: `cargo test --lib ui::add_book::tests -- --nocapture`
3. Verify UI components compile and basic logic works

**Expected:**
- ✓ test_library_screen_renders_without_books - Empty state renders
- ✓ test_library_screen_shows_book_list_when_loaded - Book list renders with data
- ✓ test_books_sorted_alphabetically_by_title - Sorting logic works
- ✓ test_add_book_button_navigates_to_add_book_route - Route::AddBook exists
- ✓ test_book_card_shows_title_and_author - BookCard displays correct fields
- ✓ test_add_book_form_renders_with_inputs - Form has title and author inputs
- ✓ test_submit_disabled_when_title_empty - Validation prevents empty title
- ✓ test_submit_disabled_when_author_empty - Validation prevents empty author
- ✓ test_submit_enabled_when_both_fields_filled - Form submits when valid

## Edge Cases

### Database NOT NULL Constraints

1. Attempt to insert book without title: `INSERT INTO books (id, author, ...) VALUES (...)`
2. Attempt to insert book without author: `INSERT INTO books (id, title, ...) VALUES (...)`

**Expected:**
- Both operations fail with SQL constraint error
- Application handles errors gracefully (no panics)

### Missing State File on First Launch

1. Delete `.shusei/app_state.json` if it exists
2. Call `AppState::load_from_prefs()`

**Expected:**
- Returns `Ok(None)` without error
- App initializes with default state

### Storage with Non-existent File

1. Call `storage.get_image("images/nonexistent.bin")` on fresh storage

**Expected:**
- Returns `Err` with descriptive error message
- No panic or crash

### Android JavaVM Not Initialized (Desktop Fallback)

1. Call `get_assets_directory()` on desktop (JavaVM not initialized)

**Expected:**
- Falls back to `std::env::current_dir()`
- Logs warning message
- Returns current directory path

## Failure Signals

- **Build failures:** `cargo build` fails with compilation errors
- **Test failures:** Any of the 48 tests fail
- **SQL errors:** "table books has no column named updated_at" (schema mismatch)
- **NOT NULL constraint errors:** Missing required columns in INSERT statements
- **File I/O errors:** "Failed to create images directory" (permission issues)
- **JNI errors:** "JavaVM not initialized" on Android (requires proper initialization)

## Requirements Proved By This UAT

- **Database foundation:** Books table schema with WAL mode, all CRUD operations working
- **Filesystem storage:** Cover photos and page images stored as files, paths in database
- **UI components:** Library screen and add book form compile and render correctly
- **State persistence:** AppState serializes to JSON and persists across lifecycle transitions
- **Android lifecycle:** onPause/onResume handlers with proper JNI memory management

## Not Proven By This UAT

- **End-to-end book creation flow:** UI not yet connected to database operations
- **Actual Android device behavior:** JNI handlers require physical device/emulator testing
- **State restoration UX:** AppState loads but router doesn't navigate to saved route yet
- **Cover photo picker UI:** Storage works but no UI for selecting/uploading photos
- **Performance under load:** No stress testing of database or storage operations
- **Concurrent access:** WAL mode enabled but concurrent read/write not tested

## Notes for Tester

**What to focus on:**
- All 48 automated tests must pass - this is the primary verification
- Build must succeed with no warnings or errors
- Database schema matches the designed structure (check db.rs line-by-line if needed)

**What can be deferred:**
- Manual Android device testing is documented in T04-SUMMARY.md but not required for S01 completion
- UI components currently show placeholder data - this is expected and will be fixed in downstream slices
- State restoration UX (actually navigating to saved route) is deferred to later slices

**Known rough edges:**
- LibraryScreen shows "No books yet" even after books are added to database (not wired up yet)
- AddBookForm navigates back without creating book (form submission not implemented)
- AppState saves route but doesn't restore it (router integration pending)

**How to verify fixes in downstream slices:**
- S02 should wire LibraryScreen to `db.get_all_books()`
- S02 should connect AddBookForm to `db.create_book()`
- S03+ should integrate AppState with Dioxus router for actual navigation restoration
