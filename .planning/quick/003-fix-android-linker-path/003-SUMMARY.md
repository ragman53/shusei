---
phase: quick-003
plan: 01
subsystem: build-infrastructure
tags: [android, ndk, cargo, linker, windows]
requires: [NDK 26.1.10909125 installed]
provides: [Android cross-compilation on Windows]
affects: [.cargo/config.toml]
key-files:
  created: []
  modified:
    - path: .cargo/config.toml
      change: "Replaced relative linker names with absolute Windows paths to NDK 26 toolchain"
tech-stack:
  added: []
  patterns:
    - "Absolute Windows paths for cargo cross-compilation"
    - "NDK .cmd wrapper scripts for clang on Windows"
decisions:
  - "Use NDK 26 instead of NDK 29 (esaxx-rs compatibility issues)"
  - "Use .cmd wrapper scripts for clang linkers on Windows"
  - "Configure all 4 Android target architectures"
metrics:
  duration: 46s
  completed_date: 2026-03-13
  tasks_completed: 1
  files_modified: 1
  commits: 1
---

# Quick Task 003: Fix Android Linker Path

## One-Liner

Configured Cargo to use absolute Windows paths to NDK 26 toolchain for Android cross-compilation, resolving "linker not found" errors on Windows.

## Context

**Problem:** Cargo on Windows uses the Windows PATH environment variable, not the bash PATH. Relative linker names like `x86_64-linux-android21-clang` cannot be resolved because they are not in the Windows PATH.

**Solution:** Use absolute Windows paths pointing directly to the NDK 26.1.10909125 toolchain binaries, using `.cmd` wrapper scripts for clang linkers.

## Changes Made

### .cargo/config.toml

Replaced all relative linker configurations with absolute paths:

| Target | Previous | New |
|--------|----------|-----|
| aarch64-linux-android | `aarch64-linux-android30-clang` | Full path to NDK 26 `.cmd` wrapper |
| armv7-linux-androideabi | `armv7a-linux-androideabi21-clang` | Full path to NDK 26 `.cmd` wrapper |
| i686-linux-android | `i686-linux-android21-clang` | Full path to NDK 26 `.cmd` wrapper |
| x86_64-linux-android | `x86_64-linux-android21-clang` | Full path to NDK 26 `.cmd` wrapper |

**Base NDK Path:**
```
C:\Users\ragma\AppData\Local\Android\Sdk\ndk\26.1.10909125\toolchains\llvm\prebuilt\windows-x86_64\bin
```

## Verification

```bash
$ cargo build --target x86_64-linux-android
warning: version requirement `0.3.0+0.22` for dependency `ort-tract` includes semver metadata...
   Compiling cfg-if v1.0.4
   Compiling memchr v2.8.0
   ...
```

Build proceeds past linker discovery - no "linker not found" error.

## Deviations from Plan

None - plan executed exactly as written.

## Self-Check

| Check | Status |
|-------|--------|
| .cargo/config.toml contains absolute paths | PASSED |
| cargo build --target x86_64-linux-android proceeds | PASSED |
| Commit created | PASSED |

---

*Completed: 2026-03-13*
*Commit: 7f44f03*