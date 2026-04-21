---
type: decision
title: "ADR 2026-04-21 Detached Installer Runs"
created: 2026-04-21
updated: 2026-04-21
tags:
  - hyperyap
  - decision
  - installer
  - codex
status: developing
related:
  - "[[HyperYap App]]"
  - "[[Hot Cache]]"
sources:
  - "[[Project Documentation]]"
---

# ADR 2026-04-21 Detached Installer Runs

## Context

Running HyperYap installer or relaunch commands directly from the active Codex terminal can spill app startup output, WebView errors, process noise, or other artifacts into the chat session. The user can still communicate, but the session becomes sloppy and harder to read.

## Decision

Future installer, upgrade, and relaunch tests must run detached from the active Codex terminal. Use a background PowerShell process, `Start-Process`, or a separate terminal window for installer execution and app relaunch. Keep the active Codex terminal focused on concise status checks and verification commands.

## Consequences

- Keeps Codex chat output clean during HyperYap install tests.
- Reduces confusion when installer or app processes emit noisy output.
- Requires explicit process checks after detached install or relaunch work.

## Validation

- Record process status after detached runs with `Get-Process hyperyap,hyperyap-hotkeys`.
- For installer builds, verify the installed app launches normally and does not show a localhost WebView error.
