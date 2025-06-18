use dioxus::prelude::*;
use std::path::PathBuf;
use std::collections::HashMap;
use crate::{get_all_reading_bookmarks, add_recent_file, load_recent_files, RecentFile, PdfPageData};

#[component]
pub fn bookmarks_popup(
    show_bookmarks_popup: Signal<bool>,
    pdf_path: Signal<Option<PathBuf>>,
    page_cache: Signal<HashMap<usize, PdfPageData>>,
    loaded_pdf_path: Signal<Option<PathBuf>>,
    is_loading: Signal<bool>,
    recent_files: Signal<Vec<RecentFile>>,
) -> Element {
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
                        {
                            let bookmarks = get_all_reading_bookmarks();
                            format!("🔖 ブックマーク ({}件)", bookmarks.len())
                        }
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
                    let bookmarks = get_all_reading_bookmarks();
                    if bookmarks.is_empty() {
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
                                for bookmark in bookmarks.iter() {
                                    div { 
                                        key: "{bookmark.pdf_path}",
                                        class: "bookmark-item",
                                        style: "background-color: #34495e; border-radius: 6px; padding: 16px; margin-bottom: 12px; cursor: pointer; transition: background-color 0.2s; border: 1px solid #445a6f;",
                                        onclick: {
                                            let bookmark_path = bookmark.pdf_path.clone();
                                            let _bookmark_page = bookmark.current_page;
                                            move |_| {
                                                // ブックマークされたPDFを開く
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
                                                    
                                                    // TODO: ブックマークされたページまでスクロール
                                                }
                                            }
                                        },
                                        div { 
                                            style: "font-weight: bold; margin-bottom: 8px; color: #3498db; font-size: 16px;",
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
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}