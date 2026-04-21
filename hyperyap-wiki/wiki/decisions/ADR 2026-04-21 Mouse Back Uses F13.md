---
type: decision
title: "ADR 2026-04-21 Mouse Back Uses F13"
created: 2026-04-21
updated: 2026-04-21
tags:
  - hyperyap
  - decision
  - input
status: developing
related:
  - "[[Hotkey Daemon]]"
sources:
  - "[[Project Documentation]]"
---

# ADR 2026-04-21 Mouse Back Uses F13

## Context

Mouse Back had been used successfully through F13 translation. A direct mouse button approach broke the desired back button behavior and could not be configured cleanly in toggle-to-talk.

## Decision

Keep Mouse Back translation to F13 for the user's toggle-to-talk workflow.

## Consequences

- Preserves the known working input path.
- Avoids requiring direct Mouse Button 4 support in toggle-to-talk configuration.
- Future hotkey changes should preserve this behavior unless the user explicitly asks to replace it.
