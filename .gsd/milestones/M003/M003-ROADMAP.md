# M003: Android Stability

**Vision:** 実装済みのカメラ撮影、PDFインポート、単語採集機能が実機でクラッシュせず安定動作する

## Success Criteria

- カメラ撮影が実機でクラッシュせず動作し、撮影した画像がRust側に渡る
- PDFインポートが実機でクラッシュせず動作し、選択したPDFがRust側に渡る
- デモPDF読み込みが実機で動作する
- 権限拒否時もクラッシュせず、適切なエラーメッセージが表示される
- OCR処理が完了し、結果が画面に表示される

## Key Risks / Unknowns

- **CameraXの依存関係追加** — build.gradle.ktsへの追加が必要、バージョン互換性の確認
- **Activity参照の初期化** — JNI_OnLoadでの初期化が正しく行われているか検証が必要
- **アセットバンドル** — デモPDFがAPKに正しく含まれているか確認が必要

## Proof Strategy

- CameraX依存関係 → retire in S01 by proving カメラが起動し撮影できる
- Activity参照 → retire in S01 by proving nativeInitが呼ばれActivity参照が保存される
- アセットバンドル → retire in S03 by proving デモPDFが読み込める

## Verification Classes

- Contract verification: Kotlinメソッドが正しくJNIコールバックを呼ぶことをユニットテストで確認
- Integration verification: 実機でカメラ撮影→OCR→保存、PDF選択→インポートのフローを検証
- Operational verification: 権限拒否時のエラーハンドリングを確認
- UAT / human verification: 実機での手動テスト（ユーザーが実際に操作）

## Milestone Definition of Done

This milestone is complete only when all are true:

- すべてのスライスが完了している
- カメラ撮影フローが実機でクラッシュせず完了する
- PDFインポートフローが実機でクラッシュせず完了する
- デモPDFが読み込める
- 権限拒否時もクラッシュしない
- 成功基準が実機での動作に対して再確認されている

## Requirement Coverage

- Covers: R001（カメラ撮影）、R002（PDFリーダー）、R004（実機動作）
- Partially covers: なし
- Leaves for later: R008（AI定義）、R009（辞書）、R010（ボイスメモ）
- Orphan risks: なし

## Slices

- [x] **S01: Kotlin Camera Implementation** `risk:high` `depends:[]`
  > After this: 「Take Photo」ボタンでカメラが起動し、撮影した画像がRust側に渡る

- [x] **S02: Kotlin File Picker Implementation** `risk:medium` `depends:[]`
  > After this: 「Import PDF」ボタンでファイル選択ダイアログが開き、選択したPDFがRust側に渡る

- [x] **S03: Asset Access Fix** `risk:medium` `depends:[S01]`
  > After this: 「Load Demo PDF」ボタンでバンドルされたPDFが読み込める

- [x] **S04: Integration Verification** `risk:low` `depends:[S01, S02, S03]`
  > After this: すべての機能が実機でクラッシュせず動作することが検証済み

## Boundary Map

### S01 → S02

Produces:
- `MainActivity.kt` — カメラ撮影メソッド（startCameraCapture, onImageCaptured）
- `PermissionHelper.kt` — カメラ権限リクエスト処理（拡張）
- AndroidManifest.xml — CAMERA権限宣言

Consumes:
- `src/platform/android.rs` — JNIコールバック定義（既存）
- `target/.../WryActivity.kt` — Activityベースクラス（Dioxus生成）

### S01 → S03

Produces:
- Activity参照 — nativeInitで保存されるグローバル参照

Consumes:
- なし（S01でActivity参照が初期化されることを前提）

### S02 → S04

Produces:
- `MainActivity.kt` — ファイルピッカー関連メソッド（pickPdfFile, onFilePicked, onFilePickFailed）
- AndroidManifest.xml — READ_EXTERNAL_STORAGE権限宣言（必要に応じて）

Consumes:
- `src/platform/android.rs` — JNIコールバック定義（既存）

### S03 → S04

Produces:
- `MainActivity.kt` — copyAssetToFilesメソッド
- アセットバンドル確認 — デモPDFがAPKに含まれる

Consumes:
- Activity参照（S01で初期化）
- AssetManager API（Android標準）