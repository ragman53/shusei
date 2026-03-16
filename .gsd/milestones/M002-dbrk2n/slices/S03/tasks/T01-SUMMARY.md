---
id: T01
parent: S03
milestone: M002-dbrk2n
provides:
  - pulldown-cmark markdown rendering with word-level span wrapping
  - HTML escape utility for XSS prevention
  - Unit tests for markdown rendering (headers, bold, italic, word spans, HTML escaping)
key_files:
  - src/ui/reader.rs
  - .gsd/milestones/M002-dbrk2n/slices/S03/S03-PLAN.md
key_decisions:
  - Use pulldown-cmark v0.12 for CommonMark-compliant parsing (already in Cargo.toml)
  - Wrap each word in <span data-word="clean-word"> for tap detection
  - Preserve original word with punctuation in span content, use cleaned version for data attribute
  - Keep deprecated render_markdown() function for backward compatibility (marked with #[deprecated])
patterns_established:
  - Semantic HTML generation with word-level granularity for interactive tap handling
  - Event-driven markdown parsing using pulldown-cmark Parser
  - Comprehensive unit testing for markdown rendering edge cases
observability_surfaces:
  - cargo test --lib reader::tests for automated verification
  - Browser dev tools inspection of data-word attributes on rendered spans
  - HTML structure validation for proper tag nesting
duration: 1h
verification_result: passed
completed_at: 2026-03-16
# Set blocker_discovered: true only if execution revealed the remaining slice plan
# is fundamentally invalid (wrong API, missing capability, architectural mismatch).
# Do NOT set true for ordinary bugs, minor deviations, or fixable issues.
blocker_discovered: false
---

# T01: Replace Markdown Renderer with pulldown-cmark

**Replaced fragile string-based markdown renderer with pulldown-cmark parser generating semantic HTML with word-level spans for tap detection.**

## What Happened

1. **Added pulldown-cmark imports** to `src/ui/reader.rs` - Parser, Event, Tag, TagEnd, CodeBlockKind, Options, HeadingLevel

2. **Created `render_markdown_to_html()` function** with:
   - pulldown-cmark Parser with extensions (strikethrough, tables)
   - Event handling for: Heading (H1-H6), Paragraph, Strong, Emphasis, Strikethrough, CodeBlock, BlockQuote, List (ordered/unordered), Item, Link, Image, Table, Rule, HardBreak, SoftBreak, TaskListMarker
   - Word-level span wrapping: `<span data-word="clean-word">original-word</span>` for tap detection
   - HTML escaping utility function to prevent XSS attacks
   - Graceful handling of unsupported events (math, inline HTML)

3. **Updated call site** in `ReaderBookView` to use `render_markdown_to_html()` instead of deprecated `render_markdown()`

4. **Added 8 unit tests** covering:
   - Header rendering (H1-H3)
   - Paragraph rendering with word spans
   - Bold text (`**text**`)
   - Italic text (`*text*`)
   - Word span generation with data-word attributes
   - Line breaks (`<br/>`)
   - Horizontal rules (`<hr/>`)
   - HTML escaping for XSS prevention

5. **Fixed pulldown-cmark API mismatches**:
   - `Tag::Heading` uses `HeadingLevel` enum (H1-H6), not integers
   - `Tag::List` takes `Option<u64>` for ordered list start number
   - `Tag::BlockQuote` takes `Option<BlockQuoteKind>` parameter
   - `TagEnd::Heading` is a tuple variant, not struct

6. **Added failure-path verification** to S03-PLAN.md for inspecting data-word attributes in browser dev tools

## Verification

- ✅ `cargo check` — Compiles without errors
- ✅ `cargo test --lib reader::tests` — All 8 tests pass:
  - test_render_headers
  - test_render_paragraph
  - test_render_bold
  - test_render_italic
  - test_render_word_spans
  - test_render_line_break
  - test_render_horizontal_rule
  - test_render_html_escape
- ✅ Verified `data-word` attributes present on rendered spans (via test assertions)
- ✅ Verified proper HTML tag nesting (tests confirm opening/closing tags match)

## Diagnostics

- **How to inspect:** Run `cargo test --lib reader::tests -- --nocapture` to see test output
- **Browser inspection:** Use browser dev tools to examine rendered HTML structure; verify `<span data-word="...">` attributes on words
- **Failure visibility:** Parse errors would cause compilation failures; unsupported markdown events are silently ignored (graceful degradation)
- **HTML structure validation:** Tests verify opening/closing tag pairs for headers, paragraphs, strong, emphasis

## Deviations

- Kept deprecated `render_markdown()` function instead of removing it (marked with `#[deprecated]` attribute for backward compatibility during transition)
- Did not add `ammonia` crate sanitization yet (marked as optional in task plan; will add in future task if OCR text security becomes a concern)

## Known Issues

- None - all must-haves met

## Files Created/Modified

- `src/ui/reader.rs` — Added `render_markdown_to_html()` function with pulldown-cmark integration, `html_escape()` utility, and 8 unit tests
- `.gsd/milestones/M002-dbrk2n/slices/S03/S03-PLAN.md` — Marked T01 as complete, added failure-path verification step
