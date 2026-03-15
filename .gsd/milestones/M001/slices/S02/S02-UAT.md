---
id: S02-UAT
parent: S02
milestone: M001
---

# S02: Paper Book Capture — UAT

**Milestone:** M001
**Written:** 2026-03-15

## UAT Type

- UAT mode: artifact-driven
- Why this mode is sufficient: This slice delivers backend infrastructure (preprocessing, database schema, quality algorithms) without UI integration. All functionality is testable via unit tests and code inspection. UI integration deferred to next phase.

## Preconditions

1. Rust toolchain installed with Android targets
2. Project builds successfully: `cargo build --lib`
3. Test environment can link ort-sys (or tests skipped if linker error occurs)

## Smoke Test

**Verify library builds:**
```bash
cd /home/devuser/develop/shusei
cargo build --lib
```
**Expected:** `Finished dev profile [unoptimized + debuginfo]` with warnings only (no errors)

## Test Cases

### 1. Image Preprocessing - Downscaling

**Purpose:** Verify large images are downscaled to 2MP while maintaining aspect ratio

**Steps:**
1. Run preprocessing tests:
   ```bash
   cargo test --lib core::ocr::preprocess::tests::test_downscale_large_image -- --nocapture
   ```
2. Inspect test assertion: output dimensions should satisfy `width * height <= 2,000,000`

**Expected:** Test passes, 4MP input produces ~2MP output with correct aspect ratio

### 2. Image Preprocessing - Small Image Passthrough

**Purpose:** Verify small images pass through without modification

**Steps:**
1. Run test:
   ```bash
   cargo test --lib core::ocr::preprocess::tests::test_small_image_passes_through
   ```

**Expected:** Test passes, images under 2MP are not downscaled

### 3. Image Preprocessing - JPEG Output

**Purpose:** Verify output is JPEG format at 85% quality

**Steps:**
1. Run test:
   ```bash
   cargo test --lib core::ocr::preprocess::tests::test_preprocess_returns_jpeg
   ```

**Expected:** Test passes, output buffer starts with JPEG magic bytes (`0xFFD8`)

### 4. Storage - Page Image Directory Structure

**Purpose:** Verify images are saved in `pages/{book_id}/` structure

**Steps:**
1. Run test:
   ```bash
   cargo test --lib core::storage::tests::test_save_page_image_creates_book_directory
   ```
2. Check test creates directory matching pattern `pages/{book_id}/`

**Expected:** Test passes, directory created with correct structure

### 5. Storage - Relative Path Format

**Purpose:** Verify returned path is relative (not absolute)

**Steps:**
1. Run test:
   ```bash
   cargo test --lib core::storage::tests::test_save_page_image_returns_relative_path
   ```
2. Verify path format: `pages/{book_id}/{timestamp}_{uuid}.jpg`

**Expected:** Test passes, path does not start with `/`

### 6. Database - Save Page with OCR Results

**Purpose:** Verify page insertion with all required fields

**Steps:**
1. Run test:
   ```bash
   cargo test --lib core::db::tests::book_pages::test_save_page_inserts_and_returns_id
   ```

**Expected:** Test passes, returns non-zero row ID

### 7. Database - Retrieve Pages by Book

**Purpose:** Verify pages are retrieved sorted by page_number

**Steps:**
1. Run test:
   ```bash
   cargo test --lib core::db::tests::book_pages::test_get_pages_by_book_returns_sorted_pages
   ```
2. Verify returned vector is ordered by page_number ascending

**Expected:** Test passes, pages returned in correct order

### 8. Quality Detection - Blur Assessment

**Purpose:** Verify Laplacian variance correctly identifies blurry images

**Steps:**
1. Run test:
   ```bash
   cargo test --lib core::ocr::postprocess::tests::test_calculate_quality_score_good_image
   ```
2. Check quality score > 0.7 for sharp image

**Expected:** Test passes, sharp images score high

### 9. Quality Detection - Brightness Assessment

**Purpose:** Verify brightness analysis identifies optimal/dark/bright images

**Steps:**
1. Run test:
   ```bash
   cargo test --lib core::ocr::postprocess::tests::test_calculate_mean_brightness
   ```

**Expected:** Test passes, brightness calculated correctly (0-255 scale)

### 10. Retry Logic - Low Confidence Trigger

**Purpose:** Verify `should_retry()` returns true for low-confidence results

**Steps:**
1. Run test:
   ```bash
   cargo test --lib core::ocr::postprocess::tests::test_should_retry_low_confidence
   ```

**Expected:** Test passes, retry triggered when confidence < 0.5

## Edge Cases

### 1. Image at Exact 2MP Threshold

**Steps:**
1. Create test image with dimensions exactly at 2MP (e.g., 2000x1000)
2. Run preprocessing
3. Verify image passes through without downscaling

**Expected:** No downscaling applied at exact threshold

### 2. Extremely Dark Image (brightness < 50)

**Steps:**
1. Create dark test image (mean brightness ~30)
2. Call `calculate_quality_score()`
3. Verify quality score is low (< 0.4)

**Expected:** Quality score reflects poor lighting conditions

### 3. Duplicate Page Number Prevention

**Steps:**
1. Insert page with book_id="test", page_number=1
2. Attempt to insert second page with same book_id and page_number
3. Verify database constraint violation

**Expected:** UNIQUE constraint prevents duplicate

### 4. Non-existent Page Retrieval

**Steps:**
1. Call `get_page()` with ID that doesn't exist
2. Verify `None` is returned (not error)

**Expected:** `Ok(None)` returned gracefully

## Failure Signals

- **Build fails:** Linker errors with ort-sys indicate environment issue (not code defect)
- **Test failures:** Any test failure indicates regression in preprocessing/storage/database logic
- **Missing warnings:** Unused import warnings are acceptable; unused function warnings in core modules indicate incomplete integration

## Requirements Proved By This UAT

- **PAPER-01** - Image capture infrastructure via storage tests
- **PAPER-02** - Image downscaling via preprocess tests
- **PAPER-03** - OCR preprocessing via preprocess tests (full OCR pending)
- **PAPER-04** - Database persistence via book_pages CRUD tests

## Not Proven By This UAT

- **End-to-end camera flow** - UI integration not yet complete
- **Real OCR accuracy** - Requires ONNX model integration
- **Quality warning UX** - UI component not yet built
- **Parallel OCR performance** - tokio::spawn integration deferred
- **Page viewing experience** - Viewer component deferred

## Notes for Tester

**Test environment:** If linker errors occur with ort-sys, this is a known environment-specific issue. The library builds successfully; test failures due to linking are not code defects.

**Focus areas:** Backend algorithms are complete and tested. Next phase integrates these components into the camera UI flow.

**Quality thresholds:** All thresholds (blur variance < 100, brightness 50-200, confidence 0.5/0.3) are module-level constants for easy tuning based on real-world testing.

**File structure:** Storage creates `pages/{book_id}/` directories on-demand. Verify these are created in the assets directory, not absolute paths.
