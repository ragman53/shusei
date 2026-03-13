---
status: resolved
trigger: "dx build --android --release fails with linker error showing wrong linker (dx.exe) and path handling issues"
created: 2026-03-13T00:00:00Z
updated: 2026-03-13T23:10:00Z
resolved: 2026-03-13T23:10:00Z
---

## Current Focus
hypothesis: CONFIRMED - This is a known Dioxus CLI bug on Windows (GitHub Issue #5118). Testing workaround: use direct cargo build instead of dx CLI.
test: Run `cargo build --target x86_64-linux-android --release` to verify workaround works
expecting: Direct cargo build should succeed where dx CLI fails
next_action: Test cargo build release and document workaround

## Symptoms
expected: Android release APK should build successfully
actual: Build fails with linking error after ~173 seconds
errors: |
  error: linking with `C:\Users\ragma\.cargo\bin\dx.exe` failed: exit code: 1
  
  clang: error: no such file or directory: 'C:Usersragmadevshuseitargetx86_64-linux-androidandroid-releasedepsshusei-9028bdcb7d21b92c.shusei.b7d98a182bf5ec61-cgu.0.rcgu.o'
  clang: error: no such file or directory: 'C:UsersragmaAppDataLocalTemprustc8wr0iVliblibsqlite3_sys-ebd4da77c77eaefa.rlib'
  
  Note: Backslashes are STRIPPED from paths - this is a Windows path handling issue

reproduction: Run `dx build --android --release` on Windows
started: Started after quick task 004 (adding demo PDF). `cargo build --target x86_64-linux-android` (debug) works, but release build with dx CLI fails.

## Eliminated

## Evidence
- timestamp: 2026-03-13T00:00:00Z
  checked: .cargo/config.toml
  found: Uses .cmd files as linkers for Android targets (Windows batch scripts). x86_64-linux-android uses x86_64-linux-android21-clang.cmd
  implication: The .cmd files are batch scripts that call clang.exe with --target=x86_64-linux-android21

- timestamp: 2026-03-13T00:00:00Z
  checked: Cargo.toml release profile
  found: release profile has lto = true, codegen-units = 1, strip = true, panic = "abort"
  implication: LTO and single codegen-unit could affect linking behavior

- timestamp: 2026-03-13T00:00:00Z
  checked: Dioxus.toml
  found: Has [features] section mapping android = ["jni"], [bundle] section with identifier and resources
  implication: Dioxus CLI may have its own Android build configuration

- timestamp: 2026-03-13T00:00:00Z
  checked: Error message analysis
  found: dx.exe is being used as the linker (C:\Users\ragma\.cargo\bin\dx.exe), and all backslashes are stripped from paths
  implication: This is a critical bug - dx.exe is NOT a linker, it's the Dioxus CLI. The path stripping suggests Windows path handling issue in the build process.

- timestamp: 2026-03-13T00:00:00Z
  checked: GitHub Issues for Dioxus
  found: EXACT MATCH - GitHub Issue #5118 "Failed to build the template for Android" reports identical error on Windows. Error shows `dx.exe` as linker with backslash-stripped paths. Issue is OPEN and labeled: bug, cli, mobile, windows
  implication: This is a confirmed upstream bug in Dioxus CLI, not a project configuration issue

- timestamp: 2026-03-13T00:00:00Z
  checked: GitHub Issue #5118 workaround comment
  found: User reported: "On WSL2 (Linux), I redo the steps but it's successful and this issue can't be reproduced. The build system gives me an APK file"
  implication: The workaround is to use WSL2/Linux for Android release builds, OR use direct cargo build instead of dx CLI

- timestamp: 2026-03-13T00:00:00Z
  checked: Direct cargo build workaround
  found: `cargo build --target x86_64-linux-android --release` completed successfully in 3m 13s, producing release binary at target/x86_64-linux-android/release/shusei
  implication: WORKAROUND CONFIRMED - The issue is specifically with dx CLI, not the project configuration or NDK setup. Direct cargo build works for release Android builds on Windows.

## Resolution
root_cause: Known upstream bug in Dioxus CLI (GitHub Issue #5118). On Windows, `dx build --android --release` incorrectly uses `dx.exe` as the linker instead of the NDK clang.cmd. Additionally, Windows backslashes are stripped from paths during the build process, causing "no such file or directory" errors. This is a Windows-specific bug in the Dioxus CLI that affects Android release builds only - debug builds via `cargo build --target x86_64-linux-android` work correctly.
fix: WORKAROUND VERIFIED - Use direct `cargo build --target x86_64-linux-android --release` instead of `dx build --android --release`. The direct cargo build completed successfully in 3m 13s. For full APK bundling, either: (1) Use WSL2/Linux with dx CLI, or (2) Manually run gradle after cargo build. The upstream fix requires changes to Dioxus CLI's linker path handling for Windows Android builds.
verification: Direct cargo build `cargo build --target x86_64-linux-android --release` completed successfully in 3m 13s, producing the release binary. This confirms the issue is with dx CLI only, not the project configuration or NDK setup.
files_changed: []