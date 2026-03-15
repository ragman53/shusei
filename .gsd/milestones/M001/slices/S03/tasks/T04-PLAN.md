# T04: 03-pdf-support 04

**Slice:** S03 — **Milestone:** M001

## Description

Wire PDF import flow to database, add conversion trigger button, and implement real-time progress display UI.

Purpose: Complete the PDF import and conversion user experience by connecting backend services to UI actions and providing visible feedback during processing.

Output: Working PDF import with database persistence, Convert button, real-time progress UI with stage indicators.

## Must-Haves

- [ ] "User can select a PDF file and see it appear in library"
- [ ] "User can tap Convert button to start OCR processing"
- [ ] "User sees real-time progress with stages (Rendering → OCR → Complete)"
- [ ] "Book record created with is_pdf=true after import"

## Files

- `src/ui/library.rs`
- `src/ui/reader.rs`
- `src/ui/components.rs`
- `src/core/pdf.rs`
