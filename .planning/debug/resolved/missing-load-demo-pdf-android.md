---
status: awaiting_human_verify
trigger: "Load Demo PDF button is missing on Android. Instead, there's a 'Go to library' button that shows 'Library view - coming soon' when pressed."
created: 2026-03-14T00:00:00Z
updated: 2026-03-14T00:06:00Z
---

## Current Focus

hypothesis: FIXED - The root cause was a combination of issues:
1. BookList/AddBook routes in app.rs were placeholders instead of using real components
2. library and add_book modules were not exported from ui/mod.rs
3. #[cfg] attributes in rsx! macro were causing syntax errors

All issues have been fixed.

test: Rebuild APK with `dx build --android` and verify Load Demo PDF button appears on Android
expecting: Load Demo PDF button should now be visible
next_action: Mark as complete and provide summary

## Symptoms

expected: Load Demo PDF button should be visible on Library screen on Android, allowing user to load a bundled test PDF
actual: No Load Demo PDF button visible. Only "Go to library" button exists, which shows "Library view - coming soon" placeholder
errors: No crash, but feature is missing
reproduction: Launch app on real Android device (arm64-v8a), observe Library screen
timeline: Just deployed APK to real device via `dx build --android`. The button was supposedly added in quick task 004.

## Eliminated

## Evidence

- timestamp: 2026-03-14T00:00:00Z
  checked: src/ui/library.rs
  found: Load Demo PDF button implementation exists at lines 378-393 for both Android and desktop platforms using #[cfg(target_os = "android")] and #[cfg(not(target_os = "android"))]
  implication: The code is correct and present in the source

- timestamp: 2026-03-14T00:00:00Z
  checked: Android handler in library.rs (lines 181-242)
  found: load_demo_pdf closure exists for Android that copies asset from APK to files directory
  implication: The Android-specific code exists and handles platform correctly

- timestamp: 2026-03-14T00:00:00Z
  checked: Dioxus.toml configuration
  found: default_platform = "desktop" is set, and [bundle] section exists for Android with resources = ["assets/models/*", "assets/test/*"]
  implication: The default_platform might be causing issues, but the bundle resources should still be included for Android builds

- timestamp: 2026-03-14T00:00:01Z
  checked: src/app.rs route definitions and components
  found: BookList component at lines 192-200 is a placeholder that just shows "Library view - coming soon" instead of using the actual LibraryScreen component from library.rs
  implication: The router is rendering a placeholder instead of the real LibraryScreen which contains the Load Demo PDF button

- timestamp: 2026-03-14T00:00:02Z
  checked: src/ui/mod.rs module exports
  found: library and add_book modules are not declared or exported, so LibraryScreen and AddBookForm are not accessible from app.rs
  implication: Need to add the modules to src/ui/mod.rs to make the components available

## Resolution

root_cause: Multiple issues were preventing the Load Demo PDF button from appearing - the BookList/AddBook routes were placeholder components, the modules weren't exported from ui/mod.rs, and #[cfg] attributes in rsx! macro were invalid.

fix: 
1. Updated src/ui/mod.rs to export library and add_book modules
2. Updated src/app.rs to import and use LibraryScreen and AddBookForm
3. Fixed library.rs by removing invalid #[cfg] attributes from rsx! macro

verification: Rebuild APK with `dx build --android` and verify Load Demo PDF button is visible on Android Library screen

files_changed:
  - src/ui/mod.rs: Added library and add_book module exports
  - src/app.rs: Updated imports and replaced placeholder components
  - src/ui/library.rs: Removed invalid #[cfg] attributes from rsx! macro
