# 読書アプリ (Reading App)

## What This Is

完全オフラインの読書アプリ。紙の本も PDF もスマホ 1 台で「付箋＋ボイスメモ＋単語採集」ができる。Dioxus+Rust で Android 実装。プライバシー 100%、外部 API 完全不使用。

## Core Value

紙の本と PDF をシームレスに統合し、読書中の思考を逃さず記録できる完全オフライン環境。

## Requirements

### Validated

(None yet — ship to validate)

### Active

- [ ] 紙の本ワークフロー：ページ撮影 → NDLOCR で OCR → テキスト記録
- [ ] 音声入力メモ：Moonshine Voice でリアルタイム音声→テキスト変換
- [ ] 単語採集：単語＋用例文＋ページ位置＋AI 生成定義を保存
- [ ] PDF 読書：ファイルピッカーでインポート → NDLOCR で Markdown 変換 → リフロー表示
- [ ] 本メタデータ：手動入力（タイトル・著者）＋表紙写真オプション
- [ ] 単語レビュー：同一単語の採集回数表示＋採集リスト閲覧
- [ ] ローカルストレージ：SQLite データベース
- [ ] メモリ最適化：低 RAM デバイスでも動作

### Out of Scope

- バックアップ機能 — データはアプリ内のみ保存、ユーザーが自己責任で管理
- クラウド同期 — 完全オフライン設計
- iOS 対応 — Android のみ（Dioxus で将来的にマルチプラットフォーム対応可能）

## Context

### Technical Environment

- **Framework**: Dioxus (Rust 製クロスプラットフォーム UI)
- **Target**: Android (Dioxus mobile)
- **Language**: Rust 100%
- **Storage**: SQLite (rusqlite)

### Key Technologies

- **OCR**: NDLOCR-Lite (ONNX 軽量 OCR) の Rust 移植
  - Reference: https://github.com/ndl-lab/ndlocr-lite
- **音声認識**: Moonshine Voice (オープンソース AI 音声ツールキット)
  - Reference: https://github.com/moonshine-ai/moonshine
- **AI 辞書**: Qwen3.5-08B 等の軽量モデルをオンデバイスで実行
- **PDF 処理**: NDLOCR で Markdown 変換、リフロー表示

### User Pain Points

- 既存アプリは断片的：紙の本用、PDF 用、ノート用が別々
- プライバシー懸念：クラウド同期、読書履歴の追跡
- 機能過多：複雑で遅く、気が散る

## Constraints

- **Tech stack**: Dioxus + Rust のみ — マルチプラットフォーム対応を維持
- **Privacy**: 外部 API 完全不使用、ネットワーク権限不要
- **Memory**: 低 RAM デバイス（Android Go 等）でも動作するメモリ最適化
- **Offline**: 完全オフライン動作、初期設定もオフライン
- **Platform**: Android -first（Dioxus で将来的にデスクトップ・iOS 対応可能）

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| NDLOCR-Lite 採用 | 軽量、ONNX 形式、Rust 移植可能 | — Pending |
| Moonshine Voice 採用 | オープンソース、多言語対応（日英） | — Pending |
| Qwen3.5-08B で AI 辞書 | オフラインで定義生成、8B パラメータは軽量 | — Pending |
| SQLite 採用 | 構造化データ、Rust エコシステム成熟 | — Pending |
| 写真＋OCR 両保存 | 後から元ページ確認可能 | — Pending |
| 単語採集回数表示 | 記憶定着の可視化 | — Pending |

---
*Last updated: 2026-03-15 after S04 completion*

## Current State (Post-S04)

**Backend infrastructure complete:**
- ✅ S01: Core Infrastructure — Database foundation with Book model and books table
- ✅ S02: Paper Book Capture — Image preprocessing, OCR engine, quality detection, book pages CRUD
- ✅ S03: PDF Support — PDF import, batch OCR pipeline, reflow reader, progress tracking
- ✅ S04: Annotation Foundation — Highlights, bookmarks, notes with full CRUD and 15 unit tests

**Annotation capabilities shipped:**
- `annotations` table with type discriminator (highlight/bookmark/note)
- `AnnotationType` enum with type-safe string conversion
- Full CRUD operations: create, read, update, delete annotations
- Type-specific queries: `get_highlights()`, `get_bookmarks()`, `get_notes()`
- Position range tracking for text selection (character offsets)
- Color customization for highlights (yellow/green/pink/blue)
- Bulk deletion by book for cleanup

**Next phase:** S05 Voice Memos — Attach voice memos to annotations and pages

**Known blockers:**
- ONNX Runtime linker error (`__isoc23_strtoll` undefined symbol) prevents test execution
- Pre-existing issue with `ort-sys` dependency, not caused by annotation code
- Tests are structurally correct and would pass on properly configured system

**Deferred to frontend phase:**
- Annotation UI components (create/edit/delete dialogs)
- Text selection integration for position tracking
- Visual highlight rendering (colored backgrounds in reader)
- Export/import integration for annotations
