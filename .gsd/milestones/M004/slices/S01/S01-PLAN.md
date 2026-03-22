# S01: Tauri Project Setup

**Risk:** low  
**Dependencies:** none  
**Estimate:** 1-2 days

## Goals

1. Initialize Tauri v2 project with proper configuration
2. Configure Android mobile support
3. Set up Leptos v0.7 frontend skeleton
4. Verify basic build works for Android target

## Tasks

- [ ] **T01: Tauri v2 Initialization** `est:2h`
  - Install Tauri CLI v2
  - Initialize Tauri project with `pnpm create tauri-app`
  - Configure `tauri.conf.json` for Android
  - Set up package.json with pnpm

- [ ] **T02: Leptos v0.7 Integration** `est:3h`
  - Add Leptos dependencies to Cargo.toml
  - Configure cargo-leptos for SSR/hydration
  - Create basic Leptos app structure
  - Wire up Tauri → Leptos communication

- [ ] **T03: Android Build Configuration** `est:4h`
  - Configure Android SDK/NDK paths
  - Set up `AndroidManifest.xml` with permissions
  - Configure Gradle build files
  - Test `pnpm tauri android init`

- [ ] **T04: Build Verification** `est:2h`
  - Run `pnpm tauri android build --apk`
  - Verify APK structure (models not bundled yet)
  - Document any build errors and fixes

## Success Criteria

- [ ] `pnpm tauri android init` completes without errors
- [ ] `pnpm tauri android build --apk` produces valid APK
- [ ] Leptos frontend compiles with `cargo leptos build`
- [ ] Basic Tauri ↔ Leptos communication works (command invocation)

## Verification

```bash
# Tauri version check
pnpm tauri --version

# Build Android APK
pnpm tauri android build --apk

# Verify APK structure
unzip -l src-tauri/target/aarch64-linux-android/release/app.apk | head -20
```

## Known Issues / Notes

- Tauri v2 requires Node.js 18+
- pnpm recommended over npm/yarn for Tauri
- Android SDK r29+ and NDK r26+ required (already installed from M003)
- Leptos v0.7 uses signals-based reactivity (different from Dioxus hooks)
