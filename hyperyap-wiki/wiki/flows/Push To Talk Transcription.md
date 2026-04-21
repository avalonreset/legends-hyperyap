---
type: flow
title: "Push To Talk Transcription"
created: 2026-04-21
updated: 2026-04-21
tags:
  - hyperyap
  - transcription
status: developing
related:
  - "[[Rust Tauri Backend]]"
  - "[[Clipboard Preservation]]"
sources:
  - "[[Project Documentation]]"
---

# Push To Talk Transcription

The push-to-talk flow records audio while the shortcut is held, runs transcription locally, places text on the clipboard, simulates paste, and restores previous clipboard content.

## Key Constraint

This flow must not persist user data beyond the last five transcriptions.
