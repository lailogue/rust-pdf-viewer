# PDF Viewer in Rust - 技術詳細解説

## 概要

このプロジェクトは、Rust言語とDioxusフレームワークを使用して構築された高機能PDFビューアアプリケーションです。PDF表示、テキスト選択、AI検索、ブックマーク機能、位置マーカー、ページ回転など、多様な機能を統合的に提供します。

## アーキテクチャ全体像

### モジュラー設計の採用

アプリケーションは以下のモジュール構成で設計されており、関心の分離と保守性を実現しています：

```
src/
├── main.rs                     # アプリケーションエントリーポイントと統合UI
├── types/                      # 型定義とデータ構造
├── pdf/                        # PDF処理コア機能
├── ai/                         # AI検索機能
├── storage/                    # データ永続化
└── ui/                         # ユーザーインターフェース
```

## 使用ライブラリと技術スタック

### コアフレームワーク

#### Dioxus 0.5
```toml
dioxus = "0.5"
dioxus-desktop = "0.5"
```

**役割**: リアクティブUIフレームワーク
**特徴**:
- **Signal-based状態管理**: `use_signal()` によるリアクティブな状態管理
- **Virtual DOM**: 効率的なUI更新機構
- **コンポーネントベース**: 再利用可能なUIコンポーネント
- **RSX記法**: Reactライクな記法でUIを記述

**実装例**:
```rust
let mut pdf_path = use_signal(|| -> Option<PathBuf> { None });
let mut page_cache = use_signal(|| HashMap::<usize, PdfPageData>::new());

rsx! {
    div { class: "app",
        // UI構成要素
    }
}
```

### PDF処理エンジン

#### PDFium-render 0.8
```toml
pdfium-render = "0.8"
```

**役割**: PDF解析とレンダリング
**技術詳細**:
- **PDFiumライブラリ**: Google Chromeで使用されているPDFエンジン
- **Native Library Binding**: C++ライブラリのRustバインディング
- **高精度レンダリング**: ベクター形式の正確な表示
- **テキスト抽出**: フォント情報と座標を含む詳細なテキストデータ

**動的ライブラリパス解決**:
```rust
pub fn get_pdfium_library_path() -> Option<PathBuf> {
    // 開発環境とアプリバンドルの自動判別
    if let Ok(current_exe) = std::env::current_exe() {
        if current_exe.to_string_lossy().contains(".app/Contents/MacOS") {
            // macOSアプリバンドル内のパス
            let app_dir = current_exe.parent().unwrap();
            let resources_lib_path = app_dir
                .parent().unwrap_or(app_dir)
                .join("Resources")
                .join("lib")
                .join("libpdfium.dylib");
            
            if resources_lib_path.exists() {
                return Some(resources_lib_path);
            }
        }
    }
    
    // 開発環境のパス
    let dev_lib_path = PathBuf::from("lib/libpdfium.dylib");
    if dev_lib_path.exists() {
        Some(dev_lib_path)
    } else {
        None
    }
}
```

### HTTP通信とAI統合

#### Reqwest 0.12
```toml
reqwest = { version = "0.12", features = ["json"] }
```

**役割**: HTTP/HTTPSクライアント
**機能**:
- **非同期HTTP通信**: async/awaitパターンでのAPI呼び出し
- **JSON自動処理**: シリアライゼーション/デシリアライゼーション
- **TLS/SSL対応**: 安全な通信チャネル

**AI プロバイダー統合**:

##### Gemini API統合
```rust
pub async fn search_with_gemini(query: String, api_key: String) -> Result<String> {
    let client = reqwest::Client::new();
    let url = format!("https://generativelanguage.googleapis.com/v1beta/models/gemini-2.0-flash-exp:generateContent?key={}", api_key);
    
    let request_body = json!({
        "contents": [{
            "parts": [{
                "text": query
            }]
        }]
    });

    let response = client
        .post(&url)
        .header("Content-Type", "application/json")
        .json(&request_body)
        .send()
        .await?;

    // レスポンス解析とエラーハンドリング
}
```

##### ChatGPT API統合
```rust
pub async fn search_with_chatgpt(query: String, api_key: String) -> Result<String> {
    let client = reqwest::Client::new();
    
    let request_body = json!({
        "model": "gpt-4o",
        "messages": [
            {
                "role": "user",
                "content": query
            }
        ],
        "max_tokens": 1000
    });

    let response = client
        .post("https://api.openai.com/v1/chat/completions")
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .json(&request_body)
        .send()
        .await?;
}
```

