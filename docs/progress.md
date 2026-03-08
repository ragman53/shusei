# Shusei MVP 開発進捗状況

**最終更新**: 2026-03-08 13:37 (JST)

---

## 完了した作業

### 1. 計画・設計フェーズ

- [x] 仕様書確認 (`docs/shusei-mvp-v2.3.md`)
- [x] レビュー文書確認 (`docs/shusei-mvp-v2.3-review.md`)
- [x] 詳細実装計画書作成 (`docs/shusei-implementation-plan.md`)
- [x] Week 1 カメラ PoC を最優先事項として計画に反映

### 2. プロジェクト初期化

- [x] Dioxus CLI インストール (v0.7.3)
- [x] プロジェクト構造作成完了

#### 作成したファイル

```
shusei/
├── Cargo.toml                    # 依存関係定義
├── Cargo.lock                    # ロックファイル（自動生成）
├── Dioxus.toml                   # Dioxus設定
├── .gitignore                    # Git除外設定
├── docs/
│   ├── shusei-implementation-plan.md  # 実装計画書
│   ├── shusei-mvp-v2.3.md            # 仕様書
│   └── shusei-mvp-v2.3-review.md     # レビュー文書
├── platform/
│   └── android/
│       └── AndroidManifest.xml   # Android権限設定
└── src/
    ├── main.rs                   # エントリーポイント
    ├── lib.rs                    # ライブラリエクスポート
    ├── app.rs                    # ルートコンポーネント + ルーティング
    ├── core/
    │   ├── mod.rs
    │   ├── error.rs              # エラー型定義
    │   ├── db.rs                 # SQLite CRUD + FTS5
    │   ├── vocab.rs              # 単語抽出
    │   ├── pdf.rs                # PDF処理（条件付き）
    │   ├── ocr/
    │   │   ├── mod.rs
    │   │   ├── engine.rs         # OCR エンジン trait
    │   │   ├── preprocess.rs     # 画像前処理
    │   │   ├── postprocess.rs    # テキスト検出・認識
    │   │   └── markdown.rs       # Markdown生成
    │   └── stt/
    │       ├── mod.rs
    │       ├── engine.rs         # STT エンジン trait
    │       ├── decoder.rs        # KVキャッシュ + デコーダー
    │       └── tokenizer.rs      # トークナイザー
    ├── platform/
    │   ├── mod.rs                # PlatformApi trait
    │   ├── android.rs            # JNI カメラ PoC
    │   └── ios.rs                # iOS用（スタブ）
    └── ui/
        ├── mod.rs
        ├── camera.rs             # カメラ撮影画面
        ├── notes.rs              # 付箋一覧画面
        ├── reader.rs             # PDF読書画面
        ├── vocab.rs              # 単語リスト画面
        └── components.rs         # 共通コンポーネント
```

---

## 残タスク（次スレッドで継続）

### 即座に対応が必要

1. **ビルドエラー修正**
   - `dioxus-router::prelude` の import エラー
   - `pdfium_render` の条件付きコンパイル設定
   - `src/core/pdf.rs` に `#[cfg(feature = "pdf")]` 追加

2. **cargo build 成功確認**

### Week 1 PoC (最重要)

3. **【最重要】Dioxus + JNI カメラ PoC**
   - Java側のカメラ実装
   - JNI ブリッジの完成
   - Android 実機テスト

4. **tract Android クロスコンパイル検証**

5. **NDLOCR-Lite ONNX tract 互換性検証** - ✅ テスト作成完了、モデル取得待ち
   - `tests/ndlocr_tract_test.rs` 作成
   - tract-onnx が正常にロード可能であることを確認
   - モデルファイルは `assets/models/ndlocr/` に配置が必要
   - 詳細は `assets/models/ndlocr/README.md` 参照

6. **Moonshine Tiny ONNX tract 互換性検証**

7. **Go/No-Go 判定**

---

## 既知の問題

### ビルドエラー

1. **dioxus-router import エラー**
   ```
   error[E0432]: unresolved import `dioxus_router::prelude`
   ```
   - 原因: dioxus-router 0.7 の API 変更の可能性
   - 対応: dioxus-router のドキュメントを確認して import を修正

2. **pdfium-render リンクエラー**
   ```
   error LNK2019: unresolved external symbol FPDFPage_TransformAnnots
   ```
   - 原因: pdfium の静的リンク設定
   - 対応: オプション機能として条件付きコンパイルに変更済み

3. **lindera-ipadic ダウンロードエラー**
   ```
   Error: Transport(Transport { kind: Dns... })
   ```
   - 原因: ネットワーク接続問題で辞書ダウンロード失敗
   - 対応: オプション機能として条件付きコンパイルに変更済み

---

## 次回の作業手順

1. `src/core/pdf.rs` に `#[cfg(feature = "pdf")]` を追加
2. `src/app.rs` の dioxus-router import を修正
3. `cargo build` でビルド成功を確認
4. JNI カメラ PoC の実装を継続

---

## 参考リンク

- [Dioxus 0.7 Documentation](https://dioxuslabs.com/)
- [tract ONNX Runtime](https://github.com/sonos/tract)
- [NDLOCR-Lite](https://github.com/ndl-lab/ndlocr-lite)
- [Moonshine ASR](https://github.com/usefulsensors/moonshine)
