# Rusic

Rusic is a modern, lightweight music player application built with Rust and the Dioxus framework. It provides a clean and responsive interface for managing and enjoying your local music collection.

![Rusic Interface](https://github.com/user-attachments/assets/8366b1ea-021f-4631-a97b-5ed6e5bf1562)

## Overview

Rusic allows you to scan your local directories for audio files, automatically organizing them into a browsable library. You can navigate by artists, albums, or explore your custom playlists. The application is built for performance and desktop integration, utilizing the power of Rust (well most of it gets demolished by webview .d).

## Features

- **Library Management**: Automatically scans your music folder to populate your library with artist and album metadata.
- **Playback Control**: Full suite of media controls including play, pause, skip, volume, and seeking.
- **Fullscreen Player**: An immersive mode that focuses on album artwork and playback details.
- **Theming**: Includes dynamic theming support to customize the visual appearance.
- **Native Integration**: Integrates with system media controls and "Now Playing" displays (only macOS for now).

## Installation

### 🚀 Quick Install (Recommended)

**One-Command Install** - works on any Linux distro:

```bash
git clone https://github.com/temidaradev/rusic
cd rusic
chmod +x install.sh
./install.sh
```

Or with Make:

```bash
git clone https://github.com/temidaradev/rusic
cd rusic
make install
```

This will:

- Build the application
- Install it to `~/.local/bin`
- Create a desktop icon automatically
- Add Rusic to your application menu

**Alternative: Download AppImage** from [Releases](https://github.com/temidaradev/rusic/releases)

```bash
chmod +x Rusic-x86_64.AppImage
./Rusic-x86_64.AppImage
```

### 📦 Other Installation Methods

<details>
<summary><b>NixOS / Nix</b></summary>

```bash
nix run github:temidaradev/rusic
```

Or add to your flake:

```nix
inputs.rusic.url = "github:temidaradev/rusic";
# Then use: inputs.rusic.packages.${system}.default
```

</details>

<details>
<summary><b>Build from Source</b></summary>

**One-command build** (auto-installs dependencies):

```bash
git clone https://github.com/temidaradev/rusic
cd rusic
chmod +x build.sh
./build.sh
```

The script automatically detects your distro (Ubuntu, Arch, Fedora, etc.) and installs dependencies.

**Manual build** (if you already have dependencies):

```bash
./build.sh --skip-deps
```

**Binary location:** `target/dx/rusic/release/linux/app/rusic`

</details>

_Note: If the application fails to start on Ubuntu 22.04+, install libfuse2: `sudo apt install libfuse2`_

## Development

```bash
# Clone
git clone https://github.com/temidaradev/rusic
cd rusic

# NixOS: Enter dev shell
nix develop

# Install npm deps
npm install

# Run in dev mode
dx serve
```

## Cache

Rusic stores its local database, configuration files, and cached album artwork in your system's cache directory (typically `~/.cache/rusic`).

## Built With

- **Dioxus**: Cross-platform UI framework
- **Rodio**: Audio playback
- **Lofty**: Audio metadata parsing
- **TailwindCSS**: Styling
