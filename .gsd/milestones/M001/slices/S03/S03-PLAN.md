# S03: Pdf Support

**Goal:** Implement PDF import flow with file picker, metadata extraction, and library integration.
**Demo:** Implement PDF import flow with file picker, metadata extraction, and library integration.

## Must-Haves


## Tasks

- [x] **T01: 03-pdf-support 01**
  - Implement PDF import flow with file picker, metadata extraction, and library integration.

Purpose: Enable users to import PDFs from device storage, extract metadata for review, and persist PDFs in the library with visual distinction from physical books.

Output: Working PDF import flow, extended Book model with PDF support, library UI with PDF badges.
- [x] **T02: 03-pdf-support 02**
  - Implement batch OCR processing pipeline with progress feedback and resume support.

Purpose: Convert PDF pages to text via OCR efficiently with visible progress, parallel processing, and ability to resume after interruption.

Output: Batch processing service, progress tracking, parallel OCR execution.
- [x] **T03: 03-pdf-support 03**
  - Build reflow reading UI with font size controls, page navigation, and progress display.

Purpose: Provide comfortable reading experience for converted PDF content with user-controlled typography and easy navigation.

Output: Reflow reader component, font size slider, page jump modal, progress indicator.
- [x] **T04: 03-pdf-support 04**
  - Wire PDF import flow to database, add conversion trigger button, and implement real-time progress display UI.

Purpose: Complete the PDF import and conversion user experience by connecting backend services to UI actions and providing visible feedback during processing.

Output: Working PDF import with database persistence, Convert button, real-time progress UI with stage indicators.
- [x] **T05: 03-pdf-support 05**
  - Complete OCR pipeline implementation with ONNX model loading and text extraction inference.

Purpose: Enable actual text extraction from PDF page images instead of returning empty results.

Output: Working OCR engine that loads NDLOCR-Lite models and extracts text from images with markdown formatting.
- [x] **T06: Plan 06**
  - Complete OCR pipeline by bundling ONNX models and implementing postprocessing logic to extract actual text from model outputs.

Purpose: Fix the blocker preventing OCR from returning real text (currently returns empty vectors).

Output: Working OCR engine that loads bundled NDLOCR-Lite models and extracts text with markdown formatting.
- [x] **T07: Plan 07**
  - Verify large PDF processing (100+ pages) works without memory crashes on target devices.

Purpose: Confirm batch processing logic handles large files safely on low-RAM devices.

Output: Documented test results confirming stability with large PDFs.

## Files Likely Touched

- `src/core/pdf.rs`
- `src/core/db.rs`
- `src/ui/library.rs`
- `src/core/pdf.rs`
- `src/core/ocr/engine.rs`
- `src/core/db.rs`
- `src/ui/reader.rs`
- `src/ui/components.rs`
- `src/ui/library.rs`
- `src/ui/reader.rs`
- `src/ui/components.rs`
- `src/core/pdf.rs`
- `src/core/ocr/engine.rs`
- `src/core/ocr/mod.rs`
- `Cargo.toml`
