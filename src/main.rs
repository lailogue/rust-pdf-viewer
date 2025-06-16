use anyhow::Result;
use eframe::egui;
use std::env;
use std::path::PathBuf;
use pdfium_render::prelude::*;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use std::thread;
use std::sync::mpsc::{self, Receiver, Sender};

fn setup_custom_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();
    
    // macOSのシステムフォントを追加
    if let Ok(font_data) = std::fs::read("/System/Library/Fonts/ヒラギノ角ゴシック W3.ttc") {
        fonts.font_data.insert(
            "hiragino".to_owned(),
            egui::FontData::from_owned(font_data)
        );
        
        fonts.families.get_mut(&egui::FontFamily::Proportional)
            .unwrap()
            .insert(0, "hiragino".to_owned());
            
        fonts.families.get_mut(&egui::FontFamily::Monospace)
            .unwrap()
            .push("hiragino".to_owned());
    }
    
    // その他の日本語フォントも試す
    let japanese_fonts = vec![
        "/System/Library/Fonts/NotoSansCJK.ttc",
        "/System/Library/Fonts/Supplemental/NotoSansCJK-Regular.ttc",
        "/Library/Fonts/Arial Unicode MS.ttf",
        "/System/Library/Fonts/PingFang.ttc",
    ];
    
    for (i, font_path) in japanese_fonts.iter().enumerate() {
        if let Ok(font_data) = std::fs::read(font_path) {
            let font_name = format!("japanese_font_{}", i);
            fonts.font_data.insert(
                font_name.clone(),
                egui::FontData::from_owned(font_data)
            );
            
            fonts.families.get_mut(&egui::FontFamily::Proportional)
                .unwrap()
                .push(font_name.clone());
        }
    }
    
    ctx.set_fonts(fonts);
}

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <pdf_file>", args[0]);
        std::process::exit(1);
    }

    let pdf_path = PathBuf::from(&args[1]);
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([1024.0, 768.0]),
        ..Default::default()
    };

    eframe::run_native(
        "PDF Viewer",
        options,
        Box::new(|cc| {
            // 日本語フォントの設定
            setup_custom_fonts(&cc.egui_ctx);
            Ok(Box::new(PdfViewerApp::new(pdf_path)))
        }),
    ).map_err(|e| anyhow::anyhow!("Failed to run GUI: {}", e))
}

#[derive(Serialize, Deserialize)]
struct GeminiRequest {
    contents: Vec<Content>,
}

#[derive(Serialize, Deserialize)]
struct Content {
    parts: Vec<Part>,
}

#[derive(Serialize, Deserialize)]
struct Part {
    text: String,
}

#[derive(Serialize, Deserialize)]
struct GeminiResponse {
    candidates: Vec<Candidate>,
}

#[derive(Serialize, Deserialize)]
struct Candidate {
    content: Content,
}

struct PdfViewerApp {
    pdf_path: PathBuf,
    pdfium: Option<Pdfium>,
    current_page: usize,
    total_pages: usize,
    pdf_info: String,
    error_message: Option<String>,
    page_textures: std::collections::HashMap<usize, egui::TextureHandle>,
    
    // Gemini AI関連のフィールド
    api_key: String,
    search_query: String,
    search_result: String,
    is_searching: bool,
    search_receiver: Option<Receiver<String>>,
}

