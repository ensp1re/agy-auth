# System Architecture

## Architectural style

Use ports and adapters with strict dependency direction:

```text
antigravity-auth-cli -> application -> domain/ports
process, storage, and provider-antigravity-cli implement infrastructure boundaries
```

The official `agy` process is the only component permitted to perform login, refresh credentials, or
make model requests. Provider code returns declarative capability and mutation plans; it does not
silently mutate authentication state.

## Current implemented core

- strongly typed non-secret profile registry;
- locked, atomic metadata persistence;
- safe executable discovery;
- direct argv execution with bounded output and timeout;
- installed `agy 1.1.2` version probe;
- capability gates that keep unverified auth-state behavior disabled.

## Antigravity provider discovery

Discovery may inspect only:

- explicit executable path or supplied `PATH`;
- regular-file and executable metadata;
- bounded output from allowlisted commands such as `agy --version` and documented diagnostics.

Discovery must not inspect credential files, keyrings, process environments, client homes, logs, or
conversation content by default.

## Capability states

Each installed client version and platform reports capabilities independently:

```text
unknown -> observed -> verified -> enabled
                    -> stale
                    -> unsupported
```

Only `verified` capabilities may produce mutation plans. Evidence includes client version, platform,
storage mode, reproduction procedure, trust level, and reverification trigger. Upgrades make storage
evidence stale until compatibility is confirmed.

## Preferred switching strategy

Choose the first verified option in this order:

1. Official Antigravity account/profile command.
2. Official documented home/config override that isolates all credential and keyring lookups.
3. Verified file-backed mode using opaque bytes and a transactional activation engine.
4. Diagnostics-only refusal.

Never infer support from filenames alone. Keyring-backed desktop state remains unsupported until
Google documents it or deterministic, reversible behavior receives independent security review.

## Transaction boundary

If file-backed activation is approved later, use a provider lock, regular-file/link checks, owner-only
permissions, rollback copy, same-directory atomic replacement, reread verification, journaled state,
and idempotent recovery. The provider supplies paths and opaque bytes; the transaction manager owns
all mutation and rollback.

## Error and redaction boundary

Errors expose stable categories and remediation, never raw output or state objects. Allowed evidence
is limited to provider enum, client version, capability booleans, operation ID, redacted paths, byte
length, and short integrity fingerprints. Full emails, tokens, authorization URLs, and environment
values are forbidden.
