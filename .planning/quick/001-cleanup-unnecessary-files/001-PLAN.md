---
phase: quick-001
plan: 01
type: execute
wave: 1
depends_on: []
files_modified: []
autonomous: true
requirements: []
must_haves:
  truths:
    - "No PDFium DLL files remain in project root"
    - "No PDFium directories (include, licenses, bin, lib) exist"
    - "Project builds successfully without PDFium dependencies"
  artifacts:
    - path: "pdfium.dll"
      provides: "Removed Windows DLL"
      should_not_exist: true
    - path: "pdfium.dll.lib"
      provides: "Removed Windows library"
      should_not_exist: true
    - path: "PDFiumConfig.cmake"
      provides: "Removed CMake config"
      should_not_exist: true
    - path: "args.gn"
      provides: "Removed GN build config"
      should_not_exist: true
    - path: "nul"
      provides: "Removed error file"
      should_not_exist: true
    - path: "include/"
      provides: "Removed C/C++ headers"
      should_not_exist: true
    - path: "licenses/"
      provides: "Removed PDFium licenses"
      should_not_exist: true
    - path: "bin/"
      provides: "Removed binary directory"
      should_not_exist: true
    - path: "lib/"
      provides: "Removed library directory"
      should_not_exist: true
  key_links: []
---

<objective>
Remove all PDFium legacy artifacts after migration to hayro (pure Rust PDF renderer).

Purpose: Clean up obsolete FFI artifacts that are no longer needed since the project migrated to hayro, which is pure Rust with no external dependencies.

Output: Removed files and directories, verified project still builds.
</objective>

<execution_context>
@$HOME/.config/opencode/get-shit-done/workflows/execute-plan.md
@$HOME/.config/opencode/get-shit-done/templates/summary.md
</execution_context>

<context>
@.planning/STATE.md

## Background

Project migrated from pdfium-render to hayro for PDF rendering. Hayro is pure Rust with no FFI, no external DLLs, no build-time dependencies. All PDFium artifacts are now obsolete.

## Files to Delete

**Root files:**
- `pdfium.dll` (7.1MB) - Windows DLL for PDFium
- `pdfium.dll.lib` (111KB) - Windows library for PDFium
- `PDFiumConfig.cmake` (1.7KB) - CMake config for PDFium
- `args.gn` (216B) - GN build system config
- `nul` (113B) - Empty/error file

**Directories:**
- `include/` - C/C++ headers for PDFium API
- `licenses/` - Third-party licenses for PDFium dependencies
- `bin/` - Contains only pdfium.dll (duplicate)
- `lib/` - Contains only pdfium.dll.lib (duplicate)

## Files to Keep

- `platform/android/` - Android manifest and app configuration (still used)
- `samples/` - Test files for PDF processing
</context>

<tasks>

<task type="auto">
  <name>task 1: Delete PDFium legacy files</name>
  <files>pdfium.dll, pdfium.dll.lib, PDFiumConfig.cmake, args.gn, nul</files>
  <action>
    Delete all PDFium-related files from the project root:
    
    ```bash
    rm -f pdfium.dll pdfium.dll.lib PDFiumConfig.cmake args.gn nul
    ```
    
    These files are obsolete after the hayro migration. Hayro is pure Rust and requires no FFI artifacts.
  </action>
  <verify>
    <automated>test ! -f pdfium.dll && test ! -f pdfium.dll.lib && test ! -f PDFiumConfig.cmake && test ! -f args.gn && test ! -f nul && echo "All files deleted"</automated>
  </verify>
  <done>All five PDFium legacy files removed from project root</done>
</task>

<task type="auto">
  <name>task 2: Delete PDFium legacy directories</name>
  <files>include/, licenses/, bin/, lib/</files>
  <action>
    Delete all PDFium-related directories:
    
    ```bash
    rm -rf include/ licenses/ bin/ lib/
    ```
    
    These directories contain only PDFium artifacts:
    - `include/` - C/C++ headers for FFI (not needed by hayro)
    - `licenses/` - Third-party licenses for PDFium dependencies
    - `bin/` - Contains duplicate pdfium.dll
    - `lib/` - Contains duplicate pdfium.dll.lib
  </action>
  <verify>
    <automated>test ! -d include && test ! -d licenses && test ! -d bin && test ! -d lib && echo "All directories deleted"</automated>
  </verify>
  <done>All four PDFium legacy directories removed</done>
</task>

</tasks>

<verification>
## Post-Cleanup Verification

After deleting all files and directories, verify the project still builds:

```bash
cargo build
```

Expected: Build succeeds without errors (hayro has no external dependencies).

## Files That Should NOT Exist

- `pdfium.dll`
- `pdfium.dll.lib`
- `PDFiumConfig.cmake`
- `args.gn`
- `nul`
- `include/` directory
- `licenses/` directory
- `bin/` directory
- `lib/` directory

## Files That Should Still Exist

- `platform/android/` - Android configuration
- `samples/` - Test files
- `build.rs` - Build script (now references hayro only)
</verification>

<success_criteria>
- [ ] All 5 PDFium files deleted from root
- [ ] All 4 PDFium directories deleted
- [ ] Project builds successfully with `cargo build`
- [ ] No references to PDFium remain in build configuration
</success_criteria>

<output>
After completion, create `.planning/quick/001-cleanup-unnecessary-files/quick-001-SUMMARY.md`
</output>