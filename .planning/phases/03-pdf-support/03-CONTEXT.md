# Phase 03: PDF Support - Context

**Gathered:** 2026-03-12
**Status:** Ready for planning

<domain>
## Phase Boundary

Import PDFs, convert to readable text via OCR, and display with reflow. Users can select PDF files from device storage, process pages through OCR with visible progress, and read converted content with adjustable font size and smooth navigation.

**In scope:**
- PDF file selection via system file picker
- PDF metadata extraction and book creation
- Batch page processing with progress feedback
- OCR conversion using Phase 2 infrastructure
- Reflow text display with font size control
- Library integration with visual distinction
- Resume processing after interruption

**Out of scope:**
- Annotations on PDF pages (Phase 4)
- Voice memos linked to PDF pages (Phase 5)
- Search within PDF content (future phase)
- Bookmarking pages (Phase 4)

</domain>

<decisions>
## Implementation Decisions

### PDF Import Flow
- **System file picker** — Standard Android file picker for PDF selection (familiar, works with all file managers)
- **Auto-extract metadata** — Pull title/author from PDF metadata, show edit form for user review before saving
- **Copy to app directory** — Duplicate PDF into Shusei's data folder (safe, isolated storage, won't break if original deleted)
- **Navigate to book detail** — After import, show book detail page where user manually starts conversion (follows Phase 2 pattern)

### Processing Strategy
- **Batch processing** — Process in chunks of 10 pages at a time (balance between progress feedback and efficiency)
- **Stage-based progress** — Show "Rendering pages → Running OCR → Saving..." instead of just percentage
- **Resume on interrupt** — Track progress, continue from last page if app is backgrounded or closed
- **10 pages per batch** — Medium batch size for good balance of feedback frequency and processing efficiency

### Reflow Display
- **Continuous scroll** — One long scrollable document (natural for reading, easy to skim)
- **Slider font control** — Continuous range (e.g., 12px-32px) for precise font size adjustment
- **Page number jumps** — Tap to enter page number and jump to that section (critical for long PDFs)
- **Progress + font size** — Show reading progress (page X of Y) and font slider in reading view

### OCR Integration
- **Immediate OCR** — Run OCR immediately after rendering each page (same pipeline as Phase 2)
- **No quality warnings** — Skip blur/darkness detection for PDFs (assume PDFs are good quality)
- **Auto-retry enabled** — Retry automatically on low OCR confidence (same as Phase 2)
- **Per-page markdown files** — Store OCR results as individual markdown files per page (follows Phase 2 pattern)
- **Retry 3 times then skip** — On OCR failure, retry 3 times then mark page as failed and continue
- **Auto-detect per page** — NDLOCR auto-detects language (Japanese/English) for each page
- **Parallel OCR** — Process 2-3 pages concurrently for faster conversion
- **Markdown format** — Store OCR output in markdown format to preserve headings, lists, basic structure

### Library Integration
- **Unified list with badge** — PDFs and physical books in same list, PDF icon/badge for visual distinction
- **Conversion progress** — Show "X/Y pages converted" in library card (indicates OCR processing status)
- **Filter toggle** — Add filter: "All" / "PDFs only" / "Physical only"
- **Open with available pages** — When tapping a processing PDF, show converted pages immediately, indicate which are still processing

### OpenCode's Discretion
- Exact batch processing interval timing
- Specific font size range min/max values
- Progress indicator animation details
- PDF badge/icon design
- Error message wording for failed pages

</decisions>

<code_context>
## Existing Code Insights

### Reusable Assets
- **PDF processor** (`src/core/pdf.rs`) — Already implemented with pdfium-render, can render pages to images
- **OCR engine** (`src/core/ocr/`) — NdlocrEngine trait and implementation from Phase 2, supports markdown output
- **Reader UI** (`src/ui/reader.rs`) — Basic reader shell with import button placeholder
- **Database layer** (`src/core/db.rs`) — Books table exists, may need extension for PDF-specific fields
- **Storage service** (`src/core/storage.rs`) — `save_image()` method with prefix support for organizing PDF page images

### Established Patterns
- **File storage for images** — Phase 1/2 pattern: images saved to filesystem, paths stored in SQLite
- **WAL mode** — Concurrent database reads supported (critical for background processing + UI)
- **Async/await** — All I/O operations use tokio async runtime
- **Progress callbacks** — Phase 2 established pattern for progress reporting during long operations
- **Parallel task support** — Phase 2: background processing with tokio spawn

### Integration Points
- **PDF → OCR pipeline** — Connect `PdfProcessor::render_all_pages()` to `NdlocrEngine::process_image()`
- **Book creation** — Extend Phase 1 book add flow to support PDF imports with metadata extraction
- **Library display** — Extend Phase 1 library UI to show PDF badge and conversion progress
- **Reader view** — Extend `ReaderPage` component with reflow text display and font controls

</code_context>

<specifics>
## Specific Ideas

- "Copy to app directory" — User's original PDF remains untouched, we work with our own copy (safe, but uses storage)
- Stage-based progress feels more informative than percentage — user knows what's happening, not just how far along
- Continuous scroll is preferred over pagination — more natural for digital reading, easier to skim content
- Auto-detect language per page handles mixed-language PDFs (e.g., Japanese textbook with English examples)
- Opening partially-processed PDFs lets users start reading immediately while background processing continues

</specifics>

<deferred>
## Deferred Ideas

- **Search within PDF content** — Full-text search across OCR'd pages — mentioned as useful, belongs in separate phase after core reading is stable
- **Bookmarking pages** — Mark specific pages for quick return — belongs in Phase 4 (Annotation Foundation)
- **Annotations on PDF pages** — Add notes/highlights to specific pages — Phase 4
- **Export/share converted text** — Export OCR results as text/markdown file — future enhancement
- **Table of Contents detection** — Auto-extract and navigate by chapters — requires PDF structure analysis, future optimization
- **Brightness/contrast controls** — Reading comfort features — mentioned but deferred to performance phase

</deferred>

---

*Phase: 03-pdf-support*
*Context gathered: 2026-03-12*
