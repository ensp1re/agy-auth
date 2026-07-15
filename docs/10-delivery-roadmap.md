# Delivery Roadmap

The phases are outcome gates, not calendar promises. Do not begin a riskier phase merely because earlier code exists.

## Phase 0 — Repository foundation

Deliverables:

- Cargo workspace and crate boundaries.
- CI on Linux/macOS/Windows.
- lint, formatting, audit, deny, secret scanning.
- domain model and stable error codes.
- synthetic fixture policy.
- `SECURITY.md`, `CONTRIBUTING.md`, initial ADRs.

Exit gate: empty CLI builds reproducibly on all targets and dependency direction is enforced.

## Phase 1 — Gemini CLI isolated profiles (MVP)

Deliverables:

- official binary discovery and override;
- profile registry;
- isolated profile homes using `GEMINI_CLI_HOME`;
- `add`, `list`, `current`, `rename`, `remove`, `exec`, `doctor`;
- shell completions;
- masked account hints only if obtainable without token decoding/backend calls.

Important simplification: no token parsing, file swapping, keyring API, import/export, or global active-home replacement.

Exit gate: two profiles can be created and launched repeatedly on each target platform without credentials crossing homes.

## Phase 2 — Storage transaction engine

Deliverables:

- registry/provider locks;
- atomic write module;
- permission/ACL module;
- journal and recovery state machine;
- OS keyring-backed `SecretStore`;
- full fault-injection suite.

Exit gate: exhaustive injected failures demonstrate old-or-new atomicity and idempotent recovery.

## Phase 3 — Antigravity CLI file mode

Deliverables:

- discovery of verified active file path;
- process guard;
- tolerant credential-envelope validation;
- capture from current official login;
- transactional `use` and rollback;
- client-version/schema fingerprint warnings;
- explicit unsupported response for keyring-backed desktop mode.

Exit gate: SSH/headless Linux smoke tests switch two test profiles through official `agy`, including an access-token refresh performed by `agy` after switching.

## Phase 4 — Hardened import and encrypted portability

Deliverables:

- encrypted export bundle;
- import with ownership/permission/schema validation;
- collision resolution and audit summary;
- backup/restore documentation;
- security review of crypto and archive parsing.

Exit gate: no plaintext secret export in default paths; corrupted/tampered bundles fail safely.

## Phase 5 — Desktop keyring research, not automatic implementation

Research first:

- determine whether Google documents account-profile storage;
- identify exact keyring service/account semantics without enumerating unrelated secrets;
- verify interaction with running desktop processes and Electron state;
- obtain an independent security review.

Only add support if the adapter can be deterministic, reversible, and compatible without copying private application databases. Otherwise retain diagnostics-only support indefinitely.

## Phase 6 — Usability, packaging, ecosystem

- Homebrew, WinGet/Scoop, cargo-binstall, Debian/RPM where maintainable.
- man pages and shell integration.
- optional minimal desktop menu wrapper that calls the same application core.
- plugin/provider SDK only if a third provider has a legitimate local-switching use case.

## Permanently excluded roadmap items

- OpenAI/Anthropic/Gemini-compatible API server.
- Direct Google backend protocol.
- Quota display obtained via private endpoints.
- Automatic account fallback/rotation.
- Shared/team vault or cloud synchronization.
- Anti-ban/fingerprint behavior.

## Work breakdown for the first implementation PRs

1. Workspace, CI, linting, domain types.
2. Registry store plus migrations and locking.
3. Process runner and official-client discovery.
4. Gemini isolated-home adapter.
5. `add` and `exec` vertical slice.
6. list/current/rename/remove.
7. doctor and environment-conflict detection.
8. platform packaging and MVP documentation.
9. Transaction engine as a separately reviewed series.
10. Antigravity adapter only after transaction engine acceptance.
