# M004: Tauri + Leptos Migration

**Vision:** Dioxus から Tauri + Leptos へ移行し、より成熟したエコシステムとパフォーマンスを獲得

## Success Criteria

- [ ] Tauri v2 + Leptos v0.7 プロジェクトがビルド可能
- [ ] 既存のデータベーススキーマが Tauri プラグインとして移植可能
- [ ] OCR 処理 (tract-onnx) が Tauri バックエンドで動作
- [ ] 既存の UI コンポーネントが Leptos で再実装可能
- [ ] Android APK が Tauri + Leptos でビルド可能

## Key Risks / Unknowns

- **Leptos 学習コスト** — Dioxus とは異なるシグナルベースのリアクティビティ
- **Tauri モバイル設定** — Android ビルドの設定が Dioxus と異なる
- **UI 移植工数** — 既存のコンポーネントをどこまで再利用可能か
- **パフォーマンス比較** — Tauri + Leptos のバンドルサイズと起動時間

## Proof Strategy

- Tauri セットアップ → retire in S01 by proving `pnpm tauri android init` が成功
- Leptos 統合 → retire in S02 by proving フロントエンドがビルド可能
- DB 移植 → retire in S03 by proving 既存のスキーマが動作
- OCR 統合 → retire in S04 by proving tract-onnx が Tauri で動作
- Android ビルド → retire in S05 by proving APK が実機にデプロイ可能

## Verification Classes

- Contract verification: Tauri + Leptos の基本機能が動作
- Integration verification: 既存の Rust コードが Tauri プラグインとして動作
- Performance verification: バンドルサイズ・起動時間の比較
- UAT / human verification: 実機での動作確認

## Dependencies

- M003 (Android Stability) — 安定した Android 実装がベース
- Node.js 18+ — Tauri の要件
- pnpm — Tauri のパッケージマネージャー推奨

## Slices

- [ ] **S01: Tauri Project Setup** `risk:low` `depends:[]`
- [ ] **S02: Leptos Frontend Integration** `risk:medium` `depends:[S01]`
- [ ] **S03: Database Plugin Migration** `risk:medium` `depends:[S01]`
- [ ] **S04: OCR Engine Integration** `risk:high` `depends:[S01, S03]`
- [ ] **S05: Android Build + Deploy** `risk:high` `depends:[S02, S03, S04]`
