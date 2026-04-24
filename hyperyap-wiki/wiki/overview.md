---
type: overview
title: "HyperYap Overview"
aliases:
  - "HyperYap Overview"
created: 2026-04-21
updated: 2026-04-24
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

The current release line includes smart terminal editing support for Windows terminal workflows. `v1.0.6` added native prepared screenshot paths in the hotkey daemon for effectively instant terminal screenshot paste. `v1.0.7` moved smart paste waits to a worker thread. `v1.0.8` adds smart terminal copy and basic HyperYap-managed paste undo.

## Core Areas

- [[Rust Tauri Backend]] owns app lifecycle, commands, audio, history, clipboard, settings, and tray behavior.
- [[React TypeScript Frontend]] owns the app UI and follows feature-first project structure.
- [[Hotkey Daemon]] owns global hotkeys and terminal smart paste behavior.
- [[Smart Screenshot Paste]] converts native Windows screenshot clipboard images into saved PNG paths for terminal paste.
- [[Terminal Editing Layer]] groups terminal-aware copy, paste, screenshot path paste, and basic paste undo behavior.

## Operating Principles

- Privacy first.
- Security first.
- Simple code over complex code.
- Small focused changes.
- Explicit validation before release-impacting work.
