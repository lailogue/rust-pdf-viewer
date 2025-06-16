# PDF Reader

A PDF viewer application built with Rust, egui, and the pdfium library, featuring AI-powered search capabilities using Gemini.

## Key Features

- **PDF Loading**: Load and display PDF documents from file paths
- **Page Navigation**: Navigate PDF pages with previous/next buttons
- **PDF Rendering**: High-quality real-time rendering of PDF pages as images
- **Caching**: Intelligent caching of page textures for improved performance
- **Error Handling**: Comprehensive error reporting for PDF loading and rendering issues
- **Japanese Font Support**: Japanese text display using macOS system fonts
- **AI Search**: AI-powered search functionality using Gemini 2.5 Flash API
- **Cross-platform GUI**: Consistent cross-platform experience with egui

## Technology Stack

- `pdfium-render` - PDF rendering using Google's PDFium library
- `eframe/egui` - Modern immediate mode GUI framework
- `anyhow` - Error handling
- `reqwest` - HTTP client (for AI search)
- `tokio` - Async runtime
- `serde` - JSON serialization

## Prerequisites

### macOS

The application requires PDFium native libraries. The required library for macOS (`libpdfium.dylib`) is included in the project.

### Gemini API Key

To use the AI search functionality, you need to obtain a Gemini API key from Google AI Studio:
https://makersuite.google.com/app/apikey

## Usage

```bash
# Build the project
cargo build

# Run with a PDF file
DYLD_LIBRARY_PATH=./lib:$DYLD_LIBRARY_PATH ./target/debug/pdf_reader <PDF_FILE_PATH>

# Example
DYLD_LIBRARY_PATH=./lib:$DYLD_LIBRARY_PATH ./target/debug/pdf_reader test.pdf
```

### Using AI Search

1. After launching the application, enter your Gemini API key in the "API Key" field in the left panel
2. Enter your search query in the "Search Query" field
3. Click the "Search" button
4. Search results will be displayed in the bottom section

## Implementation Details

### Core Components

- **PdfViewerApp**: Main application struct containing PDF state and UI logic
- **render_page()**: Renders PDF pages to textures with configurable scaling
- **Page Cache**: HashMap-based caching system for rendered page textures
- **Error Handling**: Graceful fallbacks when rendering fails
- **Japanese Font Support**: Automatic loading of macOS system fonts (Hiragino Kaku Gothic, Noto Sans CJK, etc.)
- **AI Search**: Asynchronous search functionality using Gemini 2.5 Flash API

### PDF Rendering Pipeline

1. Load PDF document using pdfium-render
2. Extract page information (count, metadata)
3. Render individual pages to bitmaps on demand
4. Convert bitmaps to egui textures with RGBA format conversion
5. Cache textures for performance optimization
6. Display with automatic scaling to fit available screen space

### UI Features

- Page counter display
- Previous/next navigation buttons
- PDF file information display
- Error message display for loading/rendering failures
- Responsive layout with scroll support
- AI search panel (API key input, search query input, result display)
- Proper Japanese text rendering

### AI Search System

- **Asynchronous Processing**: Non-blocking async API calls that don't freeze the UI
- **Gemini 2.5 Flash**: Uses the latest Gemini model
- **Error Handling**: Proper handling and display of API errors
- **Real-time Updates**: Real-time display of search results

## Architecture

The application follows clear separation of concerns:
- PDF loading and document management
- Page rendering and image conversion
- UI state management and display
- Error handling and user feedback
- AI search functionality and API communication
- Japanese font support

Built with modern Rust patterns including Result-type error handling and Option-type state management.

## License

This project is licensed under the license file included in the project.