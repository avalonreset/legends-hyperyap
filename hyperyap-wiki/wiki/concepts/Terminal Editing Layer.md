---
type: concept
title: "Terminal Editing Layer"
created: 2026-04-24
updated: 2026-04-24
tags:
  - hyperyap
  - hotkeys
  - terminals
  - release
status: developing
related:
  - "[[Hotkey Daemon]]"
  - "[[Smart Screenshot Paste]]"
  - "[[Clipboard Preservation]]"
sources:
  - "[[Project Documentation]]"
---

# Terminal Editing Layer

The terminal editing layer is the Windows hotkey daemon behavior that makes common terminal copy, paste, and undo actions match normal user expectations without modifying individual terminal applications.

## Scope

- `Ctrl+C` in supported terminals first sends terminal copy (`Ctrl+Shift+C`) so selected text is copied instead of immediately interrupting the running process.
- If terminal copy does not update the clipboard quickly, HyperYap falls back to real `Ctrl+C` so interrupt behavior still works.
- `Ctrl+V` keeps the existing smart terminal paste behavior for text and screenshot image paths.
- `Ctrl+Z` erases the most recent HyperYap-managed terminal paste with bounded backspaces when the pasted text is still pending at the prompt.

## Constraints

- This layer is Windows-only because it lives in `hotkeys/src/main.rs`.
- It must only activate for known terminal processes.
- It must not store user clipboard contents beyond short-lived process memory needed for paste undo.
- Paste undo is intentionally basic; it is not safe to assume command history can be reversed after Enter submits a command.

## Release Status

`v1.0.8` introduces the first pass of this layer. Windows build validation is local; macOS and Linux release validation must run through the platform-specific GitHub Actions builders.
