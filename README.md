[![GitHub last commit](https://img.shields.io/github/last-commit/Xylobyte/transcendia)](https://github.com/Xylobyte/transcendia/commits/main)
[![GitHub Issues](https://img.shields.io/github/issues/Xylobyte/transcendia)](https://github.com/Xylobyte/transcendia/issues)
[![GitHub License](https://img.shields.io/github/license/Xylobyte/transcendia)](LICENSE.txt)
[![GitHub Release](https://img.shields.io/github/v/release/Xylobyte/transcendia?include_prereleases)](https://github.com/Xylobyte/transcendia/releases)

# Transcendia

> **Real-time Screen Translation for Games, Videos, and More**

Transcendia is a lightweight, privacy-focused desktop application that provides instant translation of any text visible
on your screen. Simply select a region, and watch as translations appear in real-time through an always-on-top overlay
window.

**Perfect for:**

- Gaming
- Streaming and video content
- Language learning
- Accessibility support

![Transcendia Demo](/screenshots/opus-translated-in-french.jpeg)

## Features

### Core Functionality

- **Real-time OCR Translation**: Automatically detects and translates text from any screen region
- **Multi-Monitor Support**: Works seamlessly across multiple monitors
- **Customizable Capture Regions**: Select any rectangular area of your screen to monitor
- **Configurable Intervals**: Set capture frequency from 1-60 seconds

### Customization Options

- **Text Styling**: Customize font size, color, and alignment (9 position options)
- **Background Effects**: Optional blur background with customizable colors
- **Multi-Language Support**: Translate to 17+ languages including major European languages
- **Always visible overlay**: translated text appears as a floating overlay window and clicks can pass through it

### Performance

- **Optimized OCR Engine**: Uses ONNX Runtime for fast and accurate text recognition
- **Minimal Resource Usage**: Efficient background processing
- **Smart Text Detection**: Only processes changed text to reduce API calls

The app typically uses ~80 MB of RAM and up to ~80% of a single CPU core, so it runs smoothly even while gaming.

I'm working on even more optimizations for the next releases!

## Installation

### macOS (Available Now)

**Option 1: Download Pre-built Binary**

1. Visit the [Releases](https://github.com/Xylobyte/transcendia/releases) page
2. Download `Transcendia_1.0.0-beta.1_aarch64.dmg`
3. Open the DMG file and drag Transcendia to your Applications folder
4. Launch from Applications (you may need to allow the app in System Preferences > Security & Privacy)

**System Requirements:**

- Apple Silicon (M1/M2/M3/M4) with macOS 10.13 or newer
- ~30MB free disk space

### Windows & Linux

**Coming Soon!** Windows and Linux versions are almost ready and will be available in an upcoming release.

## Building from Source

### Prerequisites

- **Rust** (latest stable) - [Install Rust](https://rustup.rs/)
- **Node.js** (v18 or higher) - [Install Node.js](https://nodejs.org/)
- **npm**, **pnpm** or **yarn**

### Build Steps

1. **Clone the repository**
   ```bash
   git clone https://github.com/Xylobyte/transcendia.git
   cd transcendia
   ```

2. **Install dependencies**
   ```bash
   npm install
   ```

3. **Development mode**
   ```bash
   npm run tauri dev
   ```

4. **Build for production**
   ```bash
   npm run tauri build
   ```

The built application will be available in `src-tauri/target/release/bundle/`.

## Configuration Options

| Setting              | Description                         | Options                                             |
|----------------------|-------------------------------------|-----------------------------------------------------|
| **Target Language**  | Language to translate text into     | 17+ languages (primarily Latin alphabet)            |
| **Monitor**          | Which monitor to capture from       | All available monitors                              |
| **Screen Region**    | Area of screen to monitor           | Custom rectangular selection                        |
| **Text Color**       | Color of translated text            | RGB color picker                                    |
| **Text Size**        | Font size of overlay text           | 1-250 pixels                                        |
| **Text Align**       | Position of text in overlay         | 9 positions (Top/Center/Bottom + Left/Center/Right) |
| **Background Blur**  | Blur effect behind text             | On/Off (requires restart)                           |
| **Background Color** | Color of text background            | RGB color picker with transparency                  |
| **Capture Interval** | How often to check for text changes | 1-60 seconds                                        |

## Privacy & Security

- **Secure Translation**: Uses Google Translate API with secure HTTPS connections
- **Screen Permissions**: Only captures the specific region you select
- **Local OCR**: Text recognition is performed locally using optimized AI models
- **No Data Storage**: No screenshots or text content is permanently stored
- **Open Source**: Full source code available here 😄

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

**Getting Started:**

1. Fork the project
2. Create your feature branch (`git checkout -b feature/AmazingFeature`)
3. Commit your changes (`git commit -m 'Add some AmazingFeature'`)
4. Push to the branch (`git push origin feature/AmazingFeature`)
5. Open a Pull Request

**Areas where I need help:**

- Icon design (current placeholder needs improvement)
- Windows and Linux platform testing
- Translation accuracy improvements
- Performance optimizations
- Documentation improvements

## Changelog

- [Changelog](CHANGELOG.md) - Release history and changes

## License

This project is licensed under the GNU Affero General Public License v3.0 - see the [LICENSE](LICENSE.txt) file for
details.

---

**Made
using [Rust](https://github.com/rust-lang/rust), [Tauri](https://github.com/tauri-apps/tauri), [Ocrs](https://github.com/robertknight/ocrs)
and [Vue.js](https://github.com/vuejs)**
