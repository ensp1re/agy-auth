# Contributing

Start with `AGENTS.md`, then read the owning specification and architecture documents. Work on a
feature branch, keep one harness task active, and submit changes through a pull request.

## Product boundary

Contributions must not implement OAuth, direct Google backend calls, quota inspection, automatic
account fallback or rotation, credential sharing, fingerprint spoofing, or a proxy/server. Official
clients remain responsible for authentication, refresh, and requests.

Never use real credentials in code, tests, issues, logs, or review artifacts. Fixtures must be
unmistakably synthetic, such as `TEST_ACCESS_TOKEN_DO_NOT_USE`, and must not use real token prefixes.

## Local checks

Install the pinned toolchain from `rust-toolchain.toml`, then run:

```text
python3 scripts/check.py
```

Review the complete diff before committing. New dependencies require a purpose, license review, and
consideration of their build scripts and transitive supply-chain surface.
