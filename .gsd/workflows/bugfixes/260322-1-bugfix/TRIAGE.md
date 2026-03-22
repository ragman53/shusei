# Bug Triage Report — All Functions Crash (Missing MainActivity.kt + Assets)

**Date:** 2026-03-22  
**Workflow:** bugfix/260322-1-bugfix  
**Severity:** Critical (app is unusable — all functions crash)

---

## Bug Description

**ALL functions crash on Android device:**
- Camera capture — crashes
- File picker — crashes  
- Demo PDF loading — crashes

### Error Messages

**Build-time panic:**
```
18:40:07 [dev] Thread tokio-rt-worker panicked at /home/devuser/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/dioxus-cli-0.7.3/src/build/request.rs:5176:26:
               called `Result::unwrap()` on an `Err` value: Os { code: 2, kind: NotFound, message: "No such file or directory" }
```

**Runtime crash (all functions):**
```
E/AndroidRuntime: FATAL EXCEPTION: Thread-5
E/AndroidRuntime: java.io.FileNotFoundException: test/medium_pdf_test.pdf
```

### Expected Behavior
- Camera capture should invoke `startCameraCapture()` → CameraX → `onImageCaptured()` callback
- File picker should invoke `pickPdfFile()` → SAF → `onFilePicked()` callback
- Demo PDF should load from bundled assets via `copy_asset_to_files()`

### Actual Behavior
All functions crash because the **M003 MainActivity.kt implementation is not included in the build**. The app uses the default Dioxus-generated `MainActivity.kt` (189 bytes, just `class MainActivity : WryActivity()`) which has NO JNI methods.

---

## Root Cause Analysis

**CRITICAL ROOT CAUSE:** The patch script `scripts/android-patch.sh` is missing the step to copy the M003 `MainActivity.kt` implementation to the target directory.

### Why This Happens

1. **Missing MainActivity.kt copy step** — The M003 implementation (385 lines with CameraX, file picker, JNI callbacks) exists at:
   - `.gsd/worktrees/M003/platform/android/app/src/main/kotlin/dev/dioxus/main/MainActivity.kt`
   
   But the patch script does NOT copy it to:
   - `target/dx/shusei/debug/android/app/app/src/main/kotlin/dev/dioxus/main/MainActivity.kt`

2. **Dioxus generates default MainActivity** — When `dx build --platform android` runs, it generates a minimal `MainActivity.kt`:
   ```kotlin
   package dev.dioxus.main;
   class MainActivity : WryActivity()
   ```
   This has NO JNI methods (`startCameraCapture`, `pickPdfFile`, etc.)

3. **JNI calls fail silently** — When Rust code calls `startCameraCapture()` via JNI, the method doesn't exist, causing a crash.

4. **Assets not bundled** — Step 5 (asset copying) also has incorrect path structure.

### Current vs Required Patch Script

**Current script steps:**
1. Fix Java version (17)
2. Remove deprecated manifest attribute
3. Add CameraX dependencies
4. Disable lint tasks
5. Copy assets (broken path)

**Missing step:**
- **Copy MainActivity.kt** — Should copy from `platform/android/app/src/main/kotlin/...` to target directory

### Affected Code

| File | Issue | Impact |
|------|-------|--------|
| `scripts/android-patch.sh` | Missing MainActivity.kt copy step | ALL JNI methods missing |
| `scripts/android-patch.sh` Step 5 | Wrong asset destination path | Assets not bundled |
| `target/.../MainActivity.kt` | Default Dioxus template (189 bytes) | No CameraX, no file picker, no JNI callbacks |
| `src/platform/android.rs` | Correctly calls JNI methods | Methods don't exist in Kotlin |
| `src/ui/library.rs` | Correctly calls `copy_asset_to_files()` | Asset not bundled |

---

## Reproduction Steps

1. Run `dx serve --android --target aarch64-linux-android`
2. Dioxus generates default `MainActivity.kt` (189 bytes)
3. Patch script runs but doesn't copy M003 MainActivity.kt
4. App launches with no JNI methods
5. Tap ANY function button (camera, file picker, demo PDF)
6. **Crash**: Method not found or asset not found

### Verification Commands

