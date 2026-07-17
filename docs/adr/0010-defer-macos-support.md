# ADR 0010: Defer macOS Support

- Status: Accepted
- Date: 2026-07-17

## Context

Antigravity CLI uses Apple Keychain for its default macOS authentication state. The investigated
candidate read and replaced the `gemini` / `antigravity` item, but real validation showed two
blocking problems: third-party access produced recurring macOS authorization prompts, and a refresh
credential captured from the isolated file fallback was not interchangeable with the opaque native
Keychain value. The original imported account remained usable, while the separately enrolled account
required login again after activation.

Avoiding Keychain entirely would require a wrapper or shell integration around file-backed isolated
homes. That would switch only managed CLI launches, not the Antigravity desktop application, and no
such global integration is currently approved. Weakening Keychain access controls, asking for a
macOS password, passing credentials through command arguments, or implementing independent OAuth is
outside the security policy.

## Decision

Remove the macOS Keychain adapter, dependency, validation candidate, installation instructions,
CI job, release targets, and support claims. macOS is not built, tested, distributed, or supported.
Unknown or manually compiled macOS builds continue to fail the existing platform capability gate
before authentication-state mutation.

Linux remains unchanged and retains its verified `agy 1.1.2` and `1.1.3` SSH/headless workflow.
Windows remains a build-verified target whose real profile commands fail closed until a separate
versioned storage contract is validated.

## Consequences

- Users are not asked to weaken Apple Keychain or authorize an unsigned third-party binary.
- The repository no longer carries unvalidated credential-mutation code for macOS.
- macOS support requires a future product decision and new evidence; it cannot be restored by merely
  re-enabling the removed adapter.
- Windows support means build and diagnostics coverage only until its mutation gate passes.
