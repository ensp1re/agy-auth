# Linux Two-Account Upgrade Validation: agy 1.1.3

- Date: 2026-07-16
- Platform: Linux x86_64 over SSH
- Client: official `agy 1.1.3`
- Scope: black-box revalidation of ADR 0007 after the installed client changed from `1.1.2`

## Method

Two independently enrolled, operator-controlled credentials were copied into separate disposable
owner-only homes using the previously reviewed consumer-token envelope. Each client ran with a
cleared environment, distinct `HOME` and `XDG_RUNTIME_DIR`, direct argv, synthetic SSH markers, and
private bounded output. A third disposable home contained no credential.

The experiment:

1. launched both enrolled profiles and confirmed their expected identities without crossover;
2. replaced both access tokens and expiries with expired synthetic values while retaining their
   refresh credentials;
3. launched both profiles concurrently through bounded non-interactive requests;
4. confirmed both official clients refreshed successfully and retained distinct identities;
5. confirmed the clean third home authenticated as neither account and created no token envelope;
6. removed all disposable homes, copied credentials, client logs, and captured output.

Identity comparisons were performed locally. No credential value, full client log, or captured
output was committed.

## Result

`agy 1.1.3` preserves the reviewed Linux SSH file-backed path and consumer envelope used by `1.1.2`.
Both accounts authenticated independently across concurrent execution and forced refresh. The clean
third home did not reuse either identity. The official default homes were not changed by the
experiment.

This satisfies ADR 0007 for `agy 1.1.3` specifically. It does not imply compatibility with later
versions, desktop keyrings, macOS, or Windows.
