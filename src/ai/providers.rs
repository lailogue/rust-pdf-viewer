use anyhow::Result;
use reqwest;
use crate::types::ai::*;

pub async fn search_with_gemini(query: String, api_key: String) -> Result<String> {
    let url = format!("https://generativelanguage.googleapis.com/v1beta/models/gemini-1.5-flash-latest:generateContent?key={}", api_key);
    
    let request_body = GeminiRequest {
        contents: vec![Content {
            parts: vec![Part {
                text: query,
            }],
        }],
    };
    
    let client = reqwest::Client::new();
    let response = client
        .post(&url)
        .header("Content-Type", "application/json")
        .json(&request_body)
        .send()
        .await?;
    
    if !response.status().is_success() {
        let error_text = response.text().await?;
        return Err(anyhow::anyhow!("Gemini API error: {}", error_text));
    }
    
    let gemini_response: GeminiResponse = response.json().await?;
    
    if let Some(candidate) = gemini_response.candidates.first() {
        if let Some(part) = candidate.content.parts.first() {
            return Ok(part.text.clone());
        }
    }
    
    Err(anyhow::anyhow!("Gemini APIからの応答が空です"))
}

pub async fn search_with_chatgpt(query: String, api_key: String) -> Result<String> {
    let url = "https://api.openai.com/v1/chat/completions";
    
    let request_body = ChatGPTRequest {
        model: "gpt-3.5-turbo".to_string(),
        messages: vec![ChatMessage {
            role: "user".to_string(),
            content: query,
        }],
        max_tokens: 500,
    };
    
    let client = reqwest::Client::new();
    let response = client
        .post(url)
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", api_key))
        .json(&request_body)
        .send()
        .await?;
    
    if !response.status().is_success() {
        let error_text = response.text().await?;
        return Err(anyhow::anyhow!("ChatGPT API error: {}", error_text));
    }
    
    let chatgpt_response: ChatGPTResponse = response.json().await?;
    
    if let Some(choice) = chatgpt_response.choices.first() {
        return Ok(choice.message.content.clone());
    }
    
    Err(anyhow::anyhow!("ChatGPT APIからの応答が空です"))
}

pub async fn search_with_claude(query: String, api_key: String) -> Result<String> {
    let url = "https://api.anthropic.com/v1/messages";
    
    let request_body = ClaudeRequest {
        model: "claude-3-haiku-20240307".to_string(),
        messages: vec![ClaudeMessage {
            role: "user".to_string(),
            content: vec![ClaudeContent {
                content_type: "text".to_string(),
                text: query,
            }],
        }],
        max_tokens: 500,
    };
    
    let client = reqwest::Client::new();
    let response = client
        .post(url)
        .header("Content-Type", "application/json")
        .header("x-api-key", api_key)
        .header("anthropic-version", "2023-06-01")
        .json(&request_body)
        .send()
        .await?;
    
    if !response.status().is_success() {
        let error_text = response.text().await?;
        return Err(anyhow::anyhow!("Claude API error: {}", error_text));
    }
    
    let claude_response: ClaudeResponse = response.json().await?;
    
    if let Some(content) = claude_response.content.first() {
        return Ok(content.text.clone());
    }
    
    Err(anyhow::anyhow!("Claude APIからの応答が空です"))
}