# Deferred Items - Plan 03.1-02

## Pre-existing Blockers (Out of Scope)

### pdfium-render Linking Error

**Issue:** `pdfium-render` v0.8.37 with `static` feature fails to link on Windows with unresolved external symbol `FPDFPage_TransformAnnots`

**Impact:** Cannot compile with `--features pdf`, blocking PDF-related testing

**Status:** Pre-existing issue from Plan 03-04b, never actually resolved despite summary claiming success

**Verification:** Tested at commit da2227e (03-04b) - same linking error occurs

**Options for Resolution (Rule 4 - Architectural Decision Required):**
1. Remove pdfium-render dependency entirely
2. Switch to different PDF library (e.g., lopdf, pdf-rs)
3. Use pdfium-render without static feature (requires system PDFium installation)
4. Wait for upstream pdfium-render fix

**Current Workaround:** Proceed with OCR-only testing without pdf feature

**Logged:** 2026-03-13 during Plan 03.1-02 execution
