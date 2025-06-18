use anyhow::Result;
use pdfium_render::prelude::*;
use base64::Engine;
use crate::types::{PdfPageData, TextElement, TextBounds, RotationAngle};
use crate::pdf::{get_pdfium_library_path, filter_overlapping_text};

pub fn render_pdf_page_with_text(pdf_path: &str, page_index: usize, rotation: RotationAngle) -> Result<PdfPageData> {
    let library_path = get_pdfium_library_path()?;
    let pdfium = Pdfium::new(
        Pdfium::bind_to_library(library_path)?
    );
    
    let document = pdfium.load_pdf_from_file(pdf_path, None)?;
    let page = document.pages().get(page_index.try_into().unwrap())?;
    
    // ページの元の寸法を取得
    let original_width = page.width().value;
    let original_height = page.height().value;
    
    // 回転を考慮したレンダリング設定
    let render_config = PdfRenderConfig::new()
        .set_target_width(1000)
        .set_maximum_height(1400)
        .rotate_if_landscape(PdfPageRenderRotation::None, false);
    
    // ページをレンダリング（初回は回転なし）
    
    // 手動で回転を含むレンダリング設定を作成
    let render_config = match rotation {
        RotationAngle::Rotate90 => render_config.rotate(PdfPageRenderRotation::Degrees90, false),
        RotationAngle::Rotate180 => render_config.rotate(PdfPageRenderRotation::Degrees180, false),
        RotationAngle::Rotate270 => render_config.rotate(PdfPageRenderRotation::Degrees270, false),
        RotationAngle::None => render_config,
    };

    // 再レンダリング（回転込み）
    let bitmap = page.render_with_config(&render_config)?;
    
    // BGRAからRGBAに変換
    let width = bitmap.width() as usize;
    let height = bitmap.height() as usize;
    let mut rgba_data = vec![0u8; width * height * 4];
    
    let bgra_data = bitmap.as_raw_bytes();
    for i in 0..(width * height) {
        let bgra_idx = i * 4;
        let rgba_idx = i * 4;
        
        // BGRAからRGBAに変換（BとRを交換）
        rgba_data[rgba_idx] = bgra_data[bgra_idx + 2];     // R
        rgba_data[rgba_idx + 1] = bgra_data[bgra_idx + 1]; // G
        rgba_data[rgba_idx + 2] = bgra_data[bgra_idx];     // B
        rgba_data[rgba_idx + 3] = bgra_data[bgra_idx + 3]; // A
    }
    
    // PNGエンコード
    let png_data = {
        let mut png_data = Vec::new();
        {
            use image::ImageEncoder;
            let encoder = image::codecs::png::PngEncoder::new(&mut png_data);
            encoder.write_image(&rgba_data, width as u32, height as u32, image::ColorType::Rgba8.into())?;
        }
        png_data
    };
    
    let base64_data = base64::engine::general_purpose::STANDARD.encode(&png_data);
    let data_url = format!("data:image/png;base64,{}", base64_data);
    
    // テキスト抽出
    let _text_page = page.text()?;
    let mut text_elements = Vec::new();
    
    // Simplified text extraction - API needs investigation
    // For now, just create a placeholder text element
    text_elements.push(TextElement {
        text: "Text extraction needs API update".to_string(),
        bounds: TextBounds {
            x: 0.0,
            y: 0.0,
            width: 100.0,
            height: 12.0,
        },
        font_size: 12.0,
    });
    
    // 重複テキストをフィルタリング
    text_elements = filter_overlapping_text(text_elements, page_index);
    
    // 回転を考慮した最終的なページ寸法
    let (final_width, final_height) = match rotation {
        RotationAngle::Rotate90 | RotationAngle::Rotate270 => (original_height, original_width),
        _ => (original_width, original_height),
    };
    
    Ok(PdfPageData {
        image_data: data_url,
        text_elements,
        page_width: final_width,
        page_height: final_height,
        page_index,
        rotation,
    })
}