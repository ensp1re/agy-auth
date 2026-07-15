# Phase 3 Plan: TTY-Preserving Isolated Launcher

## Objective

Provide the process primitive needed to run the official interactive `agy` client inside a managed
profile environment while preserving terminal behavior and preventing ambient environment leakage.

## Scope

- inherit the caller's standard input, output, and error descriptors;
- pass child arguments directly without a shell and propagate the exact exit status;
- reuse owner-only home/runtime validation and the cleared HOME/XDG/PATH/locale environment;
- add only a validated terminal type to the child allowlist;
- verify behavior with fake clients, including nonzero exit and metacharacter arguments;
- probe installed `agy 1.1.2` only with `--version` under two disposable empty homes.

No CLI `exec` command, real login, account identity observation, credential access, model prompt,
logout, or production capability enablement is included.

## Acceptance

1. Fake clients observe distinct profile roots and no inherited user/environment value.
2. Standard streams use `Stdio::inherit`, and the child exit status is returned unchanged.
3. `TERM` accepts conservative terminal names and rejects shell/control syntax.
4. Existing path, permission, symlink, argv, timeout, and output-bound tests continue passing.
5. Installed `agy --version` returns 1.1.2 from two disposable isolated environments without
   creating authentication state.

## Next task

Build the managed profile-directory lifecycle and fake-client `add`/`exec` application contracts.
