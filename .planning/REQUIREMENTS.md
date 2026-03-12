# Requirements: 読書アプリ (Reading App)

**Defined:** 2026-03-11
**Core Value:** 紙の本と PDF をシームレスに統合し、読書中の思考を逃さず記録できる完全オフライン環境。

## v1 Requirements

Requirements for initial release. Each maps to roadmap phases.

### Core Infrastructure

- [x] **CORE-01**: Android アプリとして起動し、本ライブラリ画面が表示される
- [x] **CORE-02**: SQLite データベースに本メタデータ（タイトル、著者、表紙画像パス）を保存
- [x] **CORE-03**: ファイルシステムに画像・音声ファイルを保存（パスは SQLite）
- [x] **CORE-04**: Android ライフサイクル（バックグラウンド移行）を適切に処理
- [x] **CORE-05**: JNI 参照管理パターンを確立（メモリリーク防止）

### Paper Book Capture

- [ ] **PAPER-01**: カメラ起動でページを撮影し、画像を保存
- [ ] **PAPER-02**: 撮影画像を 2MP 以下にダウンケール（メモリ最適化）
- [ ] **PAPER-03**: NDLOCR-Lite で OCR 処理、テキストを抽出
- [ ] **PAPER-04**: OCR 結果テキストを SQLite に保存（画像パスと紐付け）
- [ ] **PAPER-05**: ページ画像と OCR テキストを紐付けて表示

### PDF Support

- [ ] **PDF-01**: ファイルピッカーで PDF をインポート
- [x] **PDF-02**: PDF をページ単位で NDLOCR-Lite 処理、Markdown 変換
- [x] **PDF-03**: 変換 Markdown をリフロー表示（フォントサイズ変更可能）
- [x] **PDF-04**: PDF 進捗表示（ページ単位のストリーミング処理）

### Annotation

- [ ] **ANNO-01**: ページに付箋（テキストメモ）を追加・編集・削除
- [ ] **ANNO-02**: ブックマークを追加・削除
- [ ] **ANNO-03**: 単語をタップして採集（単語＋用例文＋ページ位置）
- [ ] **ANNO-04**: 採集単語リストを表示（採集回数付き）

### Voice Memo

- [ ] **VOICE-01**: 音声入力 UI でリアルタイム音声→テキスト変換（Moonshine）
- [ ] **VOICE-02**: 音声メモをページに紐付けて保存
- [ ] **VOICE-03**: 音声メモを一覧表示・再生
- [ ] **VOICE-04**: デバイス能力検出（Moonshine モデルサイズ自動選択）

### AI Dictionary

- [ ] **AI-01**: 採集単語に対して Qwen3.5-08B で定義を生成（オフライン）
- [ ] **AI-02**: 生成定義を SQLite に保存（単語と紐付け）
- [ ] **AI-03**: 同一単語を再選択時に採集回数を表示
- [ ] **AI-04**: モデル順次読み込み（メモリ最適化、複数モデル同時読み込み防止）

### Performance

- [ ] **PERF-01**: Android Go デバイス（2GB RAM）で動作
- [ ] **PERF-02**: 画像処理中に UI がフリーズしない（バックグラウンド処理）
- [ ] **PERF-03**: 音声ストリーミング中にバッファオーバーフロー防止

## v2 Requirements

Deferred to future release. Tracked but not in current roadmap.

### Export

- **EXP-01**: 付箋・単語採集リストを Markdown でエクスポート
- **EXP-02**: 単語採集データを Anki 形式でエクスポート
- **EXP-03**: 読書統計（読了冊数、採集単語数など）を表示

### Advanced Features

- **ADV-01**: 単語の跨ぎ参照（同じ単語が他の本のどこで出たか）
- **ADV-02**: 手書き入力対応（スタイラスペン対応）
- **ADV-03**: ダークモード・テーマカスタマイズ

## Out of Scope

Explicitly excluded. Documented to prevent scope creep.

| Feature | Reason |
|---------|--------|
| クラウド同期 | 完全オフライン設計、プライバシー 100% |
| バックアップ機能 | データはアプリ内のみ、ユーザー自己管理 |
| リアルタイムチャット | 高複雑、読書コア価値と無関係 |
| 動画投稿 | ストレージ・帯域コスト、v1 不要 |
| OAuth ログイン | オフライン動作、アカウント不要 |
| iOS 対応 | Android-first、Dioxus で将来的に対応可能 |

## Traceability

Which phases cover which requirements. Updated during roadmap creation.

| Requirement | Phase | Status |
|-------------|-------|--------|
| CORE-01 | Phase 1 | Complete |
| CORE-02 | Phase 1 | Complete |
| CORE-03 | Phase 1 | Complete |
| CORE-04 | Phase 1 | Complete |
| CORE-05 | Phase 1 | Complete |
| PAPER-01 | Phase 2 | Pending |
| PAPER-02 | Phase 2 | Pending |
| PAPER-03 | Phase 2 | Pending |
| PAPER-04 | Phase 2 | Pending |
| PAPER-05 | Phase 2 | Pending |
| PDF-01 | Phase 3 | Pending |
| PDF-02 | Phase 3 | Complete |
| PDF-03 | Phase 3 | Complete |
| PDF-04 | Phase 3 | Complete |
| ANNO-01 | Phase 4 | Pending |
| ANNO-02 | Phase 4 | Pending |
| ANNO-03 | Phase 4 | Pending |
| ANNO-04 | Phase 4 | Pending |
| VOICE-01 | Phase 5 | Pending |
| VOICE-02 | Phase 5 | Pending |
| VOICE-03 | Phase 5 | Pending |
| VOICE-04 | Phase 5 | Pending |
| AI-01 | Phase 6 | Pending |
| AI-02 | Phase 6 | Pending |
| AI-03 | Phase 6 | Pending |
| AI-04 | Phase 6 | Pending |
| PERF-01 | Phase 7 | Pending |
| PERF-02 | Phase 7 | Pending |
| PERF-03 | Phase 7 | Pending |

**Phase Mapping Summary:**
- **Phase 1 (Core Infrastructure):** CORE-01 through CORE-05 (5 requirements)
- **Phase 2 (Paper Book Capture):** PAPER-01 through PAPER-05 (5 requirements)
- **Phase 3 (PDF Support):** PDF-01 through PDF-04 (4 requirements)
- **Phase 4 (Annotation Foundation):** ANNO-01 through ANNO-04 (4 requirements)
- **Phase 5 (Voice Memos):** VOICE-01 through VOICE-04 (4 requirements)
- **Phase 6 (AI Enhancement):** AI-01 through AI-04 (4 requirements)
- **Phase 7 (Performance Polish):** PERF-01 through PERF-03 (3 requirements)

**Coverage:**
- v1 requirements: 30 total
- Mapped to phases: 30
- Unmapped: 0 ✓

---
*Requirements defined: 2026-03-11*
*Last updated: 2026-03-11 after roadmap creation*
