# PDF Reader

A modern PDF viewer application built with Rust and Dioxus, featuring AI-powered search capabilities using Gemini.

## Key Features

- **Dynamic File Selection**: Open and close PDF files through an intuitive file dialog interface
- **High-Quality PDF Display**: Renders PDF pages at 1000x1400 resolution for balanced quality and performance
- **Continuous Scroll**: View all PDF pages in a seamless continuous scroll interface
- **AI-Powered Search**: Integrated Gemini 2.5 Flash API for intelligent content analysis and term explanations
- **Flashcard System**: Automatic vocabulary flashcard creation from search terms with persistent storage and popup interface
- **Modern UI**: Clean, responsive interface built with Dioxus framework
- **Optimized Layout**: Specially designed for vertical PDF documents with horizontal AI search panel
- **Smart Caching**: Intelligent page caching system with optimized loading to prevent unnecessary re-rendering
- **Cross-platform**: Desktop application with native performance

## Technology Stack

- **Dioxus** - Modern reactive UI framework for Rust
- **pdfium-render** - PDF rendering using Google's PDFium library
- **rfd** - Native file dialog integration for file selection
- **Reqwest** - HTTP client for AI search functionality
- **Tokio** - Async runtime for non-blocking operations
- **Serde** - JSON serialization for API communication
- **Base64** - Image encoding for web display

## Prerequisites

### Rust and Cargo
This application requires Rust and Cargo. Install them from the official website:

**Windows, macOS, and Linux:**
```bash
# Install Rust via rustup (recommended)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# On Windows, download and run rustup-init.exe from:
# https://rustup.rs/

# Restart your terminal or run:
source ~/.cargo/env

# Verify installation
cargo --version
rustc --version
```

**Alternative Installation Methods:**

**macOS (via Homebrew):**
```bash
brew install rust
```

**Ubuntu/Debian:**
```bash
sudo apt update
sudo apt install rustc cargo
```

**CentOS/RHEL/Fedora:**
```bash
# CentOS/RHEL
sudo yum install rust cargo

# Fedora
sudo dnf install rust cargo
```

### PDFium Library

#### macOS
The application requires PDFium native libraries. The required library for macOS (`libpdfium.dylib`) is included in the `lib/` directory.

#### Linux
For Linux systems, you'll need to download the appropriate PDFium library:
```bash
# For x64 systems
curl -L -o pdfium-linux-x64.tgz \
  "https://github.com/bblanchon/pdfium-binaries/releases/latest/download/pdfium-linux-x64.tgz" && \
  tar -xzf pdfium-linux-x64.tgz -C lib --strip-components=1 && \
  rm pdfium-linux-x64.tgz

# For ARM64 systems
curl -L -o pdfium-linux-arm64.tgz \
  "https://github.com/bblanchon/pdfium-binaries/releases/latest/download/pdfium-linux-arm64.tgz" && \
  tar -xzf pdfium-linux-arm64.tgz -C lib --strip-components=1 && \
  rm pdfium-linux-arm64.tgz
```

### Gemini API Key
To use the AI search functionality, obtain a Gemini API key from Google AI Studio:
https://makersuite.google.com/app/apikey

## Installation & Usage

### Building from Source
```bash
# Clone the repository
git clone <repository-url>
cd rust-pdf-viewer

# Setup PDFium library (macOS)
mkdir -p lib

# Download PDFium library for macOS (Intel/x64)
curl -L -o pdfium-mac-x64.tgz \
  "https://github.com/bblanchon/pdfium-binaries/releases/latest/download/pdfium-mac-x64.tgz" && \
  tar -xzf pdfium-mac-x64.tgz -C lib --strip-components=1 && \
  rm pdfium-mac-x64.tgz

# For Apple Silicon Macs, use this instead:
# curl -L -o pdfium-mac-arm64.tgz \
#   "https://github.com/bblanchon/pdfium-binaries/releases/latest/download/pdfium-mac-arm64.tgz" && \
#   tar -xzf pdfium-mac-arm64.tgz -C lib --strip-components=1 && \
#   rm pdfium-mac-arm64.tgz

# Verify the library is correctly placed
ls -la lib/libpdfium.dylib

# Build and run the project
cargo run
```

### Using the Application

1. **Launch the application** - no arguments required
2. **Open a PDF file**:
   - Click the "📁 PDFを開く" (Open PDF) button in the header
   - Select a PDF file from the file dialog
3. **View PDF content**:
   - All pages are displayed in a continuous scroll format
   - Loading progress is shown at the top during initial page rendering
4. **AI Search functionality**:
   - Enter your Gemini API key in the right panel
   - Type your search query to ask about terms or concepts
   - Click "検索" (Search) to get AI-powered explanations
5. **File management**:
   - Use "❌ 閉じる" (Close) button to close the current PDF
   - Open different files without restarting the application

## Application Layout

- **Header Bar**: File control buttons and application title
  - "📁 PDFを開く" (Open PDF) button for file selection
  - "❌ 閉じる" (Close) button to close current PDF
- **Status Bar**: Loading progress and PDF information (when PDF is loaded)
- **Main Content Area**:
  - **Left Panel**: PDF display area optimized for vertical documents with continuous scroll
  - **Right Panel**: AI search interface
    - API key input (password field)
    - Search query input for term explanations
    - Search button with loading indicator
    - Results area with clean text formatting
- **Welcome Screen**: Displayed when no PDF is loaded with usage instructions

## Data Storage

### Flashcard Data
The application automatically saves search terms and their AI-generated definitions as flashcards for future reference.

**Storage Location:**
- **macOS**: `~/.config/pdf-viewer/flashcards.json`
- **Linux**: `~/.config/pdf-viewer/flashcards.json`  
- **Windows**: `%USERPROFILE%\.config\pdf-viewer\flashcards.json`

**Data Format:**
Flashcards are stored in JSON format with the following structure:
```json
[
  {
    "id": "1734493200000-machine-lea",
    "term": "machine learning",
    "definition": "AI-generated explanation of the term...",
    "created_at": "2024-12-18 12:00:00 UTC"
  }
]
```

**Features:**
- Automatic saving when AI search is performed
- Duplicate term detection (updates existing definitions)
- Persistent storage across application sessions
- Popup interface for browsing saved flashcards
- Individual flashcard deletion capability

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
- Smart page-level caching system using HashMap with duplicate loading prevention
- Optimized parallel page rendering with CPU-aware batch processing
- Memory-efficient image handling with color space optimization
- Responsive UI updates with non-blocking async operations
- Single-load policy to prevent unnecessary re-rendering of same PDF files

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
- **PDF Rendering**: `render_pdf_page_optimized()` function handles high-resolution page rendering with optimizations
- **File Management**: Dynamic PDF loading/closing with native file dialog integration
- **AI Search**: `search_with_gemini()` async function for API integration
- **Text Processing**: `clean_markdown_text()` for formatting search results
- **UI Components**: Reactive Dioxus components with modern state management and responsive layout

### Dependencies
All dependencies are optimized for the Dioxus ecosystem, removing legacy egui dependencies for a cleaner, more maintainable codebase.

## License

This project is licensed under the terms specified in the LICENSE file.