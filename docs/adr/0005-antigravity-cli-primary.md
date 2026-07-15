# ADR 0005: Make Antigravity CLI the Primary Provider

- Status: Accepted
- Date: 2026-07-15

## Context

Google announced on May 19, 2026 that consumer Gemini CLI usage was transitioning to Antigravity CLI.
On June 18, 2026, Gemini CLI stopped serving consumer free, AI Pro, and AI Ultra requests. Enterprise
and paid API-key access remain separate compatibility cases. The target user can access Gemini models
through the installed Antigravity CLI (`agy 1.1.2`) but not Gemini CLI.

## Decision

Rename the product and binary to `agy-auth`, make Antigravity CLI the sole MVP provider, and
move Gemini CLI to deferred legacy/enterprise compatibility. Close the unmerged Gemini isolated-home
implementation. Do not enable Antigravity auth-state mutation until a supported or independently
verified, reversible storage/profile contract exists.

## Consequences

- Product naming and roadmap match the current consumer Google CLI.
- Existing registry and process infrastructure remain reusable.
- Prior Gemini isolated-home assumptions no longer drive the MVP.
- The next milestone is capability research, not credential implementation.
- Repository URL remains `ensp1re/agy-auth` until the migration PR is reviewed and merged.

## Sources

- [Google Developers Blog transition announcement](https://developers.googleblog.com/en/an-important-update-transitioning-gemini-cli-to-antigravity-cli/)
- [Google Antigravity CLI announcement](https://antigravity.google/blog/introducing-google-antigravity-cli)
- [Antigravity CLI product page](https://antigravity.google/product/antigravity-cli)
