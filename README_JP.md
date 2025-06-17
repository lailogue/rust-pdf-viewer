# PDF リーダー

RustとDioxusによるモダンなPDF閲覧アプリケーション。Gemini AIによる検索機能付き。

## 主な機能

- **動的ファイル選択**: 直感的なファイルダイアログインターフェースによるPDFファイルの開閉
- **高品質PDF表示**: 1000x1400解像度で品質とパフォーマンスのバランスを実現
- **連続スクロール**: 全PDFページをシームレスな連続スクロールで表示
- **AI検索機能**: Gemini 2.5 Flash APIによる知的なコンテンツ分析と用語解説
- **モダンUI**: Dioxusフレームワークによる美しくレスポンシブなインターフェース
- **最適化レイアウト**: 縦長PDF文書用に特別設計された横型AI検索パネル
- **スマートキャッシュ**: 不要な再レンダリングを防ぐ最適化ローディング付きインテリジェントページキャッシュシステム
- **クロスプラットフォーム**: ネイティブパフォーマンスのデスクトップアプリケーション

## 技術スタック

- **Dioxus** - Rust用モダンリアクティブUIフレームワーク
- **pdfium-render** - Google PDFiumライブラリによるPDFレンダリング
- **rfd** - ファイル選択用ネイティブファイルダイアログ統合
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

# macOS（Intel/x64）用PDFiumライブラリをダウンロード
curl -L -o pdfium-mac-x64.tgz \
  "https://github.com/bblanchon/pdfium-binaries/releases/latest/download/pdfium-mac-x64.tgz" && \
  tar -xzf pdfium-mac-x64.tgz -C lib --strip-components=1 && \
  rm pdfium-mac-x64.tgz

# Apple Silicon Mac用の場合はこちらを使用:
# curl -L -o pdfium-mac-arm64.tgz \
#   "https://github.com/bblanchon/pdfium-binaries/releases/latest/download/pdfium-mac-arm64.tgz" && \
#   tar -xzf pdfium-mac-arm64.tgz -C lib --strip-components=1 && \
#   rm pdfium-mac-arm64.tgz

# ライブラリが正しく配置されているか確認
ls -la lib/libpdfium.dylib

# プロジェクトをビルドして実行
cargo run
```

### アプリケーションの使用

1. **アプリケーション起動** - 引数は不要
2. **PDFファイルを開く**:
   - ヘッダーの「📁 PDFを開く」ボタンをクリック
   - ファイルダイアログからPDFファイルを選択
3. **PDFコンテンツの表示**:
   - 全ページが連続スクロール形式で表示
   - 初期ページレンダリング中は上部に読み込み進捗が表示
4. **AI検索機能**:
   - 右側パネルにGemini APIキーを入力
   - 用語や概念について質問する検索クエリを入力
   - 「検索」ボタンをクリックしてAI解説を取得
5. **ファイル管理**:
   - 「❌ 閉じる」ボタンで現在のPDFを閉じる
   - アプリケーションを再起動せずに異なるファイルを開く

## アプリケーションレイアウト

- **ヘッダーバー**: ファイル制御ボタンとアプリケーションタイトル
  - 「📁 PDFを開く」ボタンでファイル選択
  - 「❌ 閉じる」ボタンで現在のPDFを閉じる
- **ステータスバー**: 読み込み進捗とPDF情報（PDF読み込み時）
- **メインコンテンツエリア**:
  - **左パネル**: 連続スクロール付き縦長文書に最適化されたPDF表示エリア
  - **右パネル**: AI検索インターフェース
    - APIキー入力（パスワードフィールド）
    - 用語解説用検索クエリ入力
    - ローディングインジケーター付き検索ボタン
    - クリーンなテキスト表示の結果エリア
- **ウェルカム画面**: PDF未読み込み時に使用方法を表示

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
- 重複読み込み防止機能付きスマートページレベルキャッシュシステム（HashMap使用）
- CPUコア数認識による最適化並列ページレンダリング
- カラースペース最適化によるメモリ効率的な画像ハンドリング
- ノンブロッキング非同期操作によるレスポンシブUI更新
- 同一PDFファイルの不要な再レンダリングを防ぐシングルロードポリシー

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
- **PDFレンダリング**: `render_pdf_page_optimized()`関数が最適化付き高解像度ページレンダリングを処理
- **ファイル管理**: ネイティブファイルダイアログ統合による動的PDF読み込み/閉じる機能
- **AI検索**: `search_with_gemini()`非同期関数によるAPI統合
- **テキスト処理**: `clean_markdown_text()`による検索結果書式設定
- **UIコンポーネント**: モダンな状態管理とレスポンシブレイアウトを持つリアクティブDioxusコンポーネント

### 依存関係
すべての依存関係はDioxusエコシステム用に最適化され、レガシーeguiの依存関係を削除してよりクリーンで保守しやすいコードベースを実現。

## ライセンス

このプロジェクトはLICENSEファイルで指定された条件の下でライセンスされています。