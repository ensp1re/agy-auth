# Delivery Roadmap

Phases are evidence gates, not calendar promises.

## Phase 0 — Repository foundation (complete)

- Rust workspace and dependency boundaries.
- Local format, Clippy, test, dependency, and secret gates.
- Non-secret registry and safe process discovery.
- Contribution, security, and harness workflow.

## Phase 1 — Antigravity product transition (complete)

- Rename product and binary to `agy-auth`.
- Make `provider-antigravity-cli` the sole MVP provider.
- Record the May/June 2026 Gemini CLI transition in product truth and ADRs.
- Remove the unverified Gemini isolated-home path from the MVP.
- Confirm `agy 1.1.2` discovery and bounded diagnostic behavior.

Exit gate: repository sources consistently describe an Antigravity-first, diagnostics-only product
and no executable path claims real switching support.

## Phase 2 — Antigravity capability research (complete: diagnostics-only)

- Identify official profile/account/home commands or documented overrides.
- Observe file versus keyring modes without enumerating unrelated secrets.
- Verify paths, permissions, process behavior, refresh behavior, and version compatibility.
- Record synthetic reproduction evidence and an ADR for the selected strategy.

Exit gate: one supported mode is deterministic, reversible, policy-compatible, and independently
reviewable, or the project explicitly remains diagnostics-only.

Outcome: no supported profile or authentication-isolation contract was found for `agy 1.1.2`.
ADR 0006 keeps authentication-state mutation disabled.

## Phase 2A — Diagnostics vertical slice

- implement `agy-auth doctor` with human-readable and JSON output;
- report executable discovery, client version, and capability-gate results;
- explain that profile switching is unavailable without a supported contract;
- validate the non-secret registry without enumerating authentication state;
- test all output for secret-safe, deterministic behavior.

Exit gate: users can determine whether `agy` is installed and why switching is unavailable without
authentication access, model requests, or misleading support claims.

## Phase 3 — First vertical profile slice

- establish the version-scoped Linux isolated-home contract from ADR 0007;
- `add` through the official login flow;
- `list` and `current` using non-secret metadata;
- `exec <profile> -- agy` where an official isolated mechanism exists, otherwise transactional `use`;
- fake-client integration tests plus dedicated-account manual validation.

Exit gate: two profiles switch manually without credential disclosure, backend calls, or automatic
fallback, and interruption recovery is demonstrated.

## Phase 4 — Hardening and platform coverage

- fault injection and recovery;
- Linux, macOS, and Windows permission/atomic behavior;
- schema/version compatibility matrix;
- packaging, SBOM, provenance, and release documentation.

## Deferred

- Gemini CLI enterprise/API-key compatibility;
- desktop keyring mutation;
- encrypted export/import;
- GUI and plugin SDK.

## Permanently excluded

Backend protocol access, proxy/server modes, quota polling, automatic account rotation, shared
credentials, fingerprint spoofing, and anti-ban behavior.
