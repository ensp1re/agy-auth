# Phase 3 Plan: Linux Isolated-Home Contract

## Objective

Deliver manual work/personal account selection by launching official Antigravity CLI processes in
separate project-managed Linux homes, without reading or copying authentication credentials.

## Authorized scope

- Supersede the diagnostics-only architecture decision with ADR 0007.
- Add a synthetic, non-interactive isolated-process primitive with a cleared environment.
- Define the dedicated-account black-box evidence gate before enabling real commands.
- Keep the shipped doctor capability response and all authentication mutation disabled.

## Sequence

1. Prove two synthetic clients receive distinct `HOME`, XDG data, and runtime roots.
2. Prove non-allowlisted inherited variables are absent and unsafe roots fail closed.
3. Design an interactive TTY-preserving launcher and managed-directory lifecycle.
4. Add fake-client contracts for `add` and `exec`; do not use real accounts yet.
5. Review and explicitly approve a two-dedicated-account Linux experiment.
6. Enable Linux capability only after ADR 0007's complete evidence gate passes.

## Stop conditions

Stop without enabling switching if the client reuses an identity across isolated homes, requires
credential inspection/copying, escapes project-owned roots, or cannot be tested without private API
or keyring access.

## Verification

```bash
PATH=/root/.cargo/bin:$PATH cargo test -p agy-auth-process --lib
PATH=/root/.cargo/bin:$PATH python3 scripts/check.py
PATH=/root/.cargo/bin:$PATH cargo deny check
```
