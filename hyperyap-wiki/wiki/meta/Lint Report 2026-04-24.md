---
type: meta
title: "Lint Report 2026-04-24"
created: 2026-04-24
updated: 2026-04-24
tags:
  - hyperyap
  - lint
  - public-release
status: complete
related:
  - "[[Public Release Sanitization]]"
  - "[[Hot Cache]]"
  - "[[Dashboard]]"
sources:
  - "[[Project Documentation]]"
---

# Lint Report 2026-04-24

## Scope

Public-release scan for the HyperYap wiki before shipping the `v1.0.9` release notes and BenjaminTerm `v1.4.3` installer path.

## Sensitive Content Scan

Scanned for secret-like terms, credentials, private local paths, local usernames, machine names, and private host references.

Result: no secret values were found. The only sensitive-pattern hits were the examples documented in [[Public Release Sanitization]] and a generic log statement about ignored secret-like files.

## Release Fact Scan

Stale `v1.0.8` status appeared in [[Hot Cache]], [[Dashboard]], [[HyperYap Overview]], and [[Terminal Editing Layer]]. These notes were updated to distinguish:

- `v1.0.8`: terminal editing behavior release.
- `v1.0.9`: release path for installing latest stable BenjaminTerm, currently `v1.4.3`.

## Sanitization Fixes

- Replaced personal phrasing in [[Benjamin Term]] with product-level wording.
- Recorded that BenjaminTerm is resolved dynamically by the Windows PowerShell installer, not embedded in the HyperYap app bundle.
- Added current workflow status caveat: `.release-trigger` was pushed, but GitHub Actions run status still needs confirmation outside this blocked shell.
- Replaced a stale missing `project-source-map` wikilink in [[Project Documentation]] with an existing vault index link.

## Remaining Checks

- Confirm the `v1.0.9` GitHub Actions release run completed.
- Confirm the GitHub Release contains Windows, Linux, macOS Apple Silicon, and macOS Intel artifacts.
- Validate the published Windows installer on Windows.
