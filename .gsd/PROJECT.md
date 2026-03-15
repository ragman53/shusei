# 読書アプリ (Reading App)

## What This Is

完全オフラインの読書アプリ。紙の本も PDF もスマホ 1 台で「付箋＋ボイスメモ＋単語採集」ができる。Dioxus+Rust で Android 実装。プライバシー 100%、外部 API 完全不使用。

## Core Value

紙の本と PDF をシームレスに統合し、読書中の思考を逃さず記録できる完全オフライン環境。

## Current State

**M001 COMPLETE — Backend infrastructure fully operational**

All 7 slices delivered with 92 passing unit tests. Build completes successfully with no linker errors.

**Database schemas (5 tables):**
- `books` — Book metadata with cover photo paths
- `book_pages` — Page images with OCR results (markdown + plain text)
- `annotations` — Highlights, bookmarks, notes with type discriminator
- `words` — Vocabulary with AI-generated definitions
- `processing_progress` — PDF conversion progress with resume support

**Inference engines (tract-onnx):**
- NDLOCR-Lite OCR — Detection + recognition with preprocessing pipeline
- Moonshine STT — Encoder + decoder with mel-spectrogram preprocessing
- AI Engine — Trait-based abstraction with MockAiEngine for testing

**Core features:**
- PDF import with metadata extraction and batch OCR conversion
- Reflow reader with font controls (12-32px) and continuous scroll
- Annotation foundation with full CRUD (highlights, bookmarks, notes)
- Audio recording via JNI with 30-second limit
- Quality detection (Laplacian variance, brightness analysis)

**Next phase: M002 — Android Prototype**
- Camera UI with OCR preview and book linkage
- PDF reflow reader with progress tracking
- Word collection with example sentences (definition placeholder)
- Android APK build + deploy on Moto G66j 5G
- Model bundling (NDLOCR, Moonshine)

## Capability Contract

See `.gsd/REQUIREMENTS.md` for the explicit capability contract, requirement status, and coverage mapping.

## Milestone Sequence

- [x] M001: Backend Infrastructure — Database, OCR, STT, AI engines, 92 tests passing
- [ ] M002: Android Prototype — Camera capture, PDF reader, word collection, APK deploy on Moto G66j 5G
- [ ] M003: Dictionary Integration — JMdict/WordNet bundling, AI definitions, full word-tap experience
