# agy-auth

[![CI](https://github.com/ensp1re/agy-auth/actions/workflows/ci.yml/badge.svg)](https://github.com/ensp1re/agy-auth/actions/workflows/ci.yml)
[![Release](https://img.shields.io/github/v/release/ensp1re/agy-auth?include_prereleases)](https://github.com/ensp1re/agy-auth/releases)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/Rust-1.88%2B-orange.svg)](rust-toolchain.toml)

`agy-auth` is a local account-profile manager for Google Antigravity CLI (`agy`). It lets one person
enroll, identify, and manually switch between their own Google account contexts without repeatedly
logging out of the official client.

The official `agy` binary still owns OAuth login, token refresh, model requests, entitlements, and
Google policy enforcement. `agy-auth` does not implement Google OAuth, call private model or quota
APIs, pool usage, or rotate accounts automatically.

> [!IMPORTANT]
> The switching contract is reverse engineered, not supported by Google. It is currently verified
> only for Antigravity CLI `1.1.2` and `1.1.3` on Linux SSH/headless systems. Other versions and
> platforms fail closed.

## Features

- Enroll additional accounts through the official interactive `agy` login.
- Capture the account already logged into the official default home.
- Switch the account used by the next plain `agy` launch without opening the client.
- Browse profiles in a terminal-safe interactive selector.
- Show profile name, masked account hint, captured client version, selection, and last activity.
- Launch an isolated one-off `agy` session with `agy-auth exec`.
- Preserve refresh-token rotation and recover interrupted profile imports.
- Refuse unsafe ownership, permissions, links, schemas, versions, and concurrent profile sessions.

## Requirements

- Linux x86_64 SSH/headless environment.
- Rust `1.88` or newer when building from source.
- Official Antigravity CLI executable available as `agy`.
- Verified `agy` version `1.1.2` or `1.1.3`.

Desktop keyrings, macOS, Windows, Linux ARM64, and unverified `agy` versions are not currently
supported for authentication-state mutation.

## Installation

### Install the prebuilt Linux release

```bash
curl --proto '=https' --tlsv1.2 -LsSf \
  https://github.com/ensp1re/agy-auth/releases/download/v0.2.0-rc.1/agy-auth-installer.sh | sh
```

The installer supports Linux x86_64, verifies the archive SHA-256, and installs to
`"$HOME/.local/bin"`. To inspect it first:

```bash
curl --proto '=https' --tlsv1.2 -fLO \
  https://github.com/ensp1re/agy-auth/releases/download/v0.2.0-rc.1/agy-auth-installer.sh
less agy-auth-installer.sh
sh agy-auth-installer.sh
```

### Build from source

```bash
git clone https://github.com/ensp1re/agy-auth.git
cd agy-auth
cargo build --release -p agy-auth-cli
install -d -m 0700 "$HOME/.local/bin"
install -m 0755 target/release/agy-auth "$HOME/.local/bin/agy-auth"
```

Ensure `"$HOME/.local/bin"` is on `PATH`, then verify the installation:

```bash
agy-auth --version
agy-auth doctor
```

Alternatively, with Rust already installed:

```bash
cargo install --locked --git https://github.com/ensp1re/agy-auth \
  --tag v0.2.0-rc.1 agy-auth-cli
```

### Release artifacts

Published release candidates, checksums, SBOMs, and installers are available from
[GitHub Releases](https://github.com/ensp1re/agy-auth/releases).

## Quick start

Save the account currently logged into the default official `agy` home:

```bash
agy-auth add personal
```

Enroll another account through official `agy`:

```bash
agy-auth login work
```

Names are optional. If omitted, `agy-auth` selects the first unused `profileN` name:

```bash
agy-auth login
```

Open the interactive account selector:

```bash
agy-auth list
```

Use Up/Down to navigate, Enter to switch, and Esc, `q`, or Ctrl+C to exit without switching.
For deterministic script-friendly output:

```bash
agy-auth list --plain
agy-auth --json list
```

Switch directly:

```bash
agy-auth switch work
agy
```

`switch` does not launch `agy`; the following plain `agy` command uses the selected account.

Run one isolated session without changing the default account:

```bash
agy-auth exec personal
```

Diagnose local state or recover an interrupted import:

```bash
agy-auth doctor
agy-auth recover
```

## Command overview

| Command | Purpose |
|---|---|
| `agy-auth doctor` | Diagnose client compatibility, registry safety, and recovery state |
| `agy-auth list` | Interactively browse and switch profiles |
| `agy-auth list --plain` | Print deterministic profile metadata |
| `agy-auth add [name]` | Save the account in the current official `agy` home |
| `agy-auth login [name]` | Enroll another account through official `agy` |
| `agy-auth switch <name>` | Select the account used by future plain `agy` launches |
| `agy-auth exec <name>` | Launch an isolated one-off session |
| `agy-auth hint <name> <masked-hint>` | Set a privacy-preserving account hint |
| `agy-auth recover` | Recover interrupted profile imports |

## How it works

```text
agy-auth CLI
    │
    ├── non-secret registry and selected-profile metadata
    ├── transaction and permission checks
    └── version-gated Antigravity adapter
             │
             └── official agy owns login, refresh, and model traffic
```

Each profile has an owner-only managed home. During login, `agy-auth` launches the official client
inside a fresh isolated home, waits for verified credential and onboarding completion, then records
only the reviewed profile state. Account hints are masked before entering the registry.

Switching validates the installed client version, atomically replaces the verified official default
credential envelope, records the selected profile, and rolls back if metadata persistence fails.
The provider describes the versioned envelope; the storage layer owns filesystem mutation.

See the [architecture](docs/04-architecture.md), [CLI specification](docs/07-cli-specification.md),
and [architecture decisions](docs/adr/) for the complete contract.

## Security model

`agy-auth` is intentionally local and manual:

- no Google passwords or independent OAuth implementation;
- no private Gemini, Code Assist, quota, or entitlement API calls;
- no automatic fallback, scoring, load balancing, or account rotation;
- no credential synchronization or shared vault service;
- no full emails, tokens, authorization URLs, or private logs in routine output;
- owner-only directories and files with link, ownership, and permission checks.

This reduces accidental exposure but cannot protect credentials from malware or an attacker already
running as the same operating-system user.

Please read [SECURITY.md](SECURITY.md) before reporting a vulnerability.

## Development

Run the complete local gate:

```bash
python3 scripts/check.py
```

The gate covers formatting, Clippy, tests, dependency direction, compatibility metadata, release
preflight, secret scanning, and harness validation.

See [CONTRIBUTING.md](CONTRIBUTING.md) for contribution requirements.

## License

Licensed under the [MIT License](LICENSE). Copyright © 2026 Enspire.
