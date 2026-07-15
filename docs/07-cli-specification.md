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

### `list` and `current`

```text
agy-auth list
agy-auth current
```

These commands report project-owned non-secret registry metadata. They must distinguish registered,
selected, verified-active, unknown, stale, and unsupported states without guessing from private data.

### `add`

```text
agy-auth add <name> [--client <path>]
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

Enabled only if an official profile/home mechanism isolates all auth state. Arguments are direct argv
values, child exit status is propagated, and only reviewed child environment changes are permitted.

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
