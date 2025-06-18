
use anyhow::Result;
use dioxus::prelude::*;
use std::collections::HashMap;
use std::path::PathBuf;

// Module declarations
mod types;
mod pdf;
mod ai;
mod storage;
mod ui;

// Import all types and functions from modules
use types::*;
use pdf::*;
use ai::*;
use storage::*;
use ui::components::popups::*;


fn main() -> Result<()> {
    // 引数は任意にして、アプリケーション内でファイル選択できるようにする

    let config = dioxus_desktop::Config::new()
        .with_window(
            dioxus_desktop::tao::window::WindowBuilder::new()
                .with_title("PDF Viewer in Rust")
                .with_inner_size(dioxus_desktop::tao::dpi::LogicalSize::new(1200.0, 800.0))
        );
    dioxus_desktop::launch::launch(app, vec![], config);
    
    Ok(())
}

fn app() -> Element {
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
    let mut gemini_api_key = use_signal(|| saved_api_keys().gemini.unwrap_or_default());
    let mut chatgpt_api_key = use_signal(|| saved_api_keys().chatgpt.unwrap_or_default());
    let mut claude_api_key = use_signal(|| saved_api_keys().claude.unwrap_or_default());
    let mut search_query = use_signal(|| String::new());
    let mut search_result = use_signal(|| String::new());
    let mut is_searching = use_signal(|| false);
    let mut page_cache = use_signal(|| HashMap::<usize, PdfPageData>::new());
    let mut is_loading = use_signal(|| false);
    let mut error_message = use_signal(|| String::new());
    let mut loaded_pdf_path = use_signal(|| -> Option<PathBuf> { None }); // 読み込み済みのPDFパスを追跡
    
    // 単語帳関連の状態管理
    let mut flashcards = use_signal(|| load_flashcards());
    let selected_flashcard = use_signal(|| -> Option<FlashCard> { None });
    let mut show_flashcard_popup = use_signal(|| false);
    let show_flashcard_details = use_signal(|| false);
    
    // 最近開いたファイル関連の状態管理
    let mut recent_files = use_signal(|| load_recent_files());
    let mut show_recent_files_popup = use_signal(|| false);
    
    // ページ回転関連の状態管理
    let mut page_rotations = use_signal(|| HashMap::<usize, RotationAngle>::new());
    
    // ブックマーク関連の状態管理
    let mut current_bookmark = use_signal(|| -> Option<ReadingBookmark> { None });
    let mut show_bookmarks_popup = use_signal(|| false);
    
    // 位置マーカー関連の状態管理
    let mut position_markers = use_signal(|| Vec::<PositionMarker>::new());
    let mut show_markers_popup = use_signal(|| false);
    let mut marker_mode = use_signal(|| false); // マーカー配置モード
    
    // 単語帳リストをメモ化
    let flashcard_list = use_memo(move || flashcards());
    let recent_files_list = use_memo(move || recent_files());
    
    // PDFファイル情報の取得（PDFが選択されている場合のみ）
    let (total_pages, pdf_info) = use_memo(move || {
        if let Some(path) = pdf_path() {
            pdf::get_pdf_info(&path.to_string_lossy()).unwrap_or((0, "PDFの読み込みに失敗しました".to_string()))
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
                
                // 該当PDFの回転状態を読み込み
                let rotations = load_page_rotations(&path.to_string_lossy());
                page_rotations.set(rotations.clone());
                
                // 該当PDFのブックマークを読み込み
                let bookmark = load_reading_bookmark(&path.to_string_lossy());
                current_bookmark.set(bookmark);
                
                // 該当PDFの位置マーカーを読み込み
                let markers = load_position_markers(&path.to_string_lossy());
                position_markers.set(markers);
                
                spawn(async move {
                    // 最初の3ページを最優先で読み込み
                    for page_idx in 0..3.min(total_pages) {
                        let rotation = rotations.get(&page_idx).copied().unwrap_or(RotationAngle::None);
                        if let Ok(page_data) = render_pdf_page_with_text(&path.to_string_lossy(), page_idx, rotation) {
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
                            let rotation = rotations.get(&page_idx).copied().unwrap_or(RotationAngle::None);
                            Box::pin(async move {
                                match render_pdf_page_with_text(&path_clone.to_string_lossy(), page_idx, rotation) {
                                    Ok(data) => Some((page_idx, data)),
                                    Err(_) => None,
                                }
                            })
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
                            class: "bookmarks-btn",
                            style: "padding: 8px 16px; background-color: #f39c12; color: white; border: none; border-radius: 4px; cursor: pointer;",
                            onclick: move |_| {
                                show_bookmarks_popup.set(true);
                            },
                            "🔖 ブックマーク"
                        }
                        button {
                            class: "marker-mode-btn",
                            style: {
                                let bg_color = if marker_mode() { "#e74c3c" } else { "#34495e" };
                                format!("padding: 8px 16px; background-color: {}; color: white; border: none; border-radius: 4px; cursor: pointer;", bg_color)
                            },
                            onclick: move |_| {
                                marker_mode.set(!marker_mode());
                            },
                            {if marker_mode() { "📍 マーカーモード: ON" } else { "📍 マーカーモード" }}
                        }
                        button {
                            class: "markers-list-btn",
                            style: "padding: 8px 16px; background-color: #e67e22; color: white; border: none; border-radius: 4px; cursor: pointer;",
                            onclick: move |_| {
                                show_markers_popup.set(true);
                            },
                            {format!("📋 マーカー一覧 ({}件)", position_markers().len())}
                        }
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
                                        let _ = add_recent_file(selected_path.to_string_lossy().to_string(), selected_path.file_name().unwrap_or_default().to_string_lossy().to_string());
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
                                class: "rotate-all-btn",
                                style: "padding: 8px 16px; background-color: #9b59b6; color: white; border: none; border-radius: 4px; cursor: pointer;",
                                onclick: move |_| {
                                    if let Some(path) = pdf_path() {
                                        // 全ページを90度回転
                                        let mut new_rotations = HashMap::new();
                                        for page_idx in 0..total_pages {
                                            let current_rotation = page_rotations().get(&page_idx).copied().unwrap_or(RotationAngle::None);
                                            let new_rotation = current_rotation.next();
                                            new_rotations.insert(page_idx, new_rotation);
                                        }
                                        
                                        // 回転状態を更新
                                        page_rotations.set(new_rotations.clone());
                                        
                                        // 回転状態を保存
                                        let _ = save_page_rotations(&path.to_string_lossy(), new_rotations);
                                        
                                        // 全ページを再レンダリング
                                        page_cache.write().clear();
                                        loaded_pdf_path.set(None); // 再読み込みを強制
                                    }
                                },
                                "🔄 全て回転"
                            }
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
                            p { "LLM Search機能で調べたい語句の意味を尋ねることができます。" }
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
                                            class: "page-header",
                                            style: "display: flex; align-items: center; justify-content: center; gap: 10px; margin-bottom: 10px;",
                                            div {
                                                class: "page-number",
                                                style: "font-weight: bold; color: #2c3e50;",
                                                "ページ {page_idx + 1}"
                                            }
                                            button {
                                                class: "rotate-page-btn",
                                                style: "padding: 5px 10px; background-color: #3498db; color: white; border: none; border-radius: 3px; cursor: pointer; font-size: 12px;",
                                                onclick: {
                                                    let page_idx = *page_idx;
                                                    move |_| {
                                                        if let Some(path) = pdf_path() {
                                                            // 現在の回転状態を取得
                                                            let current_rotation = page_rotations().get(&page_idx).copied().unwrap_or(RotationAngle::None);
                                                            let new_rotation = current_rotation.next();
                                                            
                                                            // 回転状態を更新
                                                            page_rotations.write().insert(page_idx, new_rotation);
                                                            
                                                            // 回転状態を保存
                                                            let _ = save_page_rotations(&path.to_string_lossy(), page_rotations());
                                                            
                                                            // ページを再レンダリング
                                                            let path_clone = path.clone();
                                                            spawn(async move {
                                                                if let Ok(page_data) = render_pdf_page_with_text(&path_clone.to_string_lossy(), page_idx, new_rotation) {
                                                                    page_cache.write().insert(page_idx, page_data);
                                                                }
                                                            });
                                                        }
                                                    }
                                                },
                                                "🔄"
                                            }
                                            button {
                                                class: "bookmark-page-btn",
                                                style: {
                                                    let is_bookmarked = current_bookmark().map_or(false, |b| b.current_page == *page_idx);
                                                    let bg_color = if is_bookmarked { "#f39c12" } else { "#95a5a6" };
                                                    format!("padding: 5px 10px; background-color: {}; color: white; border: none; border-radius: 3px; cursor: pointer; font-size: 12px;", bg_color)
                                                },
                                                onclick: {
                                                    let page_idx = *page_idx;
                                                    move |_| {
                                                        if let Some(path) = pdf_path() {
                                                            // ブックマークを保存
                                                            if let Some(path) = pdf_path() {
                                                                let bookmark = ReadingBookmark {
                                                                    pdf_path: path.to_string_lossy().to_string(),
                                                                    current_page: page_idx,
                                                                    total_pages,
                                                                    last_read_time: chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC").to_string(),
                                                                    reading_progress: (page_idx + 1) as f32 / total_pages as f32,
                                                                };
                                                                let _ = save_reading_bookmark(bookmark);
                                                            }
                                                            
                                                            // ブックマーク状態を更新
                                                            let bookmark = load_reading_bookmark(&path.to_string_lossy());
                                                            current_bookmark.set(bookmark);
                                                            
                                                            // マーカー状態を更新
                                                            let markers = load_position_markers(&path.to_string_lossy());
                                                            position_markers.set(markers);
                                                        }
                                                    }
                                                },
                                                "🔖"
                                            }
                                        }
                                        div {
                                            class: "page-wrapper",
                                            id: "page-wrapper-{page_idx}",
                                            style: format!("position: relative; display: block; width: 100%; max-width: 800px; margin-bottom: 20px; isolation: isolate; cursor: {};", if marker_mode() { "crosshair" } else { "default" }),
                                            onclick: {
                                                let page_idx = *page_idx;
                                                move |evt| {
                                                    if marker_mode() {
                                                        if let Some(path) = pdf_path() {
                                                            // クリック位置を要素内の相対座標で取得
                                                            let coords = evt.data().element_coordinates();
                                                            
                                                            // page-wrapperの要素サイズを取得してクリック位置を正規化
                                                            // element_coordinatesは要素内の絶対位置を返す
                                                            // これを0.0-1.0の範囲に正規化する必要がある
                                                            // とりあえず固定サイズ（800px幅）で計算
                                                            let max_width = 800.0; // page-wrapperの最大幅
                                                            let aspect_ratio = 1.294; // PDF縦横比（1000x1294から）
                                                            let height = max_width * aspect_ratio;
                                                            
                                                            let x = coords.x / max_width;
                                                            let y = coords.y / height;
                                                            
                                                            // 範囲を0.0-1.0にクランプ
                                                            let x = x.max(0.0).min(1.0);
                                                            let y = y.max(0.0).min(1.0);
                                                            
                                                            // マーカーを保存
                                                            if let Some(path) = pdf_path() {
                                                                let marker = PositionMarker::new(page_idx, x as f32, y as f32, String::new());
                                                                let _ = save_position_marker(&path.to_string_lossy(), marker);
                                                            }
                                                            
                                                            // マーカー状態を更新
                                                            let markers = load_position_markers(&path.to_string_lossy());
                                                            position_markers.set(markers);
                                                        }
                                                    }
                                                }
                                            },
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
                                            div {
                                                class: "marker-overlay",
                                                style: "position: absolute; top: 0; left: 0; right: 0; bottom: 0; pointer-events: none; z-index: 3;",
                                                for marker in position_markers().iter().filter(|m| m.page_index == *page_idx) {
                                                    div {
                                                        key: "marker-{marker.id}",
                                                        class: "position-marker",
                                                        style: "position: absolute; 
                                                               left: {marker.x * 100.0}%; 
                                                               top: {marker.y * 100.0}%; 
                                                               width: 12px; 
                                                               height: 12px; 
                                                               background-color: #e74c3c; 
                                                               border: 2px solid white; 
                                                               border-radius: 50%; 
                                                               transform: translate(-50%, -50%); 
                                                               cursor: pointer; 
                                                               pointer-events: auto; 
                                                               z-index: 4; 
                                                               box-shadow: 0 2px 4px rgba(0,0,0,0.3);",
                                                        onclick: {
                                                            let marker_id = marker.id.clone();
                                                            move |evt| {
                                                                evt.stop_propagation();
                                                                if let Some(path) = pdf_path() {
                                                                    // マーカーを削除
                                                                    if let Some(path) = pdf_path() {
                                                                        let _ = delete_position_marker(&path.to_string_lossy(), &marker_id);
                                                                    }
                                                                    
                                                                    // マーカー状態を更新
                                                                    let markers = load_position_markers(&path.to_string_lossy());
                                                                    position_markers.set(markers);
                                                                }
                                                            }
                                                        },
                                                        title: "クリックして削除"
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
                                h2 { "LLM Search" }
                                
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
                                                        gemini: Some(new_value),
                                                        chatgpt: Some(chatgpt_api_key()),
                                                        claude: Some(claude_api_key()),
                                                    };
                                                    let _ = save_api_keys(&api_keys);
                                                },
                                                AIProvider::ChatGPT => {
                                                    chatgpt_api_key.set(new_value.clone());
                                                    // APIキーを保存
                                                    let api_keys = ApiKeys {
                                                        gemini: Some(gemini_api_key()),
                                                        chatgpt: Some(new_value),
                                                        claude: Some(claude_api_key()),
                                                    };
                                                    let _ = save_api_keys(&api_keys);
                                                },
                                                AIProvider::Claude => {
                                                    claude_api_key.set(new_value.clone());
                                                    // APIキーを保存
                                                    let api_keys = ApiKeys {
                                                        gemini: Some(gemini_api_key()),
                                                        chatgpt: Some(chatgpt_api_key()),
                                                        claude: Some(new_value),
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
                                                    match search_with_ai(provider, query_val.clone(), api_key_val).await {
                                                        Ok(result) => {
                                                            search_result.set(result.clone());
                                                            // 単語帳リストを更新
                                                            flashcards.set(load_flashcards());
                                                            // 検索成功時、単語帳に追加
                                                            let _ = add_flashcard(query_val, result);
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
            flashcard_popup {
                show_flashcard_popup: show_flashcard_popup,
                flashcards: flashcards,
                show_flashcard_details: show_flashcard_details,
                selected_flashcard: selected_flashcard,
            }
        }
        
        // 単語詳細ポップアップ
        if show_flashcard_details() {
            flashcard_details_popup {
                show_flashcard_details: show_flashcard_details,
                selected_flashcard: selected_flashcard,
                flashcards: flashcards,
            }
        }
        
        // ブックマーク一覧ポップアップ
        if show_bookmarks_popup() {
            bookmarks_popup {
                show_bookmarks_popup: show_bookmarks_popup,
                pdf_path: pdf_path,
                page_cache: page_cache,
                loaded_pdf_path: loaded_pdf_path,
                is_loading: is_loading,
                recent_files: recent_files,
                current_bookmark: current_bookmark,
            }
        }
        
        // 位置マーカー一覧ポップアップ
        if show_markers_popup() {
            markers_popup {
                show_markers_popup: show_markers_popup,
                position_markers: position_markers,
                pdf_path: pdf_path,
            }
        }
        
        // 最近開いたファイルのポップアップ
        if show_recent_files_popup() {
            recent_files_popup {
                show_recent_files_popup: show_recent_files_popup,
                recent_files_list: recent_files_list,
                recent_files: recent_files,
                pdf_path: pdf_path,
                page_cache: page_cache,
                loaded_pdf_path: loaded_pdf_path,
                is_loading: is_loading,
            }
        }
    }
}
