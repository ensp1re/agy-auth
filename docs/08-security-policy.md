# Security and Policy Boundaries

## Security objective

Minimize the additional risk introduced by convenient local switching. The tool necessarily handles authentication state, so correctness and restraint are more important than feature count.

## Hard product prohibitions

The project will not:

- ask for or store Google passwords;
- implement Google's OAuth client or token refresh protocol;
- call Gemini, Code Assist, Antigravity, quota, entitlement, or model backend endpoints;
- expose an HTTP API;
- automatically select another account after 401/403/429/quota errors;
- combine, pool, score, or load-balance account quota;
- spoof official client headers, request IDs, TLS signatures, installation IDs, or telemetry;
- share or synchronize credentials through a project service;
- import credentials belonging to another person;
- print full emails or token metadata by default.

These boundaries should appear in `CONTRIBUTING.md` and pull-request review templates. Contributions crossing them should be rejected even if technically useful.

## Policy analysis

Google's Gemini CLI FAQ states that harvesting or piggybacking on CLI OAuth to access backend services from third-party software violates applicable terms and may lead to suspension. This design avoids that behavior: the official client performs login, refresh, and requests; `antigravity-auth` only selects local client state.

That does not equal formal Google endorsement. Users remain responsible for account terms and employer policies. Manual switching should never be marketed as "ban safe."

Using multiple accounts for legitimate identity separation is different from switching to evade a usage limit. The latter is outside the product regardless of whether detection is possible.

## Local security controls

- Default data directories owner-only (`0700` Unix).
- Secret/rollback files `0600` Unix; restrictive Windows DACL.
- Refuse unsafe ownership or permissions rather than silently continuing.
- No symlink traversal for credential destinations.
- Exclusive creation for temp and export files.
- Bounded input sizes and recursion.
- Secrets wrapped in non-display/non-debug types.
- Structured logging allowlist; unknown fields are not logged.
- Panic/crash handlers emit operation IDs, not state objects.
- Clipboard use is forbidden for tokens.
- No shell command construction; use argv arrays.
- Environment allowlist for child modifications.

## Supply-chain controls

- Commit `Cargo.lock`.
- Pin CI actions to commit SHA.
- Run `cargo audit` and `cargo deny` for vulnerabilities, licenses, sources, and duplicate high-risk crates.
- Minimize dependencies in storage/provider crates.
- Review build scripts and proc macros.
- Generate SBOM and provenance attestations for releases.
- Sign tags and release artifacts where maintainers can support key management.

## Vulnerability handling

Before public release add `SECURITY.md` with:

- private reporting address or GitHub private vulnerability reporting;
- supported versions;
- 90-day coordinated disclosure target;
- explicit classification of credential disclosure, permission bypass, symlink overwrite, rollback failure, and secret logging as high/critical issues.

## Redaction contract

Redaction is preventive, not regex cleanup after serialization.

Allowed diagnostic fields:

- operation/transaction UUID;
- provider enum;
- profile UUID and user-chosen name;
- client version;
- capability booleans;
- paths only with home username replaced;
- byte length and SHA-256 fingerprint prefix of opaque state.

Forbidden fields:

- secret bytes or substrings;
- authorization URL query parameters;
- access/refresh/ID token;
- OAuth authorization code/state/verifier;
- full email;
- complete environment dump;
- official-client stdout/stderr when it may contain login URLs, unless streamed directly and not retained.

## Security review gates

- Threat-model review before storage implementation.
- Focused review of atomic write and permissions module.
- Cross-platform manual validation before claiming platform support.
- Secret-scanning tests in CI.
- External security review before adding encrypted export or keyring mutation for Antigravity desktop.
