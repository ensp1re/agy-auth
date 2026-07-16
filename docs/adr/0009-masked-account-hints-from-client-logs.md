# ADR 0009: Derive Masked Account Hints from Official Login Logs

- Status: Accepted for delegated login on `agy 1.1.2` and `1.1.3` Linux SSH
- Date: 2026-07-16

## Context

The reviewed consumer token envelope contains no email or display identity. During official OAuth
login, `agy` writes the authenticated email to a per-profile local log before onboarding completes.
Users need to distinguish profiles without manually entering every hint.

## Decision

After delegated login completes, the provider may inspect at most 32 regular non-linked log files,
each bounded to 2 MiB, beneath that newly managed profile home. It recognizes only the reviewed
successful-authentication log markers, validates one email-shaped value, converts it immediately to
`first-three-characters + *** + domain`, and returns only that masked hint.

The full email is never returned from the provider, stored in the registry, printed, logged by
`agy-auth`, or committed. Missing, malformed, oversized, or changed logs simply leave the hint empty.

## Consequences

- New delegated profiles normally receive a recognizable masked hint automatically.
- Existing profiles retain their current manually supplied hint.
- Log wording changes may disable automatic hints without affecting authentication or switching.
- Full email display remains intentionally unsupported.
