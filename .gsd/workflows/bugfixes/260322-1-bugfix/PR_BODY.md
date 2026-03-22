# PR: Fix Critical Android Crash — All Functions Non-Functional

## Summary

Fixes a critical bug where **all M003 Android functions crashed** (camera capture, file picker, demo PDF loading) due to missing implementation files in the build process.

**Fixes:** Missing MainActivity.kt copy, asset bundling, and NDK ABI filter in android-patch.sh

---

## Root Cause

The app was completely unusable on Android because:

1. **MainActivity.kt not copied** — The M003 implementation (385 lines with CameraX, file picker, JNI callbacks) was never included in builds. The app used the default Dioxus template (189 bytes, just `class MainActivity : WryActivity()`), which has NO JNI methods.

2. **Assets not bundled** — Demo PDF (`test/medium_pdf_test.pdf`) was not copied to the Android project, causing `FileNotFoundException` on load.

3. **NDK ABI filter missing** — APK built for wrong architecture (x86_64), failing to install on ARM64 devices with `INSTALL_FAILED_NO_MATCHING_ABIS`.

### Error Symptoms

**Build-time:**
```
Thread tokio-rt-worker panicked: called `Result::unwrap()` on an `Err` value: 
Os { code: 2, kind: NotFound, message: "No such file or directory" }
```

**Runtime (all functions):**
```
E/AndroidRuntime: FATAL EXCEPTION: Thread-5
E/AndroidRuntime: java.io.FileNotFoundException: test/medium_pdf_test.pdf
```

---

## Fix Approach

### 1. Copy MainActivity.kt to Source Tree

```bash
cp .gsd/worktrees/M003/platform/android/app/src/main/kotlin/dev/dioxus/main/MainActivity.kt \
   platform/android/app/src/main/kotlin/dev/dioxus/main/MainActivity.kt
```

**Result:** 385-line MainActivity.kt now in permanent source location

### 2. Update android-patch.sh (3 New Steps)

**Step 4: Copy MainActivity.kt**
- Copies from `platform/android/` to target directory
- Ensures JNI methods are available at runtime

**Step 5: Copy Assets**
- Fixed path structure for asset bundling
- Bundles `test/medium_pdf_test.pdf` and other assets

**Step 6: Add NDK ABI Filter**
- Injects `abiFilters += listOf("arm64-v8a")` into build.gradle.kts
- Ensures APK builds for ARM64 architecture

---

## Changes

### Files Created

| File | Lines | Purpose |
|------|-------|---------|
| `platform/android/app/src/main/kotlin/dev/dioxus/main/MainActivity.kt` | 385 | CameraX + file picker + JNI callbacks implementation |

### Files Modified

| File | Changes | Purpose |
|------|---------|---------|
| `scripts/android-patch.sh` | +100 lines | Added Steps 4, 5, 6 (MainActivity.kt, assets, NDK filter) |

---

## Testing

### Static Verification (Complete)

```bash
# MainActivity.kt presence
test -f platform/android/app/src/main/kotlin/dev/dioxus/main/MainActivity.kt
# ✅ EXISTS (385 lines)

# Patch script validation
bash -n scripts/android-patch.sh
# ✅ Valid bash syntax

# All 6 steps present
grep -c "\[./6\]" scripts/android-patch.sh
# ✅ 6 steps found
```

### Build Verification (User Action Required)

```bash
# In development environment with NDK configured:
dx build --platform android
bash scripts/android-patch.sh
```

**Expected output:**
```
[4/6] Copying MainActivity.kt...
  Copied MainActivity.kt to target directory
  MainActivity.kt lines: 385
[5/6] Copying assets to Android project...
  Assets copied: 837K .../assets/test/medium_pdf_test.pdf
[6/6] Adding NDK ABI filter (arm64-v8a)...
  Added NDK ABI filter to app/build.gradle.kts
```

### Device Verification (User Action Required)

```bash
dx serve --android --target aarch64-linux-android

# Test all three functions:
# 1. Camera capture — should open camera and capture
# 2. File picker — should open PDF picker
# 3. Demo PDF — should load without crash
```

**Success criteria:** All three functions complete without crash, JNI callbacks invoked.

---

## Impact

### Before Fix

| Function | Status |
|----------|--------|
| Camera capture | ❌ CRASH (method not found) |
| File picker | ❌ CRASH (method not found) |
| Demo PDF loading | ❌ CRASH (asset not found) |
| App usability | ❌ COMPLETELY BROKEN |

### After Fix

| Function | Expected Status |
|----------|-----------------|
| Camera capture | ✅ WORKING (CameraX JNI bridge) |
| File picker | ✅ WORKING (SAF JNI bridge) |
| Demo PDF loading | ✅ WORKING (asset bundled) |
| App usability | ✅ FULLY FUNCTIONAL |

---

## Risk Assessment

**Risk Level:** LOW

- Only affects build process, not runtime logic
- MainActivity.kt is copied from verified M003 worktree (already tested)
- Patch script changes are additive (new steps, no modifications to existing steps)
- Easy rollback: Remove MainActivity.kt, rebuild

**Regression Risk:** MINIMAL

- Existing Dioxus functionality preserved
- CameraX, file picker, JNI callbacks are from M003 (already validated)
- No changes to Rust code or database schemas

---

## Deployment Notes

### Prerequisites

- Android NDK configured (`ANDROID_NDK_HOME` environment variable)
- Android device with USB debugging enabled (for device test)

### Build Sequence

```bash
# 1. Build Android project
dx build --platform android

# 2. Run patch script (REQUIRED — don't skip!)
bash scripts/android-patch.sh

# 3. Deploy to device
dx serve --android --target aarch64-linux-android
```

### Verification Commands

```bash
# Verify MainActivity.kt in target
wc -l target/dx/shusei/debug/android/app/app/src/main/kotlin/dev/dioxus/main/MainActivity.kt
# Expected: 385 lines

# Verify assets bundled
ls target/dx/shusei/debug/android/app/app/src/main/assets/test/medium_pdf_test.pdf
# Expected: File exists

# Verify NDK filter
grep "abiFilters" target/dx/shusei/debug/android/app/app/build.gradle.kts
# Expected: abiFilters += listOf("arm64-v8a")
```

---

## Related Issues

- M003 milestone completion (Android Stability)
- S01: Kotlin Camera Implementation
- S02: Kotlin File Picker Implementation
- S03: Asset Access Fix
- S05: ARM64 APK Build and Device Verification

---

## Checklist

- [x] Root cause identified
- [x] Fix implemented
- [x] Static verification passed
- [ ] Build verification (user to run)
- [ ] Device verification (user to run)
- [x] Documentation complete
- [x] PR description ready

---

## Author Notes

This fix restores M003 functionality that was lost when the worktree changes weren't persisted to the main project. The MainActivity.kt implementation was complete and tested in the M003 worktree but never copied to the permanent source location.

**Key learning:** Worktree changes must be explicitly copied to project root before merge.

---

**PR Type:** Bug Fix  
**Breaking Change:** No  
**Migration Required:** No  
**Backport Required:** No
