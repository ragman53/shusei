---
phase: quick
plan: 9
type: execute
wave: 1
depends_on: []
files_modified: [.gitignore]
autonomous: true
requirements: []
user_setup: []

must_haves:
  truths:
    - "No build artifacts exist in patches/esaxx-rs/target/"
    - "All target/ directories are ignored by git"
    - "No redundant test PDF files in tests/"
  artifacts:
    - path: ".gitignore"
      contains: "target/"  # Must match all target dirs, not just root
    - path: "patches/esaxx-rs/target/"
      should_not_exist: true
    - path: "tests/medium_pdf_test.pdf"
      should_not_exist: true
  key_links: []
---

<objective>
Clean up garbage data generated during development: build artifacts in patched dependency, redundant test files, and fix .gitignore to prevent future accumulation.

Purpose: Free disk space (~102MB) and prevent build artifacts from being accidentally committed.
Output: Clean repository with proper gitignore patterns.
</objective>

<execution_context>
@$HOME/.config/opencode/get-shit-done/workflows/execute-plan.md
@$HOME/.config/opencode/get-shit-done/templates/summary.md
</execution_context>

<context>
@.planning/STATE.md
@.gitignore

**Discovery:**
- `patches/esaxx-rs/target/` = 101MB of Rust incremental build artifacts
- Root `.gitignore` has `/target/` (with leading slash) which only ignores ROOT target directory
- `tests/medium_pdf_test.pdf` is untracked and redundant (source was copied to `assets/test/medium_pdf_test.pdf` in quick task 004)
- Code references `assets/test/medium_pdf_test.pdf`, not `tests/medium_pdf_test.pdf`
</context>

<tasks>

<task type="auto">
  <name>task 1: Delete build artifacts from patched dependency</name>
  <files>patches/esaxx-rs/target/</files>
  <action>
    Delete the entire `patches/esaxx-rs/target/` directory (101MB of build artifacts).
    
    This directory contains:
    - Incremental compilation cache
    - .fingerprint files
    - Build outputs (.exe, .pdb, .rlib, .rmeta)
    - Dependency metadata
    
    Command: `rm -rf patches/esaxx-rs/target/`
    
    Note: This is safe to delete - it will be regenerated when building the patched crate.
  </action>
  <verify>
    <automated>test ! -d patches/esaxx-rs/target && echo "PASS: target directory deleted"</automated>
  </verify>
  <done>patches/esaxx-rs/target/ directory no longer exists, ~101MB freed</done>
</task>

<task type="auto">
  <name>task 2: Fix .gitignore to ignore all target directories</name>
  <files>.gitignore</files>
  <action>
    Update .gitignore to ignore `target/` directories in ALL locations, not just root.
    
    Current issue: `.gitignore` has `/target/` (with leading slash) which only matches the root target directory.
    
    Change line 2 from:
    ```
    /target/
    ```
    to:
    ```
    target/
    ```
    
    This will match `target/` in any directory, including `patches/esaxx-rs/target/`.
  </action>
  <verify>
    <automated>grep -q '^target/$' .gitignore && echo "PASS: gitignore fixed"</automated>
  </verify>
  <done>.gitignore ignores target/ directories in all locations</done>
</task>

<task type="auto">
  <name>task 3: Remove redundant test PDF file</name>
  <files>tests/medium_pdf_test.pdf</files>
  <action>
    Delete `tests/medium_pdf_test.pdf` - it is redundant.
    
    Background: In quick task 004, this file was copied to `assets/test/medium_pdf_test.pdf` for bundling with the APK. The code references the assets copy, not this tests copy:
    - Android: `copy_asset_to_files("test/medium_pdf_test.pdf")` reads from APK assets
    - Desktop: `PathBuf::from("assets/test/medium_pdf_test.pdf")` reads from assets directory
    
    The tests/ copy was the source file, now redundant after copying to assets/.
    
    Note: `tests/large_pdf_test.pdf` is kept - it's tracked in git and used for large PDF testing.
  </action>
  <verify>
    <automated>test ! -f tests/medium_pdf_test.pdf && echo "PASS: redundant PDF deleted"</automated>
  </verify>
  <done>tests/medium_pdf_test.pdf deleted, ~856KB freed</done>
</task>

</tasks>

<verification>
- [ ] patches/esaxx-rs/target/ does not exist
- [ ] .gitignore contains `target/` (without leading slash)
- [ ] tests/medium_pdf_test.pdf does not exist
- [ ] tests/large_pdf_test.pdf still exists (kept)
- [ ] assets/test/medium_pdf_test.pdf still exists (kept - used by app)
</verification>

<success_criteria>
- ~102MB disk space freed (101MB + 856KB)
- Build artifacts will not accumulate in patched dependencies
- No redundant test files
</success_criteria>

<output>
After completion, create `.planning/quick/9-clean-up-garbage-data-generated-during-d/9-SUMMARY.md`
</output>