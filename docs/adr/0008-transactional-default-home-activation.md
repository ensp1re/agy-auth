# ADR 0008: Permit Transactional Default-Home Activation

- Status: Accepted for `agy 1.1.2` and `1.1.3` on Linux SSH
- Date: 2026-07-16

## Context

ADR 0007 verified isolated managed homes, but an interactive `switch` alias only launched a child
and did not change the account used by a later plain `agy` command. The product owner requires manual
selection to update the same-user official default home without launching the client.

The verified Linux SSH contract uses one mode-0600 consumer token file beneath the effective home.
Materializing the same reviewed minimal envelope in independent homes selected the expected account,
survived restart, and allowed the official client to own refresh. The same-user default destination
has identical schema and permission behavior.

## Decision

Permit explicit `agy-auth switch <name>` to atomically replace only the verified official default
token file with the selected managed profile envelope. The storage transaction validates same-user
ownership, rejects links and writable ancestors, creates a mode-0600 same-directory temporary file,
flushes, renames, syncs, and rereads metadata. The provider supplies only the versioned envelope;
storage owns mutation.

Before replacing a previously selected account, `agy-auth` captures its current refresh credential
back into that managed profile so official-client rotation is retained. A project-owned non-secret
`active.json` records the selected profile for list output. `switch` never launches `agy`.

## Consequences

- Plain `agy` uses the selected profile on its next launch.
- Existing running `agy` processes are not changed; users must exit them before switching.
- Settings, history, workspace trust, and conversations remain in the default home; only
  authentication state changes.
- Direct `/logout`, manual token replacement, or another login can make `active.json` stale.
- Full account emails remain unavailable because the reviewed token envelope contains no identity
  field; users may attach a masked non-secret hint.
- Any `agy` version change invalidates this capability until reverified.