impl PdfViewerApp {
    fn render_page(&mut self, page_index: usize, ctx: &egui::Context) -> Option<&egui::TextureHandle> {
        if self.page_textures.contains_key(&page_index) {
            return self.page_textures.get(&page_index);
        }

        if let Some(pdfium) = &self.pdfium {
            if let Ok(document) = pdfium.load_pdf_from_file(&self.pdf_path, None) {
                if let Ok(page) = document.pages().get(page_index as u16) {
                let render_config = PdfRenderConfig::new()
                    .set_target_width(800)
                    .set_maximum_height(1200)
                    .rotate_if_landscape(PdfPageRenderRotation::Degrees90, true);

                match page.render_with_config(&render_config) {
                    Ok(bitmap) => {
                        let width = bitmap.width() as usize;
                        let height = bitmap.height() as usize;
                        
                        let image_buffer = bitmap.as_raw_bytes();
                        let mut rgba_pixels = Vec::with_capacity(width * height * 4);
                        
                        for chunk in image_buffer.chunks(4) {
                            if chunk.len() >= 4 {
                                rgba_pixels.push(chunk[2]); // R
                                rgba_pixels.push(chunk[1]); // G  
                                rgba_pixels.push(chunk[0]); // B
                                rgba_pixels.push(chunk[3]); // A
                            }
                        }

                        let color_image = egui::ColorImage::from_rgba_unmultiplied(
                            [width, height],
                            &rgba_pixels,
                        );
                        
                        let texture_handle = ctx.load_texture(
                            format!("pdf_page_{}", page_index),
                            color_image,
                            egui::TextureOptions::default(),
                        );
                        
                        self.page_textures.insert(page_index, texture_handle);
                        return self.page_textures.get(&page_index);
                    }
                    Err(e) => {
                        self.error_message = Some(format!("Failed to render page {}: {}", page_index + 1, e));
                    }
                }
            }
        }
        }
        None
    }
    fn new(pdf_path: PathBuf) -> Self {
        let mut app = Self {
            pdf_path,
            pdfium: None,
            current_page: 0,
            total_pages: 0,
            pdf_info: String::new(),
            error_message: None,
            page_textures: std::collections::HashMap::new(),
            api_key: String::new(),
            search_query: String::new(),
            search_result: String::new(),
            is_searching: false,
            search_receiver: None,
        };
        app.load_pdf();
        app
    }

    fn load_pdf(&mut self) {
        match Pdfium::bind_to_library(Pdfium::pdfium_platform_library_name()) {
            Ok(bindings) => {
                let pdfium = Pdfium::new(bindings);
                match pdfium.load_pdf_from_file(&self.pdf_path, None) {
                    Ok(document) => {
                        let page_count = document.pages().len() as usize;
                        self.total_pages = page_count;
                        
                        let mut info = format!("Pages: {}\n", page_count);
                        info.push_str("File: ");
                        info.push_str(self.pdf_path.file_name().unwrap_or_default().to_string_lossy().as_ref());
                        info.push_str("\n(PDF successfully loaded with pdfium-render)");
                        
                        self.pdf_info = info;
                        self.error_message = None;
                    }
                    Err(e) => {
                        self.error_message = Some(format!("Failed to load PDF: {}", e));
                        return;
                    }
                };
                self.pdfium = Some(pdfium);
            }
            Err(e) => {
                self.error_message = Some(format!("Failed to initialize Pdfium: {}", e));
            }
        }
    }

    fn get_pdf_info(&self) -> &str {
        if self.pdf_info.is_empty() {
            "No PDF loaded"
        } else {
            &self.pdf_info
        }
    }

    fn start_search(&mut self, query: String, api_key: String, ctx: egui::Context) {
        let (sender, receiver) = mpsc::channel();
        self.search_receiver = Some(receiver);
        self.is_searching = true;
        self.search_result = "検索中...".to_string();
        
        thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            let result = rt.block_on(async {
                Self::search_with_gemini(&query, &api_key).await
            });
            
            match result {
                Ok(response) => {
                    let _ = sender.send(response);
                }
                Err(e) => {
                    let _ = sender.send(format!("エラー: {}", e));
                }
            }
            
            ctx.request_repaint();
        });
    }
    
    async fn search_with_gemini(query: &str, api_key: &str) -> Result<String> {
        let client = reqwest::Client::new();
        let url = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/gemini-2.5-flash-preview-05-20:generateContent?key={}",
            api_key
        );

        let request = GeminiRequest {
            contents: vec![Content {
                parts: vec![Part {
                    text: format!("{}とは何ですか。簡潔に説明してください", query),
                }],
            }],
        };

        let response = client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await?
            .json::<GeminiResponse>()
            .await?;

        if let Some(candidate) = response.candidates.first() {
            if let Some(part) = candidate.content.parts.first() {
                Ok(part.text.clone())
            } else {
                Err(anyhow::anyhow!("レスポンスにテキストが含まれていません"))
            }
        } else {
            Err(anyhow::anyhow!("レスポンスに候補が含まれていません"))
        }
    }
}

