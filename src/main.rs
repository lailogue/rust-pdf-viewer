use anyhow::Result;
use dioxus::prelude::*;
use std::collections::HashMap;
use std::env;
use std::path::PathBuf;
use pdfium_render::prelude::*;
use serde::{Deserialize, Serialize};
use base64::Engine;

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

fn render_pdf_page(pdf_path: &PathBuf, page_index: usize) -> Option<String> {
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
    
    for chunk in image_buffer.chunks(4) {
        if chunk.len() >= 4 {
            rgba_pixels.push(chunk[2]); // R
            rgba_pixels.push(chunk[1]); // G  
            rgba_pixels.push(chunk[0]); // B
            rgba_pixels.push(chunk[3]); // A
        }
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
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <pdf_file>", args[0]);
        std::process::exit(1);
    }

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
    let args: Vec<String> = std::env::args().collect();
    let pdf_path = if args.len() >= 2 {
        PathBuf::from(&args[1])
    } else {
        PathBuf::from("test.pdf") // fallback
    };
    
    let mut api_key = use_signal(|| String::new());
    let mut search_query = use_signal(|| String::new());
    let mut search_result = use_signal(|| String::new());
    let mut is_searching = use_signal(|| false);
    let mut page_cache = use_signal(|| HashMap::<usize, String>::new());
    let mut is_loading = use_signal(|| true);
    
    let pdf_path_clone = pdf_path.clone();
    let (total_pages, pdf_info) = use_memo(move || get_pdf_info(&pdf_path_clone))();
    
    // 段階的な並列読み込み（最初の数ページを優先）
    let pdf_path_clone2 = pdf_path.clone();
    use_effect(move || {
        if total_pages > 0 && is_loading() {
            let path = pdf_path_clone2.clone();
            spawn(async move {
                // 最初の3ページを最優先で読み込み
                for page_idx in 0..3.min(total_pages) {
                    if let Some(image_data) = render_pdf_page(&path, page_idx) {
                        page_cache.write().insert(page_idx, image_data);
                    }
                }
                
                // 残りのページを並列で読み込み（4ページずつバッチ処理）
                let chunk_size = 4;
                for chunk_start in (3..total_pages).step_by(chunk_size) {
                    let chunk_end = (chunk_start + chunk_size).min(total_pages);
                    let batch_futures: Vec<_> = (chunk_start..chunk_end)
                        .map(|page_idx| {
                            let path_clone = path.clone();
                            async move {
                                render_pdf_page(&path_clone, page_idx).map(|data| (page_idx, data))
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
            });
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
                h1 { "PDF Viewer - Dioxus" }
                
                if total_pages == 0 {
                    div { class: "error",
                        "{pdf_info}"
                    }
                } else {
                    div { class: "controls",
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
                                style: "margin-left: 10px; padding: 5px 10px; background-color: #3498db; color: white; border-radius: 4px; font-size: 12px;",
                                "読み込み中..."
                            }
                        }
                    }
                    
                    div { 
                        class: "content-area",
                        style: "display: flex; flex-direction: row; flex: 1; height: calc(100vh - 120px); overflow: hidden;",
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
                                    label { "APIキー:" }
                                    input {
                                        r#type: "password",
                                        value: "{api_key}",
                                        oninput: move |evt| api_key.set(evt.value().clone())
                                    }
                                }
                                
                                div { class: "form-group",
                                    label { "検索語句:" }
                                    input {
                                        r#type: "text",
                                        value: "{search_query}",
                                        oninput: move |evt| search_query.set(evt.value().clone())
                                    }
                                }
                                
                                div { class: "form-group",
                                    button {
                                        disabled: api_key().is_empty() || search_query().is_empty() || is_searching(),
                                        onclick: move |_| {
                                            let api_key_val = api_key();
                                            let query_val = search_query();
                                            
                                            if !api_key_val.is_empty() && !query_val.is_empty() {
                                                is_searching.set(true);
                                                search_result.set("検索中...".to_string());
                                                
                                                spawn(async move {
                                                    match search_with_gemini(query_val, api_key_val).await {
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