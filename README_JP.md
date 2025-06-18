# PDF リーダー

RustとDioxusによるモダンなPDF閲覧アプリケーション。Gemini AIによる検索機能付き。

## 主な機能

- **動的ファイル選択**: 直感的なファイルダイアログインターフェースによるPDFファイルの開閉
- **高品質PDF表示**: 1000x1400解像度で品質とパフォーマンスのバランスを実現
- **連続スクロール**: 全PDFページをシームレスな連続スクロールで表示
- **AI検索機能**: Gemini、ChatGPT、Claude APIによる知的なコンテンツ分析と用語解説
- **単語帳機能**: AI検索結果の自動保存と管理
- **読書ブックマーク機能**: ページレベルのブックマークで読書進捗をセッション間で追跡
- **位置マーカー機能**: ページ内精密位置マーカーと自動スクロールナビゲーション
- **PDFページ回転**: 個別ページ回転機能とセッション間での状態永続化
- **最近のファイル**: 最近開いたPDFファイルの履歴管理（最大10件）
- **APIキー管理**: AIプロバイダーAPIキーの自動保存と復元
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

### Rust と Cargo
このアプリケーションには Rust と Cargo が必要です。公式ウェブサイトからインストールしてください：

**Windows、macOS、Linux:**
```bash
# rustup経由でRustをインストール（推奨）
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Windowsの場合は、以下からrustup-init.exeをダウンロードして実行:
# https://rustup.rs/

# ターミナルを再起動するか、以下を実行:
source ~/.cargo/env

# インストールの確認
cargo --version
rustc --version
```

**代替インストール方法:**

**macOS (Homebrew経由):**
```bash
brew install rust
```

**Ubuntu/Debian:**
```bash
sudo apt update
sudo apt install rustc cargo
```

**CentOS/RHEL/Fedora:**
```bash
# CentOS/RHEL
sudo yum install rust cargo

# Fedora
sudo dnf install rust cargo
```


### AI API キー
AI検索機能を利用するには、以下のいずれかのAPIキーを取得してください：

**Gemini API (Google):**
https://makersuite.google.com/app/apikey

**ChatGPT API (OpenAI):**
https://platform.openai.com/api-keys

**Claude API (Anthropic):**
https://console.anthropic.com/

## インストール・使用方法

### ソースからビルド

#### macOS

```bash
# リポジトリをクローン
git clone <repository-url>
cd rust-pdf-viewer

# PDFiumライブラリ用のlibディレクトリを作成
mkdir -p lib

# Intel Mac（x64）用PDFiumライブラリをダウンロード
curl -L -o pdfium-mac-x64.tgz \
  "https://github.com/bblanchon/pdfium-binaries/releases/latest/download/pdfium-mac-x64.tgz" && \
  tar -xzf pdfium-mac-x64.tgz -C lib --strip-components=1 && \
  rm pdfium-mac-x64.tgz

# Apple Silicon Mac（ARM64）用の場合はこちらを使用:
# curl -L -o pdfium-mac-arm64.tgz \
#   "https://github.com/bblanchon/pdfium-binaries/releases/latest/download/pdfium-mac-arm64.tgz" && \
#   tar -xzf pdfium-mac-arm64.tgz -C lib --strip-components=1 && \
#   rm pdfium-mac-arm64.tgz

# ライブラリが正しく配置されているか確認
ls -la lib/libpdfium.dylib

# プロジェクトをビルドして実行
cargo run

# macOSアプリバンドル（.appファイル）をビルドする場合
cargo install cargo-bundle
cargo bundle --release

# アプリバンドルは以下の場所に作成されます:
# target/release/bundle/osx/PDF Viewer.app
```

#### Linux

```bash
# リポジトリをクローン
git clone <repository-url>
cd rust-pdf-viewer

# PDFiumライブラリ用のlibディレクトリを作成
mkdir -p lib

# x64システム用PDFiumライブラリをダウンロード
curl -L -o pdfium-linux-x64.tgz \
  "https://github.com/bblanchon/pdfium-binaries/releases/latest/download/pdfium-linux-x64.tgz" && \
  tar -xzf pdfium-linux-x64.tgz -C lib --strip-components=1 && \
  rm pdfium-linux-x64.tgz

# ARM64システム用の場合はこちらを使用:
# curl -L -o pdfium-linux-arm64.tgz \
#   "https://github.com/bblanchon/pdfium-binaries/releases/latest/download/pdfium-linux-arm64.tgz" && \
#   tar -xzf pdfium-linux-arm64.tgz -C lib --strip-components=1 && \
#   rm pdfium-linux-arm64.tgz

# ライブラリが正しく配置されているか確認
ls -la lib/libpdfium.so

# プロジェクトをビルドして実行
cargo run
```

### アプリケーションの使用

1. **アプリケーション起動** - 引数は不要
2. **PDFファイルを開く**:
   - ヘッダーの「📁 PDFを開く」ボタンをクリック
   - または「📋 最近のファイル」から履歴を選択
   - ファイルダイアログからPDFファイルを選択
