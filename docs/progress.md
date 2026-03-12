# Shusei MVP 開発進捗状況

**最終更新**: 2026-03-10 (JST)

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

### 3. Android 環境セットアップ完了

- [x] Android Studio Meerkat (2025.3.1) インストール済み
- [x] Android SDK 36.1 (API 36.1) インストール済み
- [x] Build Tools 36.1.0 インストール済み
- [x] Platform Tools 37.0.0 インストール済み
- [x] Rust Android ターゲット (aarch64-linux-android) インストール済み
- [x] cargo-ndk v4.1.2 インストール済み
- [x] Dioxus CLI v0.7.3 インストール済み

### 4. JNI カメラ実装状況

- [x] Rust 側 JNI 実装完了 (src/platform/android.rs)
- [x] Java 側 Camera2 API 実装完了 (MainActivity.java)
- [x] AndroidManifest.xml 権限設定完了
- [ ] 実機ビルド検証
- [ ] カメラ動作検証

---

## 残タスク（次スレッドで継続）

### 即座に対応が必要

1. **Week 1 PoC 実機検証**
   - Android 実機ビルド検証
   - JNI カメラ PoC 動作テスト
   - tract Android クロスコンパイル検証

---


## 既知の問題

### ビルド警告

ビルドは成功していますが、以下の警告があります：

1. **未使用 import 警告**
   - `OcrError`, `SttError`, `ShuseiError` などの未使用 import
   - 対応：実装進行中に自然に解消される見込み

2. **未使用変数・関数警告**
   - 前処理・後処理関数、NMS 関数など
   - 対応：OCR 実装時に使用される予定

### 注意事項

- pdfium-render と lindera は オプション機能 (`pdf`, `lindera` feature) として設定済み
- 通常ビルドではこれらの依存関係は不要

---

## 次回の作業手順

1. **Android 実機ビルド検証**
   ```bash
   dx serve --platform android
   ```
   
2. **JNI カメラ PoC 実機テスト**
   - Moto G66j 5G でカメラ動作確認
   - 画像取得～表示のフロー検証

3. **tract Android クロスコンパイル検証**
   ```bash
   cargo build --target aarch64-linux-android
   ```

4. **Go/No-Go 判定**
   - Week 1-2 マイルストーン達成評価

---

## 参考リンク

- [Dioxus 0.7 Documentation](https://dioxuslabs.com/)
- [tract ONNX Runtime](https://github.com/sonos/tract)
- [NDLOCR-Lite](https://github.com/ndl-lab/ndlocr-lite)
- [Moonshine ASR](https://github.com/usefulsensors/moonshine)
