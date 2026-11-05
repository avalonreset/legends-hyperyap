#!/usr/bin/env python3
"""Release readiness and EDR/encoding hygiene validator for legends-hyperyap."""

from __future__ import annotations

import json
import re
import struct
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent

REQUIRED_FILES = [
    ".legends-module",
    ".legends-router-pin",
    "AGENTS.md",
    "CHANGELOG.md",
    "CITATION.cff",
    "CODE_OF_CONDUCT.md",
    "CONTRIBUTING.md",
    "LICENSE",
    "README.md",
    "SECURITY.md",
    "THIRD_PARTY_NOTICES.md",
    "package.json",
    "src-tauri/Cargo.toml",
    "src-tauri/tauri.conf.json",
    "hotkeys/Cargo.toml",
    "assets/banner.png",
    "assets/banner.webp",
    "assets/preview.png",
    "assets/manifest.json",
    "docs/AGENT-RECIPE.md",
    "skills/cto-legends/SKILL.md",
]

FORBIDDEN_EXTS = {".ps1", ".bat", ".cmd", ".vbs", ".ahk"}
FORBIDDEN_NAMES = {
    "CLAUDE.md",
    "GEMINI.md",
    "CODEX.md",
    "GROK.md",
    "LEGENDS.md",
    "gemini-extension.json",
    "skill-package.json",
}
FORBIDDEN_PATTERNS = [
    "CREATE_NO_WINDOW",
    "0x08000000",
    "-WindowStyle Hidden",
    "windowsHide",
    "-ExecutionPolicy Bypass",
    "HKCU:\\Software\\Microsoft\\Windows\\CurrentVersion\\Run",
    "HKLM:\\Software\\Microsoft\\Windows\\CurrentVersion\\Run",
    "schtasks /create",
]
MOJIBAKE_PATTERNS = [
    "\u00e2\u20ac\u201d",
    "\u00e2\u20ac\u201c",
    "\u00e2\u20ac\u2122",
    "\u00e2\u20ac\u0153",
    "\u00e2\u2020\u2019",
    "\u00c3\u00a9",
    "\ufffd",
]

SKIP_DIRS = {
    ".git",
    ".venv",
    "venv",
    "node_modules",
    "dist",
    "build",
    "target",
    "resources",
    "__pycache__",
}


def check_png_chunks(path: Path) -> list[str]:
    issues: list[str] = []
    data = path.read_bytes()
    if data[:8] != b"\x89PNG\r\n\x1a\n":
        return [f"{path}: invalid PNG signature"]
    pos = 8
    while pos < len(data):
        length = struct.unpack(">I", data[pos : pos + 4])[0]
        ctype = data[pos + 4 : pos + 8].decode("latin1")
        if ctype in {"tEXt", "iTXt", "zTXt", "eXIf"}:
            issues.append(f"{path}: forbidden metadata chunk {ctype}")
        pos += 12 + length
    return issues


def main() -> int:
    issues: list[str] = []

    for rel in REQUIRED_FILES:
        if not (ROOT / rel).is_file():
            issues.append(f"Missing required file: {rel}")

    pkg_ver = json.loads((ROOT / "package.json").read_text(encoding="utf-8"))["version"]
    tauri_ver = json.loads((ROOT / "src-tauri/tauri.conf.json").read_text(encoding="utf-8"))["version"]
    cargo_text = (ROOT / "src-tauri/Cargo.toml").read_text(encoding="utf-8")
    hotkeys_text = (ROOT / "hotkeys/Cargo.toml").read_text(encoding="utf-8")
    cff_text = (ROOT / "CITATION.cff").read_text(encoding="utf-8")
    changelog_text = (ROOT / "CHANGELOG.md").read_text(encoding="utf-8")

    m_cargo = re.search(r'^version\s*=\s*"([^"]+)"', cargo_text, re.M)
    m_hotkeys = re.search(r'^version\s*=\s*"([^"]+)"', hotkeys_text, re.M)
    m_cff = re.search(r'^version:\s*"?([0-9]+\.[0-9]+\.[0-9]+)"?', cff_text, re.M)
    m_chg = re.search(r"^##\s+\[?v?([0-9]+\.[0-9]+\.[0-9]+)\]?", changelog_text, re.M)

    versions = {
        "package.json": pkg_ver,
        "src-tauri/tauri.conf.json": tauri_ver,
        "src-tauri/Cargo.toml": m_cargo.group(1) if m_cargo else None,
        "hotkeys/Cargo.toml": m_hotkeys.group(1) if m_hotkeys else None,
        "CITATION.cff": m_cff.group(1) if m_cff else None,
        "CHANGELOG.md": m_chg.group(1) if m_chg else None,
    }
    if len(set(versions.values())) != 1 or None in versions.values():
        issues.append(f"Version mismatch across files: {versions}")

    skill_files = []
    for p in ROOT.rglob("*"):
        if any(part in SKIP_DIRS for part in p.parts):
            continue
        if not p.is_file():
            continue
        rel_str = p.relative_to(ROOT).as_posix()
        if p.name == "SKILL.md":
            skill_files.append(rel_str)
        if p.name in FORBIDDEN_NAMES:
            issues.append(f"Forbidden filename: {rel_str}")
        if p.suffix.lower() in FORBIDDEN_EXTS:
            issues.append(f"Forbidden script extension: {rel_str}")
        if p.suffix.lower() == ".png":
            issues.extend(check_png_chunks(p))
        if p.suffix.lower() in {".py", ".md", ".toml", ".yml", ".yaml", ".json", ".cff", ".sh", ".rs", ".ts", ".tsx"}:
            try:
                text = p.read_bytes().decode("utf-8")
            except UnicodeDecodeError:
                issues.append(f"Non-UTF-8 file: {rel_str}")
                continue
            if rel_str != "scripts/check_release.py":
                for pat in FORBIDDEN_PATTERNS:
                    if pat in text:
                        issues.append(f"Forbidden pattern '{pat}' in {rel_str}")
                for pat in MOJIBAKE_PATTERNS:
                    if pat in text:
                        issues.append(f"Mojibake pattern in {rel_str}")
            if p.suffix.lower() == ".md":
                if "\u2014" in text or "\u2013" in text:
                    issues.append(f"Em/en-dash found in markdown file: {rel_str}")

    if skill_files != ["skills/cto-legends/SKILL.md"]:
        issues.append(f"One-Skill Rule violation: found SKILL.md files {skill_files}")

    if issues:
        for i in issues:
            print(f"FAIL: {i}", file=sys.stderr)
        return 1

    print("Release checks: PASS (0 issues)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
