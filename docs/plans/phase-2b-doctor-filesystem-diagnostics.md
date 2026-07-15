# Phase 2B Plan: Doctor Filesystem Diagnostics

## Objective

Extend `agy-auth doctor` with read-only validation of project-owned data-root security and interrupted
transaction markers. Do not create, repair, rename, or delete any path.

## Scope

- distinguish absent, secure, and unsafe data roots;
- reject symlinks and non-directory data roots;
- on Linux, require the data root to be owned by the running user;
- on Unix, reject group/world permission bits;
- inspect only the entry count of the project-owned `transactions` directory;
- return exit `8` for unsafe filesystem state and `10` when recovery is required;
- expose only booleans, stable categories, and counts—never paths or marker names.

## Acceptance

- absent data roots remain healthy and are not created;
- mode-0700 owned roots pass while unsafe type, ownership, and permissions fail closed;
- interrupted transaction entries are counted without reading their contents or names;
- human and JSON output remain deterministic and authentication-free;
- `--repair` remains a no-op;
- all local gates and manual exit-code checks pass.
