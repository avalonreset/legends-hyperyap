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

The hotkey daemon should intercept terminal Ctrl+V using low-level Ctrl key tracking, wait briefly for image clipboard formats, and fail closed when neither image nor text paste is safe.

## Consequences

- Reduces stray `v` inserts during screenshot stress tests.
- Keeps normal text clipboard paste available.
- Needs a stale Ctrl guard so a later plain `v` is not intercepted after Ctrl has already been released.

## Validation

- Windows local install was updated to `v1.0.4`.
- GitHub release `v1.0.4` was created with installer asset.
- Ubuntu validation is still a release checklist item for future cross-platform changes.
