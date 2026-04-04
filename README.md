# HyperYap

Voice-to-text that just works. Clone it, install it, yap away.

HyperYap is a preconfigured, grab-and-go speech-to-text package built on [MURmure](https://github.com/Kieirra/murmure). It bundles NVIDIA's [Parakeet TDT 0.6B v3](https://huggingface.co/nvidia/parakeet-tdt-0.6b-v3) model for fast, fully local transcription — no internet, no cloud, no data collection.

Everything is preset: hotkeys, toggle-to-talk, mouse button mapping, auto-boot. Install once, use everywhere.

## One-Line Install

```powershell
irm https://raw.githubusercontent.com/avalonreset/hyperyap/main/install.ps1 | iex
```

Or clone and run locally:

```powershell
git clone https://github.com/avalonreset/hyperyap.git
cd hyperyap
powershell -ExecutionPolicy Bypass -File install.ps1
```

## What You Get

| Component | What It Does |
|-----------|-------------|
| **HyperYap app** | Local speech-to-text powered by Parakeet TDT 0.6B (NVIDIA) |
| **Hotkey scripts** | Mouse side buttons → F13 (record), CapsLock → F13, Mouse Forward → Enter |
| **Smart paste** | Ctrl+V in terminals auto-saves clipboard images as PNGs |
| **Auto-boot** | Everything starts on login. No setup after reboot. |
| **Preset configs** | Toggle-to-talk, English, overlay on bottom, all shortcuts pre-mapped |

## Default Hotkeys

| Key | Action |
|-----|--------|
| `F13` / `CapsLock` / Mouse Back | Start/stop recording |
| Mouse Forward | Enter |
| `Ctrl+Shift+Space` | Paste last transcript |
| `Ctrl+Alt+Space` | LLM-assisted recording |
| `Ctrl+Shift+X` | Command mode |
| `Escape` | Cancel recording |

## Requirements

- Windows 10+
- A microphone
- ~700MB disk space (model)

AutoHotkey v2 is installed automatically if not present.

## Build from Source

```bash
pnpm install
pnpm tauri dev      # development
pnpm tauri build    # production build
```

Requires: Node.js 18+, Rust, pnpm, [Tauri prerequisites](https://v2.tauri.app/start/prerequisites/)

Download the [Parakeet model](https://github.com/Kieirra/murmure-model/releases/download/1.0.0/parakeet-tdt-0.6b-v3-int8.zip) and extract to `resources/parakeet-tdt-0.6b-v3-int8/`.

## Attribution

HyperYap is a modified version of [MURmure](https://github.com/Kieirra/murmure) by [Kieirra](https://github.com/Kieirra). Full credit to the original author for building an excellent local speech-to-text application.

This project is licensed under **AGPL-3.0**, the same license as the original. See [LICENSE](LICENSE) and [NOTICE](NOTICE) for details.

Powered by NVIDIA's [Parakeet TDT 0.6B v3](https://huggingface.co/nvidia/parakeet-tdt-0.6b-v3) speech recognition model.
