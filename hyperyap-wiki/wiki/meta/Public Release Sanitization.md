---
type: meta
title: "Public Release Sanitization"
created: 2026-04-22
updated: 2026-04-24
tags:
  - hyperyap
  - sanitization
  - public-release
  - security
status: active
related:
  - "[[HyperYap Vault Index]]"
  - "[[Dashboard]]"
  - "[[Privacy First Local Processing]]"
sources:
  - "[[Project Documentation]]"
---

# Public Release Sanitization

This vault must be useful to future HyperYap developers and safe to publish with the project.

## Do Not Publish

- API keys, access tokens, passwords, credentials, private keys, signing keys, recovery codes, or `.env` values.
- Local usernames, machine names, home directories, private drive layouts, or private absolute paths.
- Private screenshots, private transcripts, private logs, raw conversation dumps, or personal account details.
- Private repo URLs, provider account details, or release credentials that are not already public project facts.

## Safe Patterns

- Use `<repo-root>` instead of a local absolute repository path.
- Use `<vault-root>` instead of a local absolute vault path.
- Use `<project-owner>`, `<username>`, or `<machine-name>` for private identity or host details.
- Use relative repo paths such as `src-tauri/src/lib.rs` and `wiki/hot.md`.
- Describe external credentials generically, such as "`GH_PAT` secret" or "release signing credentials," without including values.
- Keep durable, project-level decisions in `wiki/`; keep private source material out of the vault or under ignored private folders.

## Vault Ignore Rules

The vault `.gitignore` must exclude:

- Local Obsidian workspace state.
- Private raw transcripts, screenshots, data, and assets.
- Private attachments.
- Environment files and secret-like key material.

Tracked `.gitkeep` files may remain in empty private-prone folders so the directory structure is visible without publishing private contents.

## Pre-Publish Scan

Run from the repository root:

```powershell
rg -n "C:\\|E:\\|Users\\|API[_ -]?KEY|SECRET|TOKEN|PASSWORD|BEGIN (RSA|OPENSSH|PRIVATE)|sk-[A-Za-z0-9]|github_pat_|ghp_|gho_|AIza|AKIA|-----BEGIN" <vault-root>
```

Also scan for project-local identities and placeholder mistakes:

```powershell
rg -n "Owner:|C:/|E:/|C:\\|E:\\|Users\\|\.env|password|token|secret|api key" <vault-root>
```

Expected acceptable hits:

- This page may mention sensitive pattern names as documentation.
- Generic secret names such as `GH_PAT` are acceptable when no secret value is present.
- Public HyperYap repository and release URLs are acceptable project facts.

## Review Checklist

- Broken wikilinks are checked.
- JSON canvas, bookmark, and Obsidian config files parse successfully.
- Wiki markdown notes have YAML frontmatter.
- Sensitive-pattern hits outside this policy are inspected and replaced with placeholders when they are local or private.
- Public-release changes are recorded in [[Vault Log]] and summarized in [[Hot Cache]].

## Current Status

2026-04-24: Public-release scan found no secret values and no private local path leaks outside this policy page. Stale `v1.0.8` release status was updated to `v1.0.9`, and a personal phrasing in [[Benjamin Term]] was replaced with product-level wording. See [[Lint Report 2026-04-24]].

2026-04-22: Initial scan found one local identity detail in vault operating instructions. It was replaced with `Owner: <project-owner>`. No secret values were found outside this policy.
