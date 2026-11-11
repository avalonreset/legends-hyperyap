#!/usr/bin/env python3
"""Release readiness and EDR/encoding hygiene validator for legends-hyperyap."""

from __future__ import annotations

import hashlib
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
    "NOTICE",
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
    "assets/social-preview.png",
    "assets/manifest.json",
    "docs/AGENT-RECIPE.md",
    "skills/cto-legends/SKILL.md",
]

FORBIDDEN_EXTS = {".ps1", ".bat", ".cmd", ".vbs", ".ahk", ".pem", ".key", ".p12", ".pfx"}
FORBIDDEN_NAMES = {
    "CLAUDE.md",
    "GEMINI.md",
    "CODEX.md",
    "GROK.md",
    "LEGENDS.md",
    "gemini-extension.json",
    "skill-package.json",
    ".env",
}
FORBIDDEN_PATTERNS = [
    "CREATE_NO_WINDOW",
    "0x08000000",
    "-WindowStyle Hidden",
    "windowsHide",
    "-ExecutionPolicy Bypass",
    "GetAsyncKeyState",
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
        if ctype in {"tEXt", "iTXt", "zTXt", "eXIf", "c2pa", "caBX"}:
            issues.append(f"{path}: forbidden metadata chunk {ctype}")
        pos += 12 + length
    return issues


def check_webp_chunks(path: Path) -> list[str]:
    issues: list[str] = []
    data = path.read_bytes()
    if len(data) < 12 or data[:4] != b"RIFF" or data[8:12] != b"WEBP":
        return [f"{path}: invalid WEBP signature"]
    pos = 12
    while pos + 8 <= len(data):
        fourcc = data[pos : pos + 4].decode("latin1", errors="replace")
        size = struct.unpack("<I", data[pos + 4 : pos + 8])[0]
        if fourcc in {"EXIF", "XMP "}:
            issues.append(f"{path}: forbidden WEBP metadata chunk {fourcc.strip()}")
        pos += 8 + size + (size % 2)
    return issues


def check_jpeg_markers(path: Path) -> list[str]:
    issues: list[str] = []
    data = path.read_bytes()
    if len(data) < 4 or data[:2] != b"\xff\xd8":
        return [f"{path}: invalid JPEG signature"]
    lower = data.lower()
    for marker in (b"c2pa", b"jumbf", b"exif\x00\x00", b"http://ns.adobe.com/xap/"):
        if marker in lower:
            issues.append(f"{path}: forbidden JPEG metadata marker {marker!r}")
    return issues


def check_markdown_links(md_path: Path, text: str) -> list[str]:
    issues: list[str] = []
    cleaned = re.sub(r"```.*?```", "", text, flags=re.S)
    for match in re.finditer(r"!?\[[^\]]*\]\(([^)]+)\)", cleaned):
        raw_target = match.group(1).strip()
        if not raw_target or raw_target.startswith(("#", "http://", "https://", "mailto:")):
            continue
        target_path = raw_target.split("#", 1)[0].split("?", 1)[0].strip()
        if not target_path:
            continue
        resolved = (md_path.parent / target_path).resolve()
        if not resolved.exists():
            rel_md = md_path.relative_to(ROOT).as_posix()
            issues.append(f"Broken relative link in {rel_md}: {raw_target}")
    return issues


def check_assets_manifest() -> list[str]:
    issues: list[str] = []
    manifest_path = ROOT / "assets" / "manifest.json"
    if not manifest_path.is_file():
        return ["Missing assets/manifest.json"]
    try:
        manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
    except json.JSONDecodeError as exc:
        return [f"Invalid JSON in assets/manifest.json: {exc}"]

    files = manifest.get("files", [])
    if not files:
        issues.append("assets/manifest.json has no 'files' entries")
    for entry in files:
        rel_path = entry.get("path", "")
        target = ROOT / rel_path
        if not target.is_file():
            issues.append(f"assets/manifest.json references missing file: {rel_path}")
            continue
        data = target.read_bytes()
        digest = hashlib.sha256(data).hexdigest()
        if entry.get("bytes") != len(data):
            issues.append(f"assets/manifest.json byte size mismatch for {rel_path}: {entry.get('bytes')} != {len(data)}")
        if entry.get("sha256") != digest:
            issues.append(f"assets/manifest.json sha256 mismatch for {rel_path}")
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

    issues.extend(check_assets_manifest())

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
            issues.append(f"Forbidden script/secret extension: {rel_str}")
        if p.suffix.lower() == ".png":
            issues.extend(check_png_chunks(p))
        elif p.suffix.lower() == ".webp":
            issues.extend(check_webp_chunks(p))
        elif p.suffix.lower() in {".jpg", ".jpeg"}:
            issues.extend(check_jpeg_markers(p))
        if p.suffix.lower() in {".py", ".md", ".toml", ".yml", ".yaml", ".json", ".cff", ".sh", ".rs", ".ts", ".tsx"}:
            try:
                text = p.read_bytes().decode("utf-8")
            except UnicodeDecodeError:
                issues.append(f"Non-UTF-8 file: {rel_str}")
                continue
            if rel_str != "scripts/check_release.py":
                for pat in FORBIDDEN_PATTERNS:
                    if pat in text and rel_str != "CHANGELOG.md":
                        if (
                            pat == "GetAsyncKeyState"
                            and rel_str == "src-tauri/src/shortcuts/platform_windows.rs"
                        ):
                            continue
                        issues.append(f"Forbidden pattern '{pat}' in {rel_str}")
                for pat in MOJIBAKE_PATTERNS:
                    if pat in text:
                        issues.append(f"Mojibake pattern in {rel_str}")
            if p.suffix.lower() == ".md":
                if "\u2014" in text or "\u2013" in text:
                    issues.append(f"Em/en-dash found in markdown file: {rel_str}")
                issues.extend(check_markdown_links(p, text))

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
