# Source Catalog and Evidence Grades

This document separates stable public contracts from local observations and design inferences. Recheck current sources before implementing provider adapters.

## Primary sources

| Source | Used for | Evidence grade |
|---|---|---|
| [Gemini CLI configuration](https://github.com/google-gemini/gemini-cli/blob/main/docs/reference/configuration.md) | `GEMINI_CLI_HOME`, configuration precedence and user storage | Official documentation |
| [Gemini CLI authentication](https://github.com/google-gemini/gemini-cli/blob/main/docs/get-started/authentication.mdx) | Supported authentication modes and cached headless auth | Official documentation |
| [Gemini CLI FAQ](https://github.com/google-gemini/gemini-cli/blob/main/docs/resources/faq.md) | Security storage advice and prohibition on OAuth piggybacking | Official documentation |
| [Gemini CLI source repository](https://github.com/google-gemini/gemini-cli) | Future adapter verification, tests, configuration implementation | Official source |
| [Rust `keyring` documentation](https://docs.rs/keyring/latest/keyring/) | Cross-platform native credential-store feasibility | Upstream library documentation |
| [Rust `clap` documentation](https://docs.rs/clap/latest/clap/) | Typed CLI and completion implementation | Upstream library documentation |
| [Go keyring documentation](https://pkg.go.dev/github.com/zalando/go-keyring) | Go alternative evaluation | Upstream library documentation |

## Comparable implementations

| Project | Relevant pattern | Caution |
|---|---|---|
| [`Loongphy/codex-auth`](https://github.com/Loongphy/codex-auth) | Named account registry, explicit switching, import/export UX | Provider-specific direct API/usage behavior should not be copied |
| [`NoeFabris/opencode-antigravity-auth`](https://github.com/NoeFabris/opencode-antigravity-auth) | Multiple Google account storage and model configuration | Quota rotation/backend access are outside this project's boundary |
| [`lbjlaq/Antigravity-Manager`](https://github.com/lbjlaq/Antigravity-Manager) | Demonstrates broad compatibility complexity | It is a proxy; not a template for a local selector |
| [`badrisnarayanan/antigravity-claude-proxy`](https://github.com/badrisnarayanan/antigravity-claude-proxy) | Antigravity credential reuse ecosystem | Proxy behavior is intentionally excluded |

## Local observations

The Antigravity findings were obtained from an installed Linux build on 2026-07-15, not from public implementation source:

- file-backed token path under `.gemini/antigravity-cli/`;
- JSON envelope containing auth method and OAuth token object;
- owner-only file permissions;
- SSH-triggered file-storage log message;
- binary symbols for CLI token storage, keyring auth, save/load/remove, and OAuth completion;
- restart-time refresh performed by the official client.

Evidence grade: verified for the inspected build/environment, not a public compatibility contract.

## Inferences

- Antigravity must be stopped before file activation to prevent refresh-write races.
- The tool should preserve opaque bytes and unknown JSON fields.
- Desktop keyring support is unsafe to implement without exact identifiers and lifecycle tests.
- Isolated Gemini CLI homes are safer than copying individual credential documents.

Evidence grade: architecture conclusions derived from the primary sources and observations. Each must be validated by adapter contract tests.

## Reverification triggers

Repeat relevant research when:

- Gemini CLI changes major/minor authentication behavior or removes/changes `GEMINI_CLI_HOME`;
- Antigravity CLI changes its major version, token filename, storage log, or credential envelope;
- a provider adds official multi-account support;
- keyring crate changes its backend architecture or minimum platform versions;
- a security report concerns credential storage, symlink handling, or account switching;
- policy documentation changes materially.

## Citation rule for future docs

Every provider-storage claim should cite an official document/source line when available. Otherwise label it `observed`, with client version and OS, or `inferred`. Never convert an observed private implementation detail into an unconditional public guarantee.
