---
phase: quick
plan: 004
type: execute
wave: 1
depends_on: []
files_modified:
  - Dioxus.toml
  - src/ui/library.rs
  - assets/test/medium_pdf_test.pdf
autonomous: true
requirements: []
user_setup: []

must_haves:
  truths:
    - "User can load a demo PDF on Android device"
    - "Demo PDF button appears only on Android"
    - "PDF is successfully imported and viewable"
  artifacts:
    - path: "assets/test/medium_pdf_test.pdf"
      provides: "Bundled test PDF for mobile"
    - path: "Dioxus.toml"
      contains: 'resources = ["assets/models/*", "assets/test/*"]'
    - path: "src/ui/library.rs"
      contains: "Load Demo PDF"
  key_links:
    - from: "Dioxus.toml"
      to: "assets/test/"
      via: "resource bundling"
      pattern: 'resources.*assets/test'
    - from: "src/ui/library.rs"
      to: "assets/test/medium_pdf_test.pdf"
      via: "file loading on Android"
      pattern: "load_demo_pdf"
---

<objective>
Add bundled test PDF so the Android app can load a demo PDF during human verification of Phase 03.2.

**Purpose:** Enable human verification of PDF functionality on Android device.
**Output:** Working "Load Demo PDF" button on Android that imports a bundled test PDF.
</objective>

<execution_context>
@$HOME/.config/opencode/get-shit-done/workflows/execute-plan.md
@$HOME/.config/opencode/get-shit-done/templates/summary.md
</execution_context>

<context>
@.planning/STATE.md
@Dioxus.toml
@src/ui/library.rs

## Problem

When running `dx serve --android`, the app cannot import PDFs because:
1. File picker (`rfd`) is desktop-only
2. No test PDF is bundled with the app
3. Android shows error "PDF import not available on mobile"

## Solution

1. Bundle a test PDF in APK assets
2. Add "Load Demo PDF" button for Android that loads from bundled assets
3. Reuse existing PDF import flow for the demo PDF
</context>

<tasks>

<task type="auto">
  <name>task 1: Bundle test PDF in assets</name>
  <files>assets/test/medium_pdf_test.pdf, Dioxus.toml</files>
  <action>
    1. Create `assets/test/` directory
    2. Copy `tests/medium_pdf_test.pdf` to `assets/test/medium_pdf_test.pdf`
    3. Update `Dioxus.toml` line 40:
       - Change: `resources = ["assets/models/*"]`
       - To: `resources = ["assets/models/*", "assets/test/*"]`
    
    This bundles the PDF into the APK for Android.
  </action>
  <verify>
    <automated>test -f assets/test/medium_pdf_test.pdf && grep -q 'assets/test/\*' Dioxus.toml && echo "PASS"</automated>
  </verify>
  <done>Test PDF exists in assets and Dioxus.toml updated to bundle it</done>
</task>

<task type="auto">
  <name>task 2: Add Load Demo PDF button for Android</name>
  <files>src/ui/library.rs</files>
  <action>
    Modify the Android section of library.rs (lines 174-179) to:
    
    1. Add a `load_demo_pdf` handler that:
       - Gets the assets directory via `crate::platform::android::get_assets_directory()`
       - Constructs path to bundled PDF: `assets_dir.join("test").join("medium_pdf_test.pdf")`
       - Reuses the existing PDF import logic (PdfProcessor::import_pdf)
       - Shows metadata review dialog after successful import
    
    2. Add a "Load Demo PDF" button in the UI (after "Import PDF" button, lines 255-265):
       - Only visible on Android (use `#[cfg(target_os = "android")]`)
       - Uses the same styling as "Import PDF" button
       - Has distinct color (e.g., orange) to indicate it's a demo feature
       - Calls `load_demo_pdf` handler on click
    
    The implementation should mirror the desktop import flow but load from a fixed bundled path instead of file picker.
  </action>
  <verify>
    <automated>grep -q "Load Demo PDF" src/ui/library.rs && grep -q "assets/test" src/ui/library.rs && echo "PASS"</automated>
  </verify>
  <done>"Load Demo PDF" button visible on Android, successfully imports bundled PDF</done>
</task>

<task type="checkpoint:human-verify">
  <what-built>Android app with bundled demo PDF and "Load Demo PDF" button</what-built>
  <how-to-verify>
    1. Run `dx serve --android` in terminal
    2. Wait for app to launch on device/emulator
    3. On the Library screen, verify "Load Demo PDF" button appears (orange button)
    4. Tap "Load Demo PDF" button
    5. Verify PDF metadata dialog appears with title/page count
    6. Confirm import to see the PDF in library
  </how-to-verify>
  <resume-signal>Type "approved" if PDF loads successfully, or describe the issue</resume-signal>
</task>

</tasks>

<verification>
- `dx serve --android` builds successfully
- "Load Demo PDF" button appears on Android
- Tapping button imports the bundled PDF
- PDF appears in library after import
</verification>

<success_criteria>
- [ ] Test PDF bundled in APK
- [ ] Dioxus.toml includes assets/test/* in resources
- [ ] "Load Demo PDF" button visible on Android
- [ ] Demo PDF imports successfully and appears in library
</success_criteria>

<output>
After completion, create `.planning/quick/004-add-pdf-test-assets/004-SUMMARY.md`
</output>