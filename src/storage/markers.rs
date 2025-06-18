use anyhow::Result;
use crate::types::{PositionMarker, PdfMarkers};
use crate::storage::config::ensure_data_dir;

pub fn load_position_markers(pdf_path: &str) -> Vec<PositionMarker> {
    let data_dir = match ensure_data_dir() {
        Ok(dir) => dir,
        Err(_) => return Vec::new(),
    };
    
    let markers_path = data_dir.join("position_markers.json");
    
    if let Ok(content) = std::fs::read_to_string(&markers_path) {
        let all_markers: Vec<PdfMarkers> = serde_json::from_str(&content).unwrap_or_default();
        if let Some(pdf_markers) = all_markers.iter().find(|m| m.pdf_path == pdf_path) {
            return pdf_markers.markers.clone();
        }
    }
    
    Vec::new()
}

pub fn save_position_marker(pdf_path: &str, marker: PositionMarker) -> Result<()> {
    let data_dir = ensure_data_dir()?;
    let markers_path = data_dir.join("position_markers.json");
    
    let mut all_markers: Vec<PdfMarkers> = if let Ok(content) = std::fs::read_to_string(&markers_path) {
        serde_json::from_str(&content).unwrap_or_default()
    } else {
        Vec::new()
    };
    
    // このPDFのマーカーセットを見つけるか作成
    if let Some(existing) = all_markers.iter_mut().find(|m| m.pdf_path == pdf_path) {
        existing.markers.push(marker);
        existing.last_modified = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC").to_string();
    } else {
        let pdf_markers = PdfMarkers {
            pdf_path: pdf_path.to_string(),
            markers: vec![marker],
            last_modified: chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC").to_string(),
        };
        all_markers.push(pdf_markers);
    }
    
    let json = serde_json::to_string_pretty(&all_markers)?;
    std::fs::write(&markers_path, json)?;
    
    Ok(())
}

pub fn delete_position_marker(pdf_path: &str, marker_id: &str) -> Result<()> {
    let data_dir = ensure_data_dir()?;
    let markers_path = data_dir.join("position_markers.json");
    
    let mut all_markers: Vec<PdfMarkers> = if let Ok(content) = std::fs::read_to_string(&markers_path) {
        serde_json::from_str(&content).unwrap_or_default()
    } else {
        return Ok(()); // ファイルが存在しない場合は何もしない
    };
    
    // 指定されたマーカーを削除
    if let Some(existing) = all_markers.iter_mut().find(|m| m.pdf_path == pdf_path) {
        existing.markers.retain(|marker| marker.id != marker_id);
        existing.last_modified = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC").to_string();
    }
    
    let json = serde_json::to_string_pretty(&all_markers)?;
    std::fs::write(&markers_path, json)?;
    
    Ok(())
}