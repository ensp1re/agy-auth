# Linux Dedicated-Account Observation Protocol

## Purpose

Determine whether `agy` authentication can be isolated through a supported mechanism on Linux while
protecting the operator's existing account and credentials. This protocol is a research gate, not an
authorization to execute login, logout, keyring, or credential-state operations.

## Required approval and prerequisites

Execution requires a separate explicit approval after all prerequisites are confirmed:

- a Google account created solely for this test and permitted by its owner and applicable policies;
- a disposable, non-privileged Linux OS user with a fresh home directory;
- a dedicated user session and Secret Service/keyring instance not shared with the operator;
- a dedicated working directory owned by the test user, never the operator's current directory;
- a process environment created with `env -i` and an explicit allowlist; overriding selected
  variables in an inherited environment is forbidden;
- no existing `agy`, Gemini CLI, Antigravity desktop, browser, cloud CLI, or agent state in that user;
- `agy` version and binary digest recorded before the session;
- screen recording, shell history capture, tracing, core dumps, and verbose client logging disabled;
- a reviewer present for the observation and cleanup checklist.

Never run this protocol as `root`, in the operator's home, or on the operator's desktop/keyring
session. A container alone is not sufficient unless its D-Bus and keyring are also isolated.

## Evidence handling

The committed research record should prefer:

- UTC timestamp, Linux distribution/kernel family, architecture, and `agy` version;
- whether a native Secret Service session is available;
- normalized path templates with usernames replaced by `<test-user>`;
- file type, owner category, permission bits, byte length, and modification-time deltas;
- keyring collection and item-count deltas inside the dedicated namespace, without labels,
  attributes, or secret values;
- process names and exit status;
- capability booleans such as `browser_flow_started`, `session_reused`, and `logout_removed_session`.

During an explicitly authorized local investigation, the operator may inspect filenames, identities,
OAuth URLs or codes, token/keyring contents, environment values, client output, file bytes, hashes,
network captures, logs, conversations, and model responses when they materially help discover the
client contract. Keep raw captures outside Git with owner-only permissions and delete them when they
are no longer needed. Commit only derived findings and non-reusable examples.

## Observation sequence

Each phase requires a clean checkpoint and can be stopped independently.

1. **Baseline:** verify the isolated OS user, working directory, clean allowlisted environment, and
   keyring session; record only the allowed metadata inventory before `agy` starts. A reviewer must
   verify that no cloud, Google, OAuth, credential-helper, or operator session variable was inherited
   without printing the environment values.
2. **Unauthenticated launch:** start the official client interactively and retain local diagnostic
   output when needed to identify the login path.
3. **Dedicated login:** the account owner completes the official Google flow. Authorized tooling may
   observe the resulting URL, code exchange, cookies, or tokens when required to identify storage and
   switching behavior; it must never request or record the account password.
4. **Post-login delta:** exit without a model request, then compare allowed filesystem and dedicated
   keyring metadata with the baseline.
5. **Session reuse:** relaunch once to observe only whether silent authentication succeeds; exit
   before submitting a prompt.
6. **Logout behavior:** only after a separate destructive-action confirmation, invoke the official
   `/logout`, then observe whether the dedicated session and metadata are removed.
7. **Cleanup:** delete the disposable keyring/session and OS user, revoke the test account session
   through Google's account controls, and verify that no test process remains.

Do not attempt a second account, copy state, rename paths, override `HOME`, modify D-Bus variables,
move keyring items, interrupt writes, or test rollback in this protocol. Those are separate proposals
requiring the baseline results and another approval.

## Stop conditions

Stop immediately, preserve no additional output, and clean up when:

- the process connects to the operator's existing keyring or home;
- the official client silently identifies an account before the dedicated login completes;
- a sensitive capture cannot be kept within the operator-controlled research boundary;
- the dedicated keyring boundary cannot be proven;
- `agy` starts a model request or unrelated tool action;
- observed behavior differs materially from the official documentation;
- cleanup or account-session revocation cannot be completed;
- the installed binary/version changes during the run.

## Interpretation rules

- A discovered path or keyring item is an observation, not a supported contract.
- Silent reuse proves only that the official client can retrieve its own session.
- `/logout` removal proves destructive cleanup, not safe profile switching.
- No mutation implementation is authorized unless official support or independently reproducible,
  reversible behavior is reviewed in a new ADR.

## Completion record

The research record must state which phases ran, which were skipped, all stop conditions, cleanup
status, and the derived evidence above. It must undergo a secret scan and human review before being
committed. Raw credentials and private observation artifacts must never enter Git.

## Failed attempt supersession

The July 15, 2026 attempt used `runuser` with selected environment overrides but did not clear the
inherited root environment or change away from the operator's working directory. `agy 1.1.2`
silently identified an existing account, proving that isolation was not established. The process was
terminated without `/logout`, and the disposable user, home, runtime directory, and processes were
removed. Do not repeat that launch form. See
[`docs/research/linux-observation-2026-07-15.md`](../research/linux-observation-2026-07-15.md).