impl eframe::App for PdfViewerApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // 検索結果を確認
        if let Some(receiver) = &self.search_receiver {
            if let Ok(result) = receiver.try_recv() {
                self.search_result = result;
                self.is_searching = false;
                self.search_receiver = None;
            }
        }
        
        egui::SidePanel::left("left_panel").show(ctx, |ui| {
            ui.heading("AI検索");
            
            ui.separator();
            
            ui.label("APIキー:");
            ui.text_edit_singleline(&mut self.api_key);
            
            ui.separator();
            
            ui.label("検索語句:");
            ui.text_edit_singleline(&mut self.search_query);
            
            ui.horizontal(|ui| {
                if ui.button("検索").clicked() && !self.api_key.is_empty() && !self.search_query.is_empty() && !self.is_searching {
                    let api_key = self.api_key.clone();
                    let query = self.search_query.clone();
                    let ctx_clone = ctx.clone();
                    self.start_search(query, api_key, ctx_clone);
                }
                
                if self.is_searching {
                    ui.spinner();
                }
            });
            
            ui.separator();
            
            ui.label("検索結果:");
            egui::ScrollArea::vertical().max_height(200.0).show(ui, |ui| {
                ui.label(&self.search_result);
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("PDF Viewer");

            if let Some(ref error) = self.error_message {
                ui.colored_label(egui::Color32::RED, error);
                return;
            }

            if self.pdfium.is_none() {
                ui.label("Loading PDF...");
                return;
            }

            ui.horizontal(|ui| {
                if ui.button("Previous").clicked() && self.current_page > 0 {
                    self.current_page -= 1;
                }
                
                ui.label(format!("Page {} of {}", self.current_page + 1, self.total_pages));
                
                if ui.button("Next").clicked() && self.current_page < self.total_pages.saturating_sub(1) {
                    self.current_page += 1;
                }
            });

            ui.separator();

            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.group(|ui| {
                    ui.vertical(|ui| {
                        ui.label("PDF Information:");
                        ui.label(self.get_pdf_info());
                        
                        ui.separator();
                        
                        ui.label(format!("Currently viewing page: {}", self.current_page + 1));
                        
                        // 実際のPDFページをレンダリングして表示
                        if let Some(texture) = self.render_page(self.current_page, ctx) {
                            let image_size = texture.size_vec2();
                            let available_size = ui.available_size();
                            
                            // 利用可能なサイズに合わせてスケールを計算
                            let scale_x = available_size.x / image_size.x;
                            let scale_y = available_size.y / image_size.y;
                            let scale = scale_x.min(scale_y).min(1.0); // 最大でも1.0倍まで
                            
                            let display_size = image_size * scale;
                            
                            ui.image((texture.id(), display_size));
                        } else {
                            // フォールバック: レンダリングできない場合はプレースホルダーを表示
                            let (rect, _) = ui.allocate_exact_size(
                                egui::Vec2::new(600.0, 800.0),
                                egui::Sense::hover()
                            );
                            ui.painter().rect_stroke(
                                rect,
                                egui::Rounding::same(5.0),
                                egui::Stroke::new(2.0, egui::Color32::GRAY)
                            );
                            ui.painter().text(
                                rect.center(),
                                egui::Align2::CENTER_CENTER,
                                format!("Page {} Content\n(Rendering failed or loading...)", self.current_page + 1),
                                egui::FontId::default(),
                                egui::Color32::GRAY
                            );
                        }
                    });
                });
            });
        });
    }

}