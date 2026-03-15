# M001: Migration

**Vision:** 完全オフラインの読書アプリ。紙の本も PDF もスマホ 1 台で「付箋＋ボイスメモ＋単語採集」ができる。Dioxus+Rust で Android 実装。プライバシー 100%、外部 API 完全不使用。

## Success Criteria


## Slices

- [x] **S01: Core Infrastructure** `risk:medium` `depends:[]`
  > After this: Create database foundation with Book model and books table schema

Purpose: Establish the data layer that all book operations depend on
Output: Working database schema with tested CRUD operations
- [x] **S02: Paper Book Capture** `risk:medium` `depends:[S01]`
  > After this: Implement OCR engine and image preprocessing pipeline

Purpose: Enable the core camera → OCR workflow by implementing the NDLOCR-Lite engine and ensuring images are properly downscaled for memory efficiency

Output: Working OCR engine with image preprocessing, ready for integration with camera UI
- [x] **S03: Pdf Support** `risk:medium` `depends:[S02]`
  > After this: Implement PDF import flow with file picker, metadata extraction, and library integration.

Purpose: Enable PDF import, OCR conversion, and reflow reading with progress tracking and batch processing
Output: Working PDF import, batch OCR pipeline, reflow reader with font controls, library integration with PDF badges
- [x] **S04: Annotation Foundation** `risk:medium` `depends:[S03]`
  > After this: unit tests prove Annotation Foundation works
- [ ] **S05: Voice Memos** `risk:medium` `depends:[S04]`
  > After this: unit tests prove Voice Memos works
- [ ] **S06: AI Enhancement** `risk:medium` `depends:[S05]`
  > After this: unit tests prove AI Enhancement works
- [ ] **S07: Performance Polish** `risk:medium` `depends:[S06]`
  > After this: unit tests prove Performance Polish works
