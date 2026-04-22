---
type: meta
title: "Vault Log"
created: 2026-04-21
updated: 2026-04-22
tags:
  - hyperyap
  - log
status: developing
related:
  - "[[HyperYap Overview]]"
sources:
  - "[[Project Documentation]]"
---

# Vault Log

## 2026-04-22 save | smart paste hook responsiveness

- Patched `hotkeys/src/main.rs` so `WM_SMART_PASTE` starts a worker thread and does not block the low-level keyboard hook owner thread.
- Added a daemon guard for HyperYap-generated injected input while allowing user-generated injected Ctrl+V chords to reach smart paste.
- Rebuilt the release daemon and restarted it headless.
- Rebuilt `src-tauri/target/release/bundle/nsis/hyperyap_1.0.6_x64-setup.exe`, ran the installer, then restarted HyperYap.
- Verified the restarted daemon matched the rebuilt release binary.
- Prepared a public `v1.0.7` patch release for the same hook responsiveness fix.
- Published `v1.0.7 - Smart Paste Hook Responsiveness` on GitHub with `hyperyap_1.0.7_x64-setup.exe`.
- Validated with `cargo fmt`, `cargo clippy -- -D warnings`, and `cargo build --release`.
- Updated [[Hot Cache]], [[Smart Screenshot Paste]], [[Hotkey Daemon]], [[HyperYap Overview]], and [[HyperYap Vault Index]].

## 2026-04-21 save | native prepared screenshot paths

- Created [[ADR 2026-04-21 Native Prepared Screenshot Paths]].
- Recorded that `v1.0.6` prepares screenshot paths natively in the hotkey daemon before terminal Ctrl+V.
- Updated [[Smart Screenshot Paste]], [[Hot Cache]], [[HyperYap Overview]], [[Dashboard]], [[HyperYap Vault Index]], and [[Decisions Index]].

## 2026-04-21 save | detached installer run rule

- Created [[ADR 2026-04-21 Detached Installer Runs]].
- Recorded that installer, upgrade, and app relaunch tests should run detached from the active Codex terminal.
- Updated [[Hot Cache]], [[HyperYap Vault Index]], [[Decisions Index]], and [[Dashboard]].

## 2026-04-21 lint | project-wiki-tooling scaffold completion

- Ran the `project-wiki-tooling` skill against `hyperyap-wiki`.
- Added sub-indexes for sources, entities, concepts, modules, flows, decisions, and questions.
- Added tracked placeholders for `.raw/` source categories and `_attachments/`.
- Updated [[Hot Cache]] and [[HyperYap Vault Index]].

## 2026-04-21 scaffold | HyperYap Wiki

- Created initial Obsidian vault scaffold.
- Seeded index, hot cache, overview, source map, modules, flows, concepts, entities, and decisions.
- Source: [[Project Documentation]]
- Key insight: HyperYap needs durable notes for smart paste behavior, hotkey decisions, and release validation.
