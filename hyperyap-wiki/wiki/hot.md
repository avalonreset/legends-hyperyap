---
type: meta
title: "Hot Cache"
created: 2026-04-21
updated: 2026-04-21
tags:
  - hyperyap
  - hot-cache
status: developing
related:
  - "[[Smart Screenshot Paste]]"
  - "[[Hotkey Daemon]]"
sources:
  - "[[Project Documentation]]"
---

# Hot Cache

## Last Updated

2026-04-21. HyperYap local install is being packaged for `v1.0.6`. The release adds native prepared screenshot paths in the hotkey daemon, making vanilla WezTerm screenshot paste effectively instant in local testing. A separate private GitHub repo exists for [[Project Wiki Tooling]].

## Key Recent Facts

- Smart screenshot paste targets Benjamin Term and similar terminal windows.
- The hotkey daemon waits for fresh clipboard images before pasting a saved screenshot path.
- Recent Win+Shift+S and PrintScreen shortcuts are tracked for 10 seconds so fast Ctrl+V waits for image data instead of falling back to stale text.
- `v1.0.6` prepares screenshot PNG paths natively from CF_DIB clipboard data before terminal paste and keeps PowerShell only as fallback.
- Vanilla WezTerm receives converted paths through Ctrl+Shift+V, so Codex does not see the original Ctrl+V image-paste command.
- If the clipboard is empty or unstable during terminal Ctrl+V, the daemon suppresses fallback paste so a stray `v` should not appear.
- Mouse Back should keep translating to F13 for toggle-to-talk compatibility.
- Installer, upgrade, and relaunch tests should run detached from the active Codex terminal. Use a background PowerShell process, `Start-Process`, or a separate terminal so install noise does not pollute the chat session. See [[ADR 2026-04-21 Detached Installer Runs]].
- [[Project Wiki Tooling]] lives at `[local path redacted]` and is tracked separately at `[project setup redacted]`.

## Recent Changes

- Created [[ADR 2026-04-21 Smart Paste Guard]].
- Created [[ADR 2026-04-21 Mouse Back Uses F13]].
- Created [[Smart Screenshot Paste]] and [[Hotkey Daemon]] seed notes.
- Added folder sub-indexes and placeholder raw/attachment folders for a complete Obsidian vault structure.
- Created [[Project Wiki Tooling]] note and recorded the project setup setup.
- Created [[ADR 2026-04-21 Detached Installer Runs]] after installer output polluted the active Codex terminal.
- Created [[ADR 2026-04-21 Native Prepared Screenshot Paths]] after local testing showed the native prepared path was effectively instant.

## Active Threads

- User is stress testing very fast screenshot then Ctrl+V behavior in Benjamin Term.
- Future work should preserve native prepared screenshot paths, exact clipboard image restoration, and terminal-specific paste behavior.
- After context compaction, continue from `wiki/hot.md` first and then `wiki/index.md`.
