---
type: overview
title: "HyperYap Overview"
created: 2026-04-21
updated: 2026-04-21
tags:
  - hyperyap
  - overview
status: developing
related:
  - "[[HyperYap]]"
  - "[[Hotkey Daemon]]"
sources:
  - "[[Project Documentation]]"
---

# HyperYap Overview

HyperYap is a privacy-first local speech-to-text desktop app built with Tauri, Rust, React, and TypeScript. It runs transcription locally and is designed around a small, controlled data persistence model.

The current release line includes smart screenshot paste support for terminal workflows. The most recent release context in this vault is `v1.0.6`, which adds native prepared screenshot paths in the hotkey daemon for effectively instant terminal screenshot paste.

## Core Areas

- [[Rust Tauri Backend]] owns app lifecycle, commands, audio, history, clipboard, settings, and tray behavior.
- [[React TypeScript Frontend]] owns the app UI and follows feature-first project structure.
- [[Hotkey Daemon]] owns global hotkeys and terminal smart paste behavior.
- [[Smart Screenshot Paste]] converts native Windows screenshot clipboard images into saved PNG paths for terminal paste.

## Operating Principles

- Privacy first.
- Security first.
- Simple code over complex code.
- Small focused changes.
- Explicit validation before release-impacting work.
