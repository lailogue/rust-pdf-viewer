use dioxus::prelude::*;
use std::path::PathBuf;
use std::collections::HashMap;
use crate::{get_all_reading_bookmarks, delete_reading_bookmark, add_recent_file, load_recent_files, RecentFile, PdfPageData, ReadingBookmark};

#[component]
pub fn bookmarks_popup(
    show_bookmarks_popup: Signal<bool>,
    pdf_path: Signal<Option<PathBuf>>,
    page_cache: Signal<HashMap<usize, PdfPageData>>,
    loaded_pdf_path: Signal<Option<PathBuf>>,
    is_loading: Signal<bool>,
    recent_files: Signal<Vec<RecentFile>>,
    current_bookmark: Signal<Option<ReadingBookmark>>,
) -> Element {
    // ローカル状態でブックマークを管理
    let mut bookmarks = use_signal(|| get_all_reading_bookmarks());
    
    rsx! {
        div { 
            class: "popup-overlay",
            style: "position: fixed; top: 0; left: 0; width: 100%; height: 100%; background-color: rgba(0, 0, 0, 0.7); display: flex; align-items: center; justify-content: center; z-index: 1000;",
            onclick: move |_| {
                show_bookmarks_popup.set(false);
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
                        "🔖 ブックマーク ({bookmarks.read().len()}件)"
                    }
                    button { 
                        style: "background: none; border: none; color: #e74c3c; cursor: pointer; font-size: 24px; padding: 0;",
                        onclick: move |_| {
                            show_bookmarks_popup.set(false);
                        },
                        "×"
                    }
                }
                
                // ブックマークリスト
                {
                    if bookmarks.read().is_empty() {
                        rsx! {
                            div { 
                                style: "text-align: center; padding: 40px; color: #bdc3c7; font-size: 16px;",
                                "まだブックマークがありません。\nPDFを開いてページにブックマークを設定してみましょう！"
                            }
                        }
                    } else {
                        rsx! {
                            div { 
                                class: "bookmarks-list",
                                style: "max-height: 400px; overflow-y: auto;",
                                for bookmark in bookmarks.read().iter() {
                                    div { 
                                        key: "{bookmark.pdf_path}",
                                        class: "bookmark-item",
                                        style: "background-color: #34495e; border-radius: 6px; padding: 16px; margin-bottom: 12px; transition: background-color 0.2s; border: 1px solid #445a6f; position: relative;",
                                        
                                        // 削除ボタン
                                        button {
                                            style: "position: absolute; top: 8px; right: 8px; background: #e74c3c; border: none; color: white; border-radius: 50%; width: 24px; height: 24px; cursor: pointer; font-size: 12px; display: flex; align-items: center; justify-content: center; transition: background-color 0.2s;",
                                            onclick: {
                                                let bookmark_path = bookmark.pdf_path.clone();
                                                move |e| {
                                                    e.stop_propagation(); // クリックイベントの伝播を停止
                                                    if let Err(_) = delete_reading_bookmark(&bookmark_path) {
                                                        // エラーハンドリング（必要に応じて）
                                                    }
                                                    // ローカル状態を更新
                                                    bookmarks.set(get_all_reading_bookmarks());
                                                    
                                                    // 現在開いているPDFがこの削除されたブックマークのものと同じ場合、
                                                    // current_bookmarkを更新
                                                    if let Some(current_path) = pdf_path() {
                                                        let current_path_str = current_path.to_string_lossy().to_string();
                                                        if current_path_str == bookmark_path {
                                                            // 削除されたので、現在のブックマーク状態をクリア
                                                            current_bookmark.set(None);
                                                        }
                                                    }
                                                }
                                            },
                                            "×"
                                        }
                                        
                                        // クリック可能エリア
                                        div {
                                            style: "cursor: pointer;",
                                            onclick: {
                                                let bookmark_path = bookmark.pdf_path.clone();
                                                move |_| {
                                                    // ブックマークされたPDFを開く（スクロールなし）
                                                    let path = PathBuf::from(&bookmark_path);
                                                    if path.exists() {
                                                        let file_name = path.file_name()
                                                            .and_then(|n| n.to_str())
                                                            .unwrap_or("Unknown")
                                                            .to_string();
                                                        let _ = add_recent_file(path.to_string_lossy().to_string(), file_name);
                                                        recent_files.set(load_recent_files());
                                                        pdf_path.set(Some(path));
                                                        page_cache.write().clear();
                                                        loaded_pdf_path.set(None);
                                                        is_loading.set(false);
                                                        show_bookmarks_popup.set(false);
                                                    }
                                                }
                                            },
                                            div { 
                                                style: "font-weight: bold; margin-bottom: 8px; color: #3498db; font-size: 16px; padding-right: 32px;",
                                                {
                                                    PathBuf::from(&bookmark.pdf_path)
                                                        .file_name()
                                                        .unwrap_or_default()
                                                        .to_string_lossy()
                                                        .to_string()
                                                }
                                            }
                                            div { 
                                                style: "color: #ecf0f1; font-size: 14px; line-height: 1.4; margin-bottom: 8px;",
                                                "ページ: {bookmark.current_page + 1} / {bookmark.total_pages} ({(bookmark.reading_progress * 100.0) as u32}%)"
                                            }
                                            div { 
                                                style: "font-size: 12px; color: #95a5a6;",
                                                "最終閲覧: {bookmark.last_read_time}"
                                            }
                                        }
                                        div {
                                            style: "display: flex; gap: 8px;",
                                            button {
                                                style: "background-color: #3498db; color: white; border: none; border-radius: 4px; padding: 8px 12px; cursor: pointer; font-size: 12px;",
                                                onclick: {
                                                    let bookmark_page = bookmark.current_page;
                                                    move |_| {
                                                        // ポップアップを閉じる
                                                        show_bookmarks_popup.set(false);
                                                        
                                                        // ページにスクロール実行
                                                        let page_id = format!("page-wrapper-{}", bookmark_page);
                                                        println!("Scrolling to bookmarked page {} with ID: {}", bookmark_page + 1, page_id);
                                                        
                                                        // JavaScriptの実行でスクロールを実行
                                                        eval(&format!(
                                                            r#"
                                                            setTimeout(() => {{
                                                                const element = document.getElementById('{}');
                                                                if (element) {{
                                                                    element.scrollIntoView({{ 
                                                                        behavior: 'smooth', 
                                                                        block: 'start' 
                                                                    }});
                                                                    console.log('ブックマークスクロール実行: ページ {}');
                                                                }} else {{
                                                                    console.log('ブックマークスクロール: 要素が見つかりません: {}');
                                                                }}
                                                            }}, 200);
                                                            "#,
                                                            page_id, bookmark_page + 1, page_id
                                                        ));
                                                        
                                                        println!("ブックマークのページ {} に移動します", bookmark_page + 1);
                                                    }
                                                },
                                                {format!("P.{} へ", bookmark.current_page + 1)}
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