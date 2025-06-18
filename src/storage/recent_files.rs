use anyhow::Result;
use crate::types::RecentFile;
use crate::storage::config::ensure_data_dir;

pub fn load_recent_files() -> Vec<RecentFile> {
    let data_dir = match ensure_data_dir() {
        Ok(dir) => dir,
        Err(_) => return Vec::new(),
    };
    
    let recent_files_path = data_dir.join("recent_files.json");
    
    if let Ok(content) = std::fs::read_to_string(&recent_files_path) {
        serde_json::from_str(&content).unwrap_or_else(|_| Vec::new())
    } else {
        Vec::new()
    }
}

pub fn save_recent_files(recent_files: &[RecentFile]) -> Result<()> {
    let data_dir = ensure_data_dir()?;
    let recent_files_path = data_dir.join("recent_files.json");
    
    let json = serde_json::to_string_pretty(recent_files)?;
    std::fs::write(&recent_files_path, json)?;
    
    Ok(())
}

pub fn add_recent_file(file_path: String, file_name: String) -> Result<()> {
    let mut recent_files = load_recent_files();
    
    // 既存のファイルを削除（重複を避けるため）
    recent_files.retain(|file| file.path != file_path);
    
    // 新しいファイルを先頭に追加
    let recent_file = RecentFile {
        path: file_path.clone(),
        name: file_name.clone(),
        display_name: file_name,
        last_opened: chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC").to_string(),
    };
    
    recent_files.insert(0, recent_file);
    
    // 最大10個まで保持
    recent_files.truncate(10);
    
    save_recent_files(&recent_files)?;
    Ok(())
}