use dioxus::prelude::*;
use std::path::PathBuf;
use std::collections::HashMap;
use crate::{RecentFile, load_recent_files, add_recent_file, PdfPageData};

#[component]
pub fn recent_files_popup(
    show_recent_files_popup: Signal<bool>,
    recent_files_list: Memo<Vec<RecentFile>>,
    recent_files: Signal<Vec<RecentFile>>,
    pdf_path: Signal<Option<PathBuf>>,
    page_cache: Signal<HashMap<usize, PdfPageData>>,
    loaded_pdf_path: Signal<Option<PathBuf>>,
    is_loading: Signal<bool>,
) -> Element {
    rsx! {
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
                                            let file_name = file_path.file_name()
                                                .and_then(|n| n.to_str())
                                                .unwrap_or("Unknown")
                                                .to_string();
                                            let _ = add_recent_file(file_path.to_string_lossy().to_string(), file_name);
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