3. **PDFコンテンツの表示**:
   - 全ページが連続スクロール形式で表示
   - 初期ページレンダリング中は上部に読み込み進捗が表示
   - 個別ページ回転コントロール（90°刻み）と永続化
4. **読書進捗管理**:
   - **ブックマーク**: 「🔖 ブックマーク」でページレベルの読書ブックマークを設定
   - **位置マーカー**: 「📍 マーカーモード」を有効にしてページ内の正確な位置にマーカーを配置
   - **自動ナビゲーション**: 「📋 マーカー一覧」でマーカーをクリックして自動ページスクロール
   - **最近のファイル**: 「📋 最近のファイル」で最近開いたファイルにアクセス
5. **AI検索機能**:
   - AIプロバイダーを選択（Gemini、ChatGPT、Claude）
   - APIキーを一度入力すると自動保存され、次回起動時に復元されます
   - 用語や概念について質問する検索クエリを入力
   - 「検索」ボタンをクリックしてAI解説を取得
   - 検索した用語は自動的に単語帳に保存されます
6. **単語帳機能**:
   - 「📚 単語帳を開く」ボタンで保存された検索用語を表示
   - 各単語をクリックして詳細表示や削除が可能
7. **ファイル管理**:
   - 「❌ 閉じる」ボタンで現在のPDFを閉じる
   - アプリケーションを再起動せずに異なるファイルを開く

## アプリケーションレイアウト

- **ヘッダーバー**: ファイル制御ボタンとアプリケーションタイトル
  - 「🔖 ブックマーク」ボタンで読書進捗管理
  - 「📍 マーカーモード」トグルで位置マーカー機能
  - 「📋 マーカー一覧」で保存されたマーカーの表示とナビゲーション
  - 「📋 最近のファイル」ボタンで履歴を表示
  - 「📁 PDFを開く」ボタンでファイル選択
  - 「❌ 閉じる」ボタンで現在のPDFを閉じる
- **ステータスバー**: 読み込み進捗とPDF情報（PDF読み込み時）
- **メインコンテンツエリア**:
  - **左パネル**: 連続スクロール付き縦長文書に最適化されたPDF表示エリア
  - **右パネル**: AI検索インターフェース
    - AIプロバイダー選択（Gemini、ChatGPT、Claude）
    - APIキー入力（パスワードフィールド、自動保存・復元）
    - 用語解説用検索クエリ入力
    - ローディングインジケーター付き検索ボタン
    - クリーンなテキスト表示の結果エリア
    - 保存された検索用語を表示する単語帳ボタン
- **ウェルカム画面**: PDF未読み込み時に使用方法を表示

## データ保存

### 単語帳データ
アプリケーションは検索した用語とAI生成の定義を単語帳として自動保存し、将来の参照用に保持します。

**保存場所:**
- **macOS**: `~/.config/pdf-viewer/`
- **Linux**: `~/.config/pdf-viewer/`  
- **Windows**: `%USERPROFILE%\.config\pdf-viewer\`

**データファイル:**
- `flashcards.json` - 保存された検索用語と定義
- `recent_files.json` - 最近開いたPDFファイル（最大10件）
- `api_keys.json` - AIプロバイダーの保存されたAPIキー
- `bookmarks.json` - 各PDFファイルの読書ブックマーク
- `position_markers.json` - PDFページ内の精密位置マーカー
- `page_rotations.json` - 個別ページの回転状態

**データ形式:**
単語帳はJSON形式で以下の構造で保存されます：
```json
[
  {
    "id": "1734493200000-machine-lea",
    "term": "machine learning",
    "definition": "AI生成の用語説明...",
    "created_at": "2024-12-18 12:00:00 UTC"
  }
]
```

## 実装のハイライト

### 高品質レンダリング
- 品質とパフォーマンスのバランスを取るため1000x1400ピクセルでPDFページをレンダリング
- 全ページを一度に表示する連続スクロール表示
- 自動カラースペース変換（BGRA → RGBA）
- Web表示用のbase64データURLによるPNGエンコーディング

### AI検索統合
- UIをブロックしない非同期AI API呼び出し（Gemini、ChatGPT、Claude対応）
- APIキーの自動保存と復元機能
- 検索結果の自動単語帳保存
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
- **PDFレンダリング**: `render_pdf_page_with_text()`関数が最適化付き高解像度ページレンダリングを処理
- **ファイル管理**: ネイティブファイルダイアログ統合による動的PDF読み込み/閉じる機能と最近のファイル履歴
- **AI検索**: `search_with_ai()`非同期関数による複数AIプロバイダー統合
- **データ永続化**: 単語帳、最近のファイル、APIキーのJSON形式自動保存
- **テキスト処理**: `clean_markdown_text()`による検索結果書式設定
- **UIコンポーネント**: モダンな状態管理とレスポンシブレイアウトを持つリアクティブDioxusコンポーネント

### 依存関係
すべての依存関係はDioxusエコシステム用に最適化され、レガシーeguiの依存関係を削除してよりクリーンで保守しやすいコードベースを実現。

## ライセンス

このプロジェクトはLICENSEファイルで指定された条件の下でライセンスされています。