##### Claude API統合
```rust
pub async fn search_with_claude(query: String, api_key: String) -> Result<String> {
    let client = reqwest::Client::new();
    
    let request_body = json!({
        "model": "claude-3-5-sonnet-20241022",
        "max_tokens": 1000,
        "messages": [
            {
                "role": "user",
                "content": query
            }
        ]
    });

    let response = client
        .post("https://api.anthropic.com/v1/messages")
        .header("x-api-key", api_key)
        .header("anthropic-version", "2023-06-01")
        .header("Content-Type", "application/json")
        .json(&request_body)
        .send()
        .await?;
}
```

### 非同期ランタイム

#### Tokio 1.0
```toml
tokio = { version = "1.0", features = ["full"] }
```

**役割**: 非同期ランタイム
**機能**:
- **マルチスレッド実行器**: 効率的なタスクスケジューリング
- **非同期I/O**: ノンブロッキングなネットワーク通信
- **並行処理**: PDFページの並列レンダリング

**並列レンダリング実装**:
```rust
pub async fn render_all_pages_parallel(pdf_path: &str, total_pages: usize) -> Result<HashMap<usize, PdfPageData>> {
    let mut page_cache = HashMap::new();
    let semaphore = Arc::new(Semaphore::new(num_cpus::get()));
    
    let tasks: Vec<_> = (0..total_pages)
        .map(|page_idx| {
            let path = pdf_path.to_string();
            let sem = semaphore.clone();
            
            tokio::spawn(async move {
                let _permit = sem.acquire().await.unwrap();
                render_pdf_page_with_text(&path, page_idx, RotationAngle::None).await
            })
        })
        .collect();

    for (page_idx, task) in tasks.into_iter().enumerate() {
        if let Ok(Ok(page_data)) = task.await {
            page_cache.insert(page_idx, page_data);
        }
    }

    Ok(page_cache)
}
```

### データシリアライゼーション

#### Serde 1.0
```toml
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
```

**役割**: データシリアライゼーション/デシリアライゼーション
**実装例**:

```rust
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct FlashCard {
    pub id: String,
    pub term: String,
    pub definition: String,
    pub created_at: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ReadingBookmark {
    pub pdf_path: String,
    pub current_page: usize,
    pub total_pages: usize,
    pub last_read_time: String,
    pub reading_progress: f32,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PositionMarker {
    pub id: String,
    pub page: usize,
    pub x: f32,  // 0.0-1.0の正規化座標
    pub y: f32,  // 0.0-1.0の正規化座標
    pub note: String,
    pub created_at: String,
}
```

### 画像処理

#### Image 0.25
```toml
image = "0.25"
```

**役割**: 画像フォーマット変換
**機能**:
- **Base64エンコーディング**: PDFページ画像のWeb表示対応
- **フォーマット変換**: PNG形式での画像出力

```rust
pub fn render_pdf_page_with_text(pdf_path: &str, page_index: usize, rotation: RotationAngle) -> Result<PdfPageData> {
    // PDFページを画像としてレンダリング
    let render_config = PdfRenderConfig::new()
        .set_target_width(1000)
        .set_maximum_height(2000)
        .rotate_if_landscape(PdfPageRenderRotation::Degrees90, true);

    let page = document.pages().get(page_index)?;
    let bitmap = page.render_with_config(&render_config)?;
    
    // 画像をBase64に変換
    let image = bitmap.as_image();
    let mut buffer = Vec::new();
    let mut cursor = std::io::Cursor::new(&mut buffer);
    image.write_to(&mut cursor, image::ImageFormat::Png)?;
    let base64_image = base64::encode(&buffer);
    let image_data = format!("data:image/png;base64,{}", base64_image);

    Ok(PdfPageData {
        page_index,
        image_data,
        page_width: bitmap.width() as f32,
        page_height: bitmap.height() as f32,
        text_elements,
    })
}
```

### ファイルシステム操作

#### dirs 5.0
```toml
dirs = "5.0"
```

