# T03: 02-paper-book-capture 03

**Slice:** S02 — **Milestone:** M001

## Description

Add quality feedback, parallel processing, and auto-retry logic

Purpose: Improve user experience with quality warnings, non-blocking OCR, and automatic retry on low confidence

Output: Enhanced camera UI with quality feedback and robust OCR processing

## Must-Haves

- [ ] "User receives quality feedback before OCR (blur/darkness warning)"
- [ ] "OCR runs in parallel without blocking UI"
- [ ] "Failed OCR auto-retries with different parameters"

## Files

- `src/ui/camera.rs`
- `src/core/ocr/postprocess.rs`
- `src/ui/components.rs`
