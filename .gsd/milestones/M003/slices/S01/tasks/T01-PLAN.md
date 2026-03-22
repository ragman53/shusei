---
estimated_steps: 5
estimated_files: 2
skills_used:
  - debug-like-expert
---

# T01: Add CameraX dependencies to Gradle and implement patch

**Slice:** S01 — Kotlin Camera Implementation
**Milestone:** M003

## Description

Extend the android-patch.sh script to add CameraX dependencies to the Dioxus-generated build.gradle.kts. The current patch script only fixes Java version and lint settings; it needs to also inject CameraX libraries (camera-core, camera-camera2, camera-lifecycle, camera-view) required for camera capture functionality.

## Steps

1. Read current `scripts/android-patch.sh` to understand the patch structure
2. Add CameraX dependency injection section to android-patch.sh:
   - Define CameraX version (1.3.4 as of 2024)
   - Append dependencies block to `app/build.gradle.kts` after existing dependencies
3. Add implementation statement for CameraX in the dependencies block:
   - `implementation("androidx.camera:camera-core:$camerax_version")`
   - `implementation("androidx.camera:camera-camera2:$camerax_version")`
   - `implementation("androidx.camera:camera-lifecycle:$camerax_version")`
   - `implementation("androidx.camera:camera-view:$camerax_version")`
4. Test the patch by running `dx build --platform android && bash scripts/android-patch.sh`
5. Verify CameraX dependencies appear in generated build.gradle.kts

## Must-Haves

- [ ] CameraX dependencies added to android-patch.sh
- [ ] Patch script correctly appends dependencies without breaking existing content
- [ ] Generated build.gradle.kts contains all four CameraX libraries after patch

## Verification

- `grep -q "camera-core" scripts/android-patch.sh`
- After build and patch: `grep -q "androidx.camera:camera-core" target/dx/shusei/debug/android/app/app/build.gradle.kts`

## Inputs

- `scripts/android-patch.sh` — Current patch script to extend

## Expected Output

- `scripts/android-patch.sh` — Modified with CameraX dependency injection section

## Observability Impact

- **Runtime signals changed**: None (build-time dependency injection only)
- **How to inspect**: 
  - `grep "androidx.camera" scripts/android-patch.sh` — verify patch script contains CameraX deps
  - `grep "androidx.camera" target/dx/shusei/debug/android/app/app/build.gradle.kts` — verify generated file has deps
- **Failure visibility**: 
  - Patch script exits with error if target directory not found
  - Gradle build fails if CameraX dependencies are malformed or version incompatible
  - AWK insertion fails silently if dependencies block format differs from expected