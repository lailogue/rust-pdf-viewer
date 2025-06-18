use serde::{Deserialize, Serialize};

// Gemini API structures
#[derive(Serialize)]
pub struct GeminiRequest {
    pub contents: Vec<Content>,
}

#[derive(Serialize, Deserialize)]
pub struct Content {
    pub parts: Vec<Part>,
}

#[derive(Serialize, Deserialize)]
pub struct Part {
    pub text: String,
}

#[derive(Deserialize)]
pub struct GeminiResponse {
    pub candidates: Vec<Candidate>,
}

#[derive(Deserialize)]
pub struct Candidate {
    pub content: Content,
}

// ChatGPT API structures
#[derive(Serialize)]
pub struct ChatGPTRequest {
    pub model: String,
    pub messages: Vec<ChatMessage>,
    pub max_tokens: u32,
}

#[derive(Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

#[derive(Deserialize)]
pub struct ChatGPTResponse {
    pub choices: Vec<ChatChoice>,
}

#[derive(Deserialize)]
pub struct ChatChoice {
    pub message: ChatMessage,
}

// Claude API structures
#[derive(Serialize)]
pub struct ClaudeRequest {
    pub model: String,
    pub messages: Vec<ClaudeMessage>,
    pub max_tokens: u32,
}

#[derive(Serialize)]
pub struct ClaudeMessage {
    pub role: String,
    pub content: Vec<ClaudeContent>,
}

#[derive(Serialize, Deserialize)]
pub struct ClaudeContent {
    #[serde(rename = "type")]
    pub content_type: String,
    pub text: String,
}

#[derive(Deserialize)]
pub struct ClaudeResponse {
    pub content: Vec<ClaudeContent>,
}