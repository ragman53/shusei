# M002-dbrk2n: Android Prototype

**Gathered:** 2026-03-15
**Status:** Ready for planning

## Project Description

完全オフラインの読書アプリ。紙の本も PDF もスマホ 1 台で「付箋＋ボイスメモ＋単語採集」ができる。Dioxus+Rust で Android 実装。プライバシー 100%、外部 API 完全不使用。

## Why This Milestone

M001 でバックエンドインフラ（データベース、OCR/STT/AI エンジン、PDF 処理）が完成した。M002 では実機（Motorola Moto G66j 5G）で動作するプロトタイプを完成させ、ユーザーが実際にカメラ撮影・PDF 読書・単語収集のフローを体験できるようにする。

## User-Visible Outcome

### When this milestone is complete, the user can:

- Create a book by entering title + author → capture pages via camera → OCR extracts text → pages saved with book linkage
- Import PDF → convert to markdown → read with continuous scroll, font size control (12-32px), progress tracking
- Tap word in PDF/OCR text → save word + full example sentence → definition shows "coming soon" placeholder
- Close and reopen app → all data persists, last-read position restored

### Entry point / environment

- Entry point: Android APK installed on device
- Environment: Motorola Moto G66j 5G (mid-range Android)
- Live dependencies involved: NDLOCR OCR model, Moonshine STT model (bundled in APK)

## Completion Class

- Contract complete means: All 5 slices complete, APK builds and installs, basic flows work on device
- Integration complete means: Camera JNI, PDF pipeline, SQLite persistence all wired and functional
- Operational complete means: App launches without crashes, data persists across restarts

## Final Integrated Acceptance

To call this milestone complete, we must prove:

- APK installs on Moto G66j 5G via `adb install` or direct download
- User can create book, capture 2+ pages with OCR, see saved pages in app
- User can import PDF, convert 5+ pages, scroll through converted text
- User can tap 3+ words, save them with example sentences, see saved words after app restart
- App survives backgrounding and restore without data loss

## Risks and Unknowns

- **Android Gradle build compatibility** — Dioxus 0.7.3 generates obsolete Java 8 config; patch script required (research confirmed workaround exists)
- **Model bundling size** — NDLOCR + Moonshine ~30-40MB; APK total ~50-60MB (acceptable for prototype)
- **JNI camera stability on mid-range device** — Moto G66j 5G has moderate RAM; camera capture + OCR may stress memory
- **Performance on device** — Inference speed unknown on mid-range hardware; may need optimization

## Existing Codebase / Prior Art

- `src/platform/android.rs` — JNI camera, audio, file picker already implemented
- `src/ui/camera.rs` — Camera UI scaffold exists (needs book linkage)
- `src/ui/reader.rs` — PDF reflow reader exists (needs progress sync)
- `src/core/ocr/` — NDLOCR engine with tract-onnx (92 tests passing)
- `src/core/stt/` — Moonshine engine with tract-onnx (92 tests passing)
- `src/core/db.rs` — SQLite CRUD for books, pages, words, annotations

## Relevant Requirements

- R001 — Camera book capture (M002/S02)
- R002 — PDF reflow reader (M002/S03)
- R003 — Word collection with placeholder (M002/S04)
- R004 — APK deploy on Moto G66j (M002/S01)
- R005 — SQLite persistence (M002/S01)
- R006 — Model bundling (M002/S05)
- R007 — Gradle patch script (M002/S01)

## Scope

### In Scope

- Camera capture with book linkage
- PDF reflow reading with progress tracking
- Word tap + save with example sentence
- Android APK build + deploy on Moto G66j 5G
- NDLOCR + Moonshine model bundling
- SQLite persistence verification

### Out of Scope / Non-Goals

- Qwen AI definitions (deferred to M003)
- JMdict/WordNet dictionary bundling (deferred to M003)
- Voice memo recording UI (deferred)
- Release-signed APK (debug sufficient)
- iOS support

## Technical Constraints

- tract-onnx for inference (not ort-mobile; Qwen size dominates anyway)
- Dioxus 0.7 for UI framework
- Android min SDK 21, target SDK 36
- Debug APK for prototype (no signing required)

## Integration Points

- JNI camera capture (`src/platform/android.rs`)
- SQLite database (`src/core/db.rs`)
- NDLOCR OCR engine (`src/core/ocr/`)
- Moonshine STT engine (`src/core/stt/`)
- PDF conversion service (`src/core/pdf.rs`)

## Open Questions

- **Model file locations** — Confirm NDLOCR + Moonshine model files exist in `assets/models/` or need acquisition
- **APK size limit** — Moto G66j 5G storage capacity unknown; ~60MB APK should be fine
- **Performance targets** — OCR latency target undefined; measure on device
