![GitHub last commit](https://img.shields.io/github/last-commit/Xylobyte/transcendia)&nbsp;
![GitHub Issues or Pull Requests](https://img.shields.io/github/issues/Xylobyte/transcendia)&nbsp;
![GitHub License](https://img.shields.io/github/license/Xylobyte/transcendia)&nbsp;
![GitHub Release](https://img.shields.io/github/v/release/Xylobyte/transcendia)

# Transcendia

> **Real-time Screen Translation App**

Transcendia is a powerful desktop application that provides real-time translation of any selected region on your screen.
Perfect for translating games, videos, documents, or any content displayed on your monitor.

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
- **Always-on-Top Overlay**: Translated text appears as a floating overlay window

### System Integration

- **System Tray Integration**: Minimize to system tray for background operation
- **Global Keyboard Shortcuts**: Quick access with Ctrl+X to finish region selection
- **macOS Permissions Handling**: Automatic screen recording permission requests
- **Cross-Platform Ready**: Built with Tauri for Windows and Linux compatibility (coming soon)

### Performance

- **Optimized OCR Engine**: Uses ONNX Runtime for fast and accurate text recognition
- **Minimal Resource Usage**: Efficient background processing
- **Smart Text Detection**: Only processes changed text to reduce API calls

The app uses **~80 MB of RAM** and about **80% of a single CPU core**, so you can easily run it while playing games.

I'm working on even more optimizations for the next releases!

## 🚀 Installation

### macOS (Ready Now)

1. Go to [Releases](https://github.com/YOUR_USERNAME/transcendia/releases)
2. Download the latest `Transcendia-v1.0.0-beta.1.dmg` file
3. Open the DMG and drag Transcendia to your Applications folder
4. Launch Transcendia from Applications

### Windows & Linux

**Coming Soon!** Windows and Linux versions are almost ready and will be available in the next release.

## 🛠️ Building from Source

### Prerequisites

- **Rust** (latest stable) - [Install Rust](https://rustup.rs/)
- **Node.js** (v18 or higher) - [Install Node.js](https://nodejs.org/)
- **npm**, **pnpm** or **yarn**

### Build Steps

1. **Clone the repository**
   ```bash
   git clone https://github.com/YOUR_USERNAME/transcendia.git
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

## Usage

1. **Launch Transcendia** from your Applications folder or system tray
2. **Configure Settings** - Choose your target language and monitor
3. **Select Region** - Click "Select region" and drag to choose the area to translate
4. **Customize Appearance** - Adjust text size, color, position, and background settings
5. **Start Translating** - The overlay will show real-time translations of detected text

## 🔧 Configuration Options

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

## 🔒 Privacy & Security

- **Secure Translation**: Uses Google Translate API with secure HTTPS connections
- **Screen Permissions**: Only captures the specific region you select
- **Local OCR**: Text recognition is performed locally using optimized AI models

## 🤝 Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

1. Fork the project
2. Create your feature branch (`git checkout -b feature/AmazingFeature`)
3. Commit your changes (`git commit -m 'Add some AmazingFeature'`)
4. Push to the branch (`git push origin feature/AmazingFeature`)
5. Open a Pull Request

Or open an issue if you have a bug or need a new feature !

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE.txt) file for details.

---

**Made
using [Rust](https://github.com/rust-lang/rust), [Tauri](https://github.com/tauri-apps/tauri), [Ocrs](https://github.com/robertknight/ocrs)
and [Vue.js](https://github.com/vuejs)**
