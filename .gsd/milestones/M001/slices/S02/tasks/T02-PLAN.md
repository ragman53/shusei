# T02: 02-paper-book-capture 02

**Slice:** S02 — **Milestone:** M001

## Description

Integrate camera UI with OCR engine and database persistence

Purpose: Complete the end-to-end camera → OCR → save workflow, enabling users to capture physical book pages and store them with extracted text

Output: Working camera capture flow with OCR integration and database persistence

## Must-Haves

- [ ] "User can capture a book page with camera"
- [ ] "OCR text is saved to database linked to the page image"
- [ ] "User can view both the captured image and extracted text together"

## Files

- `src/core/db.rs`
- `src/ui/camera.rs`
- `src/app.rs`
