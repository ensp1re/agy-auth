# Phase 4 Plan: MIT-Licensed Installable Release Candidate

## Objective

Convert the verified local Linux artifact into an installable MIT-licensed release candidate while
keeping external publication as a separate explicit action.

## Scope

- add the canonical MIT license with the approved copyright identity;
- declare `MIT` in Cargo workspace metadata and generated SBOM packages;
- include `LICENSE` in the deterministic Linux archive;
- remove the obsolete not-for-distribution marker and label the artifact `rc.1`;
- repeat archive/SBOM reproducibility, checksum, content, and extracted-binary checks;
- do not create a GitHub release or upload artifacts in this task.

## Acceptance

- Cargo, README, repository license file, SBOM, and archive agree on MIT;
- archive contents include the binary, README, LICENSE, metadata, and SBOM only;
- two clean builds produce identical checksums;
- extracted binary passes installed `agy 1.1.2` diagnostics;
- all local repository and harness gates pass.