**役割**: システムディレクトリ取得
**実装**:
```rust
pub fn ensure_data_dir() -> Result<PathBuf> {
    let config_dir = dirs::config_dir()
        .ok_or_else(|| anyhow::anyhow!("設定ディレクトリが見つかりません"))?;
    
    let data_dir = config_dir.join("pdf-viewer");
    
    if !data_dir.exists() {
        std::fs::create_dir_all(&data_dir)?;
    }
    
    Ok(data_dir)
}
```

#### RFD 0.15.3
```toml
rfd = "0.15.3"
```

**役割**: ネイティブファイルダイアログ
**機能**:
- **非同期ファイル選択**: `AsyncFileDialog`
- **フィルター設定**: PDFファイルの選択制限

```rust
spawn(async move {
    if let Some(file_handle) = rfd::AsyncFileDialog::new()
        .add_filter("PDF files", &["pdf"])
        .set_title("PDFファイルを選択")
        .pick_file()
        .await 
    {
        let selected_path = file_handle.path().to_path_buf();
        pdf_path.set(Some(selected_path));
    }
});
```

## 詳細機能実装

### PDF座標系変換システム

PDFの座標系（左下原点）とHTML/CSS座標系（左上原点）の変換は、正確なテキスト配置の核心技術です：

```rust
pub fn convert_pdf_coordinates_to_html(
    pdf_x: f32, 
    pdf_y: f32, 
    pdf_width: f32, 
    pdf_height: f32,
    page_width: f32, 
    page_height: f32
) -> (f32, f32) {
    // PDF座標系 (左下原点) から HTML座標系 (左上原点) への変換
    let html_x = pdf_x;
    let html_y = page_height - pdf_y - pdf_height;
    
    // 百分率による正規化
    let x_percent = (html_x / page_width) * 100.0;
    let y_percent = (html_y / page_height) * 100.0;
    
    (x_percent, y_percent)
}
```

### テキスト重複フィルタリング

PDF解析時に発生する重複テキストを除去する高度なアルゴリズム：

```rust
pub fn filter_overlapping_text(text_elements: Vec<TextElement>) -> Vec<TextElement> {
    let mut filtered = Vec::new();
    
    for current in text_elements {
        let mut is_duplicate = false;
        
        for existing in &filtered {
            // 同一テキストで座標が近似している場合は重複と判定
            if current.text.trim() == existing.text.trim() && 
               (current.bounds.x - existing.bounds.x).abs() < 2.0 &&
               (current.bounds.y - existing.bounds.y).abs() < 2.0 {
                is_duplicate = true;
                break;
            }
            
            // 完全に包含される場合も重複と判定
            if current.bounds.is_contained_in(&existing.bounds) && 
               existing.text.contains(&current.text.trim()) {
                is_duplicate = true;
                break;
            }
        }
        
        if !is_duplicate {
            filtered.push(current);
        }
    }
    
    filtered
}
```

### AI検索システムの詳細実装

#### プロンプト戦略
- **通常検索**: `"{}とはなんですか。簡潔に説明してください"`
- **詳細検索**: `"{}とはなんですか。詳細に説明してください"`

#### 詳細検索による単語帳更新
```rust
pub fn append_detailed_explanation(card_id: &str, detailed_explanation: String) -> Result<()> {
    let mut flashcards = load_flashcards();
    
    if let Some(card) = flashcards.iter_mut().find(|card| card.id == card_id) {
        // 既存の説明に詳細説明を追記（"==========="区切り）
        card.definition = format!("{}\n===========\n{}", card.definition, detailed_explanation);
        card.created_at = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC").to_string();
        save_flashcards(&flashcards)?;
        Ok(())
    } else {
        Err(anyhow::anyhow!("Flashcard with id {} not found", card_id))
    }
}
```

### 高度な状態管理システム

Dioxusのリアクティブシステムを最大活用した状態管理：

