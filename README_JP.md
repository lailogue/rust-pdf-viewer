# PDF リーダー

RustとDioxusによるモダンなPDF閲覧アプリケーション。Gemini AIによる検索機能付き。

## 主な機能

- **高品質PDF表示**: 1000x1400解像度で品質とパフォーマンスのバランスを実現
- **連続スクロール**: 全PDFページをシームレスな連続スクロールで表示
- **AI検索機能**: Gemini 2.5 Flash APIによるインテリジェントな検索
- **モダンUI**: Dioxusフレームワークによる美しくレスポンシブなインターフェース
- **最適化レイアウト**: 縦長PDF文書用に特別設計された横型AI検索パネル
- **ページキャッシュ**: パフォーマンス向上のためのインテリジェントキャッシュシステム
- **クロスプラットフォーム**: ネイティブパフォーマンスのデスクトップアプリケーション

## 技術スタック

- **Dioxus** - Rust用モダンリアクティブUIフレームワーク
- **pdfium-render** - Google PDFiumライブラリによるPDFレンダリング
- **Reqwest** - AI検索機能用HTTPクライアント
- **Tokio** - ノンブロッキング操作用非同期ランタイム
- **Serde** - API通信用JSONシリアライゼーション
- **Base64** - Web表示用画像エンコーディング

## 前提条件

### macOS
アプリケーションにはPDFiumネイティブライブラリが必要です。macOS用の必要なライブラリ（`libpdfium.dylib`）は`lib/`ディレクトリに含まれています。

### Gemini API キー
AI検索機能を利用するには、Google AI Studio から Gemini API キーを取得してください：
https://makersuite.google.com/app/apikey

## インストール・使用方法

### ソースからビルド
```bash
# リポジトリをクローン
git clone <repository-url>
cd rust-pdf-viewer

# PDFiumライブラリのセットアップ（macOS）
mkdir -p lib

# macOS（Apple Silicon）用PDFiumライブラリをダウンロード
curl -L -o lib/libpdfium.dylib \
  "https://github.com/bblanchon/pdfium-binaries/releases/latest/download/pdfium-mac-arm64.tgz" && \
  tar -xzf lib/libpdfium.dylib -C lib --strip-components=1 lib/libpdfium.dylib

# Intel Mac用の場合はこちらを使用:
# curl -L -o pdfium-mac-x64.tgz \
#   "https://github.com/bblanchon/pdfium-binaries/releases/latest/download/pdfium-mac-x64.tgz" && \
#   tar -xzf pdfium-mac-x64.tgz -C lib --strip-components=1 lib/libpdfium.dylib && \
#   rm pdfium-mac-x64.tgz

# ライブラリが正しく配置されているか確認
ls -la lib/libpdfium.dylib

# プロジェクトをビルド
cargo build --release

# PDFファイルを指定して実行
cargo run -- <PDFファイルパス>

# 例
cargo run -- test.pdf
```

### アプリケーションの使用

1. **アプリケーション起動**: PDFファイルを引数として指定
2. **ページ移動**: 上部の前へ/次へボタンでページナビゲーション
3. **AI検索**:
   - 右側パネルにGemini APIキーを入力
   - 検索語句を入力
   - 「検索」ボタンをクリックしてAI検索を実行
4. **結果表示**: 拡張可能な検索結果エリアで結果を確認

## アプリケーションレイアウト

- **左パネル**: 縦長文書に最適化されたPDF表示エリア
- **右パネル**: AI検索インターフェース
  - APIキー入力（パスワードフィールド）
  - 検索語句入力
  - ローディングインジケーター付き検索ボタン
  - クリーンなテキスト表示の拡張可能結果エリア
- **上部バー**: ページナビゲーションコントロールとPDF情報

## 実装のハイライト

### 高品質レンダリング
- 品質とパフォーマンスのバランスを取るため1000x1400ピクセルでPDFページをレンダリング
- 全ページを一度に表示する連続スクロール表示
- 自動カラースペース変換（BGRA → RGBA）
- Web表示用のbase64データURLによるPNGエンコーディング

### AI検索統合
- UIをブロックしない非同期Gemini API呼び出し
- クリーンなテキスト表示のためのMarkdown書式削除
- リアルタイム検索ステータスインジケーター
- 包括的なエラーハンドリング

### パフォーマンス最適化
- HashMapを使用したページレベルキャッシュシステム
- PDFページの遅延ローディング
- メモリ効率的な画像ハンドリング
- レスポンシブUI更新

### モダンDioxusアーキテクチャ
- `use_signal`によるリアクティブ状態管理
- `use_memo`によるメモ化計算
- コンポーネントベース設計パターン
- CSS-in-Rustスタイリングアプローチ

## プロジェクト構造

```
├── src/
│   └── main.rs           # メインDioxusアプリケーション
├── assets/
│   └── style.css         # UIスタイリング
├── lib/
│   └── libpdfium.dylib   # macOS用PDFiumライブラリ
├── Cargo.toml            # 依存関係と設定
└── README.md            # このファイル
```

## 開発

### コード構成
- **PDF レンダリング**: `render_pdf_page()`関数が高解像度ページレンダリングを処理
- **AI検索**: `search_with_gemini()`非同期関数によるAPI統合
- **テキスト処理**: `clean_markdown_text()`による検索結果書式設定
- **UIコンポーネント**: モダンな状態管理を持つリアクティブDioxusコンポーネント

### 依存関係
すべての依存関係はDioxusエコシステム用に最適化され、レガシーeguiの依存関係を削除してよりクリーンで保守しやすいコードベースを実現。

## ライセンス

このプロジェクトはLICENSEファイルで指定された条件の下でライセンスされています。