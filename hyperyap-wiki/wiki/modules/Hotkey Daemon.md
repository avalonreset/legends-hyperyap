---
type: module
title: "Hotkey Daemon"
created: 2026-04-21
updated: 2026-04-21
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

- Detects terminal windows such as Benjamin Term.
- Intercepts Ctrl+V when image clipboard content is present or expected.
- Saves clipboard image content to `~/screenshots/`.
- Restores the image clipboard after saving.
- Suppresses fallback paste when waiting for a screenshot image.

## Validation

Run from `../hotkeys/`:

```powershell
cargo fmt
cargo clippy -- -D warnings
```
