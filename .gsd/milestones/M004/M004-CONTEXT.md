# M004: Tauri + Leptos Migration — Context

## Scope

M004 migrates the app from Dioxus 0.7 to Tauri v2 + Leptos v0.7, targeting the same Android deployment with improved ecosystem maturity and performance.

## Goals

1. **Tauri v2 setup** — Initialize Tauri project with Android mobile support
2. **Leptos integration** — Set up Leptos frontend with SSR/hydration for mobile
3. **Database migration** — Port existing SQLite plugin to Tauri plugin architecture
4. **OCR integration** — Ensure tract-onnx engines work in Tauri backend
5. **Android deployment** — Build and deploy APK to Moto G66j 5G

## Constraints

- **Offline-first** — Must remain 100% offline, no external API dependencies
- **Model bundling** — NDLOCR models (230MB) must be bundled in APK
- **Performance** — APK size should not exceed current 360MB (ideally smaller)
- **Feature parity** — All M002/M003 features must work after migration

## Out of Scope

- New features (AI definitions, voice memo) — deferred to M005
- iOS deployment — Android-only for M004
- Desktop builds — Mobile-only focus

## Key Decisions

### Why Tauri + Leptos?

**Dioxus limitations:**
- Smaller ecosystem, fewer community resources
- Android support still maturing (0.7.x)
- Limited documentation for mobile deployment
- Hook-based reactivity less intuitive for complex state

**Tauri + Leptos advantages:**
- Mature ecosystem (Tauri v2 stable, strong community)
- Better Android support with documented mobile workflow
- Leptos signals-based reactivity (fine-grained, performant)
- Smaller bundle sizes (Tauri uses system webview)
- Strong TypeScript tooling for frontend
- Active development and regular releases

**Tradeoffs:**
- Learning curve for Leptos signals vs Dioxus hooks
- Migration effort for existing UI components
- Tauri plugin architecture differs from Dioxus JNI bridge

## Migration Strategy

1. **Parallel development** — Keep Dioxus version working during migration
2. **Incremental移植** — Port one slice at a time (DB → OCR → UI)
3. **Verification at each step** — Ensure each component works before moving on
4. **Fallback plan** — If Tauri blocks progress, can revert to Dioxus

## Success Metrics

- [ ] Tauri project builds: `pnpm tauri build --target android`
- [ ] Leptos frontend compiles: `cargo leptos build --release`
- [ ] Database plugin works: existing schema + queries functional
- [ ] OCR engines load: tract-onnx models load without errors
- [ ] APK deploys: installs on Moto G66j 5G, no crashes
- [ ] Bundle size: ≤360MB (current Dioxus APK size)
