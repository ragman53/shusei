---
id: S05
parent: M003
milestone: M003
provides: []
requires:
  - slice: S01
    provides: Camera implementation
  - slice: S02
    provides: File picker implementation
  - slice: S03
    provides: Asset bundling
  - slice: S04
    provides: Integration verification infrastructure
affects: []
key_files: []
key_decisions: []
patterns_established: []
observability_surfaces: []
drill_down_paths: []
duration: 0h
verification_result: not_started
completed_at: null
---

# S05: ARM64 APK Build and Device Verification

**NOT YET STARTED**

## Status

S05 is planned but not yet implemented. This slice is responsible for:

1. Configuring ARM64 APK build (`CARGO_BUILD_TARGET=aarch64-linux-android`)
2. Adding NDK ABI filter to build.gradle.kts (`abiFilters += listOf("arm64-v8a")`)
3. Rebuilding APK with ARM64 native libraries
4. Running full integration verification on Moto G66j 5G device
5. Achieving "M003 VERIFICATION PASSED" status

## Blocker from S04

S04 UAT execution failed due to ABI mismatch:
- APK architecture: x86_64
- Device architecture: arm64-v8a (Moto G66j 5G)
- Error: `INSTALL_FAILED_NO_MATCHING_ABIS`

## Next Steps

1. Set `CARGO_BUILD_TARGET=aarch64-linux-android`
2. Rebuild APK: `dx build --platform android`
3. Run patch script: `bash scripts/android-patch.sh`
4. Verify APK architecture: `unzip -l app-debug.apk | grep lib/arm64-v8a`
5. Run integration verification: `bash scripts/verify-s04-integration.sh`
6. Document results in S05-UAT.md
