# Contributing to agy-auth

Thank you for helping improve `agy-auth`. Contributions are welcome when they preserve the project's
local-only, manual, capability-gated security model.

## Code of conduct

Be respectful, specific, and constructive. Harassment, discrimination, personal attacks, and
publishing another person's private information are not acceptable. Assume good intent while
reviewing ideas, but evaluate security claims using reproducible evidence.

Maintainers may edit or remove abusive, unsafe, off-topic, or credential-bearing content and may
restrict participation when necessary to protect contributors and users.

## Start here

Before changing code:

1. Read [AGENTS.md](AGENTS.md).
2. Run the repository context command:

   ```bash
   python3 scripts/harness/context.py
   ```

3. Read the owning specification listed by the active work item in
   [`.harness/state/work.json`](.harness/state/work.json).
4. Review the [product scope](docs/01-product-scope.md),
   [architecture](docs/04-architecture.md), [CLI specification](docs/07-cli-specification.md), and
   [security policy](docs/08-security-policy.md).

Security policy and accepted architecture decisions override convenience.

## Product and security boundaries

Contributions must not:

- implement Google OAuth, ask for Google passwords, or refresh tokens independently;
- call private Gemini, Code Assist, Antigravity, quota, entitlement, or model endpoints;
- add automatic fallback, quota-aware selection, account scoring, or rotation;
- pool, synchronize, export, share, or proxy credentials;
- spoof official-client headers, identifiers, telemetry, or TLS behavior;
- print or persist full emails, tokens, OAuth URLs, authorization codes, private logs, or raw
  environment data;
- enable an unverified `agy` version or platform from filenames or assumptions alone.

Use only unmistakably synthetic credential fixtures. Never paste real authentication state into an
issue, test, commit, pull request, or CI log.

## Prerequisites

- Git
- Python 3
- Node.js 18+ and npm for repository-local Husky hooks
- Rust toolchain pinned by [`rust-toolchain.toml`](rust-toolchain.toml)
- Linux for the currently verified profile workflow
- Optional official `agy` installation for explicitly gated local integration tests

Clone and verify the repository:

```bash
git clone https://github.com/ensp1re/agy-auth.git
cd agy-auth
npm install
python3 scripts/harness/context.py
python3 scripts/check.py
```

`npm install` runs Husky's `prepare` script and configures the repository-local pre-commit hook. The
hook blocks commits unless this command succeeds:

```bash
cargo build --locked --release -p agy-auth-cli
```

Do not bypass the hook with `--no-verify`. Run the failing build directly and fix its error.

Do not run integration tests against real credentials unless the test explicitly documents that
requirement and you control every account and system involved.

## Development workflow

1. Select or propose one bounded work item.
2. Create a focused branch:

   ```bash
   git switch -c feat/short-description
   ```

3. Keep the change vertical: implementation, owning tests, specifications, and compatibility
   evidence should move together.
4. Preserve crate dependency direction:

   ```text
   agy-auth-cli -> application -> domain/ports
   ```

   Storage, process, and provider crates implement ports. Provider code must not silently mutate
   active credentials.
5. Run the complete local gate:

   ```bash
   python3 scripts/check.py
   ```

6. Review the full diff and update `.harness/state/handoff.json` before stopping.

Never bypass hooks, weaken checks, or hide a failing verification.

## Branch and commit conventions

Use short branch names with a conventional prefix:

- `feat/interactive-selector`
- `fix/credential-rollback`
- `docs/security-reporting`
- `test/login-recovery`
- `chore/dependency-update`

Use imperative Conventional Commit-style subjects:

```text
feat: add interactive profile selector
fix: restore credential state after failed activation
docs: clarify supported client versions
test: cover interrupted import recovery
```

Keep commits reviewable and avoid mixing unrelated formatting or refactoring with behavioral changes.

## Pull requests

A pull request should include:

- the user-visible problem and intended outcome;
- linked issue, specification, plan, or ADR;
- security and compatibility impact;
- exact verification commands and results;
- screenshots or terminal captures only when sanitized;
- documentation updates for changed commands or contracts;
- a clear list of unsupported or untested platforms and versions.

Before requesting review:

- rebase or merge the current target branch as appropriate;
- run `python3 scripts/check.py`;
- inspect the complete diff;
- confirm no secret, full email, private path, or raw client log is present;
- confirm new dependencies are necessary and license-compatible.

Hosted CI may supplement local verification but does not replace the repository's complete local
gate.

## Reporting bugs

Search existing issues first. For ordinary bugs, open a GitHub issue containing:

- `agy-auth` version or Git revision;
- `agy --version`;
- operating system, architecture, and SSH/desktop context;
- exact command and exit code;
- expected and actual behavior;
- minimal reproduction using synthetic or redacted state;
- whether the issue reproduces with `agy-auth doctor`.

Do not attach registry files, credential files, full emails, OAuth URLs, environment dumps, private
logs, or screenshots containing account details.

If the problem could expose credentials, bypass filesystem safety, overwrite linked files, corrupt
transactions, or leak private state, follow [SECURITY.md](SECURITY.md) instead of opening a public
issue.

## Documentation contributions

Documentation changes should distinguish:

- official Google behavior;
- independently verified project behavior;
- assumptions or unresolved research;
- supported versus merely observed versions and platforms.

Do not present reverse-engineered behavior as Google-supported.

## License

By contributing, you agree that your contribution is licensed under the repository's
[MIT License](LICENSE).
