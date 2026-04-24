---
type: meta
title: "Dashboard"
created: 2026-04-21
updated: 2026-04-24
tags:
  - hyperyap
  - dashboard
status: seed
related:
  - "[[HyperYap Vault Index]]"
sources:
  - "[[Project Documentation]]"
---

# Dashboard

## Current Focus

- [[Smart Screenshot Paste]]
- [[Hotkey Daemon]]
- [[Terminal Editing Layer]]
- [[ADR 2026-04-21 Smart Paste Guard]]

## Release State

- Local release build: `v1.0.8`
- Windows installer: `hyperyap_1.0.8_x64-setup.exe`
- Expected release URL: `https://github.com/avalonreset/hyperyap/releases/tag/v1.0.8`

## Open Checks

- Apply [[Public Release Sanitization]] before publishing vault updates.
- Validate smart terminal copy, smart terminal paste, and paste undo in BenjaminTerm and at least one stock Windows terminal.
- Publish `v1.0.8` after GitHub Actions produces all selected platform artifacts.
- Run GitHub release workflow for Windows, Linux, macOS Apple Silicon, and macOS Intel artifacts.
- Run future installer and app relaunch tests detached from the active Codex terminal. See [[ADR 2026-04-21 Detached Installer Runs]].
- Record macOS and Linux validation results from GitHub Actions because local validation is Windows-only.
