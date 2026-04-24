---
type: meta
title: "Vault Log"
aliases:
  - "Vault Log"
created: 2026-04-21
updated: 2026-04-24
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

## 2026-04-24 save | release housekeeping and current-main retrigger

- Reframed the public README around cross-platform voice-to-text functionality and moved Windows-only CapsLock, mouse remapping, and smart terminal helper behavior into compatibility notes.
- Removed stale tracked `latest.json`; it described `1.0.0` updater assets and was not referenced by the current app because updater endpoints are disabled.
- Added `.gitignore` coverage for root-level local package artifacts and local design scratch folders.
- Removed local untracked cleanup targets: root installer copy, `assets/icon-concepts/`, `assets/originals/`, `.github-audit/`, and `dist/`.
- Verified `main` contains `package.json` and `src-tauri/tauri.conf.json` version `1.0.9`.
- Verified `main` installer logic resolves `avalonreset/benjaminterm` through the GitHub latest stable release endpoint.
- Retriggered the all-platform `v1.0.9` release workflow from current `main` with commit `424022c`.
- Remaining external check: confirm the GitHub Actions run completed and the `v1.0.9` release assets were recreated from that current-main trigger.

## 2026-04-24 lint | v1.0.9 release wiki status

- Ran [[Public Release Sanitization]] sensitive-pattern checks across the wiki.
- Found no secret values or private local path leaks outside the sanitization policy examples.
- Updated [[Hot Cache]], [[Dashboard]], [[HyperYap Overview]], and [[Terminal Editing Layer]] from stale `v1.0.8` release state to current `v1.0.9` release status.
- Updated [[Benjamin Term]] to remove personal phrasing and record that the Windows installer resolves latest stable BenjaminTerm, currently `v1.4.3`.
- Created [[Lint Report 2026-04-24]].

## 2026-04-24 save | terminal editing release prep

- Created [[Terminal Editing Layer]] to name the missing concept behind smart terminal copy, smart paste, screenshot path paste, and basic paste undo.
- Prepared `v1.0.8` release metadata for HyperYap.
- Updated README, changelog, Linux installer script, and GitHub release workflow for Windows, Linux, macOS Apple Silicon, and macOS Intel artifacts.
- Built the Windows NSIS installer at `src-tauri/target/release/bundle/nsis/hyperyap_1.0.8_x64-setup.exe`.
- Ran the `1.0.8` installer silently and launched the installed app from `%LOCALAPPDATA%/hyperyap/hyperyap.exe`.
- Validation scope: local Windows build and installer validation completed; macOS/Linux validation must come from GitHub Actions runners.

## 2026-04-22 save | public release sanitization

- Created [[Public Release Sanitization]] modeled after the BenjaminTerm wiki policy.
- Added vault `.gitignore` coverage for local Obsidian state, private raw material, private attachments, secret-like files, and environment files.
- Replaced local owner metadata in `CODEX.md` with `<project-owner>`.
- Linked the sanitization policy from [[HyperYap Vault Index]], [[Dashboard]], and `CODEX.md`; no `Meta Index.md` file exists in this vault.
- Ran public-release checks for wikilinks, JSON config validity, wiki note frontmatter, and sensitive patterns.

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
