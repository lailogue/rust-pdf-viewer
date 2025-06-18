use anyhow::Result;
use std::path::PathBuf;
use pdfium_render::prelude::Pdfium;

pub fn get_pdfium_library_path() -> Result<PathBuf> {
    // アプリケーションバンドル内のパスを最初に試す（.appファイル用）
    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(app_dir) = exe_path.parent() {
            // .app/Contents/MacOS/ から .app/Contents/Resources/lib/ へ
            let resources_lib_path = app_dir
                .parent()
                .unwrap_or(app_dir)
                .join("Resources")
                .join("lib")
                .join("libpdfium.dylib");
            
            if resources_lib_path.exists() {
                return Ok(resources_lib_path);
            }
            
            // .app/Contents/MacOS/ から .app/Contents/Frameworks/ へ（代替パス）
            let frameworks_path = app_dir
                .parent()
                .unwrap_or(app_dir)
                .join("Frameworks")
                .join("libpdfium.dylib");
            
            if frameworks_path.exists() {
                return Ok(frameworks_path);
            }
            
            // .app/Contents/MacOS/ から .app/Contents/lib/ へ（旧パス）
            let lib_path = app_dir
                .parent()
                .unwrap_or(app_dir)
                .join("lib")
                .join("libpdfium.dylib");
            
            if lib_path.exists() {
                return Ok(lib_path);
            }
        }
    }
    
    // 開発環境用: プロジェクトディレクトリのlibフォルダ
    let lib_path = PathBuf::from("lib").join(if cfg!(target_os = "macos") {
        "libpdfium.dylib"
    } else if cfg!(target_os = "windows") {
        "pdfium.dll"
    } else {
        "libpdfium.so"
    });
    
    if lib_path.exists() {
        Ok(lib_path)
    } else {
        Err(anyhow::anyhow!(
            "PDFiumライブラリが見つかりません。libディレクトリにPDFiumライブラリを配置してください。"
        ))
    }
}

pub fn get_pdf_info(pdf_path: &str) -> Result<(usize, String)> {
    let library_path = get_pdfium_library_path()?;
    let pdfium = Pdfium::new(
        Pdfium::bind_to_library(library_path)?
    );
    
    let document = pdfium.load_pdf_from_file(pdf_path, None)?;
    let page_count = document.pages().len();
    let title = "Unknown PDF".to_string(); // Metadata API needs investigation
    
    Ok((page_count.into(), title))
}