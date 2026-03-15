# T02: 03-pdf-support 02

**Slice:** S03 — **Milestone:** M001

## Description

Implement batch OCR processing pipeline with progress feedback and resume support.

Purpose: Convert PDF pages to text via OCR efficiently with visible progress, parallel processing, and ability to resume after interruption.

Output: Batch processing service, progress tracking, parallel OCR execution.

## Must-Haves

- [ ] "User can start OCR conversion on imported PDF"
- [ ] "Progress shows stage (Rendering → OCR → Saving) and page numbers"
- [ ] "Processing resumes from last page after interruption"
- [ ] "Large PDFs (100+ pages) process without crashing"

## Files

- `src/core/pdf.rs`
- `src/core/ocr/engine.rs`
- `src/core/db.rs`
