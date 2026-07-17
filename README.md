# agy-auth

[![CI](https://github.com/ensp1re/agy-auth/actions/workflows/ci.yml/badge.svg)](https://github.com/ensp1re/agy-auth/actions/workflows/ci.yml)
[![Release](https://img.shields.io/github/v/release/ensp1re/agy-auth?include_prereleases)](https://github.com/ensp1re/agy-auth/releases)
[![Rust 1.88+](https://img.shields.io/badge/Rust-1.88%2B-orange.svg)](rust-toolchain.toml)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

Switch between your personal and work accounts in Google Antigravity CLI without repeatedly using
`/logout`.

`agy-auth` saves local account profiles, opens new sign-ins through the official `agy` client, and
lets you choose which account the next plain `agy` launch will use. Login, token refresh, model
requests, entitlements, and Google policy enforcement remain owned by the official client.

> [!IMPORTANT]
> Account switching uses a reverse-engineered local contract that is not supported by Google. It is
> currently verified only for Antigravity CLI `1.1.2` and `1.1.3` on Linux x86_64 SSH/headless
> systems. Unsupported versions and platforms fail closed.

## At a glance

```console
$ agy-auth list --plain
   NAME       ACCOUNT             VERSION   LAST ACTIVITY
   personal   arc***@gmail.com    1.1.3     2m ago
-> work       ale***@gmail.com    1.1.3     now

$ agy-auth switch personal
Switched to personal. Run `agy` to start.
```

### What it does

- Saves the account already logged into the default `agy` home.
- Enrolls another account through the official interactive `agy` login.
- Switches the account used by future plain `agy` launches without opening the client.
- Provides a terminal-safe interactive profile selector.
- Runs one-off isolated sessions without changing the selected default account.
- Preserves refresh-token rotation and recovers interrupted imports.
- Rejects unsafe permissions, links, schemas, versions, and concurrent sessions.

### What it does not do

- Implement Google OAuth or ask for your Google password.
- Call private Gemini, Code Assist, quota, or entitlement APIs.
- Display account quotas or choose accounts based on remaining usage.
- Share credentials, synchronize profiles, or provide a hosted vault.
- Automatically rotate, balance, or fall back between accounts.

## Platform support

| Platform | Build and tests | Diagnostics | Account profile workflows |
|---|:---:|:---:|:---:|
| Linux x86_64 SSH/headless | ✅ | ✅ | ✅ `agy` 1.1.2–1.1.3 |
| Windows | ✅ | Build verified | ❌ fails closed |
| Linux ARM64 | Not tested | Not tested | ❌ fails closed |

Windows release compilation is exercised in CI, while installed-client diagnostics have not been
runtime validated there. Saving, enrolling, executing, and switching profiles remain disabled on
Windows until its official-client storage contract is independently verified.

## Installation

### Linux — prebuilt release

The installer downloads the pinned Linux x86_64 release, verifies its SHA-256 checksum, and places
`agy-auth` in `~/.local/bin`:

```bash
curl --proto '=https' --tlsv1.2 -fsSL \
  https://github.com/ensp1re/agy-auth/releases/download/v0.2.1/agy-auth-installer.sh |
  sh
```

To review the installer before running it:

```bash
curl --proto '=https' --tlsv1.2 -fLO \
  https://github.com/ensp1re/agy-auth/releases/download/v0.2.1/agy-auth-installer.sh
less agy-auth-installer.sh
sh agy-auth-installer.sh
```

If necessary, add the installation directory to your shell:

```bash
echo 'export PATH="$HOME/.local/bin:$PATH"' >> ~/.profile
export PATH="$HOME/.local/bin:$PATH"
```

### Windows — install with Cargo

Install Rust with the MSVC toolchain from [rustup.rs](https://rustup.rs/). The installer may prompt
for the Visual Studio C++ build tools if they are not already available.

Open a new PowerShell window, then run:

```powershell
rustup default stable-msvc
cargo install --locked `
  --git https://github.com/ensp1re/agy-auth `
  --tag v0.2.1 `
  agy-auth-cli
```

Cargo installs the executable into `%USERPROFILE%\.cargo\bin`, which rustup normally adds to
`PATH`.

### Linux or Windows — build from a checkout

Requires Git and Rust `1.88` or newer:

```bash
git clone https://github.com/ensp1re/agy-auth.git
cd agy-auth
cargo build --locked --release -p agy-auth-cli
```

The executable is written to:

- Linux: `target/release/agy-auth`
- Windows: `target\release\agy-auth.exe`

### Verify the installation

```bash
agy-auth --version
agy-auth doctor
```

`doctor` reports the installed `agy` version, profile registry health, interrupted transactions, and
whether account mutation is enabled for the current platform.

## Quick start

> [!NOTE]
> The profile workflow is production-enabled only on the verified Linux environment in the
> [platform support](#platform-support) table.

First, sign in normally with the official client:

```bash
agy
```

Exit `agy`, then save that account:

```bash
agy-auth add personal
```

Enroll a second account through official `agy`:

```bash
agy-auth login work
```

Names are optional. Without one, `agy-auth` chooses the first unused `profileN` name:

```bash
agy-auth login
```

Open the interactive selector:

```bash
agy-auth list
```

Use Up/Down to navigate, Enter to switch, and Esc, `q`, or Ctrl+C to exit without changing the
selection.

Switch directly and start the official client:

```bash
agy-auth switch work
agy
```

`switch` only updates the selected local account. It deliberately does not launch `agy`.

Run an isolated session without changing the default account:

```bash
agy-auth exec personal
```

For scripts and diagnostics:

```bash
agy-auth list --plain
agy-auth --json list
agy-auth doctor
agy-auth doctor --json
agy-auth recover
```

## Commands

| Command | Description |
|---|---|
| `agy-auth doctor` | Diagnose client compatibility, registry safety, and recovery state |
| `agy-auth list` | Browse profiles interactively and optionally switch |
| `agy-auth list --plain` | Print stable, non-interactive profile metadata |
| `agy-auth add [name]` | Save the account in the current official `agy` home |
| `agy-auth login [name]` | Enroll another account through official `agy` |
| `agy-auth switch <name>` | Select the account used by future plain `agy` launches |
| `agy-auth exec <name>` | Launch an isolated one-off official-client session |
| `agy-auth hint <name> <masked-hint>` | Set a privacy-preserving account label |
| `agy-auth recover` | Recover interrupted profile imports |

Run `agy-auth <command> --help` for command-specific options.

## How it works

```text
agy-auth
├── profile registry (non-secret metadata)
├── selected-profile marker
├── owner-only managed profile homes
├── transaction, permission, and recovery checks
└── version-gated Antigravity adapter
    └── official agy owns login, refresh, and model traffic
```

During enrollment, `agy-auth` launches the official client inside a fresh isolated home and waits
for the reviewed login and onboarding signals. Switching validates the installed client version,
atomically replaces the verified official credential envelope, updates the selection marker, and
rolls back if persistence fails.

The implementation keeps provider knowledge separate from filesystem mutation: the provider
describes the versioned contract, while the storage layer owns transactional local changes.

Read the [architecture](docs/04-architecture.md), [CLI specification](docs/07-cli-specification.md),
and [architecture decisions](docs/adr/) for the full design.

## Security

Profile homes and credential files are owner-only and checked for unsafe permissions, symlinks,
hardlinks, unexpected schemas, and concurrent use. Routine output uses profile names and masked
account hints rather than tokens or full account identifiers.

This protects against common local mistakes; it cannot protect credentials from malware or an
attacker already running as the same operating-system user.

Please report vulnerabilities privately as described in [SECURITY.md](SECURITY.md).

## Updating and uninstalling

Update a Cargo installation:

```bash
cargo install --force --locked \
  --git https://github.com/ensp1re/agy-auth \
  --tag v0.2.1 \
  agy-auth-cli
```

Remove a Cargo installation:

```bash
cargo uninstall agy-auth-cli
```

For the Linux release installer, remove `~/.local/bin/agy-auth`. Removing the executable does not
delete managed profile data. Inspect `agy-auth doctor --json` before deleting local state manually.

## Development

```bash
git clone https://github.com/ensp1re/agy-auth.git
cd agy-auth
npm install
python3 scripts/check.py
```

`npm install` enables the repository-local Husky pre-commit hook. The hook requires Node.js 18+ and
blocks commits unless the locked release binary builds successfully.

The complete check covers formatting, Clippy, tests, dependency direction, compatibility metadata,
release preflight, secret scanning, and harness validation.

See [CONTRIBUTING.md](CONTRIBUTING.md) for the contribution workflow.

## License

Licensed under the [MIT License](LICENSE). Copyright © 2026 Enspire.
