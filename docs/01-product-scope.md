# Product Scope and Requirements

## Problem statement

Google's command-line clients generally expose one active OAuth identity at a time. A developer who legitimately separates personal and work activity must repeatedly sign out, complete browser login, and reinitialize the client. The desired workflow is the same ergonomic improvement offered by account switchers in other ecosystems: save locally authenticated states under names and activate one explicitly.

The product must improve local ergonomics without changing how Google authenticates requests, increasing quota, impersonating an official client, or exposing a subscription through another protocol.

## Product definition

`gemini-auth` is a local credential-profile orchestrator. It manages references to credential states and delegates authentication and request execution to official Google clients.

It supports two operating models:

1. **Isolated-home execution**: each profile owns a separate official-client home directory. The tool launches the official client with the selected home. This is preferred because no credential copying is required.
2. **Transactional activation**: where the official client has a fixed credential path, the tool atomically installs a previously captured credential document into that path. This is a compatibility fallback.

## Target users

- A single developer with separate personal and employer-owned Google accounts.
- A consultant who keeps client identities isolated.
- A developer testing behavior across legitimate accounts or organizations.
- SSH users who cannot conveniently repeat browser login.

## Explicit non-users

- Teams sharing one person's credentials.
- Services exposing Gemini or Antigravity as a public/private API.
- Operators aggregating free or paid quotas.
- Bots automatically choosing another identity after rate limiting.
- Any workflow intended to evade account, region, entitlement, or safety controls.

## Functional requirements

### Profile lifecycle

- Add a named profile using the official client login flow.
- Import an already-active local credential state only after explicit confirmation.
- List profiles without reading or displaying secret values.
- Show the active profile and provider.
- Rename profile metadata without touching credentials.
- Remove a profile with confirmation and best-effort secret deletion.
- Diagnose storage, permissions, schema compatibility, and client-process conflicts.

### Activation

- Explicitly activate one profile.
- Atomically update the active credential state.
- Preserve a rollback point until activation is verified.
- Refuse activation while the target client is running unless `--force` is explicitly supplied; `--force` still must not corrupt files.
- Provide `exec <profile> -- <command>` to avoid global mutation when isolated homes are supported.

### Providers

- `gemini-cli`: first-class, preferred isolated-home model through `GEMINI_CLI_HOME`.
- `antigravity-cli`: v1 support for verified file-backed tokens only.
- `antigravity-desktop`: discovery/diagnostics only until keyring and state behavior are verified.

### Output

- Human-readable tables by default.
- Stable `--json` output for scripting.
- No secrets in stdout, stderr, diagnostics, crash reports, or JSON output.
- Defined exit codes documented in the CLI specification.

## Non-functional requirements

- Linux, macOS, and Windows support.
- Single self-contained executable per platform.
- Startup under 100 ms for metadata-only commands on a warm filesystem.
- No network access except when launching an official login or optional, explicit update check.
- No daemon required.
- Atomicity under crash/power loss as far as the host filesystem permits.
- Backward-compatible registry migrations.
- Reproducible release builds and signed checksums.

## Success criteria

The MVP is successful when a user can create two Gemini CLI profiles, run each in an isolated home, switch between them without another OAuth login, and recover cleanly from a deliberately interrupted activation. No secret may appear in tests, logs, or shell history.

Antigravity support is successful when the same workflow works in its verified file-token mode and refuses unsupported keyring-backed environments with a clear explanation.

## Policy-safe product boundary

Convenient manual selection is the product. Quota-aware selection is not. The codebase must not include rate-limit polling, account scoring, round-robin selection, retry-to-another-account, fingerprint spoofing, backend protocol calls, or token refresh calls made on behalf of the official client.
