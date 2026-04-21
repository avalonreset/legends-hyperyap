---
type: decision
title: "ADR 2026-04-21 Native Prepared Screenshot Paths"
created: 2026-04-21
updated: 2026-04-21
tags:
  - hyperyap
  - decision
  - smart-paste
  - performance
status: developing
related:
  - "[[Smart Screenshot Paste]]"
  - "[[Hotkey Daemon]]"
sources:
  - "[[Project Documentation]]"
---

# ADR 2026-04-21 Native Prepared Screenshot Paths

## Context

The earlier smart paste path waited until terminal Ctrl+V to launch PowerShell, read clipboard image data, save a PNG, swap the clipboard to a path, and paste. That was correct but slow enough to feel laggy in vanilla WezTerm.

## Decision

The hotkey daemon should prepare screenshot paths as soon as screenshot intent is detected. For Win+Shift+S and PrintScreen, it starts a background preparation thread, waits for Windows image clipboard data, saves the image natively from CF_DIB to `~/screenshots/`, and caches the prepared path for a short period. Terminal Ctrl+V uses that prepared path when the clipboard still contains image data.

PowerShell image conversion remains available only as a fallback when native parsing cannot handle the clipboard format.

## Consequences

- Screenshot path paste is effectively instant after the native preparation thread completes.
- Terminal paste no longer pays the PowerShell startup cost on the normal path.
- Cached screenshot paths are only used while the clipboard still advertises image content, reducing the risk of overriding copied text.
- The daemon now owns more Win32 clipboard image logic and must keep native parsing conservative.

## Validation

- Windows local hotkey daemon was updated with the native path pipeline.
- User confirmed vanilla WezTerm screenshot paste became effectively instant.
