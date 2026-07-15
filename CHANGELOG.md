# Changelog

All notable changes to this project are documented here.

## [0.1.0-rc.1] - 2026-07-16

### Added

- Diagnostics-only `agy-auth doctor` command with human and JSON output.
- Safe discovery and bounded `agy --version` probing, verified with Antigravity CLI 1.1.2.
- Read-only non-secret registry validation and filesystem ownership/permission checks.
- Interrupted project-owned transaction detection without exposing marker names or contents.
- Stable exit codes for missing clients, unsafe state, and required recovery.
- Reproducible Linux archive, SHA-256 manifest, SPDX 2.3 SBOM, and embedded build identity.
- MIT license and local installation/uninstallation documentation.

### Security boundaries

- Profile switching and authentication-state mutation are disabled.
- No OAuth implementation, keyring access, credential-file access, backend calls, quota handling, or
  automatic account rotation.

### Known limitations

- Only `x86_64-unknown-linux-gnu` has local release-candidate evidence.
- macOS, Windows, Linux ARM64, and hosted CI remain unverified.
- Antigravity CLI exposes no supported profile-isolation contract, so `add`, `use`, and profile
  `exec` are unavailable.
