---
phase: quick-006
plan: 01
type: execute
wave: 1
depends_on: []
files_modified: [src/ui/library.rs]
autonomous: true
requirements: [QUICK-006]
user_setup: []

must_haves:
  truths:
    - "Desktop users can see Load Demo PDF button"
    - "Desktop users can click Load Demo PDF to load test PDF"
    - "Android behavior remains unchanged"
  artifacts:
    - path: "src/ui/library.rs"
      provides: "LibraryScreen with demo PDF button for both platforms"
      contains: "Load Demo PDF"
  key_links:
    - from: "src/ui/library.rs"
      to: "assets/test/medium_pdf_test.pdf"
      via: "direct file path on desktop, JNI on Android"
---

<objective>
Add "Load Demo PDF" button for desktop users alongside the existing Android button.

Purpose: Desktop users currently cannot test PDF functionality without manually selecting a file. This button loads the bundled test PDF directly.
Output: Working "Load Demo PDF" button visible on both desktop AND Android.
</objective>

<execution_context>
@$HOME/.config/opencode/get-shit-done/workflows/execute-plan.md
@$HOME/.config/opencode/get-shit-done/templates/summary.md
</execution_context>

<context>
@.planning/STATE.md

## Current Implementation

From `src/ui/library.rs`:

**Android (lines 182-242):** Uses JNI to extract PDF from APK assets:
```rust
#[cfg(target_os = "android")]
let load_demo_pdf = move |_| {
    // Copy asset from APK to files directory
    let demo_path = match crate::platform::android::copy_asset_to_files("test/medium_pdf_test.pdf") {
        Ok(path) => path,
        // ... error handling
    };
    // Import using PdfProcessor
};
```

**Button rendering (lines 330-341):**
```rust
// Load Demo PDF button (Android only)
#[cfg(target_os = "android")]
button {
    class: "bg-orange-500 text-white px-4 py-2 rounded-lg",
    onclick: load_demo_pdf,
    disabled: importing(),
    if importing() {
        "Loading..."
    } else {
        "Load Demo PDF"
    }
}
```

**Asset location:** `assets/test/medium_pdf_test.pdf` (bundled via Dioxus.toml `resources = ["assets/test/*"]`)

## Desktop Asset Access

On desktop, assets are accessible relative to the executable. The path `assets/test/medium_pdf_test.pdf` should work directly when running from project root or when bundled.
</context>

<tasks>

<task type="auto">
  <name>task 1: Add desktop Load Demo PDF handler</name>
  <files>src/ui/library.rs</files>
  <action>
Add a desktop-specific `load_demo_pdf` handler after the Android handler (around line 242):

1. Add `#[cfg(not(target_os = "android"))]` handler for `load_demo_pdf`
2. Use direct path `assets/test/medium_pdf_test.pdf` for the demo PDF
3. Get app data directory using the same pattern as `handle_import_pdf` (current_exe parent or ".")
4. Import using `PdfProcessor::import_pdf()` 
5. Show metadata review dialog with title "Demo PDF" or extracted metadata

Key differences from Android:
- No JNI asset extraction needed
- Use direct file path: `std::path::PathBuf::from("assets/test/medium_pdf_test.pdf")`
- Same PdfProcessor import flow and metadata dialog
  </action>
  <verify>
    <automated>cargo check --features desktop 2>&1 | head -20</automated>
  </verify>
  <done>Desktop `load_demo_pdf` handler compiles and follows same pattern as Android version</done>
</task>

<task type="auto">
  <name>task 2: Show Load Demo PDF button on desktop</name>
  <files>src/ui/library.rs</files>
  <action>
Replace the Android-only button (lines 330-341) with platform-conditional rendering:

**Before:**
```rust
// Load Demo PDF button (Android only)
#[cfg(target_os = "android")]
button { ... }
```

**After:**
```rust
// Load Demo PDF button (both platforms)
#[cfg(target_os = "android")]
button {
    class: "bg-orange-500 text-white px-4 py-2 rounded-lg",
    onclick: load_demo_pdf,
    disabled: importing(),
    if importing() { "Loading..." } else { "Load Demo PDF" }
}

#[cfg(not(target_os = "android"))]
button {
    class: "bg-orange-500 text-white px-4 py-2 rounded-lg",
    onclick: load_demo_pdf,
    disabled: importing(),
    if importing() { "Loading..." } else { "Load Demo PDF" }
}
```

Both buttons use the same styling and text, but call their platform-specific handlers.
  </action>
  <verify>
    <automated>cargo check --features desktop 2>&1 | head -20 && cargo check --target aarch64-linux-android 2>&1 | head -20</automated>
  </verify>
  <done>"Load Demo PDF" button visible on both desktop and Android, each using their respective handlers</done>
</task>

<task type="checkpoint:human-verify">
  <what-built>Load Demo PDF button for desktop users</what-built>
  <how-to-verify>
    1. Run `dx serve` (desktop)
    2. Click "Load Demo PDF" button
    3. Verify metadata dialog appears with PDF info
    4. Click "Import" to add to library
    5. Verify PDF appears in library list
  </how-to-verify>
  <resume-signal>Type "approved" or describe issues</resume-signal>
</task>

</tasks>

<verification>
- Desktop build compiles without errors
- Android build compiles without errors
- Button visible on both platforms
- PDF loads successfully on desktop
</verification>

<success_criteria>
- "Load Demo PDF" button visible on desktop
- Clicking button loads `assets/test/medium_pdf_test.pdf`
- Metadata review dialog appears
- PDF can be imported to library
- Android behavior unchanged (still works via JNI)
</success_criteria>

<output>
After completion, create `.planning/quick/006-add-load-demo-pdf-button-for-desktop/006-SUMMARY.md`
</output>