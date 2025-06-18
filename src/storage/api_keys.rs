use anyhow::Result;
use crate::types::ApiKeys;
use crate::storage::config::ensure_data_dir;

pub fn load_api_keys() -> ApiKeys {
    let data_dir = match ensure_data_dir() {
        Ok(dir) => dir,
        Err(_) => return ApiKeys::default(),
    };
    
    let api_keys_path = data_dir.join("api_keys.json");
    
    if let Ok(content) = std::fs::read_to_string(&api_keys_path) {
        serde_json::from_str(&content).unwrap_or_default()
    } else {
        ApiKeys::default()
    }
}

pub fn save_api_keys(api_keys: &ApiKeys) -> Result<()> {
    let data_dir = ensure_data_dir()?;
    let api_keys_path = data_dir.join("api_keys.json");
    
    let json = serde_json::to_string_pretty(api_keys)?;
    std::fs::write(&api_keys_path, json)?;
    
    Ok(())
}

impl Default for ApiKeys {
    fn default() -> Self {
        Self {
            gemini: None,
            chatgpt: None,
            claude: None,
        }
    }
}