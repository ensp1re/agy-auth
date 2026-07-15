# ADR 0004: Transactional Antigravity File-Mode Support

- Status: Proposed, pending renewed `agy` 1.1.2 compatibility research
- Date: 2026-07-15

## Context

Observed Antigravity CLI SSH sessions persist OAuth state in a fixed mode-0600 JSON file. Desktop sessions may use an undocumented keyring.

## Decision

Support only verified file-backed mode through an opaque-secret vault and journaled atomic replacement. Refuse keyring-backed desktop switching until its contract is verified. Require Antigravity to be stopped during activation.

## Consequences

- Useful SSH support without backend protocol or refresh implementation.
- More complex transaction/recovery engine.
- Desktop support is intentionally incomplete.
- Unknown schemas and storage modes fail closed.
