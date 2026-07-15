# Phase 4 Plan: Release Readiness Foundation

## Objective

Make the diagnostics-only CLI testable as a release artifact by locking its end-to-end contract,
embedding reproducible build identity, and maintaining a versioned Antigravity compatibility ledger.

## Scope

- exercise the compiled `agy-auth` binary with fake official clients and isolated data roots;
- verify human and JSON output plus exit codes 0, 4, 8, and 10;
- embed package version, source revision, and Rust target without timestamps or builder paths;
- maintain a schema-validated compatibility entry for observed `agy 1.1.2` behavior;
- keep hosted builds, archives, signing, SBOM generation, and publication for a later task.

## Acceptance

- end-to-end tests execute the built binary rather than calling internal functions;
- JSON build metadata is stable, non-secret, and independently asserted;
- compatibility entries require evidence and cannot enable switching/mutation without a code change;
- the local gate validates the compatibility ledger;
- all repository and harness checks pass without hosted CI.
