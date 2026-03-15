# T01: 02-paper-book-capture 01

**Slice:** S02 — **Milestone:** M001

## Description

Implement OCR engine and image preprocessing pipeline

Purpose: Enable the core camera → OCR workflow by implementing the NDLOCR-Lite engine and ensuring images are properly downscaled for memory efficiency

Output: Working OCR engine with image preprocessing, ready for integration with camera UI

## Must-Haves

- [ ] "Captured images are downscaled to 2MP or less before processing"
- [ ] "OCR engine processes images and returns extracted text"
- [ ] "OCR processing completes within reasonable time (< 5s on mid-range device)"

## Files

- `src/core/ocr/engine.rs`
- `src/core/ocr/preprocess.rs`
- `src/core/storage.rs`
