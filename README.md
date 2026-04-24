<p align="center">
  <img src="assets/banner.webp" alt="HyperYap - local voice-to-text for Windows, macOS, and Linux" width="100%">
</p>

# HyperYap - Local Voice-to-Text for Windows, macOS, and Linux

[![CI](https://github.com/avalonreset/hyperyap/actions/workflows/ci.yaml/badge.svg)](https://github.com/avalonreset/hyperyap/actions/workflows/ci.yaml)
[![GitHub release](https://img.shields.io/github/v/release/avalonreset/hyperyap)](https://github.com/avalonreset/hyperyap/releases)
[![License: AGPL-3.0](https://img.shields.io/github/license/avalonreset/hyperyap)](LICENSE)
[![Last Commit](https://img.shields.io/github/last-commit/avalonreset/hyperyap)](https://github.com/avalonreset/hyperyap/commits/main)

HyperYap is a local voice-to-text application for Windows, macOS, and Linux. It runs NVIDIA Parakeet speech recognition on your machine, collects no data, and keeps terminal-heavy workflows fast with a Windows hotkey daemon for recording, smart terminal paste, smart terminal copy, and basic paste undo.

## Table of Contents

- [What You Get](#what-you-get)
- [Install](#install)
- [Default Hotkeys](#default-hotkeys)
  - [CapsLock Remapping](#capslock-remapping)
  - [Smart Terminal Editing](#smart-terminal-editing)
- [Requirements](#requirements)
- [How It Works](#how-it-works)
- [Configuration](#configuration)
- [Build from Source](#build-from-source)
- [Contributing](#contributing)
- [Attribution](#attribution)
- [License](#license)

## What You Get

One app, local transcription, fast terminal workflows:

- **Local speech-to-text** powered by NVIDIA [Parakeet TDT 0.6B v3](https://huggingface.co/nvidia/parakeet-tdt-0.6b-v3). No cloud, no internet after first install.
- **Cross-platform app builds** for Windows, macOS Apple Silicon, macOS Intel, and Linux x86_64.
- **Windows hotkey daemon** turns CapsLock and mouse side buttons into recording triggers. Mouse Forward becomes Enter.
- **Smart terminal editing on Windows** detects terminals and adapts Ctrl+C, Ctrl+V, and Ctrl+Z for terminal workflows.
- **Smart screenshot paste on Windows** auto-saves clipboard images as PNGs and pastes the file path instead of garbled terminal data.
- **[BenjaminTerm](https://github.com/avalonreset/benjaminterm)** is supported as the preferred terminal target for smart terminal behavior.

Windows installs are preconfigured for the full voice-to-text plus hotkey workflow. macOS and Linux builds provide the HyperYap app and platform shortcut support; the dedicated smart terminal editing daemon is Windows-only.

## Install

### Windows

Download [hyperyap_1.0.9_x64-setup.exe](https://github.com/avalonreset/hyperyap/releases/latest) from the Releases page and run it. On first launch, HyperYap will:

- Download the NVIDIA Parakeet speech model (~440MB)
- Set up the hotkey engine and register autostart
- Apply all preset settings (toggle-to-talk, F13, English)

Or use the one-line PowerShell installer. This installs everything in one shot, including BenjaminTerm and the speech model:

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

### macOS

Download the matching DMG from the [latest release](https://github.com/avalonreset/hyperyap/releases/latest):

- `HyperYap_aarch64_darwin.dmg` for Apple Silicon Macs
- `HyperYap_x86_64_darwin.dmg` for Intel Macs

macOS requires Accessibility permission for global shortcuts.

### Linux

Download `HyperYap_amd64.AppImage` or `HyperYap_amd64.deb` from the [latest release](https://github.com/avalonreset/hyperyap/releases/latest).

On Debian/Ubuntu, you can also use:

```sh
curl -fsSL https://raw.githubusercontent.com/avalonreset/hyperyap/main/install.sh | sh
```

### Upgrading

Run the installer again over an existing install. HyperYap upgrades in-place without deleting your speech model or requiring a re-download. Settings are reset to the recommended defaults on each upgrade.

## Default Hotkeys

| Key | Action |
|-----|--------|
| `F13` / `CapsLock` / Mouse Back | Start/stop recording on Windows |
| Mouse Forward | Enter on Windows |
| `Ctrl+Shift+Space` | Paste last transcript |
| `Ctrl+Alt+Space` | LLM-assisted recording |
| `Ctrl+Shift+X` | Command mode |
| `Escape` | Cancel recording |

These hotkeys are built in. Windows hotkey daemon behavior can be paused and resumed from the tray icon's right-click menu.

### CapsLock Remapping

On Windows, HyperYap disables CapsLock and repurposes it as a speech-to-text key. Press CapsLock to start recording, press it again to stop. Your transcription is pasted into whatever window is focused. CapsLock is permanently set to off so you never accidentally activate it.

### Smart Terminal Editing

On Windows, HyperYap's hotkey engine is terminal-aware. It detects which application is focused and adapts terminal copy, paste, and undo behavior accordingly:

**In regular applications** (browsers, editors, chat apps), Ctrl+C, Ctrl+V, and Ctrl+Z work normally. HyperYap does not interfere.

**In supported terminals**, HyperYap adds:

- `Ctrl+C`: copy selected terminal text through the terminal copy shortcut first; fall back to interrupt behavior when no selection is copied.
- `Ctrl+V`: paste text through the terminal paste shortcut and convert clipboard screenshots into saved PNG paths.
- `Ctrl+Z`: erase the most recent HyperYap-managed paste with bounded backspaces when the pasted text is still pending at the prompt.

When you take a Win+Shift+S or PrintScreen capture, the hotkey daemon prepares the screenshot path as soon as Windows publishes image data. It saves a timestamped PNG in `~/screenshots/`, caches the path briefly, and pastes that path through the terminal paste shortcut. If the path is not ready yet, HyperYap waits for image data instead of falling back to stale text or typing a stray `v`. Text clipboard contents paste normally.

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

- Windows 10+, macOS, or Linux x86_64
- A microphone
- ~700MB disk space (voice model)
- Internet connection for first launch (model download)

BenjaminTerm is installed by the Windows PowerShell installer, or can be downloaded separately from [its repo](https://github.com/avalonreset/benjaminterm).

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

# Build the Windows hotkey engine first when packaging on Windows
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

[BenjaminTerm](https://github.com/avalonreset/benjaminterm) is a custom distribution of [WezTerm](https://github.com/wezterm/wezterm) by Wez Furlong.

Powered by NVIDIA's [Parakeet TDT 0.6B v3](https://huggingface.co/nvidia/parakeet-tdt-0.6b-v3) speech recognition model.

## License

The voice engine is licensed under [AGPL-3.0](LICENSE). BenjaminTerm is licensed under MIT. See [NOTICE](NOTICE) for full attribution details.
