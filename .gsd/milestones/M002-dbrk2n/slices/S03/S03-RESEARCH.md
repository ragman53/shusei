# S03: PDF Reflow Reader — Research

**Date:** 2026-03-16

## Summary

S03 delivers a complete PDF reflow reading experience with progress tracking, font size control (12-32px), continuous scroll, and last-read position synchronization. The existing `reader.rs` component already implements ~80% of the required functionality including PDF import, conversion orchestration, continuous scroll view, font size slider, and page jump modal. Key gaps to address: (1) word tap detection for vocabulary collection (R003 dependency), (2) last-read position persistence to database, (3) improved markdown rendering with proper HTML parsing, and (4) progress auto-save during reading.

The PDF conversion pipeline is fully functional in `src/core/pdf.rs` using hayro (pure Rust PDF renderer) for page rendering and NDLOCR for OCR. The `PdfConversionService` handles batch rendering with parallel processing (3 concurrent threads) and OCR with progress callbacks. Database schema supports all required tables: `books`, `book_pages`, `processing_progress`, and `words` for vocabulary.

**Primary recommendation:** Complete the reader by adding (1) word tap interaction layer with span-level click handlers, (2) database persistence for reading progress using `processing_progress` table, (3) `react-markdown` or `pulldown-cmark` integration for proper markdown rendering, and (4) auto-save scroll position on scroll events. Word tap should extract the clicked word + surrounding sentence context, then save to `words` table with `definition: None` (placeholder per D007).

## Recommendation

**Build on existing reader.rs scaffold** rather than rewriting. The component already has:
- ✅ PDF import flow with metadata review dialog
- ✅ Conversion trigger with progress display
- ✅ Continuous scroll with all pages rendered
- ✅ Font size control (12-32px range)
- ✅ Page jump modal
- ✅ Book card with conversion progress

**Implementation approach:**

1. **Word Tap Detection (R003):**
   - Replace `dangerous_inner_html` with proper markdown parsing using `pulldown-cmark` crate
   - Wrap each word in a `<span>` with `onclick` handler that captures the word text
   - On tap: extract word + sentence context using `WordExtractor::extract_sentence()`
   - Save to database via `db.create_word()` with `definition: None`, `ai_generated: false`
   - Show toast notification "Word saved!" (definition shows "Coming soon" per D007)

2. **Progress Persistence:**
   - Add `last_read_page` field to `processing_progress` table (or use `last_processed_page`)
   - Auto-save on scroll: debounce scroll events (500ms), update DB with current page
   - On mount: load last position, scroll to that page using `element.scroll_into_view()`
   - Display progress as "Page X of Y" in header (already partially implemented)

3. **Markdown Rendering:**
   - Add `pulldown-cmark` and `pulldown-cmark-to-cmark` dependencies
   - Create `render_markdown_to_html()` function that parses markdown and generates semantic HTML
   - Support headers, paragraphs, bold, italic, lists, line breaks
   - Each paragraph's words wrapped in clickable spans for word tap

4. **Font Size Persistence:**
   - Store user's font size preference in database or localStorage
   - Load on reader mount, apply to container style

**Why this approach:**
- Minimal rewrite risk — existing conversion pipeline works (S01 verified)
- Word tap is the only new interaction pattern; can be isolated to reader component
- Database schema already supports all required fields
- Progress tracking table exists; just needs scroll position integration

## Don't Hand-Roll

| Problem | Existing Solution | Why Use It |
|---------|------------------|------------|
| Markdown parsing | `pulldown-cmark` crate | CommonMark-compliant, event-based parser (no intermediate AST), widely used in Rust ecosystem |
| HTML generation from markdown | Custom `render_markdown()` in reader.rs | Current implementation is fragile (simple string replace); pulldown-cmark provides proper tokenization |
| Word extraction from text | `WordExtractor` in `src/core/vocab.rs` | Already has `extract_sentence()` method; supports English/Japanese placeholder |
| PDF rendering | `hayro` crate in `src/core/pdf.rs` | Pure Rust, no native dependencies, already integrated and tested |
| Progress tracking | `processing_progress` table in DB | Schema exists with `last_processed_page`, `status`, `updated_at` fields |
| Debounced scroll save | Custom `use_effect` with timeout | Dioxus signals + `spawn(async)` pattern already used throughout codebase |

