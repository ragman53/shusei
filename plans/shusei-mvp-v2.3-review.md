# Shusei MVP v2.3 Pre-Implementation Review

**Review Date**: 2026-03-06  
**Specification Version**: v2.3  
**Reviewer**: Architect Mode  
**Status**: 🔍 Pre-Implementation Review

---

## Executive Summary

This document provides a comprehensive pre-implementation review of the Shusei MVP v2.3 specification. The specification outlines a **Pure Rust + Dioxus** multi-platform reading application with offline OCR and STT capabilities.

### Overall Assessment: ✅ **Ready for Implementation** (with noted considerations)

The specification is well-structured, technically sound, and demonstrates thoughtful architecture decisions. The migration from Tauri + Leptos to Dioxus is justified with clear benefits.

---

## 1. Technical Architecture Review

### 1.1 Dioxus Migration Decision ✅ **Well Justified**

| Aspect | Assessment |
|--------|------------|
| Single codebase | ✅ Strong benefit - eliminates Tauri/Leptos dual configuration |
| Platform support | ✅ Built-in Android/iOS/Desktop/Web from single crate |
| Hot reload | ✅ Subsecond Rust hot-patching is significant DX improvement |
| Direct JNI access | ✅ Simpler than Tauri Plugin architecture |
| TailwindCSS | ✅ Zero configuration reduces setup complexity |

**Concerns Noted**:
- ⚠️ Dioxus Mobile (Android) is still maturing - validation in Week 1 is critical
- ⚠️ WebView renderer on mobile may have performance implications for complex markdown

**Recommendation**: Proceed with Week 1 validation gate. The fallback to v2.2 (Tauri + Leptos) is well-defined.

### 1.2 Code Structure ✅ **Excellent Separation of Concerns**

```
core/      → 100% platform-agnostic (OCR, STT, DB, vocab)
ui/        → 95% shared Dioxus components
platform/  → Platform-specific bridges (camera/mic/file)
```

This architecture enables:
1. Easy testing of core logic without platform dependencies
2. Clean fallback path if Dioxus proves unstable
3. Future platform expansion with minimal code changes

### 1.3 tract ONNX Runtime ✅ **Good Choice**

Using `tract` as the unified inference runtime for both OCR and STT:
- **Pros**: Pure Rust, no C/C++ dependencies, single runtime for all models
- **Cons**: May have limited operator coverage compared to ONNX Runtime

**Risk**: tract compatibility with NDLOCR-Lite and Moonshine ONNX models
**Mitigation**: Week 1 validation gate explicitly tests this

---

## 2. Dependency Analysis

### 2.1 Core Dependencies Review

| Crate | Version | Assessment | Notes |
|-------|---------|------------|-------|
| `dioxus` | 0.7 | ⚠️ Verify mobile features | Mobile support still maturing |
| `tract-onnx` | 0.21 | ✅ Stable | Well-maintained Pure Rust ONNX |
| `pdfium-render` | 0.8 | ⚠️ Check Android | Uses FFI to pdfium - verify Android builds |
| `rusqlite` | 0.32 | ✅ Stable | bundled feature ensures no system dependency |
| `lindera` | 0.34 | ✅ Good | IPAdic bundled, Pure Rust |
| `image` | 0.25 | ✅ Stable | Standard image processing |
| `ndarray` | 0.16 | ✅ Stable | Required for tensor operations |
| `hound` | 3.5 | ✅ Stable | WAV file handling |
| `tokenizers` | 0.20 | ✅ HuggingFace | Standard tokenizer library |

### 2.2 Optional Dependencies

| Crate | Purpose | Assessment |
|-------|---------|------------|
| `jni` | Android bridge | ✅ Standard JNI bindings |

### 2.3 Missing Dependencies (Consider Adding)

| Missing | Purpose | Recommendation |
|---------|---------|----------------|
| `thiserror` | Error types | Add for clean error handling |
| `tracing` / `log` | Logging | `log` is listed, consider `tracing` for async context |
| `once_cell` | Lazy static | May need for model loading |
| `parking_lot` | Better mutex | Optional performance improvement |

---

## 3. Database Schema Review

### 3.1 Schema Design ✅ **Well Designed**

