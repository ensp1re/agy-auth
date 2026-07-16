# Research Findings

## Product transition

Google announced on May 19, 2026 that consumer Gemini CLI usage was transitioning to Antigravity CLI.
Consumer free, AI Pro, and AI Ultra requests through Gemini CLI stopped on June 18, 2026. Enterprise
and paid API-key access remain distinct legacy cases. Antigravity CLI is therefore the MVP target.

## Verified local observation

On July 15, 2026, this Linux VPS contained a regular executable at `/root/.local/bin/agy`. Bounded,
non-authenticated `agy --version`, `agy --help`, and subcommand-help probes reported `1.1.2` and
enumerated the public command surface. The available subcommands were `agent`, `agents`, `changelog`,
`help`, `install`, `models`, `plugin`, `plugins`, and `update`. None exposed account, login, logout,
profile, home, keyring, or authentication configuration flags. No credential files, keyrings, logs,
conversations, model requests, environment values, or interactive sessions were accessed.

## Official Antigravity material

The Antigravity product page describes `agy` as the terminal-first surface. Google states that
Antigravity CLI shares the Antigravity agent harness and settings model with Antigravity 2.0. The
official installation and authentication guide says the client silently loads a token profile from
the operating system's native secure keyring, falls back to browser or remote SSH OAuth when no
saved session exists, and provides the interactive `/logout` command to purge saved authentication
profiles. It does not document profile selection, an alternate credential home, keyring namespace
overrides, credential export/import, or non-destructive logout. Settings paths and `/config` are not
evidence of an authentication isolation contract.

## Capability inventory conclusion

For `agy 1.1.2`, the documented public surface supports authentication performed by the official
client and destructive logout, but not account selection. Subsequent authorized reverse engineering
identified and verified a Linux SSH file-fallback contract beneath the effective home. The contract
is unofficial and version-scoped, but two-account runtime validation now supports implementing
capability-gated profile commands.

Earlier isolated Linux attempts silently recognized an existing identity because their Secret
Service/UID boundary was not independent. ADR 0007 superseded that conclusion. A fresh-UID enrollment
plus two disposable SSH file-backed homes proved distinct concurrent identities, restart persistence,
forced refresh, clean-home non-reuse, and scoped teardown. See the July 16 two-account validation.

## What remains unverified

- official account/profile selection commands;
- desktop keyring behavior outside the verified Linux SSH file mode;
- interaction with running `agy` and Antigravity desktop processes;
- rollback/recovery behavior in the public command composition;
- compatibility across versions beyond observed `1.1.2`.

## Security conclusion

Do not call private model/quota backends or infer a contract from filenames. The official client
performs refresh and requests. The verified integration materializes only the reviewed consumer
credential envelope and remains fail-closed for other versions and platforms.

## Primary sources

- [Transitioning Gemini CLI to Antigravity CLI](https://developers.googleblog.com/en/an-important-update-transitioning-gemini-cli-to-antigravity-cli/)
- [Introducing Google Antigravity CLI](https://antigravity.google/blog/introducing-google-antigravity-cli)
- [Antigravity CLI product page](https://antigravity.google/product/antigravity-cli)
- [Antigravity CLI installation and authentication](https://antigravity.google/docs/cli-install)
- [Antigravity CLI getting started](https://antigravity.google/docs/cli-getting-started)