```rust
// PDF表示関連状態
let mut pdf_path = use_signal(|| -> Option<PathBuf> { None });
let mut page_cache = use_signal(|| HashMap::<usize, PdfPageData>::new());
let mut loaded_pdf_path = use_signal(|| -> Option<PathBuf> { None });

// AI検索関連状態
let mut selected_provider = use_signal(|| AIProvider::Gemini);
let mut search_query = use_signal(|| String::new());
let mut search_result = use_signal(|| String::new());
let mut is_searching = use_signal(|| false);

// 詳細検索専用状態
let detail_search_result = use_signal(|| String::new());
let is_detail_searching = use_signal(|| false);
let detail_search_term = use_signal(|| String::new());

// 単語帳関連状態
let mut flashcards = use_signal(|| load_flashcards());
let selected_flashcard = use_signal(|| -> Option<FlashCard> { None });
let mut show_flashcard_popup = use_signal(|| false);
let show_flashcard_details = use_signal(|| false);

// ブックマーク関連状態
let mut current_bookmark = use_signal(|| -> Option<ReadingBookmark> { None });
let mut show_bookmarks_popup = use_signal(|| false);

// 位置マーカー関連状態
let mut position_markers = use_signal(|| Vec::<PositionMarker>::new());
let mut show_markers_popup = use_signal(|| false);
let mut marker_mode = use_signal(|| false);

// ページ回転関連状態
let mut page_rotations = use_signal(|| HashMap::<usize, RotationAngle>::new());
```

### メモ化による最適化

計算量の多い処理をメモ化で最適化：

```rust
// 単語帳リストをメモ化
let flashcard_list = use_memo(move || flashcards());
let recent_files_list = use_memo(move || recent_files());

// PDFファイル情報の取得をメモ化
let (total_pages, pdf_info) = use_memo(move || {
    if let Some(path) = pdf_path() {
        pdf::get_pdf_info(&path.to_string_lossy()).unwrap_or((0, "PDFの読み込みに失敗しました".to_string()))
    } else {
        (0, "PDFファイルが選択されていません".to_string())
    }
})();
```

### データ永続化の詳細実装

#### ファイルハッシュベース保存
```rust
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

pub fn get_pdf_hash(pdf_path: &str) -> String {
    let mut hasher = DefaultHasher::new();
    pdf_path.hash(&mut hasher);
    format!("{:x}", hasher.finish())
}

pub fn save_position_markers(pdf_path: &str, markers: Vec<PositionMarker>) -> Result<()> {
    let data_dir = ensure_data_dir()?;
    let hash = get_pdf_hash(pdf_path);
    let markers_path = data_dir.join(format!("position_markers_{}.json", hash));
    
    let json = serde_json::to_string_pretty(&markers)?;
    std::fs::write(&markers_path, json)?;
    
    Ok(())
}
```

### UIコンポーネント設計

#### ポップアップコンポーネントパターン
```rust
#[component]
pub fn flashcard_details_popup(
    show_flashcard_details: Signal<bool>,
    selected_flashcard: Signal<Option<FlashCard>>,
    flashcards: Signal<Vec<FlashCard>>,
    selected_provider: Signal<AIProvider>,
    gemini_api_key: String,
    chatgpt_api_key: String,
    claude_api_key: String,
    detail_search_result: Signal<String>,
    is_detail_searching: Signal<bool>,
    detail_search_term: Signal<String>,
) -> Element {
    if let Some(ref card) = selected_flashcard() {
        rsx! {
            div { 
                class: "details-overlay",
                style: "position: fixed; top: 0; left: 0; width: 100%; height: 100%; background-color: rgba(0, 0, 0, 0.8); display: flex; align-items: center; justify-content: center; z-index: 1001;",
                onclick: move |_| {
                    show_flashcard_details.set(false);
                    selected_flashcard.set(None);
                },
                // コンポーネント内容
            }
        }
    } else {
        rsx! { div {} }
    }
}
```

### テキスト選択とスペーシング

PDF上でのテキスト選択時に適切な単語間隔を確保：

```rust
// 各テキスト要素に末尾スペースを追加
span {
    key: "p{page_idx}t{text_idx}",
    class: "selectable-text",
    style: "position: absolute; 
           left: {text_elem.bounds.x / page_data.page_width * 100.0}%; 
           top: {text_elem.bounds.y / page_data.page_height * 100.0}%;
           width: {text_elem.bounds.width / page_data.page_width * 100.0}%;
           height: {text_elem.bounds.height / page_data.page_height * 100.0}%;
           font-size: {(text_elem.font_size / page_data.page_height * 100.0).max(0.8)}%;
           color: transparent;
           pointer-events: auto;
           user-select: text;
           cursor: text;
           font-family: monospace;
           line-height: 1;
           overflow: hidden;
           white-space: nowrap;
           z-index: 2;",
    "{text_elem.text} "  // 末尾スペースで単語間隔を確保
}
```

## パフォーマンス最適化

