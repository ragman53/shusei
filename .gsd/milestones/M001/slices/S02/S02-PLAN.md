# S02: Paper Book Capture

**Goal:** Implement OCR engine and image preprocessing pipeline

Purpose: Enable the core camera → OCR workflow by implementing the NDLOCR-Lite engine and ensuring images are properly downscaled for memory efficiency

Output: Working OCR engine with image preprocessing, ready for integration with camera UI
**Demo:** Implement OCR engine and image preprocessing pipeline

Purpose: Enable the core camera → OCR workflow by implementing the NDLOCR-Lite engine and ensuring images are properly downscaled for memory efficiency

Output: Working OCR engine with image preprocessing, ready for integration with camera UI

## Must-Haves


## Tasks

- [x] **T01: 02-paper-book-capture 01**
  - Implement OCR engine and image preprocessing pipeline

Purpose: Enable the core camera → OCR workflow by implementing the NDLOCR-Lite engine and ensuring images are properly downscaled for memory efficiency

Output: Working OCR engine with image preprocessing, ready for integration with camera UI
- [x] **T02: 02-paper-book-capture 02**
  - Integrate camera UI with OCR engine and database persistence

Purpose: Complete the end-to-end camera → OCR → save workflow, enabling users to capture physical book pages and store them with extracted text

Output: Working camera capture flow with OCR integration and database persistence
- [x] **T03: 02-paper-book-capture 03**
  - Add quality feedback, parallel processing, and auto-retry logic

Purpose: Improve user experience with quality warnings, non-blocking OCR, and automatic retry on low confidence

Output: Enhanced camera UI with quality feedback and robust OCR processing

## Files Likely Touched

- `src/core/ocr/engine.rs`
- `src/core/ocr/preprocess.rs`
- `src/core/storage.rs`
- `src/core/db.rs`
- `src/ui/camera.rs`
- `src/app.rs`
- `src/ui/camera.rs`
- `src/core/ocr/postprocess.rs`
- `src/ui/components.rs`
