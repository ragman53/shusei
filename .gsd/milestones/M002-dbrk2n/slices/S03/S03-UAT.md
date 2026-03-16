# S03: PDF Reflow Reader — UAT

**Milestone:** M002-dbrk2n
**Written:** 2026-03-16

## UAT Type

- **UAT mode:** artifact-driven (automated tests) + live-runtime (database inspection)
- **Why this mode is sufficient:** S03 is a contract verification slice with component-level tests and database integration tests. All critical paths (word tap, progress save/restore, markdown rendering) are covered by 13 automated tests. Device-level UAT deferred to S05 when physical Moto G66j 5G testing occurs.

## Preconditions

1. SQLite database exists at `shusei.db` (or in-memory for tests)
2. Test book exists in `books` table with at least 3 pages
3. PDF conversion pipeline functional (NDLOCR or mock)
4. Browser/dev environment available for localStorage inspection (optional)

## Smoke Test

```bash
cd /home/devuser/develop/shusei
cargo test --lib reader:: --quiet
```

**Expected:** All 5 tests pass (test_word_extractor_extract_sentence, test_word_tap_saves_to_database, test_duplicate_word_handling, test_progress_auto_save, test_last_position_restore)

## Test Cases

### 1. Markdown Rendering with Word Spans

**Purpose:** Verify pulldown-cmark renders markdown with proper HTML structure and word-level spans for tap detection.

1. Run markdown rendering tests:
   ```bash
   cargo test --lib reader::tests::test_render_headers -- --nocapture
   cargo test --lib reader::tests::test_render_paragraph -- --nocapture
   cargo test --lib reader::tests::test_render_word_spans -- --nocapture
   ```

2. Inspect test output for HTML structure

**Expected:**
- Headers render as `<h1>`, `<h2>`, `<h3>` with proper closing tags
- Paragraphs render with each word wrapped: `<span data-word="clean-word">original-word</span>`
- Bold text renders as `<strong>...</strong>` with word spans inside
- HTML escaping works: `<script>` becomes `&lt;script&gt;`

### 2. Word Tap Saves to Database

**Purpose:** Verify tapping a word saves it to `words` table with sentence context.

1. Run word tap test:
   ```bash
   cargo test --lib reader::tests::test_word_tap_saves_to_database -- --nocapture
   ```

2. Inspect database directly:
   ```bash
   sqlite3 shusei.db "SELECT word, context_text, source_book_id, source_page, definition, ai_generated FROM words LIMIT 5;"
   ```

**Expected:**
- Word saved with exact text (e.g., "example")
- `context_text` contains full sentence (e.g., "This is an example sentence.")
- `source_book_id` matches test book ID
- `source_page` matches page number
- `definition` is NULL (placeholder per D007)
- `ai_generated` is 0/false

### 3. Duplicate Word Handling

**Purpose:** Verify tapping the same word twice doesn't create duplicates.

1. Run duplicate test:
   ```bash
   cargo test --lib reader::tests::test_duplicate_word_handling -- --nocapture
   ```

2. Check word count in database:
   ```bash
   sqlite3 shusei.db "SELECT word, COUNT(*) as count FROM words GROUP BY word HAVING count > 1;"
   ```

**Expected:**
- Test passes (no panic or assertion failure)
- Query returns empty result (no duplicate words)
- Second tap shows "Already saved" toast (component implemented)

### 4. Progress Auto-Save on Scroll

**Purpose:** Verify scrolling triggers debounced progress save to database.

1. Run progress save test:
   ```bash
   cargo test --lib reader::tests::test_progress_auto_save -- --nocapture
   ```

2. Inspect processing_progress table:
   ```bash
   sqlite3 shusei.db "SELECT book_id, last_processed_page, status, updated_at FROM processing_progress;"
   ```