**sticky_notes table**:
- ✅ FTS5 virtual table for full-text search
- ✅ Separate `ocr_text_plain` for FTS (avoids markdown in search)
- ✅ Timestamps with defaults

**books / book_pages tables**:
- ✅ Progress tracking with `converted_pages`
- ✅ Reading position with `last_read_pos`
- ✅ Proper foreign key relationship

**vocabulary table**:
- ✅ Source tracking (`source_book`, `source_page`)
- ✅ Review counters for SRS potential
- ✅ Index on `word` column

### 3.2 Schema Recommendations

```sql
-- Consider adding:
CREATE INDEX idx_sticky_notes_book ON sticky_notes(book_title);
CREATE INDEX idx_sticky_notes_created ON sticky_notes(created_at DESC);
CREATE INDEX idx_books_updated ON books(updated_at DESC);
```

---

## 4. Model & Asset Planning

### 4.1 Model Sizes

| Model | Disk Size | Memory (Peak) | Assessment |
|-------|-----------|---------------|------------|
| NDLOCR-Lite | ~70MB | ~450MB | ✅ Within target device capabilities |
| Moonshine Tiny (x2) | ~60MB | ~200MB | ✅ One language loaded at a time |
| lindera dictionary | ~50MB | - | ✅ Bundled with crate |

**Total App Size**: ~250MB (stated) - reasonable for feature-rich offline app

### 4.2 Memory Management

| State | Target Memory | Strategy |
|-------|--------------|----------|
| Idle | ~100MB | Models unloaded |
| OCR active | ~500MB | Load OCR models on demand |
| STT active | ~200MB | Load STT models on demand |

**Recommendation**: Implement model lazy-loading and explicit unloading to stay within Moto G66j 8GB RAM.

---

## 5. Development Timeline Review

### 5.1 12-Week Roadmap Analysis

| Phase | Duration | Scope | Feasibility |
|-------|----------|-------|-------------|
| Week 1-2 | 2 weeks | Foundation + validation | ⚠️ Tight - tract/NDLOCR validation critical |
| Week 3-5 | 3 weeks | OCR pipeline | ✅ Reasonable |
| Week 5-6 | 2 weeks | Camera + Phase 1 | ⚠️ Overlap with OCR, JNI complexity |
| Week 7-8 | 2 weeks | PDF + Phase 2 | ✅ Reasonable |
| Week 9-10 | 2 weeks | STT + Phase 3 | ⚠️ Decoder loop complexity |
| Week 11-12 | 2 weeks | Vocab + Polish | ✅ Reasonable |

### 5.2 Risk-Heavy Weeks

**Week 1-2 (Critical Validation Gate)**:
- Dioxus Android stability
- tract NDLOCR compatibility
- tract Moonshine compatibility
- JNI camera access

**Week 5-6 (Overlap Risk)**:
- OCR completion may spill over
- JNI camera integration could have unexpected complexity

**Week 9-10 (Technical Complexity)**:
- Autoregressive decoder with KV cache
- JNI microphone access
- Audio preprocessing pipeline

---

## 6. Risk Assessment Matrix

### 6.1 High-Risk Items (from spec)

| Risk | Probability | Impact | Mitigation | Assessment |
|------|-------------|--------|------------|------------|
| Dioxus Mobile unstable | Medium | High | Fallback to Tauri v2 | ✅ Addressed |
| tract NDLOCR ops gap | Medium | High | onnx-simplifier → PaddleOCR | ✅ Addressed |
| tract Moonshine ops gap | Low-Medium | Medium | Whisper Tiny ONNX fallback | ✅ Addressed |
| JNI camera/mic complexity | Medium | Medium | Intent-based fallback | ✅ Addressed |

### 6.2 Additional Risks Identified

| Risk | Probability | Impact | Recommendation |
|------|-------------|--------|----------------|
| pdfium-render Android build issues | Medium | Medium | Test early; have `pdf-extract` backup |
| WebView performance on markdown | Low | Medium | Virtual scrolling; lazy rendering |
| Moonshine-ja licensing (>$1M revenue) | Low | High | Document clearly; consider Whisper for commercial |
| Bundle size >250MB | Medium | Low | Model compression; on-demand download |

---

## 7. Gap Analysis

### 7.1 Specification Gaps Identified

