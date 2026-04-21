---
type: entity
title: "Benjamin Term"
created: 2026-04-21
updated: 2026-04-21
tags:
  - hyperyap
  - terminal
status: seed
related:
  - "[[Smart Screenshot Paste]]"
sources:
  - "[[Project Documentation]]"
entity_type: product
role: "Preferred terminal target for screenshot path paste"
first_mentioned: "[[Project Documentation]]"
---

# Benjamin Term

Benjamin Term is the user's preferred custom terminal. HyperYap smart screenshot paste treats it as a terminal target where Ctrl+V should paste a screenshot file path when the clipboard contains a Windows screenshot image.

## Implementation Note

Terminal detection currently matters because normal text paste should continue working, while image clipboard content needs to become a path before paste.
