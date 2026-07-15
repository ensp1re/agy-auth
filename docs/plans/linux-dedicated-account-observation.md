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
- no existing `agy`, Gemini CLI, Antigravity desktop, browser, cloud CLI, or agent state in that user;
- `agy` version and binary digest recorded before the session;
- screen recording, shell history capture, tracing, core dumps, and verbose client logging disabled;
- a reviewer present for the observation and cleanup checklist.

Never run this protocol as `root`, in the operator's home, or on the operator's desktop/keyring
session. A container alone is not sufficient unless its D-Bus and keyring are also isolated.

## Allowed evidence

Record only:

- UTC timestamp, Linux distribution/kernel family, architecture, and `agy` version;
- whether a native Secret Service session is available;
- normalized path templates with usernames replaced by `<test-user>`;
- file type, owner category, permission bits, byte length, and modification-time deltas;
- keyring collection and item-count deltas inside the dedicated namespace, without labels,
  attributes, or secret values;
- process names and exit status;
- capability booleans such as `browser_flow_started`, `session_reused`, and `logout_removed_session`.

Do not record filenames if they contain identity data. Do not record full emails, account IDs, OAuth
URLs or codes, token/keyring contents, environment values, stdout/stderr containing login material,
file bytes, hashes of secret material, network captures, logs, conversations, or model responses.

## Observation sequence

Each phase requires a clean checkpoint and can be stopped independently.

1. **Baseline:** verify the isolated OS user and keyring session; record only the allowed metadata
   inventory before `agy` starts.
2. **Unauthenticated launch:** start the official client interactively with network access only long
   enough to confirm that the documented login chooser appears; do not retain its output.
3. **Dedicated login:** the account owner completes the official Google flow directly. The observer
   never handles or records the URL, code, password, cookies, or tokens.
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
- any secret, OAuth URL/code, full identity, or credential-bearing log becomes visible to capture;
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
status, and only the allowed evidence above. It must undergo a secret scan and human review before
being committed. Raw terminal output and temporary observation artifacts must never enter Git.
