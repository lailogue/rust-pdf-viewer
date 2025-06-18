use anyhow::Result;
use std::collections::HashMap;
use crate::types::{PageRotations, RotationAngle};
use crate::storage::config::ensure_data_dir;

pub fn load_page_rotations(pdf_path: &str) -> HashMap<usize, RotationAngle> {
    let data_dir = match ensure_data_dir() {
        Ok(dir) => dir,
        Err(_) => return HashMap::new(),
    };
    
    let rotations_path = data_dir.join("page_rotations.json");
    
    if let Ok(content) = std::fs::read_to_string(&rotations_path) {
        let all_rotations: Vec<PageRotations> = serde_json::from_str(&content).unwrap_or_default();
        if let Some(pdf_rotations) = all_rotations.iter().find(|r| r.pdf_path == pdf_path) {
            return pdf_rotations.rotations.clone();
        }
    }
    
    HashMap::new()
}

pub fn save_page_rotations(pdf_path: &str, rotations: HashMap<usize, RotationAngle>) -> Result<()> {
    let data_dir = ensure_data_dir()?;
    let rotations_path = data_dir.join("page_rotations.json");
    
    let mut all_rotations: Vec<PageRotations> = if let Ok(content) = std::fs::read_to_string(&rotations_path) {
        serde_json::from_str(&content).unwrap_or_default()
    } else {
        Vec::new()
    };
    
    let page_rotations = PageRotations {
        pdf_path: pdf_path.to_string(),
        rotations,
        last_modified: chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC").to_string(),
    };
    
    // 既存のエントリを更新または新規追加
    if let Some(existing) = all_rotations.iter_mut().find(|r| r.pdf_path == pdf_path) {
        *existing = page_rotations;
    } else {
        all_rotations.push(page_rotations);
    }
    
    let json = serde_json::to_string_pretty(&all_rotations)?;
    std::fs::write(&rotations_path, json)?;
    
    Ok(())
}