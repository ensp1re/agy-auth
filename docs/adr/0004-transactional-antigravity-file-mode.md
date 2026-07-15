# ADR 0004: Transactional Antigravity File-Mode Support

- Status: Withdrawn; no supported file-backed authentication contract
- Date: 2026-07-15

## Context

An earlier observation suggested that an Antigravity CLI SSH session persisted OAuth state in a
mode-0600 JSON file. That observation has no independently reproducible, versioned provenance and
cannot establish a supported storage contract. Current official documentation describes native OS
secure-keyring token profiles for local sessions and does not document file-backed authentication
state or a credential-home override.

## Decision

Do not implement this strategy. Keep real authentication-state mutation disabled until capability
research identifies a supported, deterministic, reversible contract and a new ADR approves it.

## Consequences

- No implementation may depend on the earlier file-mode assumption.
- Keyring contents and credential files remain out of scope for the initial command-surface inventory.
- A future strategy requires new evidence and a superseding ADR.
