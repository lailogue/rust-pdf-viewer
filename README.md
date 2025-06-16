# PDF Reader

A Rust-based PDF viewer application built with egui and pdfium-render.

## Features

- **PDF Loading**: Load and display PDF documents from file paths
- **Page Navigation**: Navigate through PDF pages with Previous/Next buttons
- **PDF Rendering**: Real-time rendering of PDF pages as images with proper scaling
- **Caching**: Intelligent page caching for improved performance
- **Error Handling**: Comprehensive error reporting for PDF loading and rendering issues
- **Cross-platform GUI**: Built with egui for consistent cross-platform experience

## Dependencies

- `pdfium-render` - PDF rendering capabilities using Google's PDFium library
- `eframe/egui` - Modern immediate mode GUI framework
- `anyhow` - Error handling
- `image` - Image processing support

## Prerequisites

### macOS

The application requires the PDFium native library. For macOS, the required library (`libpdfium.dylib`) is included in the project.

## Usage

```bash
# Build the project
cargo build

# Run with a PDF file
DYLD_LIBRARY_PATH=./lib:$DYLD_LIBRARY_PATH ./target/debug/pdf_reader <path_to_pdf_file>

# Example
DYLD_LIBRARY_PATH=./lib:$DYLD_LIBRARY_PATH ./target/debug/pdf_reader test.pdf
```

## Implementation Details

### Core Components

- **PdfViewerApp**: Main application struct containing PDF state and UI logic
- **render_page()**: Renders PDF pages to textures with configurable scaling
- **Page Caching**: HashMap-based caching system for rendered page textures
- **Error Handling**: Graceful fallback for rendering failures

### PDF Rendering Pipeline

1. Load PDF document using pdfium-render
2. Extract page information (count, metadata)
3. Render individual pages to bitmaps on demand
4. Convert bitmaps to egui textures with RGBA format conversion
5. Cache textures for performance optimization
6. Display with automatic scaling to fit available screen space

### UI Features

- Page counter display
- Previous/Next navigation buttons
- PDF file information display
- Error message display for loading/rendering failures
- Responsive layout with scroll support

## Architecture

The application follows a clean separation of concerns:
- PDF loading and document management
- Page rendering and image conversion
- UI state management and display
- Error handling and user feedback

Built with modern Rust patterns including Result types for error handling and Option types for state management.