---
type: decision
title: "ADR 2026-04-21 Smart Paste Guard"
created: 2026-04-21
updated: 2026-04-21
tags:
  - hyperyap
  - decision
  - smart-paste
status: developing
related:
  - "[[Smart Screenshot Paste]]"
  - "[[Hotkey Daemon]]"
sources:
  - "[[Project Documentation]]"
---

# ADR 2026-04-21 Smart Paste Guard

## Context

Fast screenshot then Ctrl+V in Benjamin Term could race the Windows clipboard. When the screenshot image was not ready yet, the terminal could receive a literal `v`.

## Decision

The hotkey daemon should intercept terminal Ctrl+V using low-level Ctrl key tracking, wait briefly for image clipboard formats, and fail closed when neither image nor text paste is safe. It should also track recent Windows screenshot shortcuts and use the terminal paste chord after converting an image to a path, so vanilla WezTerm does not receive the original Ctrl+V command.

## Consequences

- Reduces stray `v` inserts during screenshot stress tests.
- Keeps normal text clipboard paste available.
- Needs a stale Ctrl guard so a later plain `v` is not intercepted after Ctrl has already been released.
- Allows vanilla WezTerm compatibility without depending on Benjamin Term's custom paste behavior.

## Validation

- Windows local install was updated to `v1.0.4`.
- GitHub release `v1.0.4` was created with installer asset.
- Windows local hotkey daemon was updated with the `v1.0.5` WezTerm paste behavior.
- Ubuntu validation is still a release checklist item for future cross-platform changes.
