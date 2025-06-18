use anyhow::Result;
use crate::types::FlashCard;
use crate::storage::config::ensure_data_dir;

pub fn load_flashcards() -> Vec<FlashCard> {
    let data_dir = match ensure_data_dir() {
        Ok(dir) => dir,
        Err(_) => return Vec::new(),
    };
    
    let flashcards_path = data_dir.join("flashcards.json");
    
    if let Ok(content) = std::fs::read_to_string(&flashcards_path) {
        serde_json::from_str(&content).unwrap_or_else(|_| Vec::new())
    } else {
        Vec::new()
    }
}

pub fn save_flashcards(flashcards: &[FlashCard]) -> Result<()> {
    let data_dir = ensure_data_dir()?;
    let flashcards_path = data_dir.join("flashcards.json");
    
    let json = serde_json::to_string_pretty(flashcards)?;
    std::fs::write(&flashcards_path, json)?;
    
    Ok(())
}

pub fn add_flashcard(term: String, definition: String) -> Result<()> {
    let mut flashcards = load_flashcards();
    
    // 既存の用語をチェック（大文字小文字を区別しない）
    let term_lower = term.to_lowercase();
    if let Some(existing) = flashcards.iter_mut().find(|card| card.term.to_lowercase() == term_lower) {
        // 既存の用語の定義を更新
        existing.definition = definition;
        existing.created_at = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC").to_string();
    } else {
        // 新しい単語帳エントリを作成
        let timestamp = chrono::Utc::now().timestamp_millis();
        let term_prefix = term.chars().take(10).collect::<String>().replace(" ", "-").to_lowercase();
        let id = format!("{}-{}", timestamp, term_prefix);
        
        let flashcard = FlashCard {
            id,
            term,
            definition,
            created_at: chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC").to_string(),
        };
        
        flashcards.insert(0, flashcard); // 新しいものを先頭に追加
    }
    
    save_flashcards(&flashcards)?;
    Ok(())
}

pub fn delete_flashcard(card_id: &str) -> Result<()> {
    let mut flashcards = load_flashcards();
    flashcards.retain(|card| card.id != card_id);
    save_flashcards(&flashcards)?;
    Ok(())
}

pub fn append_detailed_explanation(card_id: &str, detailed_explanation: String) -> Result<()> {
    let mut flashcards = load_flashcards();
    
    if let Some(card) = flashcards.iter_mut().find(|card| card.id == card_id) {
        // 既存の説明に詳細説明を追記
        card.definition = format!("{}\n===========\n{}", card.definition, detailed_explanation);
        card.created_at = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC").to_string();
        save_flashcards(&flashcards)?;
        Ok(())
    } else {
        Err(anyhow::anyhow!("Flashcard with id {} not found", card_id))
    }
}