# Operations and Release Engineering

## Build profiles

- `dev`: fast compile, debug assertions, never used for public artifacts.
- `release`: LTO and symbol stripping only after crash diagnostics remain useful; panic abort may reduce secret-bearing dumps but must be evaluated against recovery behavior.
- Reproducible metadata: embed version, Git commit, target triple, and build timestamp policy; never embed builder paths or environment.

## Supported targets

Initial:

- `x86_64-unknown-linux-gnu`
- `aarch64-unknown-linux-gnu`
- `x86_64-apple-darwin`
- `aarch64-apple-darwin`
- `x86_64-pc-windows-msvc`
- `aarch64-pc-windows-msvc` after keyring/process tests are stable

Musl is deferred because desktop Secret Service integration and static linking assumptions need validation. Headless users can use isolated homes without a keyring in Phase 1.

## CI pipeline

Every pull request:

1. formatting and workspace metadata checks;
2. Clippy with warnings denied;
3. unit/property/integration tests;
4. feature-matrix build to catch accidental provider coupling;
5. `cargo audit` and `cargo deny`;
6. secret scan and generated-fixture prefix check;
7. dependency graph policy check;
8. documentation link and CLI help snapshot checks.

Nightly/scheduled:

- fuzzing budget;
- minimum supported Rust version if one is declared;
- official Antigravity CLI compatibility probe using `--version`/documented diagnostics only;
- dependency updates in isolated PRs.

## Release pipeline

- Release only from a protected signed tag.
- Build on clean hosted runners.
- Produce archives, SHA-256 checksums, SBOM, and provenance attestations.
- Code-sign/notarize macOS and sign Windows binaries when sustainable.
- Publish a human-reviewed changelog with credential schema/storage changes highlighted.
- Never auto-update the binary in v1; package managers are the update channel.

## Compatibility policy

Provider compatibility is reported separately from tool version:

```text
agy-auth doctor
agy-auth 0.1.0
Antigravity CLI 1.1.2: discovered (auth storage capability unverified)
Registry schema: 1
Pending recovery: none
```

The machine-readable compatibility ledger is
[`compatibility/antigravity-cli.json`](../compatibility/antigravity-cli.json). Each entry records a
client version, platform, bounded diagnostic evidence, capability gate, and reverification trigger.
`agy-auth doctor --json` also embeds the tool version, source revision, and compilation target without
builder paths, timestamps, or environment values.

When an official client changes storage:

- discovery may continue;
- capture/activation must stop if schema fingerprint or capability changes unexpectedly;
- issue a compatibility release only after synthetic and manual tests;
- never guess and overwrite.

## Telemetry

No telemetry in v1. If crash/usage telemetry is ever proposed, it must be opt-in, documented, independently reviewable, and structurally incapable of serializing profile/provider state. Local logs are also off by default.

## Backup and uninstall

Document two independent operations:

- Remove executable/package: leaves profiles intact.
- `agy-auth purge`: removes registry, managed isolated homes, and known keyring entries after a typed confirmation; it does not touch the official default client home unless a transaction created a clearly identified rollback artifact.

Uninstall scripts must never recursively delete a path derived only from an environment variable or unvalidated config.
