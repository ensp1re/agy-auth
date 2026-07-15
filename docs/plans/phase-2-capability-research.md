# Phase 2 Plan: Antigravity Capability Research

## Objective

Determine whether Antigravity CLI exposes a supported, deterministic, reversible profile or
authentication-isolation mechanism. Do not enable or prototype real authentication-state mutation
during research.

## Evidence boundaries

- Start with official documentation plus `agy --version`, top-level help, and subcommand help.
- Do not launch an interactive session merely to enumerate state.
- Do not inspect keyring entries, credential files, OAuth URLs, logs, conversations, or environment
  dumps on an existing account.
- Any later storage observation requires a dedicated test account, explicit approval, and a record
  containing only non-secret metadata.
- Treat filenames and implementation observations as compatibility evidence, never as a supported
  contract by themselves.

## Steps

1. Inventory the installed `agy 1.1.2` public command and flag surface without authentication access.
2. Compare it with current official installation, authentication, configuration, and release docs.
3. Determine whether a supported profile selector, alternate home, keyring namespace, or reversible
   session mechanism exists.
4. If the public contract is insufficient, design the
   [dedicated-account Linux observation protocol](linux-dedicated-account-observation.md), then
   separately scope macOS and Windows evidence.
5. Record a new ADR selecting a supported strategy or explicitly retaining diagnostics-only scope.

## Exit evidence

- exact client version, platform, date, and non-secret commands;
- primary-source links and the distinction between documented and observed behavior;
- resolved or narrowed questions in `docs/12-open-questions.md`;
- a reviewed ADR before any credential mutation code is authorized.
