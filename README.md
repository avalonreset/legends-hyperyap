<p align="center">
  <img src="assets/banner.webp" alt="HyperYap - local voice-to-text for Windows" width="100%">
</p>

# HyperYap - Local Voice-to-Text for Windows

[![CI](https://github.com/avalonreset/hyperyap/actions/workflows/ci.yaml/badge.svg)](https://github.com/avalonreset/hyperyap/actions/workflows/ci.yaml)
[![GitHub release](https://img.shields.io/github/v/release/avalonreset/hyperyap)](https://github.com/avalonreset/hyperyap/releases)
[![License: AGPL-3.0](https://img.shields.io/github/license/avalonreset/hyperyap)](LICENSE)
[![Last Commit](https://img.shields.io/github/last-commit/avalonreset/hyperyap)](https://github.com/avalonreset/hyperyap/commits/main)

HyperYap is a local voice-to-text application that bundles speech recognition, a terminal emulator, and hotkey automation into a single zero-config installer for Windows. No cloud, no internet required for transcription, no data collection. Install once, use everywhere.

## Table of Contents

- [What You Get](#what-you-get)
- [Install](#install)
- [Default Hotkeys](#default-hotkeys)
  - [CapsLock Remapping](#capslock-remapping)
  - [Smart Paste](#smart-paste)
- [Requirements](#requirements)
- [How It Works](#how-it-works)
- [Configuration](#configuration)
- [Build from Source](#build-from-source)
- [Contributing](#contributing)
- [Attribution](#attribution)
- [License](#license)

## What You Get

One installer, one tray icon, everything just works:

- **Local speech-to-text** powered by NVIDIA [Parakeet TDT 0.6B v3](https://huggingface.co/nvidia/parakeet-tdt-0.6b-v3). No cloud, no internet after first install.
- **Hotkey remapping** turns CapsLock and mouse side buttons into recording triggers. Mouse Forward becomes Enter.
- **Smart paste** detects terminals and auto-saves clipboard images as PNGs, pasting the file path instead of garbled data.
- **[BenjaminTerm](https://github.com/avalonreset/BenjaminTerm)** included. Hacker-styled WezTerm terminal with smart clipboard, 86 dark themes, and borderless mode.
- **Auto-boot** on login. Preset configs. Zero setup after install.

Everything is preconfigured. You do not need to set up shortcuts, change settings, or configure anything after install.

## Install

### Option 1: Download the installer

Download [hyperyap_1.0.5_x64-setup.exe](https://github.com/avalonreset/hyperyap/releases/latest) from the Releases page and run it. On first launch, HyperYap will:

- Download the NVIDIA Parakeet speech model (~440MB)
- Set up the hotkey engine and register autostart
- Apply all preset settings (toggle-to-talk, F13, English)

### Option 2: One-line PowerShell install

This installs everything in one shot, including BenjaminTerm and the speech model:

```powershell
irm https://raw.githubusercontent.com/avalonreset/hyperyap/main/install.ps1 | iex
```

Or clone and run locally:

```powershell
git clone https://github.com/avalonreset/hyperyap.git
cd hyperyap
powershell -ExecutionPolicy Bypass -File install.ps1
```

The PowerShell installer also removes old MURmure installations if present.

### Upgrading

Run the installer again over an existing install. HyperYap upgrades in-place without deleting your speech model or requiring a re-download. Settings are reset to the recommended defaults on each upgrade.

## Default Hotkeys

| Key | Action |
|-----|--------|
| `F13` / `CapsLock` / Mouse Back | Start/stop recording |
| Mouse Forward | Enter |
| `Ctrl+Shift+Space` | Paste last transcript |
| `Ctrl+Alt+Space` | LLM-assisted recording |
| `Ctrl+Shift+X` | Command mode |
| `Escape` | Cancel recording |

These hotkeys are built in. You can pause and resume them from the tray icon's right-click menu.

### CapsLock Remapping

HyperYap disables CapsLock and repurposes it as a speech-to-text key. Press CapsLock to start recording, press it again to stop. Your transcription is pasted into whatever window is focused. CapsLock is permanently set to off so you never accidentally activate it.

### Smart Paste

HyperYap's hotkey engine is terminal-aware. It detects which application is focused and adapts Ctrl+V behavior accordingly:

**In regular applications** (browsers, editors, chat apps), Ctrl+V works exactly as it normally does. HyperYap does not interfere.

**In supported terminals**, HyperYap intercepts Ctrl+V and adds clipboard image intelligence. If your clipboard contains a screenshot or image, HyperYap waits for Windows to finish publishing the image, saves it as a timestamped PNG in `~/screenshots/`, and replaces the clipboard with the file path before pasting through the terminal paste shortcut. If Windows is still preparing a recent Win+Shift+S or PrintScreen capture, HyperYap keeps waiting instead of falling back to stale text or typing a stray `v`. Text clipboard contents paste normally.

This is especially useful for vibe coding workflows where you screenshot errors, UI mockups, or terminal output and need to reference them by path in a command or prompt.

Supported terminals:

| Terminal | Process |
|----------|---------|
| BenjaminTerm | `benjaminterm-gui.exe` |
| WezTerm | `wezterm-gui.exe` |
| Windows Terminal | `windowsterminal.exe` |
| PowerShell | `powershell.exe` |
| PowerShell 7+ | `pwsh.exe` |
| Command Prompt | `cmd.exe` |
| Alacritty | `alacritty.exe` |
| ConEmu | `conemu.exe` / `conemu64.exe` |
| Hyper | `hyper.exe` |
| Git Bash (mintty) | `mintty.exe` |
| Tabby | `tabby.exe` |
| Warp | `warp.exe` |
| MobaXterm | `mobaxterm.exe` |

## Requirements

- **Windows 10+** (Windows only)
- A microphone
- ~700MB disk space (voice model)
- Internet connection for first launch (model download)

BenjaminTerm is installed by the PowerShell installer, or can be downloaded separately from [its repo](https://github.com/avalonreset/BenjaminTerm).

## How It Works

1. Press the hotkey (F13, CapsLock, or Mouse Back) to start recording
2. Speak naturally into your microphone
3. Press the hotkey again to stop recording
4. HyperYap transcribes locally using the Parakeet TDT model
5. The transcription is automatically pasted into the active window

All processing happens on your machine. Audio never leaves your computer. The speech model runs entirely offline after the initial download.

## Configuration

HyperYap works out of the box with zero configuration. All settings can be changed from the app's Settings page.

| Setting | Default | Description |
|---------|---------|-------------|
| Record mode | Toggle-to-talk | Press once to start, press again to stop. Can be changed to push-to-talk. |
| Record shortcut | F13 | Configurable to any key or key combination |
| Language | English | Supports multiple languages via the Parakeet model |
| Overlay | Bottom of screen | Recording indicator position. Can be set to top, bottom, or hidden. |
| LLM Connect | Disabled | Post-process transcriptions with a local LLM (Ollama) or remote API |
| HTTP API | Disabled | Local API on localhost for external tool integration |
| Sound feedback | Enabled | Audio cues when recording starts and stops |
| Copy to clipboard | Disabled | Optionally keep transcriptions in the clipboard |

Settings are stored in `%APPDATA%/com.avalonreset.hyperyap/settings.json`. Each install or upgrade resets settings to the recommended defaults. If you need to preserve custom settings across upgrades, back up this file before updating.

### Hotkey Customization

All in-app hotkeys can be remapped from the Settings page. Mouse button and CapsLock remapping are handled by the bundled hotkey engine, which runs alongside the main app and can be toggled from the tray menu.

## Build from Source

```bash
pnpm install

# Build the hotkey engine first
cd hotkeys && cargo build --release && cd ..

pnpm tauri dev      # development
pnpm tauri build    # production build
```

Requires: Node.js 18+, Rust, pnpm, [Tauri prerequisites](https://v2.tauri.app/start/prerequisites/)

Download the [Parakeet model](https://github.com/Kieirra/murmure-model/releases/download/1.0.0/parakeet-tdt-0.6b-v3-int8.zip) and extract to `resources/parakeet-tdt-0.6b-v3-int8/`.

## Contributing

Contributions are welcome. See [CONTRIBUTING.md](CONTRIBUTING.md) for development setup, PR workflow, and coding guidelines.

Please read the [Code of Conduct](CODE_OF_CONDUCT.md) before contributing.

## Attribution

HyperYap's voice engine is a modified version of [MURmure](https://github.com/Kieirra/murmure) by [Kieirra](https://github.com/Kieirra). Full credit to the original author for building an excellent local speech-to-text application.

[BenjaminTerm](https://github.com/avalonreset/BenjaminTerm) is a custom distribution of [WezTerm](https://github.com/wezterm/wezterm) by Wez Furlong.

Powered by NVIDIA's [Parakeet TDT 0.6B v3](https://huggingface.co/nvidia/parakeet-tdt-0.6b-v3) speech recognition model.

## License

The voice engine is licensed under [AGPL-3.0](LICENSE). BenjaminTerm is licensed under MIT. See [NOTICE](NOTICE) for full attribution details.
