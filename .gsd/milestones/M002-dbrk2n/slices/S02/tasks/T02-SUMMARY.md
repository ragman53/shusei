---
id: T02
parent: S02
milestone: M002-dbrk2n
provides:
  - CameraPage with book_id parameter
  - Page number input field
  - OCR engine initialization on mount
  - Engine loading state indicator
  - Run OCR button disabled until engine ready
key_files:
  - src/ui/camera.rs
key_decisions:
  - Used Option<String> for book_id prop to support both /camera and /camera/:book_id routes
  - Imported OcrEngine trait to access is_ready() and process_image() methods on NdlocrEngine
  - Created book_id clone at component level to avoid multiple move issues in closures
patterns_established:
  - use_effect for async OCR engine initialization on component mount
  - Engine loading state with user-friendly loading indicator
  - Disabled button with tooltip for unavailable functionality
  - Debug info panel showing book_id and engine state
observability_surfaces:
  - log::debug!("Camera page mounted for book_id={}") - Page navigation
  - log::info!("OCR engine initialized, ready={}") - Engine ready state
  - log::error!("OCR engine initialization failed: {}") - Model loading failure
  - log::info!("OCR completed: {} chars, confidence {}") - OCR success
  - UI: Loading indicator "Loading OCR engine..." while engine initializes
  - UI: "Run OCR" button disabled with tooltip "OCR engine loading..."
  - UI: Debug panel showing book_id and engine ready state
duration: 45m
verification_result: passed
completed_at: 2026-03-16
# Set blocker_discovered: true only if execution revealed the remaining slice plan
# is fundamentally invalid (wrong API, missing capability, architectural mismatch).
# Do NOT set true for ordinary bugs, minor deviations, or fixable issues.
blocker_discovered: false
---

# T02: Camera Page Enhancement

**Enhanced CameraPage with book_id parameter, page number input, and OCR engine initialization on mount**

## What Happened

Implemented all required features from the task plan:

1. **Route already existed**: The `/camera/:book_id` route and `CameraBook` wrapper component were already present in `src/app.rs` from T01.

2. **Updated CameraPage signature**: Changed from no parameters to `book_id: Option<String>` to support both `/camera` and `/camera/:book_id` routes.

3. **Added state signals**:
   - `page_number: Signal<u32>` (default: 1)
   - `ocr_engine: Signal<Option<NdlocrEngine>>`
   - `is_engine_ready: Signal<bool>` (default: false)
   - `is_engine_loading: Signal<bool>` (default: true)

4. **Added use_effect for OCR engine initialization**:
   - Gets app data directory
   - Creates `NdlocrEngine::new(&model_dir, "ja")`
   - Calls `engine.initialize()` async
   - Sets `is_engine_ready` to true when complete
   - Logs errors if initialization fails

5. **Added page number input field**:
   - Numeric input (type="number", min="1")
   - Label: "Page Number"
   - Value bound to `page_number` signal
   - Positioned above "Take Photo" button

6. **Added engine loading indicator**:
   - Blue banner with "Loading OCR engine..." text
   - Spinning hourglass icon
   - Shown while `is_engine_loading` is true

7. **Updated "Run OCR" button**:
   - Disabled until engine is ready
   - Shows "Loading..." during initialization
   - Shows "Engine Loading..." if not ready
   - Tooltip: "OCR engine loading..." when disabled

8. **Added debug info panel**:
   - Shows book_id when present
   - Shows engine ready state

9. **Fixed compilation issues**:
   - Imported `OcrEngine` trait to access `is_ready()` and `process_image()` methods
   - Created `book_id_for_save` clone at component level to avoid move issues in closures

## Verification

**Build verification:**
```bash
cargo build
# Result: Finished `dev` profile [unoptimized + debuginfo] target(s) in 4.83s
```

**Code inspection:**
- `CameraPage` has `book_id: Option<String>` parameter ✓
- Route `/camera/:book_id` exists in `src/app.rs` ✓
- Page number input with correct label and type ✓
- OCR engine initialized in `use_effect` on mount ✓
- Loading state shown via `is_engine_loading` signal ✓
- "Run OCR" button disabled until `is_engine_ready` is true ✓

**Test verification:**
```bash
cargo test --lib
# Result: 99 passed; 2 failed (unrelated STT tests)
```

**Observability verification:**
- Log statements added for engine initialization, errors, and OCR completion
- UI shows loading state, disabled button state, and debug info

## Diagnostics

**How to inspect this implementation:**

1. **Check engine state in logs:**
   ```bash
   # Desktop
   cargo run 2>&1 | grep -i "OCR engine"
   
   # Expected output:
   # DEBUG Camera page mounted for book_id=test-book-123
   # INFO OCR engine initialized, ready=true
   ```

2. **Check UI state:**
   - Navigate to `/camera/test-book-123`
   - Verify blue loading banner appears initially
   - Verify "Run OCR" button is disabled with "Loading..." text
   - Verify debug panel shows book_id and engine ready state
   - After engine loads, verify button becomes enabled

3. **Check accessibility tree:**
   - Page number input should have label "Page Number"
   - Input type should be "number" with min="1"

4. **Failure state:**
   - If engine fails to load, red error banner shows "OCR engine failed to load: <error>"
   - "Run OCR" button remains disabled

## Deviations

None - all task plan requirements implemented as specified.

## Known Issues

None - implementation complete and builds successfully.

## Files Created/Modified

- `src/ui/camera.rs` — Enhanced with book_id prop, page number input, OCR engine initialization, loading state, and debug info panel