## Existing Code and Patterns

- `src/ui/reader.rs` — **Primary component to extend**. Already has `ReaderBookView` with font size control, continuous scroll, conversion trigger, page jump modal. Key functions: `render_markdown()` (needs replacement), scroll handler with page estimation, conversion flow with `PdfConversionService`.
- `src/core/pdf.rs` — **PDF conversion pipeline**. `PdfConversionService::convert_pdf()` handles batch rendering + OCR with progress callbacks. Uses `PdfProcessor::render_pages_batch()` with rayon parallelism (3 threads). Already tested in S01.
- `src/core/db.rs` — **Database operations**. Key methods: `create_progress()`, `update_progress()`, `get_progress()`, `create_word()`, `get_words_by_book()`, `get_page_count()`. Schema includes `processing_progress`, `words`, `book_pages` tables.
- `src/core/vocab.rs` — **Word extraction**. `WordExtractor::extract_sentence()` finds sentence containing a word. `extract_words()` splits English text (Japanese requires lindera). Use for extracting example sentences when user taps a word.
- `src/core/ocr/engine.rs` — **OCR trait**. `process_pages_parallel()` method used by `PdfConversionService`. Default implementation processes sequentially; tract engine overrides with parallel processing.
- `src/ui/components.rs` — **Shared UI components**. `ConversionProgressDisplay`, `PageJumpModal`, `LoadingSpinner` already used in reader. Reuse for consistency.
- `src/ui/library.rs` — **Book card with progress**. `BookCard` component shows conversion progress bar and "Convert" button. Navigates to `Route::ReaderBook { book_id }` on click.
- `src/core/storage.rs` — **File storage**. `save_page_image()` stores images in `pages/{book_id}/` directory. Used by OCR pipeline to save rendered page images before processing.

**Pattern to follow:** The camera page (`src/ui/camera.rs`) has OCR initialization with loading state, disabled button until ready, and error handling. Replicate this pattern for PDF conversion: show loading indicator during conversion, disable UI elements, handle errors gracefully.

**Pattern to avoid:** The current `render_markdown()` in reader.rs uses naive string replacement (`.replace("\n# ", "\n<h1>")`). This breaks with nested markdown and doesn't handle edge cases. Replace with proper parser.

## Constraints

- **Dioxus 0.7.3 limitations:** No native `onclick` for inline `<span>` elements in `dangerous_inner_html`. Must use proper component-based rendering or JS interop for word tap.
- **Android WebView:** Word tap requires touch event handling. Dioxus mobile uses WebView; ensure click handlers work on touch devices.
- **Memory constraints (Moto G66j 5G):** Rendering all pages at once with large font sizes may stress mid-range device. Consider virtual scrolling or lazy loading for books with 100+ pages.
- **Model bundling (D003):** NDLOCR models already bundled (147MB in `assets/models/ndlocr/`). PDF conversion uses these models; ensure they load on first inference.
- **Java 17 target (D008):** Patch script already configured for Java 17. No additional build configuration needed.
- **tract-onnx runtime (D001):** OCR uses tract-onnx, not ort-mobile. Consistent with M001 tests (92 passing).

## Common Pitfalls

- **`dangerous_inner_html` event handlers:** Dioxus/Webview doesn't support `onclick` on dynamically injected HTML. **Solution:** Use component-based rendering where each word is a `<button>` or `<span>` with Dioxus event handler, or use JS interop to attach listeners after render.
- **Scroll position calculation:** Current heuristic (`scroll_y / page_height`) assumes uniform page heights. OCR output varies in length. **Solution:** Use `IntersectionObserver` or measure actual page element positions.
- **Progress save thrashing:** Saving on every scroll event floods DB. **Solution:** Debounce with 500ms timeout, save only when page changes.
- **Word boundary detection:** Simple split on whitespace breaks with punctuation ("word," vs "word"). **Solution:** Use regex or `WordExtractor` with proper tokenization.
- **Markdown injection:** User-generated OCR text could contain malicious HTML. **Solution:** Sanitize output with `ammonia` crate before rendering.
- **Font size extremes:** 12px may be unreadable on small screens; 32px causes excessive scrolling. **Solution:** Respect user choice but add warning tooltip at extremes.

