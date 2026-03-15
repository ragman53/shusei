---
phase: 03-pdf-support
plan: 04b
type: fix
wave: 1
depends_on: []
files_modified: [src/core/pdf.rs, Cargo.toml]
autonomous: true
requirements: [PDF-01, PDF-02]
gap_closure: true
priority: blocker

must_haves:
  truths:
    - "src/core/pdf.rs compiles without errors"
    - "PdfProcessor::new() initializes with v0.8 API"
    - "PdfProcessor::open() loads PDF files"
    - "PdfProcessor::render_page() returns page images"
  artifacts:
    - path: "src/core/pdf.rs"
      provides: "PDF processing with pdfium-render v0.8 API"
      pattern: "Pdfium::new|load_pdf_from_file|render_with_config"
  key_links:
    - from: "src/ui/library.rs"
      to: "src/core/pdf.rs"
      via: "PdfProcessor usage"
      pattern: "PdfProcessor::new"
---

<objective>
Fix pdfium-render v0.8 API incompatibilities in src/core/pdf.rs.

Purpose: Unblock Plan 03-04 by migrating PDF processing module from v0.7 to v0.8 API.

Output: Compiling pdf.rs module with working PDF open, render, and metadata extraction.
</objective>

<tasks>

## Task 1: Update pdfium-render API calls

**Input:** src/core/pdf.rs (628 lines, 19+ compilation errors)

**Do:**
1. Fix `Pdfium::new()` initialization - v0.8 uses different binding approach
2. Fix `PdfDocument` lifetime parameters
3. Fix `PdfMetadata` extraction - v0.8 has different API
4. Fix page rendering config - `PdfRenderConfig` API changes
5. Fix page indexing - v0.8 uses u16 instead of usize

**Output:** Compiling pdf.rs with no errors

**Test:** `cargo check --package shusei` passes

---

## Task 2: Verify PDF operations

**Input:** Working pdf.rs module

**Do:**
1. Test `PdfProcessor::new()` - creates instance
2. Test `PdfProcessor::open()` - loads test PDF
3. Test `PdfProcessor::render_page()` - returns image bytes
4. Test metadata extraction - gets title/author if available

**Output:** All PDF operations working

**Test:** `cargo test --package shusei pdf` - tests pass

---

## Task 3: Update dependent code

**Input:** Any code that uses pdf.rs with old API assumptions

**Do:**
1. Check src/ui/library.rs for pdf usage
2. Check src/ui/reader.rs for pdf usage
3. Update any broken calls from v0.7 to v0.8

**Output:** All dependent code compiles

**Test:** `cargo check` passes for entire project

---

## Task 4: Documentation

**Input:** Completed fixes

**Do:**
1. Update module docs with v0.8 API notes
2. Add comments for any non-obvious v0.8 workarounds
3. Create SUMMARY.md with migration notes

**Output:** Documented migration for future reference

**Test:** SUMMARY.md created with API migration details

</tasks>

<verification>

## Must-Have Truths

- [ ] `cargo check` passes with no pdf.rs errors
- [ ] `PdfProcessor::new()` works with v0.8
- [ ] `PdfProcessor::open()` loads PDFs
- [ ] `PdfProcessor::render_page()` returns valid images

## Files to Verify

- src/core/pdf.rs - compiles, no errors
- src/ui/library.rs - compiles if uses pdf.rs
- src/ui/reader.rs - compiles if uses pdf.rs

</verification>
