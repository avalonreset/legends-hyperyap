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
- [[Lint Report 2026-04-24]]

## Release State

- Prepared release: `v1.0.9`
- Expected Windows installer: `hyperyap_1.0.9_x64-setup.exe`
- Expected release URL: `https://github.com/avalonreset/hyperyap/releases/tag/v1.0.9`
- BenjaminTerm install target: latest stable `avalonreset/benjaminterm` release, currently `v1.4.3`
- Trigger status: `.release-trigger` was refreshed on current `main` with commit `424022c`; workflow run status still needs confirmation in GitHub Actions.
- Current-main housekeeping: README is platform-neutral, local package/design scratch artifacts are ignored, and stale `latest.json` was removed.

## Open Checks

- Apply [[Public Release Sanitization]] before publishing vault updates.
- Validate smart terminal copy, smart terminal paste, and paste undo in BenjaminTerm and at least one stock Windows terminal.
- Confirm the `v1.0.9` GitHub Actions release run from commit `424022c` started and completed.
- Confirm the recreated `v1.0.9` release contains Windows, Linux, macOS Apple Silicon, and macOS Intel artifacts.
- Run future installer and app relaunch tests detached from the active Codex terminal. See [[ADR 2026-04-21 Detached Installer Runs]].
- Record macOS and Linux validation results from GitHub Actions because local validation is Windows-only.
