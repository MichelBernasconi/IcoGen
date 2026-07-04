# ✨ IcoGen Premium

[![Buy Me A Coffee](https://img.shields.io/badge/Buy%20Me%20a%20Coffee-ffdd00?style=for-the-badge&logo=buy-me-a-coffee&logoColor=black)](https://www.buymeacoffee.com/MichelBernasconi)

## About
**IcoGen** is a powerful multi-format icon generator written entirely in **Rust** and powered by a modern, fast, and fluid graphical interface based on **Slint**. 

This tool is designed specifically for software developers, mobile app creators (Android/iOS), and web designers. It allows you to take a single source image (or hundreds of images in batch) and generate all the necessary dimensions for different ecosystems with a single click, automatically applying high-quality filters (Lanczos3) and, optionally, removing unwanted monochromatic backgrounds.

![alt text](image.png)

## 🚀 Key Features

- **Asynchronous Batch Processing:** Don't limit yourself to one icon at a time. Select an entire folder of images, and IcoGen will leverage Rust's background threads to process them all simultaneously, without ever blocking or slowing down the user interface.
- **Smart Background Removal (Chroma Key):** How often do you have a `.jpg` logo with an annoying white square behind it? By activating the "Remove Monochromatic Background" tool, IcoGen will identify the main background color and make it transparent (creating a true Alpha channel). Thanks to the *tolerance slider*, you can easily eliminate those annoying blurs caused by JPEG compression!
- **Native Slint Interface:** None of the heaviness typical of Electron apps. The interface launches in a flash, consumes very little RAM, and offers a "Premium" design (dynamic shadows, rounded corners, integrated terminal).

## 📱 Included Export Profiles

Instead of making you manually enter dozens of different sizes, IcoGen natively integrates profiles for the most popular operating systems:

- **Android (`36x36`, `48x48`, `72x72`, `96x96`, `144x144`, `192x192`):** 
  Automatically generates all the pixel densities required by the Android ecosystem (`ldpi`, `mdpi`, `hdpi`, `xhdpi`, `xxhdpi`, `xxxhdpi`) needed for `ic_launcher` or for publishing on the Google Play Store.
  
- **iOS / Apple (`20x20`, `29x29`, `40x40`, `58x58`, `60x60`, `76x76`, `80x80`, `87x87`, `114x114`, `120x120`, `152x152`, `167x167`, `180x180`, `1024x1024`):**
  The Apple ecosystem is notoriously very demanding when it comes to graphic assets. This profile generates all the countless `@1x`, `@2x`, and `@3x` resolutions required for Spotlight search, Settings, app icons on iPhone/iPad, and the super-resolution needed for App Store Connect.

- **Web Favicons (`16x16`, `32x32`, `48x48`, `192x192`, `512x512`):**
  Perfect for Web development. Creates the micro-icons needed for desktop browser tab bars (`16px` and `32px`), along with the larger sizes required by PWAs (Progressive Web Apps) to allow users to save your website on their Android/iOS smartphone home screens.

- **Custom:** 
  Do you need specific formats (e.g., for a video game)? Choose the custom profile, enter a comma-separated list of dimensions (e.g., `24, 64, 256`), and IcoGen will do exactly what you ask.

## Getting Started

### Prerequisites
- [Rust](https://www.rust-lang.org/tools/install) (Cargo) installed on your system.

### Quick Start
```bash
git clone https://github.com/MichelBernasconi/IcoGen.git
cd IcoGen
cargo run --release
```

## 📦 Using IcoGen as a Library

IcoGen is designed with a clean architecture that separates the UI from the image processing engine. You can use it as a backend library in your own Rust projects (like Web Servers, CLI tools, or data pipelines) without launching the Slint interface.

Add it to your `Cargo.toml`:
```toml
[dependencies]
icogen = { git = "https://github.com/MichelBernasconi/IcoGen.git" }
```

Example usage:
```rust
use icogen::{generate_icons, IcoGenConfig};
use std::path::PathBuf;

fn main() {
    let config = IcoGenConfig {
        input_files: vec![PathBuf::from("logo.png")],
        output_dir: PathBuf::from("output/"),
        format: "png".to_string(),
        profile: "iOS".to_string(),
        custom_sizes: "".to_string(),
        remove_bg: true,
        bg_tolerance: 10,
    };

    // The engine handles the heavy lifting, sending logs back via closure
    generate_icons(config, |log_message| {
        println!("IcoGen Log: {}", log_message);
    });
}
```

## License
Distributed under the MIT License. See `LICENSE` for more information.
