
use anyhow::Result;
use dioxus::prelude::*;
use std::collections::HashMap;
use std::env;
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

impl AIProvider {
    fn display_name(&self) -> &str {
        match self {
            AIProvider::Gemini => "Gemini 2.5 Flash",
            AIProvider::ChatGPT => "ChatGPT (GPT-4o)",
            AIProvider::Claude => "Claude 3.5 Sonnet",
        }
    }
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
    match provider {
        AIProvider::Gemini => search_with_gemini(query, api_key).await,
        AIProvider::ChatGPT => search_with_chatgpt(query, api_key).await,
        AIProvider::Claude => search_with_claude(query, api_key).await,
    }
}

fn render_pdf_page_optimized(pdf_path: &PathBuf, page_index: usize) -> Option<String> {
    let lib_path = std::env::current_dir().ok()?.join("lib/libpdfium.dylib");
    let bindings = Pdfium::bind_to_library(&lib_path).ok()?;
    let pdfium = Pdfium::new(bindings);
    let document = pdfium.load_pdf_from_file(pdf_path, None).ok()?;
    let page = document.pages().get(page_index as u16).ok()?;
    
    let render_config = PdfRenderConfig::new()
        .set_target_width(1000)
        .set_maximum_height(1400)
        .rotate_if_landscape(PdfPageRenderRotation::Degrees90, true);

    let bitmap = page.render_with_config(&render_config).ok()?;
    let width = bitmap.width() as usize;
    let height = bitmap.height() as usize;
    
    let image_buffer = bitmap.as_raw_bytes();
    let mut rgba_pixels = Vec::with_capacity(width * height * 4);
    
    // より効率的なカラー変換（chunk_exactを使用してbound checkを削減）
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
    
    Some(format!("data:image/png;base64,{}", base64::engine::general_purpose::STANDARD.encode(&png_data)))
}

fn get_pdf_info(pdf_path: &PathBuf) -> (usize, String) {
    let lib_path = std::env::current_dir().unwrap_or_default().join("lib/libpdfium.dylib");
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
    let mut gemini_api_key = use_signal(|| String::new());
    let mut chatgpt_api_key = use_signal(|| String::new());
    let mut claude_api_key = use_signal(|| String::new());
    let mut search_query = use_signal(|| String::new());
    let mut search_result = use_signal(|| String::new());
    let mut is_searching = use_signal(|| false);
    let mut page_cache = use_signal(|| HashMap::<usize, String>::new());
    let mut is_loading = use_signal(|| false);
    let mut error_message = use_signal(|| String::new());
    let mut loaded_pdf_path = use_signal(|| -> Option<PathBuf> { None }); // 読み込み済みのPDFパスを追跡
    
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
                        if let Some(image_data) = render_pdf_page_optimized(&path, page_idx) {
                            page_cache.write().insert(page_idx, image_data);
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
                                render_pdf_page_optimized(&path_clone, page_idx).map(|data| (page_idx, data))
                            }
                        })
                        .collect();
                    
                    // バッチを並列実行
                    let results = futures::future::join_all(batch_futures).await;
                    for result in results {
                        if let Some((page_idx, image_data)) = result {
                            page_cache.write().insert(page_idx, image_data);
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
            if let Some(image_data) = page_cache().get(&page_idx) {
                pages.push((page_idx, image_data.clone()));
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
                                class: "pdf-info",
                                style: "flex-shrink: 0; margin-bottom: 15px; padding: 10px; background-color: #f8f9fa; border-radius: 4px; border-left: 3px solid #3498db; font-size: 14px;",
                                h3 { "PDF情報:" }
                                pre { "{pdf_info}" }
                            }
                            
                            div { 
                                class: "pdf-viewer",
                                style: "flex: 1; display: flex; flex-direction: column; overflow-y: auto; overflow-x: hidden; padding: 10px; gap: 20px; height: 100%; max-height: calc(100vh - 200px);",
                                for (page_idx, image_data) in rendered_pages() {
                                    div {
                                        key: "{page_idx}",
                                        class: "page-container",
                                        style: "display: flex; flex-direction: column; align-items: center;",
                                        div {
                                            class: "page-number",
                                            style: "margin-bottom: 10px; font-weight: bold; color: #2c3e50;",
                                            "ページ {page_idx + 1}"
                                        }
                                        img {
                                            src: "{image_data}",
                                            alt: "PDF Page {page_idx + 1}",
                                            class: "pdf-page",
                                            style: "max-width: 100%; height: auto; border: 1px solid #ddd; border-radius: 4px; box-shadow: 0 4px 8px rgba(0, 0, 0, 0.1); background-color: white; margin-bottom: 10px;"
                                        }
                                    }
                                }
                                
                                if is_loading() && rendered_pages().len() < total_pages {
                                    div {
                                        class: "loading-pages-placeholder",
                                        style: "text-align: center; padding: 40px; color: #3498db; font-style: italic; border: 2px dashed #3498db; border-radius: 8px; background-color: #ecf0f1;",
                                        "残り {total_pages - rendered_pages().len()} ページを読み込み中..."
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
                                            match selected_provider() {
                                                AIProvider::Gemini => gemini_api_key.set(evt.value().clone()),
                                                AIProvider::ChatGPT => chatgpt_api_key.set(evt.value().clone()),
                                                AIProvider::Claude => claude_api_key.set(evt.value().clone()),
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
                                                        Ok(result) => search_result.set(result),
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
                        }
                    }
                }
            }
        }
    }
}