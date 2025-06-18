use anyhow::Result;
use crate::types::AIProvider;
use crate::ai::providers::{search_with_gemini, search_with_chatgpt, search_with_claude};

pub async fn search_with_ai(provider: AIProvider, query: String, api_key: String) -> Result<String> {
    match provider {
        AIProvider::Gemini => search_with_gemini(query, api_key).await,
        AIProvider::ChatGPT => search_with_chatgpt(query, api_key).await,
        AIProvider::Claude => search_with_claude(query, api_key).await,
    }
}

pub async fn detailed_search_with_ai(provider: AIProvider, term: String, api_key: String) -> Result<String> {
    let detailed_query = format!("{}とはなんですか。詳細に説明してください", term);
    search_with_ai(provider, detailed_query, api_key).await
}

#[allow(dead_code)]
pub fn clean_markdown_text(text: &str) -> String {
    text.lines()
        .map(|line| {
            // Markdownの見出し記号を削除
            let line = line.trim_start_matches('#').trim();
            // Markdownの太字記号を削除
            let line = line.replace("**", "");
            // Markdownのイタリック記号を削除
            let line = line.replace("*", "");
            // Markdownのコードブロック記号を削除
            let line = line.replace("`", "");
            line
        })
        .collect::<Vec<_>>()
        .join("\n")
        .trim()
        .to_string()
}