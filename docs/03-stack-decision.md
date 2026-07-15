# Technology Stack Decision

## Decision summary

Build the CLI in stable Rust as a Cargo workspace. Produce one native binary for Linux, macOS, and Windows. Use typed provider adapters, a JSON metadata registry, native OS credential storage where available, and an encrypted-file fallback only after the MVP.

## Evaluation criteria

Weights reflect the product's risk profile.

| Criterion | Weight | Rust | Go | TypeScript/Node |
|---|---:|---:|---:|---:|
| Single binary and distribution | 20 | 19 | 20 | 8 |
| Safe filesystem transactions | 18 | 18 | 16 | 12 |
| Secret-memory discipline | 15 | 14 | 10 | 6 |
| Cross-platform keyring | 12 | 10 | 10 | 9 |
| Type-safe schema/adapters | 12 | 12 | 10 | 9 |
| Contributor accessibility | 8 | 5 | 7 | 8 |
| Startup/runtime footprint | 7 | 7 | 7 | 3 |
| Testing/tooling | 5 | 5 | 5 | 5 |
| Supply-chain surface | 3 | 2 | 3 | 1 |
| **Weighted total / 100** | | **92** | **88** | **61** |

## Why Rust

- Strong ownership and type modeling reduce accidental secret duplication and adapter cross-contamination.
- `Zeroizing`/secret wrapper types can clear sensitive buffers on drop, acknowledging that complete zeroization cannot be guaranteed across all copies and OS layers.
- Native executables require no Node/Python runtime and start quickly.
- Cross-compilation and release artifacts are well-supported.
- `clap` derive provides a typed CLI contract and generated help/completions.
- `serde` supports tolerant read/strict write schema handling.
- The current `keyring` ecosystem supports macOS Keychain, Windows Credential Store, and Linux Secret Service. Headless Linux remains a known limitation and therefore needs a file-vault fallback design.

## Why not TypeScript

TypeScript would match `codex-auth` and lower the contribution barrier, but it adds a Node runtime, larger dependency graph, more ways for secrets to become ordinary immutable strings, and less predictable single-file distribution. Native keyring modules also complicate installation across architectures.

## Why not Go

Go is a credible second choice and offers excellent cross-compilation. [`zalando/go-keyring`](https://pkg.go.dev/github.com/zalando/go-keyring) covers the three desktop platforms and is simple to mock. Rust wins narrowly because the design benefits from richer enums/traits for capability gates, RAII cleanup, and secret wrapper types. If maintainers are substantially more experienced in Go, choosing Go would be reasonable; architecture matters more than the eight-point score difference.

## Proposed Rust components

Versions should be pinned when implementation begins; do not copy the research-date versions blindly.

| Concern | Preferred crate/approach | Notes |
|---|---|---|
| CLI | `clap` derive | Commands, aliases, help, completions |
| Serialization | `serde`, `serde_json` | Registry and provider schema probes |
| Errors | `thiserror`; `anyhow` only at binary boundary | Preserve typed domain errors |
| Directories | `directories` | Platform data/config/cache paths |
| Secrets | `secrecy` and `zeroize` | Prevent debug/display; clear owned buffers |
| Keyring | `keyring` with explicit platform features | Feature-gate and test each backend |
| File locking | `fs2` or platform-specific lock abstraction | Registry/activation lock |
| Atomic writes | same-directory temp file + fsync + rename | Implement small audited module |
| Hashing | `sha2` | Integrity fingerprints, never token logging |
| Encryption fallback | defer; later `age` or AEAD + Argon2id | Avoid inventing cryptography in MVP |
| Process discovery | `sysinfo` behind adapter interface | Advisory detection, never kill by default |
| Testing | built-in tests, `tempfile`, `assert_cmd`, `predicates`, `proptest` | Unit, CLI, property tests |
| Snapshot tests | `insta` with redaction review | Help/output only; never secret fixtures |
| Logging | `tracing` with a redacting field layer | Disabled/minimal by default |

## Runtime architecture

The executable is synchronous. Operations are filesystem/keyring/process-bound and do not need an async runtime. Avoiding Tokio reduces binary size and complexity. The official client subprocess may be awaited using `std::process::Command`.

## Data formats

- Registry: versioned JSON for transparency and easy recovery.
- Journal: newline-delimited JSON or one JSON transaction file per activation.
- Secret payload: opaque bytes/string stored through `SecretStore`.
- No SQLite in v1; expected account counts are tiny and atomic JSON is simpler to audit.

## Rejected stack decisions

- Shell scripts: unsafe quoting, weak Windows support, hard-to-test failure recovery.
- Python: good prototype language but packaging/keyring variability undermines the single-binary goal.
- Tauri/Electron UI: unnecessary for an account switcher; expand only after the CLI is stable.
- Background daemon: no functional need and increases secret exposure.
