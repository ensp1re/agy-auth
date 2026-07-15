# agy-auth

`agy-auth` is a planned local profile manager for Google Antigravity CLI (`agy`). It aims to
let one person select among their own legitimate Google account contexts without implementing OAuth,
calling Google model backends, pooling quota, or sharing credentials.

## Why Antigravity CLI

Google announced on May 19, 2026 that consumer Gemini CLI access was transitioning to Antigravity
CLI. Consumer requests through Gemini CLI stopped on June 18, 2026; enterprise and paid API-key cases
remain separate compatibility paths. Antigravity CLI is therefore the primary product target.

The exact `agy` profile and credential-storage contract is still under research. Until a supported,
reversible isolation or switching mechanism is verified, this project must remain diagnostics-only
for real Antigravity authentication state.

## Intended experience

The target interface, subject to capability verification, is:

```text
agy-auth add personal
agy-auth add work
agy-auth list
agy-auth use personal
agy-auth current
agy-auth exec work -- agy
```

Commands that require an unverified storage capability must fail closed with a clear diagnostic.

## Current status

The repository has a diagnostics-only `agy-auth doctor` command, non-secret profile registry, safe
client discovery, bounded process execution, filesystem safety checks, and a verified `agy 1.1.2
--version` compatibility entry. It does not read, write, capture, or switch real authentication
state.

## Non-negotiable principles

1. Official login only; the tool never requests a Google password or implements OAuth.
2. Manual profile selection only; no quota-aware fallback, scoring, or rotation.
3. Local-only secrets; nothing is uploaded, synchronized, logged, or committed.
4. Fail closed on undocumented storage, schemas, permissions, links, or running-client conflicts.
5. Reversible mutations only after a versioned Antigravity storage contract is verified.
6. The official `agy` binary remains responsible for login, refresh, and model requests.

See [product scope](docs/01-product-scope.md), [architecture](docs/04-architecture.md),
[security policy](docs/08-security-policy.md), and [roadmap](docs/10-delivery-roadmap.md).

## License

Choose a license before publication or package distribution.
