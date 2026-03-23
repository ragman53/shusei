---
estimated_steps: 6
estimated_files: 2
skills_used:
  - debug-like-expert
---

# T02: Build APK and Verify on Device

**Slice:** S07 — JNI Symbol Fix
**Milestone:** M003

## Description

Build the ARM64 APK with the new JNI bindings and verify the app launches without `UnsatisfiedLinkError` on the Moto G66j 5G device. This is the critical validation that the fix works.

## Steps

1. Run `bash scripts/android-build.sh` to build the APK with updated Rust library
2. Verify APK contains arm64-v8a native library: `unzip -l target/dx/shusei/debug/android/app/app/build/outputs/apk/debug/app-debug.apk | grep "lib/arm64-v8a/libdioxusmain.so"`
3. Connect Moto G66j 5G device via USB and verify with `adb devices`
4. Install APK: `adb install -r target/dx/shusei/debug/android/app/app/build/outputs/apk/debug/app-debug.apk`
5. Clear logcat: `adb logcat -c`
6. Launch app: `adb shell am start -n com.shusei.app/.MainActivity`
7. Monitor logcat for success signals: `adb logcat -d | grep -E "WryActivity|MainActivity|UnsatisfiedLinkError|Shusei"`
8. Verify NO `UnsatisfiedLinkError` in logs and WryActivity lifecycle methods ARE called

## Must-Haves

- [ ] APK built successfully with `bash scripts/android-build.sh`
- [ ] APK installs on device without error
- [ ] App launches without crash (no `UnsatisfiedLinkError`)
- [ ] Logcat shows WryActivity lifecycle methods being called
- [ ] No JNI symbol errors in logcat

## Verification

- `adb logcat -d | grep "UnsatisfiedLinkError"` returns empty (no crash)
- `adb logcat -d | grep "WryActivity.create"` returns at least 1 match (lifecycle called)
- `adb shell pm list packages | grep com.shusei.app` returns the installed package

## Observability Impact

- Signals added: WryActivity lifecycle log messages (create, start, resume, etc.)
- How a future agent inspects this: `adb logcat | grep -E "WryActivity|Shusei|MainActivity"`
- Failure state exposed: `UnsatisfiedLinkError` with method name, or absence of lifecycle logs indicates bindings not called

## Inputs

- `src/platform/android_bindings.rs` — JNI bindings created in T01
- `scripts/android-build.sh` — Build script to compile and package APK
- `target/dx/shusei/debug/android/app/app/build/outputs/apk/debug/app-debug.apk` — APK to install

## Expected Output

- `target/dx/shusei/debug/android/app/app/build/outputs/apk/debug/app-debug.apk` — APK with JNI symbols
- Device logcat showing successful app launch without `UnsatisfiedLinkError`