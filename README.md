<p align="center">
  <img src="assets/banner.webp" alt="HyperYap - local voice-to-text for your desktop" width="100%">
</p>

# HyperYap - Local Voice-to-Text for Your Desktop

[![CI](https://github.com/avalonreset/hyperyap/actions/workflows/ci.yaml/badge.svg)](https://github.com/avalonreset/hyperyap/actions/workflows/ci.yaml)
[![GitHub release](https://img.shields.io/github/v/release/avalonreset/hyperyap)](https://github.com/avalonreset/hyperyap/releases)
[![License: AGPL-3.0](https://img.shields.io/github/license/avalonreset/hyperyap)](LICENSE)
[![Last Commit](https://img.shields.io/github/last-commit/avalonreset/hyperyap)](https://github.com/avalonreset/hyperyap/commits/main)

HyperYap is a privacy-first voice-to-text app that runs speech recognition locally on your machine. It uses NVIDIA Parakeet, collects no data, and gives you fast recording, automatic paste, configurable shortcuts, optional LLM cleanup, and terminal-friendly workflow helpers.

## Table of Contents

- [What You Get](#what-you-get)
- [Install](#install)
- [Shortcut Support](#shortcut-support)
- [Terminal-Friendly Workflows](#terminal-friendly-workflows)
- [Platform Compatibility](#platform-compatibility)
- [Requirements](#requirements)
- [How It Works](#how-it-works)
- [Configuration](#configuration)
- [Build from Source](#build-from-source)
- [Contributing](#contributing)
- [Attribution](#attribution)
- [License](#license)

## What You Get

One app for private, local transcription:

- **Local speech-to-text** powered by NVIDIA [Parakeet TDT 0.6B v3](https://huggingface.co/nvidia/parakeet-tdt-0.6b-v3). No cloud, no internet after first install.
- **Fast record-and-paste workflow** that transcribes speech and inserts the result into the active app.
- **Configurable shortcuts** for recording, last transcript paste, command mode, cancellation, and LLM-assisted modes.
- **Toggle-to-talk and push-to-talk** recording modes.
- **Optional LLM post-processing** through local Ollama or a configured remote endpoint.
- **Custom dictionary and formatting rules** for names, commands, casing, and recurring phrases.
- **Import/export settings** for moving configurations between installs.
- **Desktop builds** for Windows, macOS Apple Silicon, macOS Intel, and Linux x86_64.

## Install

Download the package for your platform from the [latest release](https://github.com/avalonreset/hyperyap/releases/latest).

### Windows

Run `hyperyap_1.0.9_x64-setup.exe` from the Releases page.

For the full workstation setup, use the PowerShell installer. It installs HyperYap, downloads the speech model, installs the latest [BenjaminTerm](https://github.com/avalonreset/benjaminterm) release, and configures the optional hotkey helper:

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

Run the installer again over an existing install. HyperYap upgrades in place without deleting your speech model or requiring a re-download. Settings are reset to the recommended defaults on each upgrade.

## Shortcut Support

HyperYap records through configurable shortcuts. The exact binding can be changed from Settings.

| Shortcut | Action |
|----------|--------|
| Record shortcut | Start or stop recording |
| `Ctrl+Shift+Space` | Paste last transcript |
| `Ctrl+Alt+Space` | LLM-assisted recording |
| `Ctrl+Shift+X` | Command mode |
| `Ctrl+Shift+1` through `Ctrl+Shift+4` | LLM mode slots |
| `Escape` | Cancel recording |

The Windows workstation preset maps `F13`, `CapsLock`, and Mouse Back to the record shortcut, and maps Mouse Forward to Enter. Those mappings come from the optional hotkey helper and can be paused from its tray menu.

## Terminal-Friendly Workflows

HyperYap can be used with any app that accepts pasted text. It is especially useful in editors, chat apps, terminals, issue trackers, and coding tools where fast dictation reduces context switching.

The optional hotkey helper adds terminal-aware behavior where supported:

- Smart copy, paste, and undo handling for terminal windows.
- Clipboard screenshot conversion into a saved PNG path before paste.
- Bounded paste undo for recent HyperYap-managed terminal inserts.

[BenjaminTerm](https://github.com/avalonreset/benjaminterm) is the preferred terminal target for this workflow, but HyperYap does not require it for normal transcription.

Supported terminal process names for the helper:

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

## Platform Compatibility

The core app is built for Windows, macOS, and Linux. Global shortcut behavior is implemented separately per platform, so support depends on the operating system permissions and desktop environment.

| Capability | Windows | macOS | Linux |
|------------|---------|-------|-------|
| Local Parakeet transcription | Supported | Supported | Supported |
| Configurable recording shortcuts | Supported | Supported with Accessibility permission | Supported where the desktop/session allows global input hooks |
| Automatic paste into active app | Supported | Supported | Supported |
| CapsLock and mouse side-button remapping | Supported through the optional hotkey helper | Not currently bundled | Not currently bundled |
| Smart terminal copy/paste/undo helper | Supported through the optional hotkey helper | Not currently bundled | Not currently bundled |
| PowerShell one-line workstation installer | Supported | Not applicable | Not applicable |

In short: the app should work as a local voice-to-text tool on macOS and Linux, but the CapsLock remapper and smart terminal helper are currently Windows-only. macOS users must grant Accessibility permission. Linux global shortcuts may vary by distribution, window manager, and Wayland/X11 session.

## Requirements

- Windows 10+, macOS, or Linux x86_64
- A microphone
- ~700MB disk space for the speech model
- Internet connection for first launch model download

## How It Works

1. Press your configured record shortcut.
2. Speak naturally into your microphone.
3. Press the shortcut again to stop recording.
4. HyperYap transcribes locally using the Parakeet TDT model.
5. The transcription is automatically pasted into the active window.

All processing happens on your machine. Audio never leaves your computer. The speech model runs entirely offline after the initial download.

## Configuration

HyperYap works out of the box with minimal setup. All settings can be changed from the app's Settings page.

| Setting | Default | Description |
|---------|---------|-------------|
| Record mode | Toggle-to-talk | Press once to start, press again to stop. Can be changed to push-to-talk. |
| Record shortcut | Configurable | Remap to any supported key or key combination. |
| Language | English | Supports multiple languages via the Parakeet model. |
| Overlay | Bottom of screen | Recording indicator position. Can be set to top, bottom, or hidden. |
| LLM Connect | Disabled | Post-process transcriptions with a local LLM through Ollama or a configured remote endpoint. |
| HTTP API | Disabled | Local API on localhost for external tool integration. |
| Sound feedback | Enabled | Audio cues when recording starts and stops. |
| Copy to clipboard | Disabled | Optionally keep transcriptions in the clipboard. |

Settings are stored in `%APPDATA%/com.avalonreset.hyperyap/settings.json` on Windows. Each install or upgrade resets settings to the recommended defaults. If you need to preserve custom settings across upgrades, back up this file before updating.

### Hotkey Customization

All in-app shortcuts can be remapped from the Settings page. Platform-level remapping such as CapsLock or mouse side buttons is handled by the optional hotkey helper where available.

## Build from Source

```bash
pnpm install

# Build the optional hotkey helper before packaging the Windows installer
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
