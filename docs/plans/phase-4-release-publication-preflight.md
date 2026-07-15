# Phase 4 Plan: v0.1.0-rc.1 Publication Preflight

## Objective

Make the diagnostics-only release candidate internally consistent and approval-ready without
creating a Git tag, GitHub release, or external artifact upload.

## Scope

- set every workspace package to semantic prerelease version `0.1.0-rc.1`;
- align binary, archive, SBOM, checksum, release-note, and proposed tag names;
- document available diagnostics, security boundaries, platform evidence, and deferred hosted CI;
- validate the proposed `v0.1.0-rc.1` tag does not already exist;
- rebuild deterministic artifacts from the final preflight revision;
- stop for explicit approval before tag creation or publication.

## Acceptance

- all workspace packages and internal exact dependencies use `0.1.0-rc.1`;
- release notes truthfully describe diagnostics-only scope and untested targets;
- artifact names derive from the Cargo version without duplicated prerelease suffixes;
- local gates and reproducibility checks pass from a clean revision;
- proposed tag remains absent and no external release exists.
