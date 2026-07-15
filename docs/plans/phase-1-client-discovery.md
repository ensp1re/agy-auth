# Phase 1 client discovery execution plan

## Objective

Discover supported official client executables and run bounded, non-authenticated probes without
shell evaluation, credential access, or provider-specific mutation.

## Constraints

- Commands are executable plus argv arrays; shell command construction is forbidden.
- Explicit overrides fail closed unless they are regular executable files.
- Captured output is bounded and intended only for allowlisted probes such as `--version`.
- Timeouts terminate probes and return typed evidence.
- Discovery never searches credential files, keyrings, process environments, or client homes.

## Steps

1. Define provider executable names and typed discovery/process errors.
2. Implement PATH search and explicit-path validation without adding platform-specific shell calls.
3. Implement bounded stdout/stderr capture, timeout, exit status, and truncation reporting.
4. Add fake executable tests for overrides, PATH ordering, argv preservation, failures, and timeout.
5. Exercise `agy --version` through the abstraction and record only path, version, and capability.
6. Run project, dependency-policy, secret, and harness checks and open a review PR.

## Not in scope

Login, model requests, client home discovery, credential file inspection, keyring access, process
enumeration, environment dumps, profile creation, and provider activation.
