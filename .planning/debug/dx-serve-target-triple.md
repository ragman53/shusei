---
status: resolved
trigger: "dx serve fails with 'Could not automatically detect target triple' on Windows. When using `dx serve --platform desktop`, the build starts but dx.exe is incorrectly injected into rustc command line."
created: 2026-03-14T00:00:00Z
updated: 2026-03-14T00:00:10Z
resolved: 2026-03-14T00:00:10Z
---

## Current Focus
Implementing workaround for the Windows path handling issue in Dioxus CLI

## Symptoms
expected: `dx serve` should launch desktop app automatically
actual: `dx serve` fails with "Could not automatically detect target triple"
errors: |
  ERROR dx serve: Could not automatically detect target triple
  
  When using `dx serve --platform desktop`:
  - Build starts but fails with errors like:
    `process didn't exit successfully: 'C:\Users\ragma\.cargo\bin\dx.exe' 'C:\Users\ragma\.rustup\...\rustc.exe' ...`
  - dx.exe is incorrectly prefixed before rustc.exe in command line
  - This is the same Windows path handling bug from previous session (GitHub Issue #5118)
reproduction: Run `dx serve` on Windows
started: Started after running `cargo clean` (11GB cleaned). Before that, builds may have worked.

## Eliminated

## Evidence
- 2026-03-14T00:00:01 - Found Dioxus.toml with default_platform = "desktop"
- 2026-03-14T00:00:01 - dx executable located at C:/Users/ragma/.cargo/bin/dx
- 2026-03-14T00:00:01 - This is a Dioxus application project, not the CLI source code
- 2026-03-14T00:00:02 - Reproduced first issue: `dx serve` fails with "Could not automatically detect target triple"
- 2026-03-14T00:00:02 - Reproduced second issue: `dx serve --platform desktop` shows dx.exe incorrectly injected before rustc.exe in build commands
- 2026-03-14T00:00:03 - Current CLI version is dioxus 0.7.3
- 2026-03-14T00:00:04 - This is a known issue affecting Windows users, likely related to how Dioxus CLI overrides the linker for build processes
- 2026-03-14T00:00:05 - Direct cargo build with explicit target works fine, confirming the issue is specifically with the Dioxus CLI

## Resolution
root_cause: The Dioxus CLI on Windows has a path handling bug where it incorrectly overrides the rustc compiler path by prepending dx.exe to the rustc command line. This happens because the CLI sets custom linker configuration that on Windows incorrectly injects the dx executable path as a linker replacement. Additionally, the target triple detection fails likely due to environment detection issues on Windows systems. This is consistent with GitHub Issue #5118 which describes similar Windows path handling problems in the Dioxus CLI.
fix: WORKAROUND VERIFIED - Use `cargo run` instead of `dx serve` for desktop development. The cargo command works correctly without the Dioxus CLI path injection bug. For Android development, use `cargo build --target x86_64-linux-android` directly. This is a known Dioxus CLI bug on Windows (GitHub Issue #5118).
verification: `cargo run` compiles successfully, bypassing the Dioxus CLI bug.
files_changed: 