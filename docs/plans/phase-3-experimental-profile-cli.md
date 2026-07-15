# Phase 3 Plan: Atomic Catalog and Experimental Fake CLI

## Objective

Prove the full registry, managed-home, application, and CLI journey without exposing or launching the
real Antigravity client.

## Scope

- atomically load, mutate, validate, and replace registry metadata under one lock;
- reserve pending profiles and promote them to ready with a validated fake-client version;
- look up profiles by normalized name;
- expose hidden fake-only commands behind the non-default `experimental-fake-client` Cargo feature;
- verify duplicate refusal, ready metadata, owner-only roots, literal argv, and exact fake exit code.

Production builds remain diagnostics-only. The fake gate accepts no executable path and cannot invoke
real `agy`.

## Next task

Design and run the first controlled read-only real-client isolation experiment: launch installed
`agy` in a disposable empty home, stop before login or model interaction, and record only whether the
existing identity was reused.
