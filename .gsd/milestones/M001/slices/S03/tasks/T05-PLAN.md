# T05: 03-pdf-support 05

**Slice:** S03 — **Milestone:** M001

## Description

Complete OCR pipeline implementation with ONNX model loading and text extraction inference.

Purpose: Enable actual text extraction from PDF page images instead of returning empty results.

Output: Working OCR engine that loads NDLOCR-Lite models and extracts text from images with markdown formatting.

## Must-Haves

- [ ] "OCR extracts actual text from PDF page images"
- [ ] "Extracted text saved to database with markdown formatting"
- [ ] "Reader displays real OCR content, not empty placeholders"

## Files

- `src/core/ocr/engine.rs`
- `src/core/ocr/mod.rs`
- `Cargo.toml`
