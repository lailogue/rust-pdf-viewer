use crate::types::TextElement;

pub fn filter_overlapping_text(text_elements: Vec<TextElement>, _page_index: usize) -> Vec<TextElement> {
    let mut filtered_elements = Vec::new();
    
    for element in text_elements {
        let text = element.text.trim();
        
        // 空のテキストや意味のない文字列をスキップ
        if text.is_empty() || text.len() < 2 {
            continue;
        }
        
        // 数字のみの短い文字列をスキップ（ページ番号など）
        if text.len() <= 3 && text.chars().all(|c| c.is_numeric()) {
            continue;
        }
        
        // 重複チェック: 同じテキストが既に存在するかチェック
        let is_duplicate = filtered_elements.iter().any(|existing: &TextElement| {
            let existing_text = existing.text.trim();
            
            // 完全一致
            if existing_text == text {
                return true;
            }
            
            // 一方が他方を含む場合（より長い方を保持）
            if text.len() > existing_text.len() && text.contains(existing_text) {
                return false; // 新しい要素の方が長いので、既存の要素を置き換える
            }
            
            if existing_text.len() > text.len() && existing_text.contains(text) {
                return true; // 既存の要素の方が長いので、新しい要素をスキップ
            }
            
            false
        });
        
        if !is_duplicate {
            // より長いテキストで置き換える場合は、既存の短い要素を削除
            filtered_elements.retain(|existing| {
                let existing_text = existing.text.trim();
                !(text.len() > existing_text.len() && text.contains(existing_text))
            });
            
            filtered_elements.push(element);
        }
    }
    
    filtered_elements
}