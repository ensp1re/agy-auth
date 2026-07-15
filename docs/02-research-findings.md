# Research Findings

## Product transition

Google announced on May 19, 2026 that consumer Gemini CLI usage was transitioning to Antigravity CLI.
Consumer free, AI Pro, and AI Ultra requests through Gemini CLI stopped on June 18, 2026. Enterprise
and paid API-key access remain distinct legacy cases. Antigravity CLI is therefore the MVP target.

## Verified local observation

On July 15, 2026, this Linux VPS contained a regular executable at `/root/.local/bin/agy`. A bounded,
non-authenticated `agy --version` probe reported `1.1.2`; `agy --help` described the terminal agent
surface. No credential files, keyrings, logs, conversations, model requests, or environment values
were accessed.

## Official Antigravity material

The Antigravity product page describes `agy` as the terminal-first surface. Google states that
Antigravity CLI shares the Antigravity agent harness and settings model with Antigravity 2.0. Current
official documentation shows settings beneath `~/.gemini/antigravity-cli/`, but a settings location is
not evidence of an authentication isolation or switching contract.

## What remains unverified

- official account/profile selection commands;
- complete credential and keyring storage by platform and mode;
- whether any home override isolates credentials as well as settings;
- refresh behavior after selection;
- interaction with running `agy` and Antigravity desktop processes;
- atomic/reversible behavior for any observed file-backed mode;
- compatibility across versions beyond observed `1.1.2`.

## Security conclusion

Do not harvest or piggyback on CLI OAuth, call private model/quota backends, or infer a contract from
filenames. The official client performs login, refresh, and requests. Until capability evidence is
verified, real authentication-state mutation remains disabled.

## Primary sources

- [Transitioning Gemini CLI to Antigravity CLI](https://developers.googleblog.com/en/an-important-update-transitioning-gemini-cli-to-antigravity-cli/)
- [Introducing Google Antigravity CLI](https://antigravity.google/blog/introducing-google-antigravity-cli)
- [Antigravity CLI product page](https://antigravity.google/product/antigravity-cli)
- [Antigravity CLI documentation](https://www.antigravity.google/docs)
