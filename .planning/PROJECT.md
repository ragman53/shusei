# 読書アプリ (Reading App)

## What This Is

Rust と Dioxus で作る Android 読書アプリ。本のページを撮影して高精度 OCR で Markdown に変換し、付箋のようにメモを追加できる。紙の本の読書体験とデジタルの利便性を包括的にカバーする。

## Core Value

紙の本の読書を助けることと、デジタル読書の両方を包括すること。

## Requirements

### Validated

(None yet — ship to validate)

### Active

- [ ] **OCR-01**: ユーザーは本のページを撮影できる
- [ ] **OCR-02**: 撮影した画像を NDLOCR-lite ONNX モデルで高精度に文字認識
- [ ] **OCR-03**: 認識結果を Markdown 形式で保存
- [ ] **NOTE-01**: 変換したページに付箋のようにメモを追加できる
- [ ] **NOTE-02**: 付箋メモは編集・削除可能
- [ ] **VIEW-01**: 保存したページと付箋をアプリ内で閲覧できる

### Out of Scope

- 音声入力メモ — v2 以降
- 単語・用例採集機能 — v2 以降
- PDF → Markdown 変換 — v2 以降
- リフロービュー — v2 以降
- iOS 対応 — Android 専用

## Context

- 既存コードベースあり（Dioxus プロジェクトとしてセット済み）
- 技術スタック：Rust + Dioxus
- OCR エンジン：NDLOCR-lite の ONNX モデルを使用

## Constraints

- **Tech stack**: Rust と Dioxus を使用すること
- **Platform**: Android 専用
- **OCR quality**: 高精度 OCR 必須（NDLOCR-lite ONNX モデル）

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| OCR エンジンに NDLOCR-lite 採用 | ONNX モデルで高精度、Rust と相性が良い | — Pending |
| v1 は写真＋付箋に集中 | コア機能を確実に実装 | — Pending |
| Dioxus で UI 構築 | Rust エコシステムで統一 | — Pending |

---
*Last updated: 2026-03-11 after initialization*
