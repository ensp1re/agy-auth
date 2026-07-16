# System Architecture

## Architectural style

Use ports and adapters with strict dependency direction:

```text
agy-auth-cli -> application -> domain/ports
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
- non-default experimental workflow connecting the versioned consumer envelope to protected
  profile-home materialization and refreshed-token extraction;
- capability gates that keep unverified auth-state behavior disabled.

## Antigravity provider discovery

Discovery may inspect only:

- explicit executable path or supplied `PATH`;
- regular-file and executable metadata;
- bounded output from allowlisted commands such as `agy --version` and documented diagnostics.

Discovery may inspect credential files, keyrings, process environments, client homes, logs, and
network behavior when the operator explicitly authorizes reverse engineering on accounts and systems
they control. Sensitive observations stay local, use restrictive temporary storage, and must not be
committed or copied into reports.

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
3. Version-scoped, independently verified isolated environment that delegates login and refresh to
   the official client without reading or copying credential state.
4. Verified file-backed activation using opaque bytes and a transactional engine, if separately
   approved.
5. Diagnostics-only refusal.

Never infer support from filenames alone. A reverse-engineered behavior becomes a project contract
only after the versioned black-box gates in ADR 0007 pass. Keyring-backed desktop state remains
unsupported until deterministic, reversible behavior receives independent security review.

## Transaction boundary

If file-backed activation is approved later, use a provider lock, regular-file/link checks, owner-only
permissions, rollback copy, same-directory atomic replacement, reread verification, journaled state,
and idempotent recovery. The provider supplies paths and opaque bytes; the transaction manager owns
all mutation and rollback.

## Error and redaction boundary

Normal command errors expose stable categories and remediation rather than raw authentication state.
Opt-in research diagnostics may capture raw client output, paths, environment values, and protocol
metadata locally when needed to establish the profile contract. Those artifacts are sensitive and
must remain outside Git and user-facing reports unless the operator explicitly requests a specific
value.
