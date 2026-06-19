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
role: "Decommissioned former companion terminal target"
first_mentioned: "[[Project Documentation]]"
---

# Benjamin Term

Benjamin Term was previously a supported companion terminal for HyperYap's Windows terminal workflow. As of the `v1.0.10` release prep, HyperYap no longer installs, launches, documents, or targets BenjaminTerm as a current companion dependency.

## Release Note

Historical note: the `v1.0.9` Windows PowerShell installer resolved the latest stable `avalonreset/benjaminterm` GitHub release and installed its setup asset when available. That behavior was removed for `v1.0.10`.

## Implementation Note

Terminal detection currently matters because normal text paste should continue working, while image clipboard content needs to become a path before paste.
