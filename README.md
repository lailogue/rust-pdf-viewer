# PDF Reader

A modern PDF viewer application built with Rust and Dioxus, featuring AI-powered search capabilities using Gemini.

## Key Features

- **High-Quality PDF Display**: Renders PDF pages at 1000x1400 resolution for balanced quality and performance
- **Continuous Scroll**: View all PDF pages in a seamless continuous scroll interface
- **AI-Powered Search**: Integrated Gemini 2.5 Flash API for intelligent content search
- **Modern UI**: Clean, responsive interface built with Dioxus framework
- **Optimized Layout**: Specially designed for vertical PDF documents with horizontal AI search panel
- **Page Caching**: Intelligent caching system for improved performance
- **Cross-platform**: Desktop application with native performance

## Technology Stack

- **Dioxus** - Modern reactive UI framework for Rust
- **pdfium-render** - PDF rendering using Google's PDFium library
- **Reqwest** - HTTP client for AI search functionality
- **Tokio** - Async runtime for non-blocking operations
- **Serde** - JSON serialization for API communication
- **Base64** - Image encoding for web display

## Prerequisites

### macOS
The application requires PDFium native libraries. The required library for macOS (`libpdfium.dylib`) is included in the `lib/` directory.

### Gemini API Key
To use the AI search functionality, obtain a Gemini API key from Google AI Studio:
https://makersuite.google.com/app/apikey

## Installation & Usage

### Building from Source
```bash
# Clone the repository
git clone <repository-url>
cd claude_code

# Build the project
cargo build --release

# Run with a PDF file
cargo run -- <PDF_FILE_PATH>

# Example
cargo run -- test.pdf
```

### Using the Application

1. **Launch the application** with a PDF file as argument
2. **Navigate pages** using the Previous/Next buttons at the top
3. **AI Search**:
   - Enter your Gemini API key in the right panel
   - Type your search query
   - Click "検索" (Search) to get AI-powered explanations
4. **View results** in the expandable search results area

## Application Layout

- **Left Panel**: PDF display area optimized for vertical documents
- **Right Panel**: AI search interface
  - API key input (password field)
  - Search query input
  - Search button with loading indicator
  - Expandable results area with clean text formatting
- **Top Bar**: Page navigation controls and PDF information

## Implementation Highlights

### High-Quality Rendering
- PDF pages rendered at 1000x1400 pixels for balanced quality and performance
- Continuous scroll display for all pages at once
- Automatic color space conversion (BGRA → RGBA)
- PNG encoding with base64 data URLs for web display

### AI Search Integration
- Asynchronous Gemini API calls that don't block the UI
- Markdown formatting removal for clean text display
- Real-time search status indicators
- Comprehensive error handling

### Performance Optimizations
- Page-level caching system using HashMap
- Lazy loading of PDF pages
- Memory-efficient image handling
- Responsive UI updates

### Modern Dioxus Architecture
- Reactive state management with `use_signal`
- Memoized computations with `use_memo`
- Component-based design patterns
- CSS-in-Rust styling approach

## Project Structure

```
├── src/
│   └── main.rs           # Main Dioxus application
├── assets/
│   └── style.css         # UI styling
├── lib/
│   └── libpdfium.dylib   # PDFium library for macOS
├── Cargo.toml            # Dependencies and configuration
└── README.md            # This file
```

## Development

### Code Organization
- **PDF Rendering**: `render_pdf_page()` function handles high-resolution page rendering
- **AI Search**: `search_with_gemini()` async function for API integration
- **Text Processing**: `clean_markdown_text()` for formatting search results
- **UI Components**: Reactive Dioxus components with modern state management

### Dependencies
All dependencies are optimized for the Dioxus ecosystem, removing legacy egui dependencies for a cleaner, more maintainable codebase.

## License

This project is licensed under the terms specified in the LICENSE file.