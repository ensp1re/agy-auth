# Linux Two-Account Validation: 2026-07-16

## Scope

- Platform: Ubuntu 24.04 family, Linux x86_64 VPS
- Client: `agy 1.1.2`
- Binary SHA-256: `70bf6eaf2e82fbb243db999b9c7c61fcf7f6e537f41980650eb2341ed84b24de`
- Storage mode: SSH file fallback beneath each effective home
- Accounts: two distinct operator-controlled consumer accounts, recorded as Profile A and Profile B

No reusable credential, OAuth value, private log, transcript, or full account identity is committed
with this record.

## Enrollment boundary

Each source account had authenticated through the official `agy` client. Profile A used the
operator's existing official-client state. Profile B was enrolled under a fresh non-root OS user
with a new UID, owner-only home/runtime directories, a separate D-Bus session, and a separate Secret
Service collection. The client-owned Profile B token was then materialized into that user's
mode-`0600` SSH fallback file for the isolated-home test.

An earlier attempted validation user reused UID `1000`, which was also running an unrelated process
and retained stale Secret Service behavior. The test stopped without killing or changing that
process. A fresh UID (`1101`) superseded that boundary.

## Concurrent isolation and restart test

Two disposable mode-`0700` profile homes and runtime directories received their corresponding
credential records. Both real clients launched concurrently with:

- cleared environments;
- distinct `HOME` and `XDG_RUNTIME_DIR` values;
- direct `agy` execution under pseudo-terminals;
- synthetic loopback SSH markers selecting file fallback;
- closed stdin and no prompt or model request;
- bounded timeouts with cleanup after each launch.

Each profile launched twice. Derived log evidence showed:

- Profile A authenticated exactly one identity on both launches;
- Profile B authenticated a different identity on both launches;
- neither profile observed the other identity;
- neither profile started browser login;
- both reached `v1internal:loadCodeAssist`;
- Profile A refreshed an expired access token during the concurrent run.

## Forced refresh and clean-profile controls

A separate disposable copy of Profile B replaced only its access token and expiry with the reviewed
expired placeholder envelope while preserving its refresh credential. The real client refreshed it,
authenticated the same Profile B identity, avoided browser login, and reached `loadCodeAssist`.

A clean third disposable home with no credential record:

- authenticated no identity;
- observed neither known test identity;
- loaded no token;
- reported that login was required.

## Cleanup and consequence

All disposable homes, runtimes, transcripts, copied credentials, logs, and client processes were
removed. The operator's default official-client state was not modified. The separate Profile B
enrollment account remains isolated for the next controlled integration step.

This satisfies the ADR 0007 black-box isolation gates for `agy 1.1.2` on Linux SSH:
distinct enrollment, restart persistence, forced refresh, clean-home non-reuse, concurrent
separation, version gating, and scoped teardown are reproducible. The compatibility state is
`verified`, not `enabled`; public commands remain capability-gated until their application and CLI
workflow is reviewed.