| Gap | Section | Recommendation |
|-----|---------|----------------|
| No error handling strategy defined | Throughout | Add error handling architecture section |
| No offline model download strategy | Section 3 | Define first-run experience for model loading |
| No data migration strategy | Section 7 | Define schema versioning for updates |
| No accessibility (a11y) considerations | Section 7 | Add screen reader support for reading view |
| No unit/integration test strategy | Section 10 | Add testing milestones |
| No CI/CD pipeline definition | Section 10 | Add automated build/test pipeline |

### 7.2 Technical Clarifications Needed

1. **Image preprocessing pipeline details**: What specific preprocessing steps? (resize, normalize, binarize?)
2. **Moonshine tokenizer format**: Is it SentencePiece or BPE? Does `tokenizers` crate support it?
3. **Reading order algorithm**: Vertical text reading order - what algorithm?
4. **Audio preprocessing format**: 16kHz mono PCM - is hound sufficient or need additional conversion?

### 7.3 Minor Inconsistencies

| Location | Issue | Correction |
|----------|-------|------------|
| Section 4.3 | `Dioxus.toml` - should be `Dioxus.toml` or `dioxus.toml`? | Verify with Dioxus 0.7 docs |
| Section 12 | `features = ["mobile", "desktop", "web"]` | May need platform-specific feature flags |
| Section 10 | Week 5-6 shows overlap | Clarify if Week 5 ends OCR or continues |

---

## 8. Licensing Compliance Review

| Component | License | Commercial Use | Notes |
|-----------|---------|----------------|-------|
| shusei app | TBD | - | Define before distribution |
| Dioxus | MIT/Apache 2.0 | ✅ | Standard OSI approved |
| tract | Apache 2.0/MIT | ✅ | Standard OSI approved |
| NDLOCR-Lite | CC BY 4.0 | ✅ | Attribution required |
| Moonshine Tiny (EN) | MIT | ✅ | Standard OSI approved |
| Moonshine Tiny-ja | Moonshine Community License | ⚠️ | Revenue limit: $1M/year |
| lindera | MIT | ✅ | Standard OSI approved |

**Recommendation**: Document attribution requirements for NDLOCR-Lite (CC BY 4.0) in app credits. Monitor revenue if commercializing beyond Moonshine-ja license limits.

---

## 9. Recommendations Summary

### 9.1 Before Implementation

1. ✅ **Validate Week 1 gates rigorously** - This is the most critical week
2. 📝 Add error handling architecture to spec
3. 📝 Define model first-run download strategy
4. 📝 Add test strategy (unit, integration, E2E)
5. 📝 Set up CI/CD pipeline from day 1

### 9.2 During Implementation

1. Start with `core/` layer - can be developed and tested without Dioxus
2. Create mock platform layer for early development
3. Implement model lazy-loading from the start
4. Add tracing/logging infrastructure early
5. Consider performance benchmarking infrastructure

### 9.3 Post-MVP

1. Document lessons learned for iOS port
2. Evaluate WebView vs WGPU renderer performance
3. Consider model compression for smaller bundle

---

## 10. Decision Checklist

Before proceeding to implementation, confirm:

- [ ] **Week 1 Validation Plan Ready**: Dioxus Android, tract NDLOCR, tract Moonshine, JNI camera
- [ ] **Fallback Path Clear**: v2.2 Tauri + Leptos spec available
- [ ] **Development Environment**: Moto G66j device or equivalent available
- [ ] **Model Files**: NDLOCR-Lite ONNX and Moonshine ONNX files accessible
- [ ] **License Attribution**: CC BY 4.0 attribution plan for NDLOCR

---

## 11. Verdict

### Overall Assessment: ✅ **APPROVED FOR IMPLEMENTATION**

The specification is comprehensive, well-researched, and demonstrates strong architectural thinking. The migration to Dioxus from Tauri + Leptos is justified. The fallback strategy to v2.2 provides appropriate risk mitigation.

### Critical Path Items:

1. **Week 1 validation must pass** - Do not proceed if tract compatibility fails
2. **JNI camera access PoC** - Core functionality depends on this
3. **Model memory management** - Critical for 8GB RAM target device

### Suggested Next Steps:

1. Set up development environment and CI/CD
2. Execute Week 1 validation gates
3. Make go/no-go decision based on validation results
4. Proceed to Code mode for implementation

---

*Review completed: 2026-03-06*