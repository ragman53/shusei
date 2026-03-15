---
estimated_steps: 4
estimated_files: 1
---

# T03: Install APK on Moto G66j 5G

**Slice:** S01 — Android Build + Deploy
**Milestone:** M002-dbrk2n

## Description

Install debug APK on target device (Motorola Moto G66j 5G) via adb. Handle any install errors.

## Steps

1. Enable USB debugging on Moto G66j 5G (Settings → Developer options → USB debugging)
2. Connect device via USB, verify connection: `adb devices`
3. Install APK: `adb install -r target/dx/shusei/debug/android/app/app/build/outputs/apk/debug/app-debug.apk`
4. Verify installation: `adb shell pm list packages | grep com.shusei.app`
5. Handle errors if any:
   - INSTALL_FAILED_UPDATE_INCOMPATIBLE: uninstall first, then reinstall
   - INSTALL_FAILED_INSUFFICIENT_STORAGE: free device storage
   - device unauthorized: check device for authorization prompt

## Must-Haves

- [ ] Device connected and visible via `adb devices`
- [ ] APK installs without errors
- [ ] Package visible in installed packages list
- [ ] App icon appears in device app drawer

## Verification

- `adb shell pm list packages | grep com.shusei.app` returns `package:com.shusei.app`
- App icon visible in device app drawer
- `adb shell dumpsys package com.shusei.app` shows package info

## Observability Impact

- Signals added/changed: None (deployment task)
- How a future agent inspects this: `adb shell pm list packages`, `adb logcat` for install logs
- Failure state exposed: adb install error messages, device storage/permission issues

## Inputs

- Debug APK from T01/T02
- Moto G66j 5G device with USB debugging enabled

## Expected Output

- APK installed on device
- Package com.shusei.app registered in Android package manager
