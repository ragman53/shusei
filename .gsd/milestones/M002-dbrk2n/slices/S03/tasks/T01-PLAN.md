# T01: Replace Markdown Renderer with pulldown-cmark

**Slice:** S03 — PDF Reflow Reader
**Milestone:** M002-dbrk2n

## Description

Replace the fragile `render_markdown()` function (which uses naive string replacement) with proper CommonMark-compliant parsing using the `pulldown-cmark` crate. This provides the foundation for word tap detection by generating semantic HTML structure where each word can be wrapped in a clickable span.

## Steps

1. Add `pulldown-cmark` dependency to `Cargo.toml` (version 0.10 or later)
2. Create `render_markdown_to_html()` function in `src/ui/reader.rs` that:
   - Uses `pulldown-cmark` Parser to tokenize markdown
   - Handles events: Start, End, Text, Heading, Paragraph, Strong, Emphasis, Rule, LineBreak
   - Generates semantic HTML with proper tag nesting
   - Wraps each word in `<span data-word="word-text">word</span>` for word tap detection
3. Replace call to `render_markdown()` in `ReaderBookView` with new function
4. Add sanitization using `ammonia` crate (optional, for security if OCR text could contain malicious HTML)
5. Test with various markdown inputs: headers, bold, italic, lists, line breaks

## Must-Haves

- [ ] `pulldown-cmark` added to `Cargo.toml` with version constraint
- [ ] `render_markdown_to_html()` function handles CommonMark events correctly
- [ ] Each word wrapped in `<span data-word="...">` for tap detection
- [ ] Proper HTML tag nesting (no unclosed tags)
- [ ] Support for headers (h1-h3), paragraphs, bold, italic, line breaks

## Verification

- `cargo check` — Compiles without errors
- `cargo test --lib reader::test_markdown_rendering` — Renders test markdown correctly
- Manual test: Import PDF with markdown content → verify headers, bold, italic render correctly

## Observability Impact

- **Signals added/changed:** 
  - Log warning if markdown parsing fails (fallback to plain text)
  - Log word count per page for performance monitoring
- **How a future agent inspects this:** 
  - Check rendered HTML structure in browser dev tools
  - Verify `data-word` attributes present on spans
  - Run `cargo test --lib reader::` for automated checks
- **Failure state exposed:** 
  - Parse errors logged with markdown snippet that failed
  - Fallback renders as plain text (no formatting, no word tap)

## Inputs

- `src/ui/reader.rs` — Existing `render_markdown()` function to replace
- `S03-PLAN.md` — Slice requirements and verification criteria

## Expected Output

- `src/ui/reader.rs` — New `render_markdown_to_html()` function with pulldown-cmark integration
- `Cargo.toml` — `pulldown-cmark` dependency added
- Test function `test_markdown_rendering()` verifying correct HTML output
