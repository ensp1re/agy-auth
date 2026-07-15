# CLI Specification

Binary name: `gemini-auth`. Short alias `gauth` may be packaged only if it does not conflict on target systems.

## Global flags

```text
--json                 machine-readable output
--no-color             disable ANSI
--config <path>        alternate non-secret config path
--data-dir <path>      alternate data root, mainly tests/portable use
--non-interactive      never prompt; fail if input is required
-v, --verbose          diagnostics; still redacted
--version
--help
```

`--verbose` must never enable HTTP body, environment, or secret logging.

## Commands

### `add`

```text
gemini-auth add <name> --provider <gemini-cli|antigravity-cli>
                       [--from-current]
                       [--client <path>]
```

Default behavior launches the official client's login in a new isolated profile state. `--from-current` captures an existing supported state after confirmation.

Gemini flow:

```text
$ gemini-auth add personal --provider gemini-cli
Creating isolated Gemini CLI profile "personal".
The official Gemini CLI will open for Google sign-in.
...
Profile saved. Run: gemini-auth exec personal -- gemini
```

Antigravity file-mode flow must explain that `agy` must be stopped and that keyring-backed desktop storage is unsupported.

### `list`

```text
gemini-auth list [--provider <provider>]
```

```text
NAME       PROVIDER          ACTIVE  ACCOUNT          STATUS
personal   gemini-cli       yes     a***@example.com ready
work       gemini-cli       no      d***@company.com ready
ssh-agy    antigravity-cli  yes     —                ready
```

Account hints are opt-in and masked. `--json` emits stable field names and never locators that reveal usernames unless `--show-paths` is explicitly added in a future version.

### `use`

```text
gemini-auth use <name> [--force]
```

For isolated-home providers, `use` changes the default selection used by `gemini-auth run`/shell integration. It does not copy the home into the official default location.

For transactional providers, it activates the credential state after process and permission checks.

### `exec`

```text
gemini-auth exec <name> -- <command> [args...]
```

Rules:

- only provider-approved environment changes are injected;
- inherited variables that override authentication are diagnosed and, with confirmation, removed for the child;
- exit code and signals are propagated;
- command arguments are never logged unless safe and explicitly requested;
- default command can be inferred from provider, but `--` form remains canonical.

### `current`

```text
gemini-auth current [--provider <provider>]
```

Reports registry selection plus whether discovered active state matches. A mismatch returns a distinct status without reading network state.

### `rename`

```text
gemini-auth rename <old> <new>
```

Changes metadata only. UUID/keyring identity stays constant.

### `remove`

```text
gemini-auth remove <name> [--yes]
```

Refuses to remove the active transactional profile until another profile is activated or `--deactivate` is provided. Shows which local data categories will be removed, never their contents.

### `doctor`

```text
gemini-auth doctor [--provider <provider>] [--repair]
```

Checks:

- official binary discovery/version;
- registry schema/invariants;
- secret store availability;
- data directory owner/permissions;
- symlinks and unexpected file types;
- interrupted transactions;
- active-state hash mismatch;
- conflicting authentication environment variables;
- running client processes;
- provider capability status.

`--repair` prompts per mutation. In noninteractive mode it only performs explicitly safe, idempotent repairs selected by flags.

### `completion`

```text
gemini-auth completion <bash|zsh|fish|powershell>
```

Generated through `clap_complete` and tested as release artifacts.

## Deferred commands

- `export`/`import`: after encrypted bundle design.
- `migrate`: once a second registry version exists.
- `desktop`: no GUI until core stability.
- `quota`, `rotate`, `proxy`, `serve`: permanently out of scope.

## Exit codes

| Code | Meaning |
|---:|---|
| 0 | Success |
| 2 | CLI usage error |
| 3 | Profile not found/conflict |
| 4 | Provider/client unavailable |
| 5 | Unsupported provider mode |
| 6 | Authentication/login cancelled or failed |
| 7 | Secret store unavailable/locked |
| 8 | Unsafe permissions or corrupt/unsupported state |
| 9 | Client running/concurrency conflict |
| 10 | Transaction recovery required |
| 11 | Internal invariant failure |

For `exec`, child exit codes take precedence; launcher failures use 125-127 conventions where appropriate.

## Wording guidelines

Use "profile" for a locally stored state and "Google account" only when confirmed. Use "select" or "activate," not "rotate." Never promise secure deletion, ban avoidance, quota aggregation, or official endorsement.
