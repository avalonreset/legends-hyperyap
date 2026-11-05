---
type: meta
title: "HyperYap Wiki Instructions"
created: 2026-04-21
updated: 2026-04-22
tags:
  - hyperyap
  - codex
  - operating-guide
status: active
related:
  - "[[Public Release Sanitization]]"
sources:
  - "[[Project Documentation]]"
---

# HyperYap Wiki

Mode: GitHub repository wiki
Purpose: Maintain a persistent Obsidian knowledge base for HyperYap architecture, releases, workflows, and decisions.
Owner: <project-owner>
Created: 2026-04-21

## Structure

```text
hyperyap-wiki/
├── .raw/
├── _attachments/
├── _templates/
└── wiki/
    ├── index.md
    ├── log.md
    ├── hot.md
    ├── overview.md
    ├── sources/
    ├── entities/
    ├── concepts/
    ├── modules/
    ├── flows/
    ├── decisions/
    ├── questions/
    └── meta/
```

## Operating Rules

- Read `wiki/hot.md` first for recent context.
- Read `wiki/index.md` before creating new notes.
- Keep `.raw/` as immutable source material.
- Use Obsidian wikilinks such as `[[Smart Screenshot Paste]]`.
- Every wiki note must have flat YAML frontmatter.
- Update `wiki/index.md`, `wiki/hot.md`, and `wiki/log.md` after meaningful changes.
- New log entries go at the top of `wiki/log.md`.
- Do not store private user data in this vault.
- Before public release, apply [[Public Release Sanitization]] and inspect sensitive-pattern scan hits.
- Preserve HyperYap project rules from `../CONTRIBUTING.md` and `../GUIDELINES.md`.

## HyperYap Constraints

- Processing must stay local.
- Do not persist user data beyond the last five transcriptions.
- Security shortcuts are not acceptable.
- Rust changes must pass `cargo fmt` and `cargo clippy`.
- Frontend changes must avoid `any` and follow feature-first structure.
- Windows and Ubuntu validation status should be recorded for release-impacting changes.
