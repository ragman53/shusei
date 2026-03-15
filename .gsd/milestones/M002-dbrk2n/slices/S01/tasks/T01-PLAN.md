---
estimated_steps: 8
estimated_files: 3
---

# T01: Create Gradle patch script

**Slice:** S01 — Android Build + Deploy
**Milestone:** M002-dbrk2n

## Description

Create post-generation patch script that fixes Dioxus-generated Gradle files for modern Android tooling. Based on GitHub issue #5251 workaround.

## Steps

1. Create `scripts/android-patch.sh` with sed commands to fix:
   - Java version: VERSION_1_8 → VERSION_21
   - Kotlin JVM target: jvmTarget = "1.8" → jvmTarget = "21"
   - Manifest: Remove android:extractNativeLibs="false"
   - Skip lint tasks in gradlew command
2. Create `scripts/android-build.sh` wrapper that:
   - Runs `dx build --platform android` (generates files)
   - Applies patch script
   - Runs gradlew assembleRelease with lint skipped
3. Make scripts executable: `chmod +x scripts/*.sh`
4. Test build locally (desktop or emulator)
5. Document usage in script headers

## Must-Haves

- [ ] Patch script fixes all three issues (Java version, manifest, lint)
- [ ] Build wrapper script runs dx build then applies patch automatically
- [ ] Scripts are executable and documented
- [ ] Build completes without Gradle errors

## Verification

- `bash scripts/android-build.sh` exits with code 0
- Generated APK exists at `target/dx/shusei/debug/android/app/app/build/outputs/apk/debug/app-debug.apk`
- No Java version warnings in build output
- No lintVital errors in build output

## Observability Impact

- Signals added/changed: Build script logs patch application steps
- How a future agent inspects this: Check `scripts/android-build.sh` output, review `target/dx/*/build.gradle.kts` for Java version
- Failure state exposed: Script exits non-zero on patch failure, logs which sed command failed

## Inputs

- GitHub issue #5251 workaround script — reference implementation for patch commands
- Dioxus 0.7.3 generated Gradle files — target of patch

## Expected Output

- `scripts/android-patch.sh` — Patch script with sed commands
- `scripts/android-build.sh` — Wrapper build script
- Working debug APK at expected path
