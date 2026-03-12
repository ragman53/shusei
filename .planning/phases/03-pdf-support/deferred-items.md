# Deferred Items - Plan 03-04

## Blocked by Pre-existing Technical Debt

The following items from Plan 03-04 could not be completed due to pre-existing pdfium-render v0.8 API incompatibilities:

### Task 1: Wire PDF import to database with metadata review
**Status:** BLOCKED  
**Reason:** `PdfProcessor::import_pdf()` depends on pdfium-render v0.8 API which has breaking changes from v0.7

**Specific Issues:**
- `Pdfium::bind_to_system_library()` removed in v0.8
- `PdfDocument` now requires lifetime parameter
- `PdfMetadata` API changed (methods like `title()`, `author()` no longer available)
- `PdfRenderConfig::set_render_flags()` replaced with new API
- `PdfPageIndex` now uses `u16` instead of `usize`

**Files Affected:**
- `src/core/pdf.rs` - requires extensive refactoring
- `src/ui/library.rs` - cannot call PdfProcessor until fixed

### Task 2: Add Convert button to trigger OCR processing
**Status:** BLOCKED  
**Reason:** `PdfConversionService::convert_pdf()` has compilation errors due to:
- progress_cb callback ownership issues
- pdfium-render API incompatibilities from Task 1

### Task 4: Wire conversion service to UI and test end-to-end
**Status:** BLOCKED  
**Reason:** Depends on Tasks 1 and 2 being resolved

---

## Completed

### Task 3: Create real-time progress display component
**Status:** NOT STARTED (blocked by pdf.rs compilation)

The `ConversionProgressDisplay` component requires `ConversionProgress` and `ConversionStage` types from `src/core/pdf.rs`, which doesn't compile.

---

## Resolution Required

This plan requires **pdfium-render v0.8 API migration** to be completed first. This is a known issue documented in STATE.md under "Active TODOs":
- "Update pdfium-render integration for v0.8 API changes"

**Recommended Next Steps:**
1. Create dedicated plan for pdfium-render v0.8 migration
2. Refactor `src/core/pdf.rs` to use v0.8 API
3. Re-execute this plan once pdf.rs compiles successfully

**Reference:** pdfium-render v0.8 documentation: https://docs.rs/pdfium-render/0.8.37/pdfium_render/
