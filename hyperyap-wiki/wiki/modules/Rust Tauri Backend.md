---
type: module
title: "Rust Tauri Backend"
created: 2026-04-21
updated: 2026-04-21
tags:
  - hyperyap
  - rust
  - tauri
status: developing
related:
  - "[[Push To Talk Transcription]]"
  - "[[Clipboard Preservation]]"
sources:
  - "[[Project Documentation]]"
path: "../src-tauri/"
---

# Rust Tauri Backend

The backend owns app setup, commands, audio recording, transcription, clipboard integration, history, settings, overlay, tray behavior, and local HTTP API.

## Validation

Run from `../src-tauri/`:

```powershell
cargo fmt
cargo clippy -- -D warnings
```