```bash
# Check if M003 MainActivity.kt exists in source
ls -la platform/android/app/src/main/kotlin/dev/dioxus/main/MainActivity.kt
# Result: NOT FOUND (doesn't exist in worktree)

# Check target MainActivity.kt
ls -la target/dx/shusei/debug/android/app/app/src/main/kotlin/dev/dioxus/main/MainActivity.kt
# Result: 189 bytes (default Dioxus template)

# Check if patch script copies MainActivity.kt
grep -i "MainActivity.kt" scripts/android-patch.sh
# Result: NO MATCH (step missing)
```

---

## Blast Radius Assessment

**CRITICAL: All M003 functionality is broken**

| Feature | Status | Reason |
|---------|--------|--------|
| Camera capture | ❌ CRASH | `startCameraCapture()` method doesn't exist |
| File picker | ❌ CRASH | `pickPdfFile()` method doesn't exist |
| Demo PDF loading | ❌ CRASH | Asset not bundled + method doesn't exist |
| Permission handling | ❌ CRASH | `hasCameraPermission()`, `requestCameraPermission()` don't exist |
| Vibration | ❌ CRASH | `vibrate()` method doesn't exist |
| JNI callbacks | ❌ CRASH | `onImageCaptured`, `onFilePicked` not implemented |

**User Impact:**
- **App is completely unusable** on Android
- All M003 stability improvements are non-functional
- Cannot test or demo any features

---

## Proposed Fix Approach

### Immediate Fix: Restore Patch Script Steps

**Fix:** Add missing steps to `scripts/android-patch.sh`:

1. **Step 4: Copy MainActivity.kt** — Copy from `platform/android/app/src/main/kotlin/dev/dioxus/main/MainActivity.kt` to target directory
2. **Step 5: Copy assets** — Fix path to copy from `assets/` to `target/.../app/src/main/assets/`
3. **Step 6: Add NDK ABI filter** — Inject `abiFilters += listOf("arm64-v8a")` for ARM64 devices

**Changes to patch script:**
```bash
# Step 4: Copy MainActivity.kt
echo "[4/6] Copying MainActivity.kt..."
cp "$PROJECT_ROOT/platform/android/app/src/main/kotlin/dev/dioxus/main/MainActivity.kt" \
   "$ANDROID_DIR/app/src/main/kotlin/dev/dioxus/main/MainActivity.kt"

# Step 5: Copy assets (fixed path)
echo "[5/6] Copying assets..."
cp -r "$PROJECT_ROOT/assets" "$ANDROID_DIR/app/src/main/assets/"

# Step 6: Add NDK ABI filter
echo "[6/6] Adding NDK ABI filter..."
# AWK injection for abiFilters
```

### Source File Location Issue

**Problem:** The M003 MainActivity.kt is in the worktree (`.gsd/worktrees/M003/`) but the patch script expects it at `platform/android/`.

**Solution:** Copy MainActivity.kt from worktree to project root:
```bash
cp .gsd/worktrees/M003/platform/android/app/src/main/kotlin/dev/dioxus/main/MainActivity.kt \
   platform/android/app/src/main/kotlin/dev/dioxus/main/MainActivity.kt
```

---

## Files to Modify

1. `platform/android/app/src/main/kotlin/dev/dioxus/main/MainActivity.kt` — Copy from worktree
2. `scripts/android-patch.sh` — Add Steps 4, 5, 6 (MainActivity.kt, assets, NDK filter)
3. `.gsd/workflows/bugfixes/260322-1-bugfix/FIX.md` — Document the fix

---

## Verification Plan

1. **Copy MainActivity.kt** from worktree to `platform/android/`
2. **Update patch script** with Steps 4, 5, 6
3. **Rebuild**: `dx build --platform android && bash scripts/android-patch.sh`
4. **Verify MainActivity.kt**: Check target file is 385 lines (not 189 bytes)
5. **Verify assets**: Check `target/.../assets/test/medium_pdf_test.pdf` exists
6. **Deploy**: `dx serve --android`
7. **Test all functions**:
   - Camera capture — should open camera, capture image
   - File picker — should open PDF picker
   - Demo PDF — should load without crash

---

## Gate: User Confirmation Required

**Proposed fix:**
1. Copy MainActivity.kt from worktree to `platform/android/`
2. Update patch script with 3 missing steps (MainActivity.kt copy, asset copy, NDK filter)
3. Rebuild and test all three functions

**Estimated effort:** 1 hour  
**Risk:** Medium (requires correct file paths and patch script updates)

**Proceed with this fix?** (yes/no)
