# Phase 3 Plan: Managed Profile and Fake Add/Exec Contracts

## Objective

Establish secure project-owned directory lifecycle and application orchestration for future profile
`add` and `exec` commands without enabling either command or invoking a real login.

## Scope

- create `profiles/<uuid>/home` and `profiles/<uuid>/runtime` beneath an absolute data root;
- create directories with owner-only permissions and revalidate type, permissions, and Linux owner;
- refuse links, non-directories, group/other access, relative roots, and unsafe existing paths;
- define application ports for the catalog, directory manager, and official client;
- reserve pending metadata before login and mark ready only after successful fake login;
- require ready state before fake execution and preserve literal argv/exit status.

## Failure semantics

A failure after catalog reservation leaves the profile pending for explicit recovery. This slice does
not delete a home automatically, because a future official login might have created state before
failing. No adapter reads, copies, hashes, or logs files beneath the managed home.

## Exclusions

- CLI `add`, `list`, `current`, or `exec` exposure;
- real Antigravity login or identity observation;
- registry adapter implementation for the new catalog port;
- automatic cleanup, keyring access, credential parsing, or capability enablement.

## Verification

```bash
PATH=/root/.cargo/bin:$PATH cargo test -p agy-auth-app -p agy-auth-storage
PATH=/root/.cargo/bin:$PATH python3 scripts/check.py
PATH=/root/.cargo/bin:$PATH cargo deny check
```

## Next task

Implement the atomic registry catalog adapter and fake-client CLI commands behind an explicit
experimental capability gate.
