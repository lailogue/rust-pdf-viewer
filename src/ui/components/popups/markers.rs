use dioxus::prelude::*;
use std::path::PathBuf;
use crate::{PositionMarker, load_position_markers, delete_position_marker};

#[component]
pub fn markers_popup(
    show_markers_popup: Signal<bool>,
    position_markers: Signal<Vec<PositionMarker>>,
    pdf_path: Signal<Option<PathBuf>>,
) -> Element {
    rsx! {
        div { 
            class: "popup-overlay",
            style: "position: fixed; top: 0; left: 0; width: 100%; height: 100%; background-color: rgba(0, 0, 0, 0.7); display: flex; align-items: center; justify-content: center; z-index: 1000;",
            onclick: move |_| {
                show_markers_popup.set(false);
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
                            let marker_count = position_markers().len();
                            format!("📍 位置マーカー ({}件)", marker_count)
                        }
                    }
                    button { 
                        style: "background: none; border: none; color: #e74c3c; cursor: pointer; font-size: 24px; padding: 0;",
                        onclick: move |_| {
                            show_markers_popup.set(false);
                        },
                        "×"
                    }
                }
                
                // マーカーリスト
                {
                    let markers = position_markers();
                    if markers.is_empty() {
                        rsx! {
                            div { 
                                style: "text-align: center; padding: 40px; color: #bdc3c7; font-size: 16px;",
                                "まだ位置マーカーがありません。\nマーカーモードをONにしてPDFページをクリックしてマーカーを設置してみましょう！"
                            }
                        }
                    } else {
                        rsx! {
                            div { 
                                class: "markers-list",
                                style: "max-height: 400px; overflow-y: auto;",
                                for marker in markers.iter() {
                                    div { 
                                        key: "{marker.id}",
                                        class: "marker-item",
                                        style: "background-color: #34495e; border-radius: 6px; padding: 16px; margin-bottom: 12px; border: 1px solid #445a6f; display: flex; justify-content: space-between; align-items: center;",
                                        div {
                                            style: "flex: 1; cursor: pointer;",
                                            onclick: {
                                                let page_index = marker.page_index;
                                                move |_| {
                                                    // ポップアップを閉じる
                                                    show_markers_popup.set(false);
                                                    
                                                    // ページにスクロール実行
                                                    let page_id = format!("page-wrapper-{}", page_index);
                                                    println!("Scrolling to page {} with ID: {}", page_index + 1, page_id);
                                                    
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
                                                                console.log('スクロール実行: ページ {}');
                                                            }} else {{
                                                                console.log('要素が見つかりません: {}');
                                                            }}
                                                        }}, 200);
                                                        "#,
                                                        page_id, page_index + 1, page_id
                                                    ));
                                                    
                                                    println!("マーカーのページ {} に移動します", page_index + 1);
                                                }
                                            },
                                            title: "クリックしてページに移動",
                                            div { 
                                                style: "font-weight: bold; margin-bottom: 8px; color: #f39c12; font-size: 18px;",
                                                {format!("📄 ページ {} のマーカー", marker.page_index + 1)}
                                            }
                                            div { 
                                                style: "color: #ecf0f1; font-size: 14px; line-height: 1.4; margin-bottom: 8px;",
                                                {format!("位置: X={:.1}%, Y={:.1}%", marker.x * 100.0, marker.y * 100.0)}
                                            }
                                            div { 
                                                style: "font-size: 12px; color: #95a5a6;",
                                                {format!("作成日時: {}", marker.created_at)}
                                            }
                                        }
                                        div {
                                            style: "display: flex; gap: 8px;",
                                            button {
                                                style: "background-color: #3498db; color: white; border: none; border-radius: 4px; padding: 8px 12px; cursor: pointer; font-size: 12px;",
                                                onclick: {
                                                    let page_index = marker.page_index;
                                                    move |_| {
                                                        // ポップアップを閉じる
                                                        show_markers_popup.set(false);
                                                        
                                                        // ページにスクロール実行
                                                        let page_id = format!("page-wrapper-{}", page_index);
                                                        println!("Scrolling to page {} with ID: {}", page_index + 1, page_id);
                                                        
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
                                                                    console.log('スクロール実行: ページ {}');
                                                                }} else {{
                                                                    console.log('要素が見つかりません: {}');
                                                                }}
                                                            }}, 200);
                                                            "#,
                                                            page_id, page_index + 1, page_id
                                                        ));
                                                        
                                                        println!("マーカーのページ {} に移動します", page_index + 1);
                                                    }
                                                },
                                                {format!("P.{} へ", marker.page_index + 1)}
                                            }
                                            button {
                                                style: "background-color: #e74c3c; color: white; border: none; border-radius: 4px; padding: 8px 12px; cursor: pointer; font-size: 12px;",
                                            onclick: {
                                                let marker_id = marker.id.clone();
                                                move |_| {
                                                    if let Some(path) = pdf_path() {
                                                        // マーカーを削除
                                                        let _ = delete_position_marker(&path.to_string_lossy(), &marker_id);
                                                        
                                                        // マーカー状態を更新
                                                        let markers = load_position_markers(&path.to_string_lossy());
                                                        position_markers.set(markers);
                                                    }
                                                }
                                            },
                                            "削除"
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