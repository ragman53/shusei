# Large PDF Test Procedure

## Test PDF Information

**File:** `tests/large_pdf_test.pdf`

**Source:** "Difference and Repetition" by Gilles Deleuze (public domain PDF from samples directory)

**Metadata:**
- Page count: 373 pages
- File size: 13.15 MB
- Content type: Academic text (philosophy)
- Language: English
- Page complexity: Mixed (text-heavy with some formatting)

## Test Objectives

Verify that the batch processing infrastructure handles large PDFs (100+ pages) without:
1. Memory crashes (OOM errors)
2. App freezes or ANR (Application Not Responding)
3. Memory leaks (continuous growth)

## Monitoring Infrastructure

The following logging has been added to track processing:

### PDF Processing (src/core/pdf.rs)

1. **Conversion start:** Logs book ID and PDF path
2. **PDF opened:** Logs total page count
3. **Batch completion:** For each batch of 10 pages:
   - Batch number
   - Pages rendered in batch
   - Cumulative total
   - Time taken for batch
4. **OCR processing start:** Logs page count
5. **OCR progress:** Every 10 pages
6. **OCR completion:** Total time and pages/second rate
7. **Overall completion:** Total processing time

### OCR Engine (src/core/ocr/engine.rs)

1. **Parallel processing start:** Page count and concurrency limit
2. **Page completion:** Individual page OCR results with confidence
3. **Retry attempts:** Warnings for failed OCR attempts
4. **Parallel processing complete:** Total time and throughput

## Test Procedure

### Step 1: Build the Application

```bash
cargo build --release --features pdf
```

### Step 2: Deploy to Android Device

Deploy the built application to an Android device (preferably a mid-range device with 4-6GB RAM).

### Step 3: Import the Large PDF

1. Open the Shusei app
2. Click the "+" button to add a book
3. Select "Import PDF"
4. Navigate to and select `tests/large_pdf_test.pdf`
5. Verify metadata dialog appears showing:
   - Title: "Difference and Repetition"
   - Author: "Gilles Deleuze"
   - Page count: 373

### Step 4: Start Conversion

1. Go to the book detail page
2. Click the "Convert" button
3. Verify progress display appears showing:
   - Stage indicator (📄 Rendering → 🔍 OCR → ✓ Complete)
   - Current page number (e.g., "Processing page 15/373")
   - Progress bar

### Step 5: Monitor Processing

Watch for the following indicators:

**Success indicators:**
- Progress advances steadily
- Batch completion logs appear (every 10 pages)
- Memory usage remains stable (check via Android Studio Profiler or device settings)
- App remains responsive (can navigate, no ANR dialogs)

**Failure indicators:**
- App freezes completely (not just busy indicator)
- App closes unexpectedly
- "Out of memory" errors
- ANR (Application Not Responding) dialogs
- Progress stalls for extended periods (>1 minute per batch)

### Step 6: Test Multitasking

1. While processing is ongoing, press the home button to background the app
2. Use other apps for 1-2 minutes
3. Return to the Shusei app
4. Verify:
   - Processing resumed from last page (not restarted from beginning)
   - No crash on resume
   - Progress continues normally

### Step 7: Verify Completion

1. Wait for full conversion to complete
2. Verify:
   - Progress shows "373/373 pages converted" or similar
   - Status shows "Complete" with checkmark
3. Open the reader view
4. Spot-check random pages (e.g., pages 50, 150, 300)
5. Verify:
   - All pages display OCR text
   - No missing pages or empty content
   - Text is readable and correctly extracted

### Step 8: Document Results

Record the following metrics:

```
Test Date: _______________
Device Model: _______________
Android Version: _______________
RAM: _______________

Total processing time: ___ minutes ___ seconds
Average time per page: ___ seconds
Peak memory usage: ___ MB (if measurable)

Batch processing:
- Batch size: 10 pages
- Average batch time: ___ seconds
- Concurrency limit: 3 OCR operations

Crashes or errors: Yes/No (describe: _______________)
Resume after backgrounding: Worked/Failed
Final page count converted: ___ / 373

Observations:
_________________________________
_________________________________
```

## Expected Outcomes

**Pass criteria:**
- [ ] Processing completes without crash
- [ ] Memory usage stays stable (no continuous growth)
- [ ] App remains responsive during processing
- [ ] Resume works correctly after backgrounding
- [ ] All 373 pages converted successfully
- [ ] OCR text is readable on spot-checked pages

**Performance expectations:**
- Total processing time: 10-30 minutes (depending on device)
- Average time per page: 2-5 seconds
- Batch time (10 pages): 20-50 seconds

## Troubleshooting

**If processing is slow:**
- Check device specifications (CPU, RAM)
- Verify concurrency limit is appropriate (3 parallel OCR operations)
- Consider reducing batch size if memory issues occur

**If crashes occur:**
- Check logcat for OOM (Out of Memory) errors
- Verify batch processing is working (should process 10 pages at a time)
- Check if concurrency limit needs reduction

**If resume fails:**
- Verify processing_progress table is being updated
- Check database save operations are completing

## Log Analysis

After testing, review logs for:

1. **Batch timing consistency:** Should be relatively uniform
2. **Memory patterns:** Should not show continuous growth
3. **Error patterns:** Should be minimal (retry logic handles transient failures)
4. **Throughput:** Pages/second rate should be stable

Example log entries to look for:
```
INFO Starting PDF conversion for book 1
INFO PDF opened: 373 pages
INFO Batch 1 complete: rendered 10 pages (total: 10/373), time: 2.34s
INFO Batch 2 complete: rendered 10 pages (total: 20/373), time: 2.12s
...
INFO Starting OCR processing for 373 pages
INFO Parallel OCR complete: 373 pages processed in 15m 32s (0.40 pages/sec)
INFO PDF conversion complete for book 1: 373 pages, total time: 18m 45s
```
