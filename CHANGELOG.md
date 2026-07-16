# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project
adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Production profile workflow with `add`, delegated `login`, `list`, `switch`, `exec`, `hint`, and
  `recover` commands.
- Interactive terminal-safe account selector with navigation, explicit cancellation, selected
  profile indication, masked account hints, captured client version, and last activity.
- Automatic profile naming when `add` or `login` is called without a name.
- Automatic masked account hints derived from bounded official-client successful-login logs.
- Transactional default-home account activation for subsequent plain `agy` launches.
- Project-owned selected-profile metadata and last-activity tracking.
- Interrupted import journaling and idempotent recovery.
- Verified Linux SSH compatibility contracts for Antigravity CLI `1.1.2` and `1.1.3`.

### Changed

- Promoted profile management from hidden experimental composition to the default CLI feature.
- Updated capability diagnostics to report verified switching and authentication mutation for
  supported Linux/client combinations.
- Changed `switch` from an isolated-client launcher alias to an atomic default-state selection
  command that does not launch `agy`.
- Changed `list` to use `NAME`, `ACCOUNT`, `VERSION`, and `LAST ACTIVITY` columns.
- Changed delegated login completion to wait for official consumer onboarding and terminate the
  client gracefully before workspace prompts.

### Fixed

- Prevented delegated login from stopping immediately after early credential persistence.
- Normalized official-client-created credential ancestors from non-writable `0755` to managed
  owner-only `0700` before validation.
- Restored terminal mode and cursor state on every interactive-selector exit path.
- Prevented diagonal terminal output by using raw-mode-safe line endings.
- Stopped the selector from clearing terminal history.
- Added rollback when selected-profile metadata cannot be persisted after credential activation.

### Security

- Full account emails are discarded after masking and never stored in the registry.
- Official default credential replacement validates ownership, permissions, file type, link count,
  bounded size, and same-directory atomic replacement.
- Private backend usage/quota calls and automatic account rotation remain explicitly excluded.

## [0.1.0-rc.1] - 2026-07-15

### Added

- Diagnostics-only `agy-auth doctor` command with human-readable and JSON output.
- Safe official-client discovery and bounded `agy --version` probing.
- Read-only non-secret registry validation.
- Data-directory ownership, permission, type, and interrupted-transaction diagnostics.
- Stable exit codes for missing clients, unsafe state, and required recovery.
- Linux x86_64 release archive, SHA-256 manifest, SPDX 2.3 SBOM, and embedded build identity.
- MIT license and local installation documentation.

### Changed

- Selected Google Antigravity CLI as the primary provider target.
- Established capability-gated architecture and strict CLI → application → domain/ports dependency
  direction.

### Fixed

- Rejected unsafe registry directories, symlinks, malformed documents, and unsupported schemas
  without exposing local paths or private state.

### Security

- Shipped with profile switching and authentication-state mutation disabled.
- Added explicit prohibitions on OAuth implementation, backend calls, quota handling, credential
  sharing, and automatic account rotation.

[Unreleased]: https://github.com/ensp1re/agy-auth/compare/v0.1.0-rc.1...HEAD
[0.1.0-rc.1]: https://github.com/ensp1re/agy-auth/releases/tag/v0.1.0-rc.1