## Open Risks

- **Word tap performance:** Wrapping every word in a clickable component creates hundreds/thousands of DOM nodes for long pages. May cause sluggish scrolling on Moto G66j 5G. **Mitigation:** Benchmark with 10-page book; consider lazy rendering or only enabling word tap on long-press.
- **PDF conversion time:** Large PDFs (50+ pages) may take minutes to convert on mid-range device. User may abandon flow. **Mitigation:** Show estimated time remaining, allow background conversion with notification.
- **Last-read position drift:** If user scrolls quickly, auto-save may capture wrong page. **Mitigation:** Use scroll-end detection (no scroll for 1s) before saving.
- **Cross-platform file paths:** Android uses content URIs, not file paths. `PdfProcessor::import_pdf()` assumes file paths. **Mitigation:** Test with Android file picker; may need JNI bridge for content URI → file path conversion.

## Skills Discovered

| Technology | Skill | Status |
|------------|-------|--------|
| Dioxus Mobile | `dioxus-mobile` for Android | Already configured in Dioxus.toml |
| hayro PDF | Pure Rust PDF renderer | Integrated in `src/core/pdf.rs` |
| pulldown-cmark | CommonMark parser | Not yet installed (recommend adding) |
| lindera | Japanese morphological analyzer | Referenced in `src/core/vocab.rs`, not yet enabled |
| tract-onnx | ONNX inference runtime | Integrated for NDLOCR (92 tests passing) |

**Recommended skill installation:** None required for S03. Existing Dioxus + hayro + tract setup is sufficient. Consider installing `frontend-design` skill for UI polish (word tap highlight animations, progress indicators).

## Sources

- **Dioxus event handling** — `dangerous_inner_html` doesn't support event handlers on child elements (source: [Dioxus docs](https://dioxuslabs.com/learn/0.7/cookbook/dangerous_inner_html))
- **pulldown-cmark** — Event-based CommonMark parser (source: [pulldown-cmark crate](https://docs.rs/pulldown-cmark/latest/pulldown_cmark/))
- **hayro PDF renderer** — Pure Rust PDF 1.7 renderer (source: [hayro GitHub](https://github.com/LaurenzV/hayro))
- **S01 Summary** — APK build, database persistence, model bundling verified (source: `.gsd/milestones/M002-dbrk2n/slices/S01/S01-SUMMARY.md`)
- **M002 Roadmap** — S03 depends on S01, provides PDF reflow for S04 word collection (source: `.gsd/milestones/M002-dbrk2n/M002-dbrk2n-ROADMAP.md`)
- **Requirements R002/R003** — PDF reflow reader with progress tracking, word tap with placeholder definition (source: `.gsd/REQUIREMENTS.md`)

---

## Implementation Checklist (for Planning)

**Core Features:**
- [ ] Replace `render_markdown()` with `pulldown-cmark` parser
- [ ] Add word tap handler: extract word + sentence, save to DB
- [ ] Persist last-read page to `processing_progress` table
- [ ] Auto-save scroll position on scroll (debounced)
- [ ] Restore last-read position on mount
- [ ] Persist font size preference

**UI Polish:**
- [ ] Show "Word saved!" toast on tap
- [ ] Highlight tapped word briefly (visual feedback)
- [ ] Show "Definition coming soon" placeholder in word detail
- [ ] Add "Tap words to save" hint in empty vocabulary state

**Performance:**
- [ ] Benchmark word tap with 1000+ words per page
- [ ] Test scroll performance with 50+ page book
- [ ] Verify conversion progress on Moto G66j 5G (when device connected)

**Verification:**
- [ ] Import PDF → convert → read → close → reopen → verify last position restored
- [ ] Tap 3+ words → save → check `words` table → restart app → verify persistence
- [ ] Change font size → navigate away → return → verify size preserved
- [ ] Scroll through 10-page book → verify progress saves correctly