### 並列処理による高速化

CPUコア数を考慮した並列レンダリング：

```rust
use std::sync::Arc;
use tokio::sync::Semaphore;

pub async fn render_pages_in_batches(pdf_path: &str, total_pages: usize) -> Result<HashMap<usize, PdfPageData>> {
    let cpu_count = num_cpus::get();
    let semaphore = Arc::new(Semaphore::new(cpu_count));
    
    let mut page_cache = HashMap::new();
    let mut tasks = Vec::new();
    
    for page_idx in 0..total_pages {
        let path = pdf_path.to_string();
        let sem = semaphore.clone();
        
        let task = tokio::spawn(async move {
            let _permit = sem.acquire().await.unwrap();
            render_pdf_page_with_text(&path, page_idx, RotationAngle::None).await
        });
        
        tasks.push((page_idx, task));
    }
    
    // 結果を順次処理
    for (page_idx, task) in tasks {
        if let Ok(Ok(page_data)) = task.await {
            page_cache.insert(page_idx, page_data);
        }
    }
    
    Ok(page_cache)
}
```

### メモリ効率的なキャッシュ管理

```rust
// ページキャッシュの動的管理
let mut page_cache = use_signal(|| HashMap::<usize, PdfPageData>::new());

// 新しいPDFファイル読み込み時にキャッシュクリア
pdf_path.set(Some(selected_path));
page_cache.write().clear();
loaded_pdf_path.set(None);
```

## エラーハンドリング戦略

### 包括的エラー処理

anyhowクレートによる統一的エラーハンドリング：

```rust
use anyhow::{Result, Context};

pub async fn search_with_ai(provider: AIProvider, query: String, api_key: String) -> Result<String> {
    match provider {
        AIProvider::Gemini => {
            search_with_gemini(query, api_key)
                .await
                .context("Gemini API呼び出しに失敗しました")
        },
        AIProvider::ChatGPT => {
            search_with_chatgpt(query, api_key)
                .await
                .context("ChatGPT API呼び出しに失敗しました")
        },
        AIProvider::Claude => {
            search_with_claude(query, api_key)
                .await
                .context("Claude API呼び出しに失敗しました")
        },
    }
}
```

## セキュリティ考慮事項

### APIキー管理

現在の実装ではローカルストレージに平文で保存されており、将来的な暗号化対応が推奨されます：

```rust
pub fn save_api_keys(api_keys: &ApiKeys) -> Result<()> {
    let data_dir = ensure_data_dir()?;
    let api_keys_path = data_dir.join("api_keys.json");
    
    // TODO: 暗号化して保存
    let json = serde_json::to_string_pretty(api_keys)?;
    std::fs::write(&api_keys_path, json)?;
    
    Ok(())
}
```

## 今後の拡張可能性

### テスト戦略
現在はテストケースが未実装ですが、以下の構造でテストを追加することが推奨されます：

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_pdf_rendering() {
        // PDFレンダリングのテスト
    }
    
    #[tokio::test]
    async fn test_ai_search_integration() {
        // AI検索機能のテスト
    }
    
    #[test]
    fn test_coordinate_conversion() {
        // 座標変換のテスト
    }
}
```

### 性能監視
```rust
use std::time::Instant;

pub async fn render_pdf_page_with_monitoring(pdf_path: &str, page_index: usize) -> Result<PdfPageData> {
    let start = Instant::now();
    
    let result = render_pdf_page_with_text(pdf_path, page_index, RotationAngle::None).await;
    
    let duration = start.elapsed();
    log::info!("Page {} rendered in {:?}", page_index, duration);
    
    result
}
```

## まとめ

このPDFビューアアプリケーションは、Rustエコシステムの最新技術を活用して構築された高機能なデスクトップアプリケーションです。モジュラー設計、非同期処理、リアクティブUI、AI統合など、現代的なソフトウェア開発のベストプラクティスを実装しています。

特に注目すべき技術的成果：
- **PDFium統合**: ネイティブライブラリとの効率的な統合
- **座標系変換**: PDF座標とHTML座標の正確な変換
- **AI統合**: 複数プロバイダーの統一的な処理
- **リアクティブUI**: Signalベースの効率的な状態管理
- **並列処理**: CPU効率を最大化したレンダリング

これらの実装により、ユーザーにとって直感的で高性能なPDF閲覧体験を提供しています。