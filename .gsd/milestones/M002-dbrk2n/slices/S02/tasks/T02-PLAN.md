# T02: Camera Page Enhancement

**Slice:** S02 — Camera Book Capture
**Milestone:** M002-dbrk2n

## Description

Enhance `CameraPage` component to accept `book_id` parameter, add page number input, and initialize OCR engine on mount. This sets up the UI structure needed for OCR integration in T03.

## Steps

1. Modify `src/app.rs` to add route parameter: `#[route("/camera/:book_id")]` for `CameraBook { book_id: String }`
2. Create new `CameraBook` component wrapper that passes `book_id` to `CameraPage`
3. Update `CameraPage` signature: `pub fn CameraPage(book_id: String) -> Element`
4. Add state signals:
   - `page_number: Signal<u32>` (default: 1)
   - `ocr_engine: Signal<Option<NdlocrEngineTract>>`
   - `is_engine_ready: Signal<bool>` (default: false)
5. Add `use_effect` to initialize OCR engine on mount:
   - Get app data directory
   - Create `NdlocrEngineTract::new(&model_dir, "ja")`
   - Call `engine.initialize()` async
   - Set `is_engine_ready` to true when done
6. Add page number input field to UI:
   - Numeric input (type="number", min="1")
   - Label: "Page Number"
   - Value bound to `page_number` signal
   - Positioned above "Take Photo" button
7. Display `book_id` in debug area (can be styled subtly)
8. Add loading indicator for OCR engine initialization

## Must-Haves

- [ ] Route `/camera/:book_id` works (test with manual navigation)
- [ ] `CameraPage` receives `book_id` as prop
- [ ] Page number input visible and functional
- [ ] OCR engine initialized on mount (check logcat for "NDLOCR-Lite engine initialized")
- [ ] Loading state shown while engine loads
- [ ] "Run OCR" button disabled until engine is ready

## Verification

- Desktop test: `dx serve`, navigate to `/camera/test-book-123`, verify page renders
- Check accessibility tree: page number input has correct label and type
- Log inspection: OCR engine initialization logs appear in console
- Code inspection: `CameraPage` has `book_id: String` parameter

## Observability Impact

- **Signals added/changed:**
  - `log::info!("OCR engine initialized, ready={}", is_ready)` — Engine ready state
  - `log::error!("OCR engine initialization failed: {}", e)` — Model loading failure
  - `log::debug!("Camera page mounted for book_id={}", book_id)` — Page navigation
- **How a future agent inspects this:**
  - Check `is_engine_ready` signal state in Dioxus devtools (desktop)
  - Monitor logcat for engine initialization messages
- **Failure state exposed:**
  - "Run OCR" button disabled with tooltip "OCR engine loading..."
  - Error message if engine fails to initialize

## Inputs

- `src/core/ocr/engine_tract.rs` — `NdlocrEngineTract::new()` and `initialize()` methods
- `src/app.rs` — Route definitions (T01 may have already modified this)
- `src/ui/reader.rs` — Example of OCR engine initialization pattern to follow

## Expected Output

- `src/app.rs` — New route `/camera/:book_id` with `CameraBook` wrapper component
- `src/ui/camera.rs` — Enhanced with `book_id` prop, page number input, OCR engine initialization
- Working camera page that accepts book_id, has page number input, and has OCR engine ready for T03 integration
