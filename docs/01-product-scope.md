# Product Scope and Requirements

## Product definition

`agy-auth` is a local profile-state orchestrator for Google Antigravity CLI (`agy`). It
manages non-secret profile metadata and, only where an official or independently verified local
contract permits, selects the local authentication context used by the official client.

It does not authenticate to Google itself. The official client owns login, token refresh, requests,
entitlements, safety controls, and account policy.

## Transition context

Google announced the transition from consumer Gemini CLI to Antigravity CLI on May 19, 2026. Consumer
Gemini CLI service stopped on June 18, 2026. Gemini CLI remains a possible legacy enterprise/API-key
compatibility target, but it is not part of the MVP.

## Target users

- One developer separating their own personal and employer-approved Google accounts.
- A consultant isolating legitimate client identities on one workstation.
- SSH/headless users who need explicit, manual account-context selection.

## First-release scope

The first release is capability-gated:

1. Discover and diagnose the installed `agy` version without reading authentication state.
2. Maintain a local non-secret profile registry.
3. Use an official Antigravity profile/home mechanism if Google documents one.
4. Otherwise support a clearly labeled, version-scoped unofficial isolated-home mode only after
   black-box tests prove separation, refresh behavior, restart behavior, permissions, and
   running-process safety without reading or copying credential contents.
5. Refuse mutation in unsupported desktop/keyring or unknown modes.

Supported build targets are Linux and Windows. The real profile workflow is currently enabled only
for the verified Linux SSH/headless contract. Windows remains build-verified and mutation-gated.
macOS is deferred and is not built, tested, distributed, or claimed as supported.

## Explicit exclusions

- Gemini/model backend calls, proxies, protocol emulation, or OAuth implementation.
- Quota display through private endpoints or automatic switching after errors or limits.
- Shared/team vaults, credential synchronization, impersonation, or account pooling.
- Keyring enumeration, private database modification, fingerprint spoofing, or anti-ban claims.
- Gemini CLI consumer support; legacy enterprise compatibility requires a separate approved task.
- macOS Keychain integration or macOS distribution.

## Success criteria

Research is successful when an official or project-verified unofficial `agy` isolation contract is
documented with version, platform, non-secret reproduction steps, reversibility, and expiry triggers.

The MVP is successful only when two synthetic or dedicated test accounts can be selected manually
through the official `agy` client on each claimed platform, switching survives refresh and restart,
interrupted mutation recovers safely, and no secret appears in output, logs, tests, or Git.

Until those criteria are met, the product may expose diagnostics but must not claim account switching.
