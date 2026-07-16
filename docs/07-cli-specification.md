# CLI Specification

Binary name: `agy-auth`. The official client executable is `agy`.

## Global flags

```text
--json
--no-color
--config <path>
--data-dir <path>
--non-interactive
-v, --verbose
--version
--help
```

Diagnostics never include credential values, authorization URLs, full emails, or environment values.

## Capability-gated commands

### `doctor`

```text
agy-auth doctor [--repair]
```

MVP-safe diagnostics include executable discovery/version, registry validity, local directory
permissions, interrupted project-owned transactions, and provider capability status. `--repair`
must not alter Antigravity authentication state until a mutation contract is verified.

Diagnostics report `profileSwitching: true` and `authStateMutation: true` for verified
`agy 1.1.2` and `1.1.3` Linux builds. Their stable reason is `verified_contract_enabled`;
unsupported or unverified combinations use
`no_verified_antigravity_profile_contract`. JSON output uses `schemaVersion: 1` and never includes
executable paths, registry paths, environment values, account identity, or client output other than
the validated version line. Until safe repair actions exist, `--repair` is an explicit no-op.

Filesystem diagnostics report `dataDirectoryState`, `ownerMatches`, `permissionsSecure`, and only the
count of `interruptedTransactions`. They never emit paths or transaction names. An absent data root
is healthy and is not created. Unsafe ownership, permissions, symlinks, or file types return exit 8;
one or more project-owned transaction markers return exit 10 without reading marker contents.

### `list` and `current`

```text
agy-auth list
agy-auth current
```

These commands report project-owned non-secret registry metadata. They must distinguish registered,
selected, verified-active, unknown, stale, and unsupported states without guessing from private data.

### `add`

```text
agy-auth add [name] [--client <path>]
```

Enabled only after a verified strategy delegates login to official `agy` under an isolated or
transaction-safe profile context. Otherwise return `unsupported_provider_mode` with remediation.

### `use`

```text
agy-auth use <name> [--force]
```

Enabled only for a verified reversible selection strategy. `--force` may bypass a running-process
warning after confirmation but must never bypass unsafe paths, permissions, schemas, or recovery.

### `exec`

```text
agy-auth exec <name> -- agy [args...]
```

Enabled only if an official or version-scoped project-verified isolated-home mechanism separates all
relevant client state. Arguments are direct argv values, child exit status is propagated, and only
reviewed child environment changes are permitted. Interactive launches inherit the caller's standard
streams, clear ambient environment variables, and forward only validated terminal metadata.

### `rename` and `remove`

Metadata rename never changes provider state. Remove requires confirmation and lists data categories,
not contents. Secret deletion claims are best-effort and platform-specific.

## Exit codes

| Code | Meaning |
|---:|---|
| 0 | Success |
| 2 | CLI usage error |
| 3 | Profile not found/conflict |
| 4 | Official client unavailable |
| 5 | Unsupported or unverified provider mode |
| 6 | Official login cancelled or failed |
| 7 | Secret store unavailable/locked |
| 8 | Unsafe permissions or corrupt/unsupported state |
| 9 | Client running/concurrency conflict |
| 10 | Transaction recovery required |
| 11 | Internal invariant failure |

For `exec`, child exit codes take precedence; launcher failures use 125–127 where applicable.

## Permanently excluded commands

`quota`, `rotate`, `proxy`, `serve`, automatic fallback, and backend protocol commands.

## Compile-time feature gates

The non-default Cargo feature `experimental-fake-client` exposes hidden `experimental-add` and
`experimental-exec` commands for end-to-end orchestration tests. They use an in-process fake only;
they cannot accept an executable path or launch real `agy`. Release builds omit this feature.

The separate non-default feature `experimental-profile-credentials` compiles only the internal
application/provider/storage credential workflow, the real interactive client adapter, and synthetic
end-to-end tests. It adds no CLI command by itself. The default `profile-cli` feature composes those
adapters into the production command surface. The legacy `experimental-real-profile-cli` feature is
an alias for `profile-cli`.

The default CLI exposes `add`, `login`, `list`, `switch`, `exec`, `hint`, and `recover`.
`add [name]` captures the
account already logged into the caller's official `agy` home; `--from-home` selects another
same-user official home. `login [name]` launches official `agy` in a newly managed isolated home so
another account can be enrolled without logging out the default home. Enrollment waits for both the
verified credential and official consumer-onboarding completion marker, then gracefully interrupts
the client before workspace/project prompts, validates the envelope, commits the profile, and prints
the next switch command. An omitted name selects the first unused `profileN`. These commands are
restricted to the verified
`agy 1.1.2` or `1.1.3` Linux contract. Import reads a bounded secure official-client envelope,
extracts only its refresh credential, and materializes a minimal envelope in an owner-only managed
profile home.
Execution uses direct argv, the isolated SSH environment, and an exclusive profile lock; the
official client continues to own login, token refresh, and backend traffic. The mode-0600 credential
file remains inside its mode-0700 isolated official-client home as approved by the storage contract;
it is not exported into a separate plaintext vault.

`switch <name>` is the convenient interactive alias for `exec <name>` with no child arguments.
`hint <name> <masked-hint>` stores only a user-supplied masked value such as `a***@gmail.com`;
`list` includes that hint so account names remain recognizable without printing full emails.

Import creates a non-secret durable marker
before registry reservation and advances it after reservation, credential materialization, and the
ready commit. Recovery removes only pending or absent profile metadata and its project-owned managed
home. If readiness committed before marker cleanup, recovery preserves the ready profile and removes
only the stale marker. Repeated recovery is idempotent.
