# ADR 0006: Remain Diagnostics-Only Without a Supported Profile Contract

- Status: Superseded by ADR 0007
- Date: 2026-07-15

## Context

Official Antigravity CLI documentation describes OS-keyring-backed silent authentication and the
destructive interactive `/logout` command. It does not document account selection, credential-home
overrides, keyring namespaces, credential export/import, or a reversible profile switch.

The `agy 1.1.2` public command surface likewise exposes no account or profile mechanism. Two Linux
observations used disposable non-root users and stopped when the client silently recognized an
existing identity before dedicated login. The corrected retry used a clean environment, dedicated
working directory, fresh home/runtime, D-Bus session, and Secret Service. It still reused an identity.
The reuse mechanism was intentionally not investigated through credential, environment, keyring,
log, or network inspection.

## Decision

Keep `agy-auth` diagnostics-only. It may discover and version-check the official executable, report
documented capability availability, validate its own non-secret registry, and explain why switching
is unavailable. It must not capture, copy, select, delete, move, or otherwise mutate Antigravity
authentication state.

Do not repeat authentication experiments unless new official documentation or a materially new,
reviewed hypothesis identifies a supported isolation boundary. Any future mutation proposal requires
a superseding ADR with deterministic, reversible, independently reproducible evidence.

ADR 0007 supersedes this decision after the product owner explicitly approved a version-scoped,
reverse-engineered isolated-home strategy. The diagnostics-only behavior remains the production
default until that strategy passes its stated evidence gates.

## Consequences

- Phase 2 exits with no supported profile-switching mode.
- The next vertical slice is a truthful `agy-auth doctor` diagnostics experience.
- `add`, `use`, and `exec <profile>` remain unavailable rather than simulating unsafe support.
- The registry remains reusable infrastructure but does not imply credentials are manageable.
- The product can progress on diagnostics, packaging, and compatibility reporting without handling
  authentication secrets.
