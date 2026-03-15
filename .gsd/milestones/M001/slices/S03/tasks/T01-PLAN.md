# T01: 03-pdf-support 01

**Slice:** S03 — **Milestone:** M001

## Description

Implement PDF import flow with file picker, metadata extraction, and library integration.

Purpose: Enable users to import PDFs from device storage, extract metadata for review, and persist PDFs in the library with visual distinction from physical books.

Output: Working PDF import flow, extended Book model with PDF support, library UI with PDF badges.

## Must-Haves

- [ ] "User can select a PDF file from device storage"
- [ ] "PDF metadata is extracted and shown for review"
- [ ] "PDF is copied to app directory and book record created"
- [ ] "Library shows PDF badge and page count"

## Files

- `src/core/pdf.rs`
- `src/core/db.rs`
- `src/ui/library.rs`
