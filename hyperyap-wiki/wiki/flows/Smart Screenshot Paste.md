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
5. The hotkey daemon detects terminal context, image clipboard formats, and recent screenshot intent.
6. The daemon saves the image to `~/screenshots/screenshot_*.png`.
7. The daemon places the path on the clipboard and simulates the terminal paste shortcut.
8. The daemon restores the original image clipboard.

## Timing Guard

The current guard waits for fresh image clipboard content when Ctrl+V arrives before the screenshot is fully available. If image content does not arrive within the wait window, the paste fails closed instead of allowing a raw `v` to appear.

## WezTerm Compatibility

Vanilla WezTerm does not need Benjamin Term's custom smart paste behavior. HyperYap owns the conversion, writes the saved PNG path to the clipboard, and pastes with Ctrl+Shift+V so terminal apps receive text instead of the original Ctrl+V command.
