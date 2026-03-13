---
phase: quick
plan: 002
type: execute
wave: 1
depends_on: []
files_modified:
  - Cargo.toml
  - src/core/stt/mod.rs
  - src/core/stt/tokenizer.rs
autonomous: true
requirements: []
must_haves:
  truths:
    - "dx serve --android builds successfully"
    - "Desktop build still includes STT support"
  artifacts:
    - path: "Cargo.toml"
      provides: "Platform-specific tokenizers dependency"
    - path: "src/core/stt/tokenizer.rs"
      provides: "Conditional tokenizer module"
  key_links:
    - from: "Cargo.toml"
      to: "tokenizers crate"
      via: "target cfg"
---

<objective>
Fix `dx serve --android` build failure by making `tokenizers` crate desktop-only.

**Problem:** Dioxus CLI uses NDK 29 despite NDK 26 being configured. The `esaxx-rs` crate (dependency of `tokenizers`) fails with NDK 29 due to `pthread_cond_clockwait` requiring Android API 30+.

**Solution:** Make `tokenizers` desktop-only since STT is Phase 5 (not yet implemented).

Purpose: Unblock Android build for Phase 03.2 human verification.
Output: Working Android build, preserved desktop STT support.
</objective>

<execution_context>
@$HOME/.config/opencode/get-shit-done/workflows/execute-plan.md
@$HOME/.config/opencode/get-shit-done/templates/summary.md
</execution_context>

<context>
@.planning/STATE.md

## Diagnosis Summary

**Error:**
```
error: use of undeclared identifier 'pthread_cond_clockwait'
```

**Failed crate:** `esaxx-rs v0.1.10` (dependency of `tokenizers v0.20`)

**Dependency chain:**
```
esaxx-rs v0.1.10
└── tokenizers v0.20.4
    └── shusei v0.1.0
```

**Root cause:** Dioxus CLI uses NDK 29 instead of configured NDK 26, causing C++ compatibility issues.

**Impact:** STT is Phase 5, so tokenizers is not needed for current Android builds.
</context>

<tasks>

<task type="auto">
  <name>task 1: Make tokenizers desktop-only dependency</name>
  <files>Cargo.toml</files>
  <action>
Move `tokenizers` from `[dependencies]` to `[target.'cfg(not(target_os = "android"))'.dependencies]`.

**Current (line 42-43):**
```toml
# Tokenizer (for STT)
tokenizers = "0.20"
```

**Change to:** Remove from `[dependencies]` and add to desktop-only section:
```toml
[target.'cfg(not(target_os = "android"))'.dependencies]
rfd = "0.15"
ort = "2.0.0-rc.12"
tokenizers = "0.20"  # STT tokenizer (Phase 5 - desktop only for now)
```

This ensures tokenizers is only compiled for desktop builds, avoiding the NDK 29 compatibility issue.
  </action>
  <verify>
    <automated>cargo check --target aarch64-linux-android 2>&1 | grep -c "tokenizers" || echo "tokenizers excluded from android"</automated>
  </verify>
  <done>tokenizers not included in Android build dependencies</done>
</task>

<task type="auto">
  <name>task 2: Add conditional compilation for STT tokenizer module</name>
  <files>src/core/stt/mod.rs, src/core/stt/tokenizer.rs</files>
  <action>
Wrap the tokenizer module and re-export with platform-specific cfg attribute.

**In `src/core/stt/mod.rs`:**
- Add `#[cfg(not(target_os = "android"))]` before `mod tokenizer;`
- Add `#[cfg(not(target_os = "android"))]` before `pub use tokenizer::Tokenizer;`

**In `src/core/stt/tokenizer.rs`:**
- Add `#![cfg(not(target_os = "android"))]` at the top of the file

This ensures the tokenizer module is only compiled for desktop builds, matching the tokenizers dependency.
  </action>
  <verify>
    <automated>cargo check --target aarch64-linux-android 2>&1 | head -20</automated>
  </verify>
  <done>STT tokenizer module conditionally compiled for desktop only</done>
</task>

<task type="auto">
  <name>task 3: Verify Android build succeeds</name>
  <files>N/A (verification only)</files>
  <action>
Run `dx serve --android` to verify the build completes without the `pthread_cond_clockwait` error.

Expected outcome: Build starts successfully and deploys to emulator/device.
  </action>
  <verify>
    <automated>dx serve --android --no-open 2>&1 | head -50</automated>
  </verify>
  <done>Android build starts without NDK 29 compatibility errors</done>
</task>

</tasks>

<verification>
1. `cargo check --target aarch64-linux-android` passes
2. `dx serve --android` starts without build errors
3. Desktop build still compiles with STT support
</verification>

<success_criteria>
- Android build completes successfully
- Desktop build retains STT functionality
- No regression in existing features
</success_criteria>

<output>
After completion, create `.planning/quick/002-debug-dx-serve-android/002-SUMMARY.md`
</output>