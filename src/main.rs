use anyhow::{Context, Result};
use pdf::file::File as PdfFile;
use pdf::object::*;
use std::env;

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <pdf_file>", args[0]);
        std::process::exit(1);
    }

    let pdf_path = &args[1];
    read_pdf(pdf_path)?;
    Ok(())
}

fn read_pdf(path: &str) -> Result<()> {
    let file = PdfFile::open(path)
        .with_context(|| format!("Failed to open PDF file: {}", path))?;

    println!("PDF Information:");
    println!("Pages: {}", file.num_pages());
    
    if let Ok(info) = file.trailer.info_dict {
        if let Some(title) = info.title {
            println!("Title: {}", title.to_string_lossy());
        }
        if let Some(author) = info.author {
            println!("Author: {}", author.to_string_lossy());
        }
        if let Some(subject) = info.subject {
            println!("Subject: {}", subject.to_string_lossy());
        }
    }

    println!("\nPage Contents:");
    for page_num in 1..=file.num_pages() {
        println!("\n--- Page {} ---", page_num);
        match extract_page_text(&file, page_num) {
            Ok(text) => {
                if text.trim().is_empty() {
                    println!("(No text content found)");
                } else {
                    println!("{}", text);
                }
            }
            Err(e) => println!("Error reading page {}: {}", page_num, e),
        }
    }

    Ok(())
}

fn extract_page_text(file: &PdfFile, page_num: u32) -> Result<String> {
    let page = file.get_page(page_num)
        .with_context(|| format!("Failed to get page {}", page_num))?;
    
    let mut text = String::new();
    
    if let Some(contents) = &page.contents {
        for content in contents {
            if let Ok(content_stream) = file.get_object(content.get_inner()) {
                if let Object::Stream(stream) = content_stream {
                    if let Ok(data) = stream.data() {
                        let content_str = String::from_utf8_lossy(&data);
                        text.push_str(&extract_text_from_stream(&content_str));
                    }
                }
            }
        }
    }
    
    Ok(text)
}

fn extract_text_from_stream(stream_content: &str) -> String {
    let mut text = String::new();
    let mut in_text_object = false;
    
    for line in stream_content.lines() {
        let line = line.trim();
        
        if line == "BT" {
            in_text_object = true;
            continue;
        }
        
        if line == "ET" {
            in_text_object = false;
            continue;
        }
        
        if in_text_object && line.contains("Tj") {
            if let Some(start) = line.find('(') {
                if let Some(end) = line.rfind(')') {
                    if start < end {
                        let extracted = &line[start + 1..end];
                        text.push_str(extracted);
                        text.push(' ');
                    }
                }
            }
        }
        
        if in_text_object && line.contains("TJ") {
            text.push_str(&extract_from_array(line));
        }
    }
    
    text
}

fn extract_from_array(line: &str) -> String {
    let mut text = String::new();
    let mut in_string = false;
    let mut current_string = String::new();
    
    for ch in line.chars() {
        match ch {
            '(' => {
                in_string = true;
                current_string.clear();
            }
            ')' => {
                if in_string {
                    text.push_str(&current_string);
                    text.push(' ');
                    in_string = false;
                }
            }
            _ => {
                if in_string {
                    current_string.push(ch);
                }
            }
        }
    }
    
    text
}