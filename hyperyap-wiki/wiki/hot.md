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

2026-04-21. HyperYap local install and GitHub release are at `v1.0.4`. The `project-wiki-tooling` skill was used after restart to lint and complete the `hyperyap-wiki` scaffold. A separate private GitHub repo now exists for [[Project Wiki Tooling]].

## Key Recent Facts

- Smart screenshot paste targets Benjamin Term and similar terminal windows.
- The hotkey daemon waits for fresh clipboard images before pasting a saved screenshot path.
- If the clipboard is empty or unstable during terminal Ctrl+V, the daemon suppresses fallback paste so a stray `v` should not appear.
- Mouse Back should keep translating to F13 for toggle-to-talk compatibility.
- [[Project Wiki Tooling]] lives at `[local path redacted]` and is tracked separately at `[project setup redacted]`.

## Recent Changes

- Created [[ADR 2026-04-21 Smart Paste Guard]].
- Created [[ADR 2026-04-21 Mouse Back Uses F13]].
- Created [[Smart Screenshot Paste]] and [[Hotkey Daemon]] seed notes.
- Added folder sub-indexes and placeholder raw/attachment folders for a complete Obsidian vault structure.
- Created [[Project Wiki Tooling]] note and recorded the project setup setup.

## Active Threads

- User is stress testing very fast screenshot then Ctrl+V behavior in Benjamin Term.
- Future work should preserve exact clipboard image restoration and terminal-specific paste behavior.
- After context compaction, continue from `wiki/hot.md` first and then `wiki/index.md`.
