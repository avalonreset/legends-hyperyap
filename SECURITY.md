# Security Policy

## Supported Versions

| Version | Supported          |
|---------|--------------------|
| 1.0.x   | :white_check_mark: |

## Reporting a Vulnerability

If you discover a security vulnerability in HyperYap, please report it responsibly.

**Do not open a public issue.** Instead:

1. Email **avalonreset** via [GitHub private vulnerability reporting](https://github.com/avalonreset/hyperyap/security/advisories/new)
2. Include a description of the vulnerability, steps to reproduce, and any relevant logs or screenshots

### What to expect

- **Acknowledgment** within 48 hours
- **Status update** within 7 days
- **Fix timeline** depends on severity:
  - Critical (remote code execution, data exfiltration): patch within 7 days
  - High (privilege escalation, auth bypass): patch within 14 days
  - Medium (information disclosure, denial of service): patch within 30 days
  - Low (minor issues): addressed in next regular release

### Scope

The following are in scope:

- HyperYap desktop application (Tauri/Rust backend, React frontend)
- AutoHotkey hotkey scripts (`presets/scripts/`)
- PowerShell installer (`install.ps1`)
- HTTP API (when enabled in settings)

The following are out of scope:

- NVIDIA Parakeet speech model (report to NVIDIA)
- BenjaminTerm terminal (report to [BenjaminTerm](https://github.com/avalonreset/benjaminterm))
- Upstream MURmure issues (report to [MURmure](https://github.com/Kieirra/murmure))

## Security Design

HyperYap is designed with privacy and security in mind:

- All speech recognition runs locally -- no audio data leaves your machine
- No telemetry, analytics, or data collection
- HTTP API is disabled by default and binds to localhost only
- Clipboard access is temporary (original content is restored after paste)
