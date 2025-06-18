use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use super::pdf::RotationAngle;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FlashCard {
    pub id: String,
    pub term: String,
    pub definition: String,
    pub created_at: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RecentFile {
    pub path: String,
    pub name: String,
    pub display_name: String,
    pub last_opened: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ApiKeys {
    pub gemini: Option<String>,
    pub chatgpt: Option<String>,
    pub claude: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PageRotations {
    pub pdf_path: String,
    pub rotations: HashMap<usize, RotationAngle>, // ページインデックス -> 回転角度
    pub last_modified: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ReadingBookmark {
    pub pdf_path: String,
    pub current_page: usize,          // 現在読んでいるページ
    pub total_pages: usize,           // PDF総ページ数
    pub last_read_time: String,       // 最後に読んだ時間
    pub reading_progress: f32,        // 読書進捗率（0.0-1.0）
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PositionMarker {
    pub id: String,
    pub page_index: usize,            // ページ番号（0から始まる）
    pub x: f32,                       // ページ内のX座標（相対位置 0.0-1.0）
    pub y: f32,                       // ページ内のY座標（相対位置 0.0-1.0）
    pub created_at: String,           // 作成日時
    pub note: String,                 // オプションのメモ
}

impl PositionMarker {
    pub fn new(page_index: usize, x: f32, y: f32, note: String) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            page_index,
            x,
            y,
            created_at: chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC").to_string(),
            note,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PdfMarkers {
    pub pdf_path: String,
    pub markers: Vec<PositionMarker>,
    pub last_modified: String,
}