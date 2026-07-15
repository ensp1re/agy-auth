# Phase 4 Plan: Linux Local Release Candidate

## Objective

Produce a reproducible, verifiable Linux release-candidate archive for local testing. This initial
plan predated the approved MIT license and was superseded by the installable RC plan.

## Scope

- build the host target with `--release --locked` from a clean Git worktree;
- embed the full source revision through the existing build identity contract;
- create a deterministic tar/gzip archive with normalized order, time, owner, and group;
- generate a deterministic SPDX 2.3 JSON dependency SBOM from Cargo metadata and `Cargo.lock`;
- generate and immediately verify SHA-256 checksums;
- include an explicit not-for-distribution notice until a license is approved (superseded: MIT was
  approved July 16, 2026);
- document checksum verification, local install, binary uninstall, and preserved data.

## Acceptance

- dirty worktrees and non-host targets fail before packaging;
- two builds from the same revision produce identical archive and SBOM checksums;
- archive contents are limited to the binary, README, release metadata, SBOM, and notice;
- the extracted binary reports the embedded revision/target and passes doctor smoke testing;
- no GitHub release, package registry, or external publication occurs.
