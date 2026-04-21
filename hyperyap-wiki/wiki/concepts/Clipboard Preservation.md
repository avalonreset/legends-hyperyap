---
type: concept
title: "Clipboard Preservation"
created: 2026-04-21
updated: 2026-04-21
tags:
  - hyperyap
  - clipboard
status: developing
related:
  - "[[Smart Screenshot Paste]]"
  - "[[Push To Talk Transcription]]"
sources:
  - "[[Project Documentation]]"
complexity: intermediate
domain: "HyperYap"
---

# Clipboard Preservation

Clipboard preservation is the pattern of saving the user's current clipboard, replacing it just long enough to paste HyperYap output, then restoring the original clipboard content.

## Why It Matters

The clipboard often contains sensitive or useful user data. HyperYap should minimize the time it owns clipboard content and should restore the user's previous state when possible.
