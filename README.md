# Cat Paw 🐾

[English](README.md) | [中文](README_ZH.md)

**Cat Paw** is a fun, lightweight desktop overlay application written in Rust using the [Bevy](https://bevyengine.org/)
game engine. It replaces your mouse cursor with an animated cat's paw that follows your movements and reacts to your
clicks!

## Features

- **Mouse Following**: The paw naturally follows your mouse cursor across the screen.
- **Interactive Animations**:
    - **Left Click**: The paw clenches (grabs).
    - **Right Click**: The paw spreads its fingers (opens wide).
- **Global Overlay**: Runs in a transparent, always-on-top window that overlays your entire screen.
- **Shortcuts**:
    - **Toggle Visibility**: Press **Left Click + Right Click** simultaneously (short press) to toggle the paw on/off.
      When the paw is hidden, the system cursor returns.
    - **Exit**: Press and hold **Left Click + Right Click** for **2 seconds** to close the application.
- **Cross-Platform**: Designed for macOS and Windows.

## Installation & Building

### Prerequisites

You need to have **Rust** and **Cargo** installed. If you don't have them, get them
from [rustup.rs](https://rustup.rs/).

### Development

To run the application in development mode:

```bash
cargo run
```

### Build for macOS

To create a standalone `.app` bundle for macOS:

1. Open a terminal in the project root.
2. Run the build script:
   ```bash
   ./build_macos.sh
   ```
3. The application will be built at `target/release/bundle/CatPaw.app`.
4. You can open it with `open target/release/bundle/CatPaw.app` or drag it to your Applications folder.

### Build for Windows

To create a distribution folder for Windows:

1. Open a Command Prompt or PowerShell in the project root.
2. Run the build script:
   ```cmd
   build_windows.bat
   ```
3. The application will be located at `target\release\distribution\CatPaw`.
4. Run `CatPaw.exe` inside that folder.

## Usage

Once the application is running:

- **Move your mouse**: The paw follows.
- **Left Click**: Watch the paw clench.
- **Right Click**: Watch the paw open.
- **Hide/Show**: Press Left+Right buttons together quickly.
- **Quit**: Press and hold Left+Right buttons together for 2 seconds.

## Known Issues

- **Vulkan Window Transparency**: In some environments using the Vulkan graphics API, window transparency may fail,
  resulting in a black background instead of a transparent overlay. This is often related to GPU driver support or
  system compositor settings.

## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.
