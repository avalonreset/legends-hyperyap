---
type: entity
title: "Benjamin Term"
created: 2026-04-21
updated: 2026-04-24
tags:
  - hyperyap
  - terminal
status: seed
related:
  - "[[Smart Screenshot Paste]]"
sources:
  - "[[Project Documentation]]"
entity_type: product
role: "Supported terminal target for screenshot path paste"
first_mentioned: "[[Project Documentation]]"
---

# Benjamin Term

Benjamin Term is a supported companion terminal for HyperYap's Windows terminal workflow. HyperYap smart screenshot paste treats it as a terminal target where Ctrl+V should paste a screenshot file path when the clipboard contains a Windows screenshot image.

## Release Note

HyperYap does not embed BenjaminTerm in the app bundle. The Windows PowerShell installer resolves the latest stable `avalonreset/benjaminterm` GitHub release and installs its setup asset when available. As of 2026-04-24, that release line is BenjaminTerm `v1.4.3`.

## Implementation Note

Terminal detection currently matters because normal text paste should continue working, while image clipboard content needs to become a path before paste.
