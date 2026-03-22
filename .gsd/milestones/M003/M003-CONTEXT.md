# M003: Android Stability — Context

**Gathered:** 2026-03-22
**Status:** Ready for planning

## Project Description

読書アプリ（Shusei）の実機安定化マイルストーン。M002で実装したカメラ撮影、PDFインポート、単語採集機能がAndroid実機でクラッシュせず動作するようにする。

## Why This Milestone

M002でAPKのビルドとデプロイは成功したが、実機での操作時にクラッシュが発生する。根本原因はKotlin側（MainActivity.kt）の実装不足。Rust側のJNIコードは呼び出し可能だが、Java/Kotlin側に呼び出されるメソッドが存在しない。

ユーザーは以下の操作でクラッシュを確認：
- 「Take Photo」ボタン押下 → `startCameraCapture()`が未実装
- 「Import PDF」ボタン押下 → `pickPdfFile()`が未実装
- 「Load Demo PDF」ボタン押下 → アセットアクセスに失敗

このマイルストーンで実機での安定動作を実現し、プロトタイプを実際に使用可能な状態にする。

## User-Visible Outcome

### When this milestone is complete, the user can:

- 「Take Photo」ボタンを押してカメラを起動し、撮影した画像がOCR処理されて保存される
- 「Import PDF」ボタンを押してPDFファイルを選択し、ライブラリに追加される
- 「Load Demo PDF」ボタンを押してバンドルされたデモPDFを読み込める

### Entry point / environment

- Entry point: APKインストール後、アプリを起動
- Environment: Moto G66j 5G（実機）、Android 14
- Live dependencies involved: カメラ、ストレージ、ファイルシステム

## Completion Class

- Contract complete means: 各Kotlinメソッドが正しくRust側のJNIコールバックを呼び出し、データが正しく渡る
- Integration complete means: カメラ撮影→OCR→保存、PDF選択→インポート→表示の全フローが実機で動作
- Operational complete means: 権限拒否時もクラッシュせず、適切なエラーメッセージを表示

## Final Integrated Acceptance

To call this milestone complete, we must prove:

- カメラ撮影フロー: 起動 → 撮影 → OCR → 保存が実機で完了
- PDFインポートフロー: 選択 → インポート → 表示が実機で完了
- 権限拒否時: クラッシュせず、エラーメッセージが表示される

## Risks and Unknowns

- **CameraXの依存関係追加** — build.gradle.ktsへの追加が必要、バージョン互換性の確認が必要
- **Android APIレベル互換性** — ターゲットSDK（現在不明）に合わせた実装が必要
- **アセットバンドルの確認** — デモPDFがAPKに正しく含まれているか確認が必要
- **Activity参照の初期化タイミング** — JNI_OnLoadでの初期化が正しく行われているか検証が必要

## Existing Codebase / Prior Art

- `src/platform/android.rs` — Rust側のJNIコード（コールバック定義済み）
- `target/dx/shusei/debug/android/app/app/src/main/kotlin/dev/dioxus/main/MainActivity.kt` — Dioxus生成（空のクラス）
- `target/dx/shusei/debug/android/app/app/src/main/kotlin/dev/dioxus/main/WryActivity.kt` — Dioxus生成のベースActivity
- `scripts/android-patch.sh` — ビルド後のパッチスクリプト

> See `.gsd/DECISIONS.md` for all architectural and pattern decisions — it is an append-only register; read it during planning, append to it during execution.

## Relevant Requirements

- R001 — Camera book capture（現在validatedだが、実機で動作していない）
- R002 — PDF reflow reader（現在validatedだが、実機で動作していない）
- R004 — APK deploys on Moto G66j 5G（実機でクラッシュするため再検証が必要）

## Scope

### In Scope

- MainActivity.ktへのカメラ実装（CameraX使用）
- MainActivity.ktへのファイルピッカー実装（Storage Access Framework使用）
- 権限リクエスト処理の実装
- アセットアクセスの修正
- AndroidManifest.xmlの権限宣言確認・追加
- 実機での動作検証

### Out of Scope / Non-Goals

- 新機能の追加（ボイスメモ、AI定義など）
- UIの変更
- Rustコードの大幅な変更
- iOS対応

## Technical Constraints

- Kotlin 1.9+（Dioxusが生成するGradle設定に依存）
- CameraX 1.3+（最新安定版）
- Android SDK 34（最新のDioxusプロジェクト設定）
- 最小SDK 24（Android 7.0）

## Integration Points

- **CameraX** — Android標準のカメラAPI。画像キャプチャ → JPEGバイト配列 → JNIでRustに渡す
- **Storage Access Framework** — ファイル選択。選択されたURI → パス解決 → JNIでRustに渡す
- **AssetManager** — APK内アセットへのアクセス。Activity参照が必要
- **JNI** — RustとKotlin間のデータブリッジ。バイト配列、文字列、数値の受け渡し

## Open Questions

- **カメラの向き** — フロント/バックどちらを使用するか？（デフォルトはバック）
- **画像サイズ** — カメラ撮影の解像度設定（OCR効率のため1024px程度が望ましい）
- **デモPDFのアセットパス** — `test/medium_pdf_test.pdf`がAPKに含まれているか確認が必要