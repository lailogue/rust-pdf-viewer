use std::path::PathBuf;

pub fn get_data_dir() -> PathBuf {
    if let Some(config_dir) = dirs::config_dir() {
        config_dir.join("pdf-viewer")
    } else {
        PathBuf::from(".config/pdf-viewer")
    }
}

pub fn ensure_data_dir() -> std::io::Result<PathBuf> {
    let data_dir = get_data_dir();
    std::fs::create_dir_all(&data_dir)?;
    Ok(data_dir)
}