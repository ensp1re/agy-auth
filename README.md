# gemini-auth

`gemini-auth` is a planned local account-profile switcher for Google Gemini CLI and Antigravity CLI.

The project has one narrow purpose: let one person authenticate each of their own Google accounts through the official client, save those local credential states as named profiles, and switch the active profile without repeatedly logging out and back in.

It is **not** an API proxy, credential-sharing service, traffic interceptor, quota pool, or automatic rate-limit failover system.

## Proposed experience

```text
gemini-auth add personal --provider gemini-cli
gemini-auth add work --provider gemini-cli
gemini-auth list
gemini-auth use personal
gemini-auth current
gemini-auth exec work -- gemini
```

Antigravity CLI support is intentionally capability-gated:

```text
gemini-auth add personal-agy --provider antigravity-cli
gemini-auth use personal-agy
```

The first Antigravity implementation supports only the verified file-backed credential mode used in SSH/headless environments. Desktop keyring modification remains out of scope until its storage contract is documented or independently verified.

## Status

This repository contains the research, implementation plan, and compiling Phase 0 Rust workspace.
The CLI currently exposes only its global help/version shell; no credential-management feature has
been implemented yet.

## Documentation

- [Product scope](docs/01-product-scope.md)
- [Research findings](docs/02-research-findings.md)
- [Stack decision](docs/03-stack-decision.md)
- [Architecture](docs/04-architecture.md)
- [Repository structure](docs/05-repository-structure.md)
- [Credential and storage design](docs/06-credential-storage.md)
- [CLI specification](docs/07-cli-specification.md)
- [Security and policy boundaries](docs/08-security-policy.md)
- [Testing strategy](docs/09-testing-strategy.md)
- [Delivery phases](docs/10-delivery-roadmap.md)
- [Operations and release engineering](docs/11-operations-release.md)
- [Open questions](docs/12-open-questions.md)
- [Source catalog](docs/13-source-catalog.md)
- [Architecture decisions](docs/adr/)

## Non-negotiable principles

1. Official login only. The tool invokes or guides the official client's login flow; it never asks for a Google password.
2. Manual switching only. No automatic rotation based on quota, errors, or rate limits.
3. Local-only secrets. Credentials are never uploaded, synchronized, logged, or committed.
4. Fail closed. Unknown credential schemas, active client processes, unsafe permissions, or unsupported keyrings stop the operation.
5. Reversible changes. Every activation uses a transaction, backup, atomic replacement, and recovery journal.
6. Adapter isolation. Gemini CLI and Antigravity CLI storage rules never leak into generic profile-management code.

## License

Choose a license before implementation. MIT or Apache-2.0 are reasonable; Apache-2.0 adds an explicit patent grant.