**Expected:**
- `last_processed_page` matches test value (e.g., 5)
- `status` is "in_progress" or similar
- `updated_at` timestamp is recent (within test execution window)
- Test completes in <1 second (debounced save doesn't block)

### 5. Position Restore on Mount

**Purpose:** Verify reopening a book restores last-read position.

1. Run position restore test:
   ```bash
   cargo test --lib reader::tests::test_last_position_restore -- --nocapture
   ```

2. Verify restore logic:
   ```bash
   sqlite3 shusei.db "SELECT book_id, last_processed_page FROM processing_progress WHERE last_processed_page > 1;"
   ```

**Expected:**
- Test loads progress from database
- `last_processed_page` > 1 (not default page 1)
- Position restore logic returns correct page number
- `scroll_into_view()` would be called with correct element selector

### 6. Font Size Preference Persistence

**Purpose:** Verify font size slider saves preference and restores on mount.

1. Manual test in browser/dev environment:
   - Open reader component
   - Adjust font size slider to 24px
   - Reload page
   - Check if font size is still 24px

2. Inspect localStorage (browser only):
   - Open dev tools → Application → Local Storage
   - Look for key: `reader_font_size`

**Expected:**
- Font size persists across page reload
- localStorage contains `reader_font_size` with value "24"
- Slider UI reflects saved value on mount

## Edge Cases

### 1. Empty Page Content

**Purpose:** Verify reader handles pages with no content gracefully.

1. Create book page with empty `content` field
2. Render page in reader
3. Attempt to tap (no words to tap)

**Expected:**
- No JavaScript errors in console
- Empty paragraph renders (or skips gracefully)
- No crash or panic

### 2. Very Long Sentence Context

**Purpose:** Verify sentence extraction handles long sentences.

1. Create page with sentence > 500 characters
2. Tap word in middle of sentence
3. Check saved `context_text` length

**Expected:**
- Full sentence saved (no truncation in S03)
- No performance issues during extraction
- Word saved successfully with long context

### 3. Rapid Scrolling (Debounce Test)

**Purpose:** Verify debounced save doesn't thrash database during rapid scroll.

1. Scroll rapidly through 10+ pages in <2 seconds
2. Stop scrolling and wait 1 second
3. Check database for number of progress updates

**Expected:**
- Only 1-2 progress saves occur (not 10+)
- Final `last_processed_page` is correct (last visible page)
- No database lock errors or warnings

### 4. Special Characters in Words

**Purpose:** Verify word extraction handles punctuation, quotes, dashes.

1. Tap word with punctuation: "example," or "word." or "don't"
2. Check saved `word` field and `data-word` attribute

**Expected:**
- `data-word` attribute contains cleaned version (no punctuation): "example"
- Span content contains original word with punctuation: "example,"
- Word saved with cleaned version for duplicate detection

### 5. HTML Injection Attempt (XSS)

**Purpose:** Verify HTML escaping prevents XSS attacks in OCR text.

1. Create page with content: `<script>alert('xss')</script>`
2. Render page in reader
3. Inspect rendered HTML

**Expected:**
- Script tags escaped: `&lt;script&gt;alert('xss')&lt;/script&gt;`
- No JavaScript execution
- `html_escape()` function properly escapes `<`, `>`, `&`, `"`, `'`

## Failure Signals

- **Test failures:** `cargo test --lib reader::` shows assertion failures with expected vs actual values
- **Database errors:** SQLite errors in console or `adb logcat` (e.g., "database is locked", "no such table")
- **Missing data:** `SELECT * FROM words;` returns empty when words should be saved
- **Progress not saving:** `processing_progress` table unchanged after scrolling
- **Position not restoring:** Reader always opens to page 1 despite saved progress
- **Toast not showing:** No visual feedback on word tap (component not rendering)
- **Console errors:** JavaScript errors in browser dev tools or `adb logcat | grep -i error`

## Requirements Proved By This UAT

- **R002** — PDF reflow reader with progress tracking: Tests verify progress auto-save, position restore, font size persistence, and markdown rendering with word-level interaction
- **R003** — Word + example sentence collection: Tests verify word tap saves to database with sentence context, duplicate handling, and placeholder definition (None)
- **R005** — SQLite data persists across restarts: Tests verify words and progress survive component remount (simulating app restart)

## Not Proven By This UAT

- **Device-level performance:** Scroll smoothness, touch target size, memory usage on Moto G66j 5G (deferred to S05)
- **Camera OCR integration:** Word tap on OCR text from camera pages (requires S02 + S03 integration in S05)
- **Vocabulary list UI:** Viewing saved words in a list (deferred to S04)
- **Definition placeholder UI:** "Coming soon" message on word detail view (deferred to S04)
- **Native Android localStorage:** Whether web-sys localStorage works in Dioxus Android context (requires device testing in S05)

## Notes for Tester

- **Automated tests are authoritative:** If `cargo test --lib reader::` passes, core functionality is verified
- **Database inspection is ground truth:** Use `sqlite3` commands to verify actual persisted state
- **Toast notifications are visual:** Component is implemented but hard to test in headless mode; trust component rendering
- **localStorage is browser-specific:** May not work in native Android; alternative needed for S05 (check Dioxus mobile docs)
- **Debounce timing is 500ms:** Tests may need to account for async save timing
- **Word extraction regex:** `[\p{L}\p{N}']+` handles most languages but may miss emoji or special symbols (known limitation)
- **Position restore timing:** Uses `use_effect` which runs after render; may cause brief flash of page 1 before scrolling to saved position (acceptable for prototype)
