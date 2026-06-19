---
type: meta
title: "Hot Cache"
aliases:
  - "Hot Cache"
created: 2026-04-21
updated: 2026-04-24
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

2026-06-19. HyperYap `main` is being prepared for `v1.0.10`: English-first Parakeet v2 INT8 default, targeted Murmure stability fixes, Dependabot hygiene, and BenjaminTerm decommissioning from the default Windows installer. The current installer path is HyperYap + Parakeet model + optional hotkey helper only.

## Key Recent Facts

- Smart screenshot paste targets common terminal windows such as WezTerm, Windows Terminal, PowerShell, and cmd.
- BenjaminTerm is no longer installed, launched, or documented as a current HyperYap companion dependency.
- `main` now has platform-neutral README positioning, cleanup ignore rules, and no tracked stale `latest.json` updater manifest.
- The Windows hotkey daemon now treats smart copy, smart paste, smart screenshot paste, and paste undo as one terminal editing layer.
- Terminal Ctrl+C first attempts Ctrl+Shift+C copy, then falls back to Ctrl+C when no clipboard change is detected.
- Terminal Ctrl+Z only undoes recent HyperYap-managed pastes; it does not reverse commands after Enter submits them.
- The hotkey daemon waits for fresh clipboard images before pasting a saved screenshot path.
- Recent Win+Shift+S and PrintScreen shortcuts are tracked for 10 seconds so fast Ctrl+V waits for image data instead of falling back to stale text.
- `v1.0.6` prepares screenshot PNG paths natively from CF_DIB clipboard data before terminal paste and keeps PowerShell only as fallback.
- Vanilla WezTerm receives converted paths through Ctrl+Shift+V, so Codex does not see the original Ctrl+V image-paste command.
- If the clipboard is empty or unstable during terminal Ctrl+V, the daemon suppresses fallback paste so a stray `v` should not appear.
- Smart paste work must not block the hook owner thread. Long waits, clipboard conversion, and PowerShell fallback belong on a worker thread.
- User-generated injected Ctrl+V chords from keyboard tools should still be eligible for smart paste; only HyperYap's own recent `SendInput` events should be skipped.
- Mouse Back should keep translating to F13 for toggle-to-talk compatibility.
- Installer, upgrade, and relaunch tests should run detached from the active Codex terminal. Use a background PowerShell process, `Start-Process`, or a separate terminal so install noise does not pollute the chat session. See [[ADR 2026-04-21 Detached Installer Runs]].

## Recent Changes

- Created [[Terminal Editing Layer]].
- Bumped release metadata to `1.0.9`.
- Removed the Windows installer path that selected and installed BenjaminTerm.
- Added `.release-trigger` and release workflow support so a deliberate trigger-file commit can start the all-platform release path.
- Retriggered `.release-trigger` from current `main` after documentation and cleanup commits.
- Reframed the README around cross-platform functionality and moved Windows-only behavior into compatibility notes.
- Removed the stale tracked `latest.json` updater manifest and ignored local release/design scratch artifacts.
- Created [[Lint Report 2026-04-24]] for the public-release wiki scan.
- Created [[Public Release Sanitization]] and linked it from [[HyperYap Vault Index]], [[Dashboard]], and `CODEX.md`.
- Added vault `.gitignore` rules for local/private/sensitive material.
- Replaced local owner metadata in `CODEX.md` with `Owner: <project-owner>`.
- Created [[ADR 2026-04-21 Smart Paste Guard]].
- Created [[ADR 2026-04-21 Mouse Back Uses F13]].
- Created [[Smart Screenshot Paste]] and [[Hotkey Daemon]] seed notes.
- Added folder sub-indexes and placeholder raw/attachment folders for a complete Obsidian vault structure.
- Created [[ADR 2026-04-21 Detached Installer Runs]] after installer output polluted the active Codex terminal.
- Created [[ADR 2026-04-21 Native Prepared Screenshot Paths]] after local testing showed the native prepared path was effectively instant.

## Active Threads

- Run [[Public Release Sanitization]] checks before publishing vault changes.
- Monitor the all-platform GitHub Actions release run started by commit `424022c` for `v1.0.9`.
- Validate the `v1.0.9` Windows installer after the release artifact is published.
- Validate smart terminal copy/paste/undo manually on Windows before declaring terminal editing complete.
- Use GitHub Actions for macOS and Linux build validation; local machine validation is Windows-only.
- Future work should preserve native prepared screenshot paths, exact clipboard image restoration, and terminal-specific paste behavior.
- If raw `V` still appears after this patch, next checks are elevated-terminal integrity mismatch, unsupported foreground process name, or the daemon not running.
- Target release: `https://github.com/avalonreset/hyperyap/releases/tag/v1.0.9`.
- Expected Windows release asset: `hyperyap_1.0.9_x64-setup.exe`.
- After context compaction, continue from `wiki/hot.md` first and then `wiki/index.md`.
