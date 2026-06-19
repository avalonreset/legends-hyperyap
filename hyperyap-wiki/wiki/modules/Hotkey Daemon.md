---
type: module
title: "Hotkey Daemon"
created: 2026-04-21
updated: 2026-04-22
tags:
  - hyperyap
  - hotkeys
status: developing
related:
  - "[[Smart Screenshot Paste]]"
  - "[[ADR 2026-04-21 Mouse Back Uses F13]]"
sources:
  - "[[Project Documentation]]"
path: "../hotkeys/"
---

# Hotkey Daemon

The hotkey daemon handles global input behaviors outside the main Tauri app. It includes terminal-aware smart screenshot paste and Mouse Back to F13 translation.

## Current Smart Paste State

- Detects common terminal windows such as WezTerm, Windows Terminal, PowerShell, and cmd.
- Intercepts Ctrl+V when image clipboard content is present or expected.
- Saves clipboard image content to `~/screenshots/`.
- Restores the image clipboard after saving.
- Suppresses fallback paste when waiting for a screenshot image.
- Runs smart paste work on a background thread to keep the keyboard hook message pump responsive.
- Ignores only recent HyperYap-generated injected key events, allowing user remappers to trigger smart paste.

## Validation

Run from `../hotkeys/`:

```powershell
cargo fmt
cargo clippy -- -D warnings
```
