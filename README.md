# PDF Reader

A modern PDF viewer application built with Rust and Dioxus, featuring AI-powered search capabilities using Gemini.

## Key Features

- **Dynamic File Selection**: Open and close PDF files through an intuitive file dialog interface
- **High-Quality PDF Display**: Renders PDF pages at 1000x1400 resolution for balanced quality and performance
- **Continuous Scroll**: View all PDF pages in a seamless continuous scroll interface
- **AI-Powered Search**: Integrated multi-provider AI API (Gemini, ChatGPT, Claude) for intelligent content analysis and term explanations
- **Flashcard System**: Automatic vocabulary flashcard creation from search terms with persistent storage and popup interface
- **Reading Bookmark System**: Page-level bookmark functionality to track reading progress across sessions
- **Position Marker System**: Precise in-page position markers with automatic scrolling navigation
- **PDF Page Rotation**: Individual page rotation with persistent state across sessions
- **Recent Files History**: Quick access to recently opened PDF files (up to 10 files)
- **API Key Management**: Automatic saving and restoration of AI provider API keys
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


### AI API Keys
To use the AI search functionality, obtain one or more of the following API keys:

**Gemini API (Google):**
https://makersuite.google.com/app/apikey

**ChatGPT API (OpenAI):**
https://platform.openai.com/api-keys

**Claude API (Anthropic):**
https://console.anthropic.com/

## Installation & Usage

### Building from Source

#### macOS

```bash
# Clone the repository
git clone <repository-url>
cd rust-pdf-viewer

# Create lib directory for PDFium library
mkdir -p lib

# Download PDFium library for Intel Macs (x64)
curl -L -o pdfium-mac-x64.tgz \
  "https://github.com/bblanchon/pdfium-binaries/releases/latest/download/pdfium-mac-x64.tgz" && \
  tar -xzf pdfium-mac-x64.tgz -C lib --strip-components=1 && \
  rm pdfium-mac-x64.tgz

# For Apple Silicon Macs (ARM64), use this instead:
# curl -L -o pdfium-mac-arm64.tgz \
#   "https://github.com/bblanchon/pdfium-binaries/releases/latest/download/pdfium-mac-arm64.tgz" && \
#   tar -xzf pdfium-mac-arm64.tgz -C lib --strip-components=1 && \
#   rm pdfium-mac-arm64.tgz

# Verify the library is correctly placed
ls -la lib/libpdfium.dylib

# Build and run the project
cargo run

# To build a macOS app bundle (.app file)
cargo install cargo-bundle
cargo bundle --release

# The app bundle will be created at:
# target/release/bundle/osx/PDF Viewer.app
```

#### Linux

```bash
# Clone the repository
git clone <repository-url>
cd rust-pdf-viewer

# Create lib directory for PDFium library
mkdir -p lib

# Download PDFium library for x64 systems
curl -L -o pdfium-linux-x64.tgz \
  "https://github.com/bblanchon/pdfium-binaries/releases/latest/download/pdfium-linux-x64.tgz" && \
  tar -xzf pdfium-linux-x64.tgz -C lib --strip-components=1 && \
  rm pdfium-linux-x64.tgz

# For ARM64 systems, use this instead:
# curl -L -o pdfium-linux-arm64.tgz \
#   "https://github.com/bblanchon/pdfium-binaries/releases/latest/download/pdfium-linux-arm64.tgz" && \
#   tar -xzf pdfium-linux-arm64.tgz -C lib --strip-components=1 && \
#   rm pdfium-linux-arm64.tgz

# Verify the library is correctly placed
ls -la lib/libpdfium.so

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
   - Individual page rotation controls (90° increments) with persistence
4. **Reading Progress Management**:
   - **Bookmarks**: Use "🔖 ブックマーク" to set page-level reading bookmarks
   - **Position Markers**: Enable "📍 マーカーモード" to place precise location markers within pages
   - **Automatic Navigation**: Click markers in "📋 マーカー一覧" for automatic page scrolling
   - **Recent Files**: Access recently opened files via "📋 最近のファイル"
5. **AI Search functionality**:
   - Select AI provider: Gemini, ChatGPT, or Claude
   - Enter your API key once - it will be automatically saved and restored in future sessions
   - Type your search query to ask about terms or concepts
   - Click "検索" (Search) to get AI-powered explanations
   - Searched terms are automatically saved to your flashcard collection
6. **File management**:
   - Use "❌ 閉じる" (Close) button to close the current PDF
   - Open different files without restarting the application

## Application Layout

- **Header Bar**: File control buttons and application title
  - "🔖 ブックマーク" (Bookmarks) button for reading progress management
  - "📍 マーカーモード" (Marker Mode) toggle for position marking
  - "📋 マーカー一覧" (Marker List) for viewing and navigating to saved markers
  - "📋 最近のファイル" (Recent Files) for quick file access
  - "📁 PDFを開く" (Open PDF) button for file selection
  - "❌ 閉じる" (Close) button to close current PDF
- **Status Bar**: Loading progress and PDF information (when PDF is loaded)
- **Main Content Area**:
  - **Left Panel**: PDF display area optimized for vertical documents with continuous scroll
  - **Right Panel**: AI search interface
    - AI provider selection (Gemini, ChatGPT, Claude)
    - API key input (password field) with automatic save/restore
    - Search query input for term explanations
    - Search button with loading indicator
    - Results area with clean text formatting
    - Flashcard button to view saved search terms
- **Welcome Screen**: Displayed when no PDF is loaded with usage instructions

## Data Storage

### Flashcard Data
The application automatically saves search terms and their AI-generated definitions as flashcards for future reference.

**Storage Location:**
- **macOS**: `~/.config/pdf-viewer/`
- **Linux**: `~/.config/pdf-viewer/`  
- **Windows**: `%USERPROFILE%\.config\pdf-viewer\`

**Data Files:**
- `flashcards.json` - Saved search terms and definitions
- `recent_files.json` - Recently opened PDF files (max 10)
- `api_keys.json` - Saved API keys for AI providers
- `bookmarks.json` - Reading bookmarks for each PDF file
- `position_markers.json` - Precise position markers within PDF pages
- `page_rotations.json` - Individual page rotation states

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
- **PDF Rendering**: `render_pdf_page_with_text()` function handles high-resolution page rendering with text extraction
- **File Management**: Dynamic PDF loading/closing with native file dialog integration and recent files history
- **Reading Progress**: Bookmark and position marker systems with JSON persistence and automatic navigation
- **Page Rotation**: Individual page rotation controls with state persistence
- **AI Search**: `search_with_ai()` async function for multi-provider API integration
- **Text Processing**: `clean_markdown_text()` for formatting search results
- **UI Components**: Reactive Dioxus components with modern state management and responsive layout
- **JavaScript Integration**: `eval()` function for automatic page scrolling and DOM manipulation

### Dependencies
All dependencies are optimized for the Dioxus ecosystem, removing legacy egui dependencies for a cleaner, more maintainable codebase.

## License

This project is licensed under the terms specified in the LICENSE file.