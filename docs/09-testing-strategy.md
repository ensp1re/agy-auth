# Testing Strategy

## Test pyramid

```text
                 Small manual platform matrix
                /                              \
          End-to-end official-client smoke tests
         /                                      \
       CLI integration + transaction fault injection
      /                                            \
  Provider contract tests + storage integration tests
 /                                                  \
Domain unit tests + property tests + parser fuzzing
```

Most tests must not require a Google account or network.

## Synthetic credentials only

Fixtures use unmistakably fake values such as `TEST_ACCESS_TOKEN_DO_NOT_USE`. Avoid real token prefixes (`ya29`, `1//`) so secret scanners and reviewers can distinguish fixtures. Never copy a developer's credential file into the repository or CI artifacts.

## Unit tests

- Profile-name normalization and Unicode/case conflicts.
- Registry invariants and migrations.
- Capability gating.
- Transaction state machine transitions.
- Error-to-exit-code mapping.
- Redacted `Debug` output.
- Schema probes with unknown fields, missing fields, wrong types, oversized values, and invalid UTF-8.
- Permission policy decisions.

## Property tests

- Arbitrary crashes at every activation transition either preserve old active bytes or install complete new bytes; never partial bytes.
- Registry serialize/parse round trips preserve invariants.
- Random provider JSON cannot cause panic, path traversal, or secret display.
- Profile names cannot escape managed directories.
- Repeated recovery is idempotent.

## Fault injection

The storage port must allow deterministic failures at:

- lock acquisition;
- backup read/write;
- temp creation;
- short write;
- flush/fsync;
- chmod/ACL;
- rename;
- parent-directory fsync;
- verification read;
- registry commit;
- cleanup.

Each scenario asserts resulting bytes, journal state, rollback availability, and remediation text.

## Provider contract tests

Every adapter runs the same suite:

- discovery does not mutate;
- unsupported storage mode fails closed;
- validation never logs payload;
- capture and activation preserve opaque bytes;
- activation plan stays within declared provider paths;
- login delegates to the configured official executable;
- no network library is linked or invoked by provider code.

## CLI integration tests

Use `assert_cmd` with temporary config/data/home directories and fake official-client executables.

Scenarios:

- add/list/current/rename/remove happy paths;
- cancelled browser/login subprocess;
- duplicate name;
- missing official binary;
- child exit/signal propagation;
- JSON output schema snapshots;
- noninteractive refusal to prompt;
- no ANSI under `--json`/`--no-color`;
- no secret in stdout/stderr on every error path.

## Process/concurrency tests

- Two simultaneous activations: exactly one succeeds or waits; no corruption.
- Official client writes during attempted activation: process guard blocks operation.
- Stale lock recovery requires PID/start-time validation, not time alone.
- Ctrl-C during login leaves only a reserved/failed profile without secret.
- Ctrl-C during activation produces a recoverable journal.

## Platform matrix

| Platform | Unit/CLI CI | Keyring integration | Permission/atomic manual |
|---|---|---|---|
| Ubuntu latest | Required | Secret Service test container/session | Required |
| Ubuntu headless | Required | Expected unavailable path | Required |
| macOS current + previous | Required | Keychain test namespace | Required |
| Windows current | Required | Credential Manager test namespace | Required |

Keyring tests use a unique service namespace and delete it in teardown. They must never enumerate unrelated entries.

## Official-client smoke tests

Run manually or in a protected, non-fork CI environment with dedicated test accounts:

Linux authentication-state research must first follow the approval and isolation requirements in
[`docs/plans/linux-dedicated-account-observation.md`](plans/linux-dedicated-account-observation.md).

1. Antigravity CLI profile A login and repeat launch through a verified strategy.
2. Antigravity CLI profile B login; verify the selected identity without backend or token inspection.
3. Alternate manual selection and confirm isolated or transaction-safe behavior.
4. Interrupt activation at each supported transition and recover safely.
5. Upgrade `agy` and repeat schema/capability probes before re-enabling mutation.

No quota exhaustion tests and no multi-account retry tests.

## Fuzzing

Fuzz parsers for registry, journal, Antigravity envelope, imported bundle, and CLI JSON output inputs. Corpus must be synthetic. Fuzz targets run with low memory/file-size limits.

## Release acceptance

- All CI platforms green.
- Zero high/critical dependency advisories or documented exception.
- Secret scan clean.
- Migration up/down recovery tested.
- Installation, switching, rollback, and uninstall manually exercised.
- Documentation matches `--help` snapshots.
