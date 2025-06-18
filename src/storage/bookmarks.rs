use anyhow::Result;
use crate::types::ReadingBookmark;
use crate::storage::config::ensure_data_dir;

pub fn load_reading_bookmark(pdf_path: &str) -> Option<ReadingBookmark> {
    let data_dir = match ensure_data_dir() {
        Ok(dir) => dir,
        Err(_) => return None,
    };
    
    let bookmarks_path = data_dir.join("bookmarks.json");
    
    if let Ok(content) = std::fs::read_to_string(&bookmarks_path) {
        let bookmarks: Vec<ReadingBookmark> = serde_json::from_str(&content).unwrap_or_default();
        bookmarks.into_iter().find(|bookmark| bookmark.pdf_path == pdf_path)
    } else {
        None
    }
}

pub fn save_reading_bookmark(bookmark: ReadingBookmark) -> Result<()> {
    let data_dir = ensure_data_dir()?;
    let bookmarks_path = data_dir.join("bookmarks.json");
    
    let mut bookmarks: Vec<ReadingBookmark> = if let Ok(content) = std::fs::read_to_string(&bookmarks_path) {
        serde_json::from_str(&content).unwrap_or_default()
    } else {
        Vec::new()
    };
    
    // 既存のブックマークを更新または新規追加
    if let Some(existing) = bookmarks.iter_mut().find(|b| b.pdf_path == bookmark.pdf_path) {
        *existing = bookmark;
    } else {
        bookmarks.push(bookmark);
    }
    
    let json = serde_json::to_string_pretty(&bookmarks)?;
    std::fs::write(&bookmarks_path, json)?;
    
    Ok(())
}

pub fn get_all_reading_bookmarks() -> Vec<ReadingBookmark> {
    let data_dir = match ensure_data_dir() {
        Ok(dir) => dir,
        Err(_) => return Vec::new(),
    };
    
    let bookmarks_path = data_dir.join("bookmarks.json");
    
    if let Ok(content) = std::fs::read_to_string(&bookmarks_path) {
        serde_json::from_str(&content).unwrap_or_default()
    } else {
        Vec::new()
    }
}

pub fn delete_reading_bookmark(pdf_path: &str) -> Result<()> {
    let data_dir = ensure_data_dir()?;
    let bookmarks_path = data_dir.join("bookmarks.json");
    
    let mut bookmarks: Vec<ReadingBookmark> = if let Ok(content) = std::fs::read_to_string(&bookmarks_path) {
        serde_json::from_str(&content).unwrap_or_default()
    } else {
        Vec::new()
    };
    
    // 指定されたパスのブックマークを削除
    bookmarks.retain(|b| b.pdf_path != pdf_path);
    
    let json = serde_json::to_string_pretty(&bookmarks)?;
    std::fs::write(&bookmarks_path, json)?;
    
    Ok(())
}