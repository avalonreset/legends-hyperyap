---
type: flow
title: "Smart Screenshot Paste"
created: 2026-04-21
updated: 2026-04-21
tags:
  - hyperyap
  - smart-paste
status: developing
related:
  - "[[Hotkey Daemon]]"
  - "[[Clipboard Preservation]]"
sources:
  - "[[Project Documentation]]"
---

# Smart Screenshot Paste

Smart screenshot paste lets the user take a native Windows screenshot, switch to a terminal, press Ctrl+V, and paste a saved PNG path instead of trying to paste binary image data into the terminal.

## Flow

1. User takes a Windows screenshot.
2. The screenshot appears on the clipboard as image data.
3. User focuses Benjamin Term, vanilla WezTerm, or another supported terminal.
4. User presses Ctrl+V.
5. The hotkey daemon detects screenshot intent from Win+Shift+S or PrintScreen.
6. A background preparation thread waits for Windows clipboard image data.
7. The daemon saves the image natively to `~/screenshots/screenshot_*.png` and caches the path.
8. User focuses Benjamin Term, vanilla WezTerm, or another supported terminal.
9. User presses Ctrl+V.
10. The daemon pastes the prepared path through the terminal paste shortcut.
11. The daemon restores the original image clipboard.

## Timing Guard

The current guard waits for fresh image clipboard content when Ctrl+V arrives before the screenshot is fully available. If image content does not arrive within the wait window, the paste fails closed instead of allowing a raw `v` to appear.

## Native Prepared Path

The `v1.0.6` pipeline prepares screenshot paths before paste. The hotkey daemon reads Windows CF_DIB clipboard image data natively in Rust, writes a PNG path under `~/screenshots/`, and keeps that path in a short-lived cache while the clipboard still contains image data. PowerShell remains only as a fallback for image formats the native path cannot parse.

## WezTerm Compatibility

Vanilla WezTerm does not need Benjamin Term's custom smart paste behavior. HyperYap owns the conversion, writes the saved PNG path to the clipboard, and pastes with Ctrl+Shift+V so terminal apps receive text instead of the original Ctrl+V command.
