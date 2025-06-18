use dioxus::prelude::*;
use crate::{FlashCard, load_flashcards, delete_flashcard, detailed_search_with_ai, append_detailed_explanation, AIProvider};

#[component]
pub fn flashcard_popup(
    show_flashcard_popup: Signal<bool>,
    flashcards: Signal<Vec<FlashCard>>,
    show_flashcard_details: Signal<bool>,
    selected_flashcard: Signal<Option<FlashCard>>,
) -> Element {
    let flashcard_list = flashcards;
    
    rsx! {
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
                        "まだ単語が保存されていません。\nLLM Searchを使って単語を保存してみましょう！"
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
}

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
                                disabled: is_detail_searching(),
                                style: {
                                    let bg_color = if is_detail_searching() { "#95a5a6" } else { "#27ae60" };
                                    format!("background-color: {}; color: white; border: none; border-radius: 4px; padding: 6px 12px; cursor: pointer; font-size: 12px;", bg_color)
                                },
                                onclick: {
                                    let card_id = card.id.clone();
                                    let term = card.term.clone();
                                    let provider = selected_provider();
                                    let api_key = match provider {
                                        AIProvider::Gemini => gemini_api_key.clone(),
                                        AIProvider::ChatGPT => chatgpt_api_key.clone(),
                                        AIProvider::Claude => claude_api_key.clone(),
                                    };
                                    move |_| {
                                        if api_key.is_empty() {
                                            return;
                                        }
                                        
                                        is_detail_searching.set(true);
                                        
                                        let card_id_clone = card_id.clone();
                                        let term_clone = term.clone();
                                        let api_key_clone = api_key.clone();
                                        let provider_clone = provider.clone();
                                        
                                        spawn(async move {
                                            match detailed_search_with_ai(provider_clone, term_clone, api_key_clone).await {
                                                Ok(result) => {
                                                    // 詳細説明を単語帳に追記
                                                    if let Ok(_) = append_detailed_explanation(&card_id_clone, result) {
                                                        // 単語帳リストを更新して即座に反映
                                                        flashcards.set(load_flashcards());
                                                        // 現在選択中のカードも更新
                                                        if let Some(updated_card) = load_flashcards().iter().find(|c| c.id == card_id_clone) {
                                                            selected_flashcard.set(Some(updated_card.clone()));
                                                        }
                                                    }
                                                }
                                                Err(_) => {
                                                    // エラーハンドリング（必要に応じて）
                                                }
                                            }
                                            is_detail_searching.set(false);
                                        });
                                    }
                                },
                                if is_detail_searching() { "🔍 検索中..." } else { "🔍 さらに詳しく" }
                            }
                            button { 
                                style: "background-color: #e74c3c; color: white; border: none; border-radius: 4px; padding: 6px 12px; cursor: pointer; font-size: 12px;",
                                onclick: {
                                    let card_id = card.id.clone();
                                    move |_| {
                                        if let Ok(_) = delete_flashcard(&card_id) {
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
    } else {
        rsx! { div {} }
    }
}