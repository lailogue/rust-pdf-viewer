
use anyhow::Result;
use dioxus::prelude::*;
use std::collections::HashMap;
use std::path::PathBuf;
use pdfium_render::prelude::*;
use serde::{Deserialize, Serialize};
use base64::Engine;

#[derive(Clone, PartialEq)]
enum AIProvider {
    Gemini,
    ChatGPT,
    Claude,
}

#[derive(Clone, Debug, PartialEq)]
struct PdfPageData {
    image_data: String,
    text_elements: Vec<TextElement>,
    page_width: f32,
    page_height: f32,
    page_index: usize, // 混入チェック用
}

#[derive(Clone, Debug, PartialEq)]
struct TextElement {
    text: String,
    bounds: TextBounds,
    font_size: f32,
}

#[derive(Clone, Debug, PartialEq)]
struct TextBounds {
    x: f32,
    y: f32,
    width: f32,
    height: f32,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct FlashCard {
    id: String,
    term: String,
    definition: String,
    created_at: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct RecentFile {
    path: String,
    display_name: String,
    last_opened: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct ApiKeys {
    gemini: String,
    chatgpt: String,
    claude: String,
}

impl Default for ApiKeys {
    fn default() -> Self {
        Self {
            gemini: String::new(),
            chatgpt: String::new(),
            claude: String::new(),
        }
    }
}

impl FlashCard {
    fn new(term: String, definition: String) -> Self {
        let id = format!("{}-{}", 
            chrono::Utc::now().timestamp_millis(),
            term.chars().take(10).collect::<String>().replace(" ", "-")
        );
        Self {
            id,
            term,
            definition,
            created_at: chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC").to_string(),
        }
    }
}

// FlashCard管理用のヘルパー関数
fn load_flashcards() -> Vec<FlashCard> {
    let path = get_flashcards_file_path();
    if !path.exists() {
        return Vec::new();
    }
    
    match std::fs::read_to_string(&path) {
        Ok(content) => {
            serde_json::from_str(&content).unwrap_or_else(|_| Vec::new())
        }
        Err(_) => Vec::new()
    }
}

fn save_flashcards(flashcards: &Vec<FlashCard>) -> Result<()> {
    let path = get_flashcards_file_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    
    let content = serde_json::to_string_pretty(flashcards)?;
    std::fs::write(&path, content)?;
    Ok(())
}

fn get_flashcards_file_path() -> PathBuf {
    let mut path = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
    path.push("pdf-viewer");
    path.push("flashcards.json");
    path
}

fn add_flashcard(term: String, definition: String) -> Result<()> {
    let mut flashcards = load_flashcards();
    
    // 重複チェック（同じ単語の場合は更新）
    if let Some(existing) = flashcards.iter_mut().find(|card| card.term == term) {
        existing.definition = definition;
        existing.created_at = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC").to_string();
    } else {
        flashcards.push(FlashCard::new(term, definition));
    }
    
    save_flashcards(&flashcards)
}

fn delete_flashcard(card_id: String) -> Result<()> {
    let mut flashcards = load_flashcards();
    flashcards.retain(|card| card.id != card_id);
    save_flashcards(&flashcards)
}

// 最近開いたファイル管理用のヘルパー関数
fn load_recent_files() -> Vec<RecentFile> {
    let path = get_recent_files_path();
    if !path.exists() {
        return Vec::new();
    }
    
    match std::fs::read_to_string(&path) {
        Ok(content) => {
            serde_json::from_str(&content).unwrap_or_else(|_| Vec::new())
        }
        Err(_) => Vec::new()
    }
}

fn save_recent_files(recent_files: &Vec<RecentFile>) -> Result<()> {
    let path = get_recent_files_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    
    let content = serde_json::to_string_pretty(recent_files)?;
    std::fs::write(&path, content)?;
    Ok(())
}

fn get_recent_files_path() -> PathBuf {
    let mut path = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
    path.push("pdf-viewer");
    path.push("recent_files.json");
    path
}

fn add_recent_file(file_path: &PathBuf) -> Result<()> {
    let mut recent_files = load_recent_files();
    
    let display_name = file_path
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();
    
    let path_str = file_path.to_string_lossy().to_string();
    let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC").to_string();
    
    // 既存のエントリを削除（重複回避）
    recent_files.retain(|file| file.path != path_str);
    
    // 新しいエントリを先頭に追加
    recent_files.insert(0, RecentFile {
        path: path_str,
        display_name,
        last_opened: now,
    });
    
    // 最大10件に制限
    recent_files.truncate(10);
    
    save_recent_files(&recent_files)
}

// APIキー管理用のヘルパー関数
fn load_api_keys() -> ApiKeys {
    let path = get_api_keys_path();
    if !path.exists() {
        return ApiKeys::default();
    }
    
    match std::fs::read_to_string(&path) {
        Ok(content) => {
            serde_json::from_str(&content).unwrap_or_else(|_| ApiKeys::default())
        }
        Err(_) => ApiKeys::default()
    }
}

fn save_api_keys(api_keys: &ApiKeys) -> Result<()> {
    let path = get_api_keys_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    
    let content = serde_json::to_string_pretty(api_keys)?;
    std::fs::write(&path, content)?;
    Ok(())
}

fn get_api_keys_path() -> PathBuf {
    let mut path = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
    path.push("pdf-viewer");
    path.push("api_keys.json");
    path
}


#[derive(Serialize, Deserialize)]
struct GeminiRequest {
    contents: Vec<Content>,
}

#[derive(Serialize, Deserialize)]
struct Content {
    parts: Vec<Part>,
}

#[derive(Serialize, Deserialize)]
struct Part {
    text: String,
}

#[derive(Serialize, Deserialize)]
struct GeminiResponse {
    candidates: Vec<Candidate>,
}

#[derive(Serialize, Deserialize)]
struct Candidate {
    content: Content,
}

// ChatGPT API用の構造体
#[derive(Serialize, Deserialize)]
struct ChatGPTRequest {
    model: String,
    messages: Vec<ChatMessage>,
    max_tokens: u32,
    temperature: f32,
}

#[derive(Serialize, Deserialize)]
struct ChatMessage {
    role: String,
    content: String,
}

#[derive(Serialize, Deserialize)]
struct ChatGPTResponse {
    choices: Vec<ChatChoice>,
}

#[derive(Serialize, Deserialize)]
struct ChatChoice {
    message: ChatMessage,
}

// Claude API用の構造体
#[derive(Serialize, Deserialize)]
struct ClaudeRequest {
    model: String,
    max_tokens: u32,
    messages: Vec<ClaudeMessage>,
}

#[derive(Serialize, Deserialize)]
struct ClaudeMessage {
    role: String,
    content: String,
}

#[derive(Serialize, Deserialize)]
struct ClaudeResponse {
    content: Vec<ClaudeContent>,
}

#[derive(Serialize, Deserialize)]
struct ClaudeContent {
    text: String,
}

fn get_pdfium_library_path() -> PathBuf {
    // Check if we're running in a macOS app bundle
    if let Ok(exe_path) = std::env::current_exe() {
        // Check if we're in an app bundle structure (Contents/MacOS/)
        if let Some(macos_dir) = exe_path.parent() {
            if let Some(contents_dir) = macos_dir.parent() {
                if let Some(dir_name) = contents_dir.file_name() {
                    if dir_name == "Contents" {
                        // We're in an app bundle, look in Resources/lib/
                        let bundle_lib_path = contents_dir.join("Resources").join("lib").join("libpdfium.dylib");
                        if bundle_lib_path.exists() {
                            println!("Using PDFium from app bundle: {:?}", bundle_lib_path);
                            return bundle_lib_path;
                        }
                    }
                }
            }
        }
    }
    
    // Fallback to development path
    let dev_path = std::env::current_dir()
        .unwrap_or_default()
        .join("lib")
        .join("libpdfium.dylib");
    
    println!("Using PDFium from development path: {:?}", dev_path);
    dev_path
}

fn filter_overlapping_text(mut text_elements: Vec<TextElement>) -> Vec<TextElement> {
    // ページ内のテキスト重なりを防ぐためのフィルタリング
    let mut filtered_elements: Vec<TextElement> = Vec::new();
    
    // Y座標、次にX座標でソートして順序良く処理
    text_elements.sort_by(|a, b| {
        match a.bounds.y.partial_cmp(&b.bounds.y) {
            Some(std::cmp::Ordering::Equal) => a.bounds.x.partial_cmp(&b.bounds.x).unwrap_or(std::cmp::Ordering::Equal),
            Some(other) => other,
            None => std::cmp::Ordering::Equal,
        }
    });
    
    for element in text_elements {
        let mut should_add = true;
        
        // 既に追加された要素との重なりをチェック（より厳格に）
        for existing in &filtered_elements {
            let overlap_threshold = 0.7; // 70%以上の重なりで除外
            let significant_overlap_x = (element.bounds.x < existing.bounds.x + existing.bounds.width * overlap_threshold) &&
                                       (element.bounds.x + element.bounds.width * overlap_threshold > existing.bounds.x);
            let significant_overlap_y = (element.bounds.y < existing.bounds.y + existing.bounds.height * overlap_threshold) &&
                                       (element.bounds.y + element.bounds.height * overlap_threshold > existing.bounds.y);
            
            if significant_overlap_x && significant_overlap_y {
                // 重なりがある場合、より意味のあるテキスト（長さと位置）を保持
                if element.text.trim().len() <= existing.text.trim().len() {
                    should_add = false;
                    break;
                }
            }
        }
        
        // 空のテキストや意味のないテキストを除外
        if element.text.trim().is_empty() || element.text.trim().len() < 1 {
            should_add = false;
        }
        
        if should_add {
            filtered_elements.push(element);
        }
    }
    
    filtered_elements
}

fn clean_markdown_text(text: &str) -> String {
    // **太字** の記号を削除
    text.replace("**", "")
        .replace("*", "")
        .replace("###", "")
        .replace("##", "")
        .replace("#", "")
        .replace("```", "")
        .replace("`", "")
}

async fn search_with_gemini(query: String, api_key: String) -> Result<String> {
    let client = reqwest::Client::new();
    let url = format!(
        "https://generativelanguage.googleapis.com/v1beta/models/gemini-2.5-flash-preview-05-20:generateContent?key={}",
        api_key
    );

    let request = GeminiRequest {
        contents: vec![Content {
            parts: vec![Part {
                text: format!("{}とは何ですか。簡潔に説明してください", query),
            }],
        }],
    };

    let response = client
        .post(&url)
        .header("Content-Type", "application/json")
        .json(&request)
        .send()
        .await?
        .json::<GeminiResponse>()
        .await?;

    if let Some(candidate) = response.candidates.first() {
        if let Some(part) = candidate.content.parts.first() {
            Ok(clean_markdown_text(&part.text))
        } else {
            Err(anyhow::anyhow!("レスポンスにテキストが含まれていません"))
        }
    } else {
        Err(anyhow::anyhow!("レスポンスに候補が含まれていません"))
    }
}

async fn search_with_chatgpt(query: String, api_key: String) -> Result<String> {
    let client = reqwest::Client::new();
    let url = "https://api.openai.com/v1/chat/completions";

    let request = ChatGPTRequest {
        model: "gpt-4o".to_string(),
        messages: vec![ChatMessage {
            role: "user".to_string(),
            content: format!("{}とは何ですか。簡潔に説明してください", query),
        }],
        max_tokens: 500,
        temperature: 0.7,
    };

    let response = client
        .post(url)
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", api_key))
        .json(&request)
        .send()
        .await?
        .json::<ChatGPTResponse>()
        .await?;

    if let Some(choice) = response.choices.first() {
        Ok(clean_markdown_text(&choice.message.content))
    } else {
        Err(anyhow::anyhow!("レスポンスに候補が含まれていません"))
    }
}

async fn search_with_claude(query: String, api_key: String) -> Result<String> {
    let client = reqwest::Client::new();
    let url = "https://api.anthropic.com/v1/messages";

    let request = ClaudeRequest {
        model: "claude-3-5-sonnet-20241022".to_string(),
        max_tokens: 500,
        messages: vec![ClaudeMessage {
            role: "user".to_string(),
            content: format!("{}とは何ですか。簡潔に説明してください", query),
        }],
    };

    let response = client
        .post(url)
        .header("Content-Type", "application/json")
        .header("x-api-key", api_key)
        .header("anthropic-version", "2023-06-01")
        .json(&request)
        .send()
        .await?
        .json::<ClaudeResponse>()
        .await?;

    if let Some(content) = response.content.first() {
        Ok(clean_markdown_text(&content.text))
    } else {
        Err(anyhow::anyhow!("レスポンスに候補が含まれていません"))
    }
}

async fn search_with_ai(provider: AIProvider, query: String, api_key: String) -> Result<String> {
    let result = match provider {
        AIProvider::Gemini => search_with_gemini(query.clone(), api_key).await,
        AIProvider::ChatGPT => search_with_chatgpt(query.clone(), api_key).await,
        AIProvider::Claude => search_with_claude(query.clone(), api_key).await,
    };
    
    // 検索が成功した場合、単語帳に保存
    if let Ok(ref definition) = result {
        if !query.trim().is_empty() && !definition.trim().is_empty() {
            let _ = add_flashcard(query.trim().to_string(), definition.clone());
        }
    }
    
    result
}

fn render_pdf_page_with_text(pdf_path: &PathBuf, page_index: usize) -> Option<PdfPageData> {
    println!("DEBUG: Starting page {} processing", page_index);
    let lib_path = get_pdfium_library_path();
    let bindings = Pdfium::bind_to_library(&lib_path).ok()?;
    let pdfium = Pdfium::new(bindings);
    let document = pdfium.load_pdf_from_file(pdf_path, None).ok()?;
    let page = document.pages().get(page_index as u16).ok()?;
    
    // ページが正しく取得されているか確認
    let actual_page_count = document.pages().len();
    if page_index >= actual_page_count as usize {
        println!("ERROR: Page index {} out of bounds (total pages: {})", page_index, actual_page_count);
        return None;
    }
    
    // レンダリング設定で適切な処理を行う
    let render_config = PdfRenderConfig::new()
        .set_target_width(1000)
        .set_maximum_height(1400)
        .rotate_if_landscape(PdfPageRenderRotation::Degrees90, true);

    let bitmap = page.render_with_config(&render_config).ok()?;
    let width = bitmap.width() as usize;
    let height = bitmap.height() as usize;
    
    // Get page dimensions for coordinate scaling
    let page_width = page.width().value;
    let page_height = page.height().value;
    let scale_x = bitmap.width() as f32 / page_width;
    let scale_y = bitmap.height() as f32 / page_height;
    
    // ページ固有の情報を調査
    let _rotation = page.rotation();
    let is_landscape = page_width > page_height;
    let _was_rotated = is_landscape && (bitmap.width() < bitmap.height());
    
    if page_index < 3 {
        println!("Page {}: PDF {}x{} -> Bitmap {}x{}, Scale: {:.3}x{:.3}", 
            page_index, page_width, page_height, bitmap.width(), bitmap.height(), scale_x, scale_y);
        println!("  Landscape: {}, Rotation: {:?}", is_landscape, page.rotation());
    }
    
    // Extract text with coordinates using proper pdfium-render API
    let mut text_elements = Vec::new();
    
    if let Ok(page_text) = page.text() {
        let chars = page_text.chars();
        let mut current_word = String::new();
        let mut word_bounds: Option<TextBounds> = None;
        let mut word_font_size = 0.0;
        
        // レンダリング設定を考慮した座標変換の計算
        let render_width = bitmap.width() as f32;
        let render_height = bitmap.height() as f32;
        
        for char_index in 0..chars.len() {
            if let Ok(char_obj) = chars.get(char_index as usize) {
                if let Some(char_string) = char_obj.unicode_string() {
                    if let Ok(bounds) = char_obj.loose_bounds() {
                        // PDF座標を取得
                        let pdf_left = bounds.left().value;
                        let pdf_right = bounds.right().value;
                        let pdf_top = bounds.top().value;
                        let pdf_bottom = bounds.bottom().value;
                        
                        // 基本的な座標変換
                        // PDFは左下原点、HTMLは左上原点なので、Y座標を反転
                        let transformed_x = pdf_left * scale_x;
                        let transformed_y = render_height - (pdf_top * scale_y);
                        
                        let scaled_bounds = TextBounds {
                            x: transformed_x,
                            y: transformed_y,
                            width: (pdf_right - pdf_left) * scale_x,
                            height: (pdf_top - pdf_bottom) * scale_y,
                        };
                        
                        // 詳細なデバッグ情報
                        if char_index < 5 && page_index == 0 {
                            println!("Page {} Char '{}': PDF({:.1},{:.1},{:.1},{:.1}) -> Rendered({:.1},{:.1}) Size({:.1}x{:.1})", 
                                page_index, char_string, pdf_left, pdf_bottom, pdf_right, pdf_top,
                                scaled_bounds.x, scaled_bounds.y, scaled_bounds.width, scaled_bounds.height);
                        }
                        
                        // 文字を単語にグループ化（より良い選択体験のため）
                        if char_string.trim().is_empty() {
                            // スペースまたは空白 - 現在の単語を終了
                            if !current_word.is_empty() {
                                if let Some(bounds) = word_bounds {
                                    text_elements.push(TextElement {
                                        text: current_word.clone(),
                                        bounds,
                                        font_size: word_font_size,
                                    });
                                }
                                current_word.clear();
                                word_bounds = None;
                            }
                        } else {
                            // 通常の文字 - 現在の単語に追加
                            current_word.push_str(&char_string);
                            word_font_size = scaled_bounds.height;
                            
                            if let Some(ref mut existing_bounds) = word_bounds {
                                // 既存の単語境界を拡張
                                let right_edge = scaled_bounds.x + scaled_bounds.width;
                                existing_bounds.width = right_edge - existing_bounds.x;
                                existing_bounds.height = existing_bounds.height.max(scaled_bounds.height);
                                // Y座標は最初の文字の位置を維持
                            } else {
                                // 新しい単語境界を開始
                                word_bounds = Some(scaled_bounds);
                            }
                        }
                    }
                }
            }
        }
        
        // 最後の単語を忘れないように
        if !current_word.is_empty() {
            if let Some(bounds) = word_bounds {
                text_elements.push(TextElement {
                    text: current_word,
                    bounds,
                    font_size: word_font_size,
                });
            }
        }
        
        // 重なりを防ぐためのフィルタリング
        text_elements = filter_overlapping_text(text_elements);
        
        println!("Page {}: Extracted {} text elements (after filtering)", page_index, text_elements.len());
        
        // デバッグ: ページの最初と最後のテキストを表示（混入チェック）
        if !text_elements.is_empty() && page_index < 5 {
            println!("  Page {} first 3 texts: {:?}", page_index, 
                text_elements.iter().take(3).map(|e| &e.text).collect::<Vec<_>>());
            if text_elements.len() > 3 {
                println!("  Page {} last 3 texts: {:?}", page_index,
                    text_elements.iter().rev().take(3).map(|e| &e.text).collect::<Vec<_>>());
            }
            
            // 座標の妥当性をチェック
            let sample = &text_elements[0];
            println!("  Sample text '{}' at ({:.1}%, {:.1}%) size {:.1}%", 
                sample.text, 
                sample.bounds.x / render_width * 100.0,
                sample.bounds.y / render_height * 100.0,
                sample.bounds.height / render_height * 100.0);
        }
    }
    
    // Render image (existing logic)
    let image_buffer = bitmap.as_raw_bytes();
    let mut rgba_pixels = Vec::with_capacity(width * height * 4);
    
    for chunk in image_buffer.chunks_exact(4) {
        rgba_pixels.push(chunk[2]); // R
        rgba_pixels.push(chunk[1]); // G  
        rgba_pixels.push(chunk[0]); // B
        rgba_pixels.push(chunk[3]); // A
    }

    let image = image::RgbaImage::from_vec(width as u32, height as u32, rgba_pixels)?;
    let mut png_data = Vec::new();
    image::DynamicImage::ImageRgba8(image)
        .write_to(&mut std::io::Cursor::new(&mut png_data), image::ImageFormat::Png)
        .ok()?;
    
    let image_data = format!("data:image/png;base64,{}", base64::engine::general_purpose::STANDARD.encode(&png_data));
    
    let result = PdfPageData {
        image_data,
        text_elements,
        page_width: bitmap.width() as f32,
        page_height: bitmap.height() as f32,
        page_index,
    };
    
    println!("DEBUG: Completed page {} processing successfully", page_index);
    Some(result)
}


fn get_pdf_info(pdf_path: &PathBuf) -> (usize, String) {
    let lib_path = get_pdfium_library_path();
    match Pdfium::bind_to_library(&lib_path) {
        Ok(bindings) => {
            let pdfium = Pdfium::new(bindings);
            let document_result = pdfium.load_pdf_from_file(pdf_path, None);
            match document_result {
                Ok(doc) => {
                    let page_count = doc.pages().len() as usize;
                    let mut info = format!("Pages: {}\n", page_count);
                    info.push_str("File: ");
                    info.push_str(pdf_path.file_name().unwrap_or_default().to_string_lossy().as_ref());
                    info.push_str("\n(PDF successfully loaded with pdfium-render)");
                    (page_count, info)
                }
                Err(e) => (0, format!("Failed to load PDF: {}", e))
            }
        }
        Err(e) => (0, format!("Failed to initialize Pdfium: {}", e))
    }
}

fn main() -> Result<()> {
    // 引数は任意にして、アプリケーション内でファイル選択できるようにする

    let config = dioxus_desktop::Config::new()
        .with_window(
            dioxus_desktop::tao::window::WindowBuilder::new()
                .with_title("PDF Viewer - Dioxus")
                .with_inner_size(dioxus_desktop::tao::dpi::LogicalSize::new(1200.0, 800.0))
        );
    dioxus_desktop::launch::launch(App, vec![], config);
    
    Ok(())
}

fn App() -> Element {
    // PDFファイルパスの状態管理（初期値はNone）
    let mut pdf_path = use_signal(|| -> Option<PathBuf> {
        let args: Vec<String> = std::env::args().collect();
        if args.len() >= 2 {
            Some(PathBuf::from(&args[1]))
        } else {
            None
        }
    });
    
    let mut selected_provider = use_signal(|| AIProvider::Gemini);
    
    // APIキーを保存済みのものから読み込み
    let saved_api_keys = use_signal(|| load_api_keys());
    let mut gemini_api_key = use_signal(|| saved_api_keys().gemini);
    let mut chatgpt_api_key = use_signal(|| saved_api_keys().chatgpt);
    let mut claude_api_key = use_signal(|| saved_api_keys().claude);
    let mut search_query = use_signal(|| String::new());
    let mut search_result = use_signal(|| String::new());
    let mut is_searching = use_signal(|| false);
    let mut page_cache = use_signal(|| HashMap::<usize, PdfPageData>::new());
    let mut is_loading = use_signal(|| false);
    let mut error_message = use_signal(|| String::new());
    let mut loaded_pdf_path = use_signal(|| -> Option<PathBuf> { None }); // 読み込み済みのPDFパスを追跡
    
    // 単語帳関連の状態管理
    let mut flashcards = use_signal(|| load_flashcards());
    let mut selected_flashcard = use_signal(|| -> Option<FlashCard> { None });
    let mut show_flashcard_popup = use_signal(|| false);
    let mut show_flashcard_details = use_signal(|| false);
    
    // 最近開いたファイル関連の状態管理
    let mut recent_files = use_signal(|| load_recent_files());
    let mut show_recent_files_popup = use_signal(|| false);
    
    // 単語帳リストをメモ化
    let flashcard_list = use_memo(move || flashcards());
    let recent_files_list = use_memo(move || recent_files());
    
    // PDFファイル情報の取得（PDFが選択されている場合のみ）
    let (total_pages, pdf_info) = use_memo(move || {
        if let Some(path) = pdf_path() {
            get_pdf_info(&path)
        } else {
            (0, "PDFファイルが選択されていません".to_string())
        }
    })();
    
    // PDFが選択されたときの読み込み処理（新しいファイルの場合のみ）
    use_effect(move || {
        if let Some(path) = pdf_path() {
            // 新しいファイルかどうかチェック
            let should_load = loaded_pdf_path().as_ref() != Some(&path);
            
            if total_pages > 0 && !is_loading() && should_load {
                is_loading.set(true);
                page_cache.write().clear(); // 既存のキャッシュをクリア
                error_message.set(String::new());
                
                spawn(async move {
                    // 最初の3ページを最優先で読み込み
                    for page_idx in 0..3.min(total_pages) {
                        if let Some(page_data) = render_pdf_page_with_text(&path, page_idx) {
                            // 混入チェック: ページデータのインデックスが正しいか確認
                            if page_data.page_index != page_idx {
                                eprintln!("CRITICAL: Page data contamination detected in priority load! Expected page {}, got page {}", page_idx, page_data.page_index);
                                continue; // 混入したデータは破棄
                            }
                            page_cache.write().insert(page_idx, page_data);
                        }
                    }
                
                // 残りのページを並列で読み込み（CPUコア数に基づいてバッチサイズを決定）
                let chunk_size = std::thread::available_parallelism().map(|n| n.get()).unwrap_or(4).min(8);
                for chunk_start in (3..total_pages).step_by(chunk_size) {
                    let chunk_end = (chunk_start + chunk_size).min(total_pages);
                    let batch_futures: Vec<_> = (chunk_start..chunk_end)
                        .map(|page_idx| {
                            let path_clone = path.clone();
                            async move {
                                render_pdf_page_with_text(&path_clone, page_idx).map(|data| (page_idx, data))
                            }
                        })
                        .collect();
                    
                    // バッチを並列実行
                    let results = futures::future::join_all(batch_futures).await;
                    for result in results {
                        if let Some((page_idx, page_data)) = result {
                            // 混入チェック: ページデータのインデックスが正しいか確認
                            if page_data.page_index != page_idx {
                                eprintln!("CRITICAL: Page data contamination detected! Expected page {}, got page {}", page_idx, page_data.page_index);
                                continue; // 混入したデータは破棄
                            }
                            page_cache.write().insert(page_idx, page_data);
                        }
                    }
                    
                    // 少し待機してUIの応答性を保つ
                    tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
                }
                
                    is_loading.set(false);
                    loaded_pdf_path.set(Some(path)); // 読み込み完了したパスを記録
                });
            }
        }
    });

    // レンダリング済みページのリストを取得
    let rendered_pages = use_memo(move || {
        let mut pages = Vec::new();
        for page_idx in 0..total_pages {
            if let Some(page_data) = page_cache().get(&page_idx) {
                // 混入チェック: ページインデックスが一致するか確認
                if page_data.page_index != page_idx {
                    println!("WARNING: Page data mismatch detected! Expected page {}, got page {}", page_idx, page_data.page_index);
                    continue; // 混入したページはスキップ
                }
                pages.push((page_idx, page_data.clone()));
            }
        }
        pages
    });

    rsx! {
        head {
            style { {include_str!("../assets/style.css")} }
        }
        div { class: "app",
            div { class: "main-content",
                div { 
                    class: "header",
                    style: "display: flex; justify-content: space-between; align-items: center; margin-bottom: 20px; padding: 10px; background-color: #f8f9fa; border-radius: 4px;",
                    h1 { "PDF Viewer - Dioxus" }
                    div { 
                        class: "file-controls",
                        style: "display: flex; gap: 10px;",
                        button {
                            class: "recent-files-btn",
                            style: "padding: 8px 16px; background-color: #9b59b6; color: white; border: none; border-radius: 4px; cursor: pointer;",
                            onclick: move |_| {
                                show_recent_files_popup.set(true);
                            },
                            "📋 最近のファイル"
                        }
                        button {
                            class: "file-select-btn",
                            style: "padding: 8px 16px; background-color: #3498db; color: white; border: none; border-radius: 4px; cursor: pointer;",
                            onclick: move |_| {
                                spawn(async move {
                                    if let Some(file_handle) = rfd::AsyncFileDialog::new()
                                        .add_filter("PDF files", &["pdf"])
                                        .set_title("PDFファイルを選択")
                                        .pick_file()
                                        .await 
                                    {
                                        let selected_path = file_handle.path().to_path_buf();
                                        let _ = add_recent_file(&selected_path);
                                        recent_files.set(load_recent_files());
                                        pdf_path.set(Some(selected_path));
                                        page_cache.write().clear();
                                        loaded_pdf_path.set(None); // 新しいファイル選択時にリセット
                                        is_loading.set(false);
                                    }
                                });
                            },
                            "📁 PDFを開く"
                        }
                        if pdf_path().is_some() {
                            button {
                                class: "file-close-btn",
                                style: "padding: 8px 16px; background-color: #e74c3c; color: white; border: none; border-radius: 4px; cursor: pointer;",
                                onclick: move |_| {
                                    pdf_path.set(None);
                                    page_cache.write().clear();
                                    loaded_pdf_path.set(None); // ファイル閉じる時にもリセット
                                    is_loading.set(false);
                                    error_message.set(String::new());
                                },
                                "❌ 閉じる"
                            }
                        }
                    }
                }
                
                if !error_message().is_empty() {
                    div { 
                        class: "error",
                        style: "margin-bottom: 15px; padding: 10px; background-color: #f8d7da; color: #721c24; border: 1px solid #f5c6cb; border-radius: 4px;",
                        "{error_message()}"
                    }
                }
                
                if pdf_path().is_some() && total_pages > 0 {
                    div { class: "controls",
                        style: "margin-bottom: 15px; padding: 10px; background-color: #f8f9fa; border-radius: 4px; display: flex; align-items: center; gap: 10px;",
                        span { class: "page-info",
                            if is_loading() {
                                "全 {total_pages} ページ読み込み中... ({rendered_pages().len()}/{total_pages})"
                            } else {
                                "全 {total_pages} ページ読み込み完了"
                            }
                        }
                        
                        if is_loading() {
                            div {
                                class: "loading-indicator",
                                style: "padding: 5px 10px; background-color: #3498db; color: white; border-radius: 4px; font-size: 12px;",
                                "読み込み中..."
                            }
                        }
                    }
                }
                
                div { 
                    class: "content-area",
                    style: "display: flex; flex-direction: row; flex: 1; height: calc(100vh - 160px); overflow: hidden;",
                    
                    if pdf_path().is_none() {
                        div { 
                            class: "welcome",
                            style: "flex: 1; text-align: center; padding: 40px; color: #6c757d; border: 2px dashed #dee2e6; border-radius: 8px; margin: 20px; display: flex; flex-direction: column; justify-content: center;",
                            h2 { "PDFビューアーへようこそ" }
                            p { "上の「📁 PDFを開く」ボタンをクリックしてPDFファイルを選択してください。" }
                            p { "AI検索機能で調べたい語句の意味を尋ねることができます。" }
                        }
                    } else if total_pages == 0 {
                        div { 
                            class: "error",
                            style: "flex: 1; padding: 20px;",
                            "{pdf_info}"
                        }
                    } else {
                        div { 
                            class: "pdf-section",
                            style: "flex: 1; display: flex; flex-direction: column; overflow: hidden; height: 100%;",
                            
                            div { 
                                class: "pdf-viewer",
                                style: "flex: 1; display: flex; flex-direction: column; overflow-y: auto; overflow-x: hidden; padding: 10px; gap: 20px; height: 100%; max-height: calc(100vh - 200px);",
                                for (page_idx, page_data) in rendered_pages.read().iter() {
                                    div {
                                        key: "{page_idx}",
                                        class: "page-container",
                                        style: "display: flex; flex-direction: column; align-items: center;",
                                        div {
                                            class: "page-number",
                                            style: "margin-bottom: 10px; font-weight: bold; color: #2c3e50;",
                                            "ページ {page_idx + 1}"
                                        }
                                        div {
                                            class: "page-wrapper",
                                            id: "page-wrapper-{page_idx}",
                                            style: "position: relative; display: block; width: 100%; max-width: 800px; margin-bottom: 20px; isolation: isolate;",
                                            img {
                                                src: "{page_data.image_data}",
                                                alt: "PDF Page {page_idx + 1}",
                                                class: "pdf-page",
                                                style: "display: block; width: 100%; height: auto; border: 1px solid #ddd; border-radius: 4px; box-shadow: 0 4px 8px rgba(0, 0, 0, 0.1); background-color: white;"
                                            }
                                            div {
                                                class: "text-overlay",
                                                id: "text-overlay-{page_idx}",
                                                style: "position: absolute; top: 0; left: 0; right: 0; bottom: 0; pointer-events: none; border-radius: 4px; z-index: 1; overflow: hidden;",
                                                for (text_idx, text_elem) in page_data.text_elements.iter().enumerate() {
                                                    span {
                                                        key: "p{page_idx}t{text_idx}",
                                                        class: "selectable-text",
                                                        "data-page": "{page_idx}",
                                                        "data-text-idx": "{text_idx}",
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
                                                        "{text_elem.text}"
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                                
                                if is_loading() && rendered_pages.read().len() < total_pages {
                                    div {
                                        class: "loading-pages-placeholder",
                                        style: "text-align: center; padding: 40px; color: #3498db; font-style: italic; border: 2px dashed #3498db; border-radius: 8px; background-color: #ecf0f1;",
                                        "残り {total_pages - rendered_pages.read().len()} ページを読み込み中..."
                                    }
                                }
                            }
                        }
                        
                        div { 
                            class: "sidebar",
                            style: "width: 350px; background-color: #2c3e50; color: white; padding: 20px; display: flex; flex-direction: column; height: 100%;",
                            
                            div {
                                style: "flex-shrink: 0;",
                                h2 { "AI検索" }
                                
                                div { class: "form-group",
                                    label { "AIモデル:" }
                                    select {
                                        value: match selected_provider() {
                                            AIProvider::Gemini => "gemini",
                                            AIProvider::ChatGPT => "chatgpt",
                                            AIProvider::Claude => "claude",
                                        },
                                        onchange: move |evt| {
                                            match evt.value().as_str() {
                                                "gemini" => selected_provider.set(AIProvider::Gemini),
                                                "chatgpt" => selected_provider.set(AIProvider::ChatGPT),
                                                "claude" => selected_provider.set(AIProvider::Claude),
                                                _ => {}
                                            }
                                        },
                                        style: "width: 100%; padding: 8px; border-radius: 4px; border: 1px solid #bdc3c7; background-color: white; color: black;",
                                        option { value: "gemini", "Gemini 2.5 Flash" }
                                        option { value: "chatgpt", "ChatGPT (GPT-4o)" }
                                        option { value: "claude", "Claude 3.5 Sonnet" }
                                    }
                                }
                                
                                div { class: "form-group",
                                    label { 
                                        match selected_provider() {
                                            AIProvider::Gemini => "Gemini APIキー:",
                                            AIProvider::ChatGPT => "ChatGPT APIキー:",
                                            AIProvider::Claude => "Claude APIキー:",
                                        }
                                    }
                                    input {
                                        r#type: "password",
                                        placeholder: match selected_provider() {
                                            AIProvider::Gemini => "Gemini APIキーを入力",
                                            AIProvider::ChatGPT => "OpenAI APIキーを入力",
                                            AIProvider::Claude => "Anthropic APIキーを入力",
                                        },
                                        value: match selected_provider() {
                                            AIProvider::Gemini => gemini_api_key(),
                                            AIProvider::ChatGPT => chatgpt_api_key(),
                                            AIProvider::Claude => claude_api_key(),
                                        },
                                        oninput: move |evt| {
                                            let new_value = evt.value().clone();
                                            match selected_provider() {
                                                AIProvider::Gemini => {
                                                    gemini_api_key.set(new_value.clone());
                                                    // APIキーを保存
                                                    let api_keys = ApiKeys {
                                                        gemini: new_value,
                                                        chatgpt: chatgpt_api_key(),
                                                        claude: claude_api_key(),
                                                    };
                                                    let _ = save_api_keys(&api_keys);
                                                },
                                                AIProvider::ChatGPT => {
                                                    chatgpt_api_key.set(new_value.clone());
                                                    // APIキーを保存
                                                    let api_keys = ApiKeys {
                                                        gemini: gemini_api_key(),
                                                        chatgpt: new_value,
                                                        claude: claude_api_key(),
                                                    };
                                                    let _ = save_api_keys(&api_keys);
                                                },
                                                AIProvider::Claude => {
                                                    claude_api_key.set(new_value.clone());
                                                    // APIキーを保存
                                                    let api_keys = ApiKeys {
                                                        gemini: gemini_api_key(),
                                                        chatgpt: chatgpt_api_key(),
                                                        claude: new_value,
                                                    };
                                                    let _ = save_api_keys(&api_keys);
                                                },
                                            }
                                        }
                                    }
                                }
                                
                                div { class: "form-group",
                                    label { "検索語句:" }
                                    input {
                                        r#type: "text",
                                        value: search_query(),
                                        oninput: move |evt| search_query.set(evt.value().clone())
                                    }
                                }
                                
                                div { class: "form-group",
                                    button {
                                        disabled: {
                                            let api_key_empty = match selected_provider() {
                                                AIProvider::Gemini => gemini_api_key().is_empty(),
                                                AIProvider::ChatGPT => chatgpt_api_key().is_empty(),
                                                AIProvider::Claude => claude_api_key().is_empty(),
                                            };
                                            api_key_empty || search_query().is_empty() || is_searching()
                                        },
                                        onclick: move |_| {
                                            let provider = selected_provider();
                                            let api_key_val = match provider {
                                                AIProvider::Gemini => gemini_api_key(),
                                                AIProvider::ChatGPT => chatgpt_api_key(),
                                                AIProvider::Claude => claude_api_key(),
                                            };
                                            let query_val = search_query();
                                            
                                            if !api_key_val.is_empty() && !query_val.is_empty() {
                                                is_searching.set(true);
                                                search_result.set("検索中...".to_string());
                                                
                                                spawn(async move {
                                                    match search_with_ai(provider, query_val, api_key_val).await {
                                                        Ok(result) => {
                                                            search_result.set(result);
                                                            // 単語帳リストを更新
                                                            flashcards.set(load_flashcards());
                                                        },
                                                        Err(e) => search_result.set(format!("エラー: {}", e)),
                                                    }
                                                    is_searching.set(false);
                                                });
                                            }
                                        },
                                        if is_searching() { "検索中..." } else { "検索" }
                                    }
                                }
                            }
                            
                            div { 
                                class: "search-result",
                                style: "flex: 1; display: flex; flex-direction: column; overflow: hidden; margin-top: 20px;",
                                h3 { 
                                    style: "flex-shrink: 0; margin-bottom: 10px; color: #ecf0f1; font-size: 16px;",
                                    "検索結果:" 
                                }
                                div { 
                                    class: "result-content",
                                    style: "flex: 1; background-color: #34495e; padding: 12px; border-radius: 4px; overflow-y: auto; font-size: 13px; line-height: 1.4; color: #ecf0f1; white-space: pre-wrap;",
                                    "{search_result}"
                                }
                            }
                            
                            // 単語帳開くボタン
                            div { 
                                style: "margin-top: 20px;",
                                button { 
                                    style: "width: 100%; padding: 10px; background-color: #27ae60; color: white; border: none; border-radius: 4px; cursor: pointer; font-size: 14px;",
                                    onclick: move |_| {
                                        show_flashcard_popup.set(true);
                                    },
                                    "📚 単語帳を開く ({flashcard_list().len()}件)"
                                }
                            }
                        }
                    }
                }
            }
        }
        
        // 単語帳ポップアップ
        if show_flashcard_popup() {
            div { 
                class: "popup-overlay",
                style: "position: fixed; top: 0; left: 0; width: 100%; height: 100%; background-color: rgba(0, 0, 0, 0.7); display: flex; align-items: center; justify-content: center; z-index: 1000;",
                onclick: move |_| {
                    show_flashcard_popup.set(false);
                    show_flashcard_details.set(false);
                    selected_flashcard.set(None);
                },
                div { 
                    class: "popup-content",
                    style: "background-color: #2c3e50; border-radius: 8px; padding: 20px; max-width: 600px; max-height: 80vh; overflow-y: auto; position: relative;",
                    onclick: move |e| {
                        e.stop_propagation();
                    },
                    
                    // ヘッダー
                    div { 
                        style: "display: flex; justify-content: space-between; align-items: center; margin-bottom: 20px; border-bottom: 1px solid #34495e; padding-bottom: 10px;",
                        h2 { 
                            style: "color: #ecf0f1; margin: 0; font-size: 18px;",
                            "📚 単語帳 ({flashcard_list().len()}件)"
                        }
                        button { 
                            style: "background: none; border: none; color: #e74c3c; cursor: pointer; font-size: 24px; padding: 0;",
                            onclick: move |_| {
                                show_flashcard_popup.set(false);
                                show_flashcard_details.set(false);
                                selected_flashcard.set(None);
                            },
                            "×"
                        }
                    }
                    
                    // 単語帳リスト
                    if flashcard_list().is_empty() {
                        div { 
                            style: "text-align: center; padding: 40px; color: #bdc3c7; font-size: 16px;",
                            "まだ単語が保存されていません。\nAI検索を使って単語を保存してみましょう！"
                        }
                    } else {
                        div { 
                            class: "flashcards-list",
                            style: "max-height: 400px; overflow-y: auto;",
                            for (_index, flashcard) in flashcard_list().iter().enumerate() {
                                div { 
                                    key: "{flashcard.id}",
                                    class: "flashcard-item",
                                    style: "background-color: #34495e; border-radius: 6px; padding: 16px; margin-bottom: 12px; cursor: pointer; transition: background-color 0.2s; border: 1px solid #445a6f;",
                                    onclick: {
                                        let card = flashcard.clone();
                                        move |_| {
                                            selected_flashcard.set(Some(card.clone()));
                                            show_flashcard_details.set(true);
                                        }
                                    },
                                    div { 
                                        style: "font-weight: bold; margin-bottom: 8px; color: #3498db; font-size: 16px;",
                                        "{flashcard.term}"
                                    }
                                    div { 
                                        style: "color: #ecf0f1; font-size: 14px; line-height: 1.4; max-height: 60px; overflow: hidden; margin-bottom: 8px;",
                                        {
                                            if flashcard.definition.len() > 100 {
                                                format!("{}...", flashcard.definition.chars().take(100).collect::<String>())
                                            } else {
                                                flashcard.definition.clone()
                                            }
                                        }
                                    }
                                    div { 
                                        style: "font-size: 12px; color: #95a5a6;",
                                        "{flashcard.created_at}"
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        
        // 単語詳細ポップアップ
        if show_flashcard_details() {
            if let Some(ref card) = selected_flashcard() {
                div { 
                    class: "details-overlay",
                    style: "position: fixed; top: 0; left: 0; width: 100%; height: 100%; background-color: rgba(0, 0, 0, 0.8); display: flex; align-items: center; justify-content: center; z-index: 1001;",
                    onclick: move |_| {
                        show_flashcard_details.set(false);
                        selected_flashcard.set(None);
                    },
                    div { 
                        class: "details-content",
                        style: "background-color: #2c3e50; border-radius: 8px; padding: 24px; max-width: 700px; max-height: 80vh; overflow-y: auto; position: relative; margin: 20px;",
                        onclick: move |e| {
                            e.stop_propagation();
                        },
                        
                        // ヘッダー
                        div { 
                            style: "display: flex; justify-content: space-between; align-items: center; margin-bottom: 20px; border-bottom: 1px solid #34495e; padding-bottom: 12px;",
                            h3 { 
                                style: "color: #3498db; margin: 0; font-size: 20px;",
                                "{card.term}"
                            }
                            div { 
                                style: "display: flex; gap: 10px; align-items: center;",
                                button { 
                                    style: "background-color: #e74c3c; color: white; border: none; border-radius: 4px; padding: 6px 12px; cursor: pointer; font-size: 12px;",
                                    onclick: {
                                        let card_id = card.id.clone();
                                        move |_| {
                                            if let Ok(_) = delete_flashcard(card_id.clone()) {
                                                flashcards.set(load_flashcards());
                                                show_flashcard_details.set(false);
                                                selected_flashcard.set(None);
                                            }
                                        }
                                    },
                                    "🗑️ 削除"
                                }
                                button { 
                                    style: "background-color: #3498db; color: white; border: none; border-radius: 4px; padding: 6px 12px; cursor: pointer; font-size: 12px;",
                                    onclick: move |_| {
                                        show_flashcard_details.set(false);
                                        selected_flashcard.set(None);
                                    },
                                    "戻る"
                                }
                                button { 
                                    style: "background: none; border: none; color: #e74c3c; cursor: pointer; font-size: 24px; padding: 0;",
                                    onclick: move |_| {
                                        show_flashcard_details.set(false);
                                        selected_flashcard.set(None);
                                    },
                                    "×"
                                }
                            }
                        }
                        
                        // 詳細内容
                        div { 
                            style: "color: #ecf0f1; font-size: 15px; line-height: 1.6; white-space: pre-wrap; margin-bottom: 16px;",
                            "{card.definition}"
                        }
                        
                        // フッター
                        div { 
                            style: "border-top: 1px solid #34495e; padding-top: 12px; font-size: 13px; color: #95a5a6;",
                            "保存日時: {card.created_at}"
                        }
                    }
                }
            }
        }
        
        // 最近開いたファイルのポップアップ
        if show_recent_files_popup() {
            div { 
                class: "popup-overlay",
                style: "position: fixed; top: 0; left: 0; width: 100%; height: 100%; background-color: rgba(0, 0, 0, 0.7); display: flex; align-items: center; justify-content: center; z-index: 1000;",
                onclick: move |_| {
                    show_recent_files_popup.set(false);
                },
                div { 
                    class: "popup-content",
                    style: "background-color: #2c3e50; border-radius: 8px; padding: 20px; max-width: 600px; max-height: 80vh; overflow-y: auto; position: relative;",
                    onclick: move |e| {
                        e.stop_propagation();
                    },
                    
                    // ヘッダー
                    div { 
                        style: "display: flex; justify-content: space-between; align-items: center; margin-bottom: 20px; border-bottom: 1px solid #34495e; padding-bottom: 10px;",
                        h2 { 
                            style: "color: #ecf0f1; margin: 0; font-size: 18px;",
                            "📋 最近開いたファイル ({recent_files_list().len()}件)"
                        }
                        button { 
                            style: "background: none; border: none; color: #e74c3c; cursor: pointer; font-size: 24px; padding: 0;",
                            onclick: move |_| {
                                show_recent_files_popup.set(false);
                            },
                            "×"
                        }
                    }
                    
                    // 最近開いたファイルリスト
                    if recent_files_list().is_empty() {
                        div { 
                            style: "text-align: center; padding: 40px; color: #bdc3c7; font-size: 16px;",
                            "まだファイルを開いていません。\n\"📁 PDFを開く\"ボタンでファイルを選択してみましょう！"
                        }
                    } else {
                        div { 
                            class: "recent-files-list",
                            style: "max-height: 400px; overflow-y: auto;",
                            for (_index, recent_file) in recent_files_list().iter().enumerate() {
                                div { 
                                    key: "{recent_file.path}",
                                    class: "recent-file-item",
                                    style: "background-color: #34495e; border-radius: 6px; padding: 16px; margin-bottom: 12px; cursor: pointer; transition: background-color 0.2s; border: 1px solid #445a6f;",
                                    onclick: {
                                        let file_path = PathBuf::from(recent_file.path.clone());
                                        move |_| {
                                            if file_path.exists() {
                                                let _ = add_recent_file(&file_path);
                                                recent_files.set(load_recent_files());
                                                pdf_path.set(Some(file_path.clone()));
                                                page_cache.write().clear();
                                                loaded_pdf_path.set(None);
                                                is_loading.set(false);
                                                show_recent_files_popup.set(false);
                                            }
                                        }
                                    },
                                    div { 
                                        style: "display: flex; justify-content: space-between; align-items: start;",
                                        div { 
                                            style: "flex: 1;",
                                            div { 
                                                style: "font-weight: bold; margin-bottom: 4px; color: #3498db; font-size: 16px;",
                                                "{recent_file.display_name}"
                                            }
                                            div { 
                                                style: "color: #bdc3c7; font-size: 13px; margin-bottom: 4px; word-break: break-all;",
                                                "{recent_file.path}"
                                            }
                                            div { 
                                                style: "color: #95a5a6; font-size: 12px;",
                                                "最後に開いた日時: {recent_file.last_opened}"
                                            }
                                        }
                                        div { 
                                            style: "margin-left: 10px;",
                                            if !PathBuf::from(&recent_file.path).exists() {
                                                span { 
                                                    style: "color: #e74c3c; font-size: 12px;",
                                                    "❌ ファイルが見つかりません"
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}