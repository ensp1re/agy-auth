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
4. Otherwise support a versioned, independently verified local file mode only after reversible
   behavior, permissions, refresh behavior, and running-process interaction are tested.
5. Refuse mutation in unsupported desktop/keyring or unknown modes.

## Explicit exclusions

- Gemini/model backend calls, proxies, protocol emulation, or OAuth implementation.
- Quota display through private endpoints or automatic switching after errors or limits.
- Shared/team vaults, credential synchronization, impersonation, or account pooling.
- Keyring enumeration, private database modification, fingerprint spoofing, or anti-ban claims.
- Gemini CLI consumer support; legacy enterprise compatibility requires a separate approved task.

## Success criteria

Research is successful when an `agy` storage/profile contract is documented with version, platform,
non-secret reproduction steps, reversibility, and expiry triggers.

The MVP is successful only when two synthetic or dedicated test accounts can be selected manually
through the official `agy` client on each claimed platform, switching survives refresh and restart,
interrupted mutation recovers safely, and no secret appears in output, logs, tests, or Git.

Until those criteria are met, the product may expose diagnostics but must not claim account switching.
