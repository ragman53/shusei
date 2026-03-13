---
phase: quick-003
plan: 01
type: execute
wave: 1
depends_on: []
files_modified: [.cargo/config.toml]
autonomous: true
requirements: [INFRA-01]
must_haves:
  truths:
    - "Cargo finds Android linker without PATH errors"
    - "Build proceeds past linker discovery stage"
  artifacts:
    - path: ".cargo/config.toml"
      provides: "Absolute Windows paths to NDK toolchain"
      contains: "C:\\\\Users\\\\ragma\\\\AppData\\\\Local\\\\Android\\\\Sdk\\\\ndk\\\\26"
  key_links:
    - from: ".cargo/config.toml"
      to: "NDK 26 toolchain"
      via: "absolute path"
      pattern: "C:\\\\.*/ndk/26.1.10909125/.*clang.cmd"
---

<objective>
Fix Android linker path configuration to use absolute Windows paths.

Purpose: Cargo on Windows uses the Windows PATH, not bash PATH. Relative linker names fail because cargo cannot find them.
Output: Working Android cross-compilation with NDK 26.
</objective>

<execution_context>
@$HOME/.config/opencode/get-shit-done/workflows/execute-plan.md
@$HOME/.config/opencode/get-shit-done/templates/summary.md
</execution_context>

<context>
@.planning/STATE.md
@.cargo/config.toml

## Diagnosis

**Error:** `error: linker 'x86_64-linux-android21-clang' not found`

**Root Cause:** Cargo on Windows uses Windows PATH (not bash PATH). The relative linker names cannot be resolved.

**Solution:** Use absolute Windows paths to NDK 26 toolchain (NDK 29 has compatibility issues with `esaxx-rs`).

**NDK Base Path:**
```
C:\Users\ragma\AppData\Local\Android\Sdk\ndk\26.1.10909125\toolchains\llvm\prebuilt\windows-x86_64\bin
```
</context>

<tasks>

<task type="auto">
  <name>task 1: Update .cargo/config.toml with absolute Windows paths</name>
  <files>.cargo/config.toml</files>
  <action>
Replace the entire `.cargo/config.toml` with absolute Windows paths for all Android targets.
Use NDK 26.1.10909125 (not NDK 29 - has esaxx-rs compatibility issues).
On Windows, use the `.cmd` wrapper scripts for clang linkers.

New content:
```toml
[target.aarch64-linux-android]
linker = "C:\\Users\\ragma\\AppData\\Local\\Android\\Sdk\\ndk\\26.1.10909125\\toolchains\\llvm\\prebuilt\\windows-x86_64\\bin\\aarch64-linux-android30-clang.cmd"
ar = "C:\\Users\\ragma\\AppData\\Local\\Android\\Sdk\\ndk\\26.1.10909125\\toolchains\\llvm\\prebuilt\\windows-x86_64\\bin\\llvm-ar.exe"

[target.armv7-linux-androideabi]
linker = "C:\\Users\\ragma\\AppData\\Local\\Android\\Sdk\\ndk\\26.1.10909125\\toolchains\\llvm\\prebuilt\\windows-x86_64\\bin\\armv7a-linux-androideabi21-clang.cmd"
ar = "C:\\Users\\ragma\\AppData\\Local\\Android\\Sdk\\ndk\\26.1.10909125\\toolchains\\llvm\\prebuilt\\windows-x86_64\\bin\\llvm-ar.exe"

[target.i686-linux-android]
linker = "C:\\Users\\ragma\\AppData\\Local\\Android\\Sdk\\ndk\\26.1.10909125\\toolchains\\llvm\\prebuilt\\windows-x86_64\\bin\\i686-linux-android21-clang.cmd"
ar = "C:\\Users\\ragma\\AppData\\Local\\Android\\Sdk\\ndk\\26.1.10909125\\toolchains\\llvm\\prebuilt\\windows-x86_64\\bin\\llvm-ar.exe"

[target.x86_64-linux-android]
linker = "C:\\Users\\ragma\\AppData\\Local\\Android\\Sdk\\ndk\\26.1.10909125\\toolchains\\llvm\\prebuilt\\windows-x86_64\\bin\\x86_64-linux-android21-clang.cmd"
ar = "C:\\Users\\ragma\\AppData\\Local\\Android\\Sdk\\ndk\\26.1.10909125\\toolchains\\llvm\\prebuilt\\windows-x86_64\\bin\\llvm-ar.exe"
```
  </action>
  <verify>
    <automated>cargo build --target x86_64-linux-android 2>&1 | head -5</automated>
  </verify>
  <done>Android linker found and build proceeds past linker stage</done>
</task>

</tasks>

<verification>
- Verify cargo can find the linker: `cargo build --target x86_64-linux-android` should not show "linker not found" error
- Verify config.toml uses absolute paths with double backslashes
</verification>

<success_criteria>
- `.cargo/config.toml` contains absolute Windows paths
- `cargo build --target x86_64-linux-android` proceeds past linker discovery
</success_criteria>

<output>
After completion, create `.planning/quick/003-fix-android-linker-path/003-SUMMARY.md`
</output>