# ADR 0007: Permit Versioned Unofficial Isolated Homes

- Status: Accepted; production capability remains gated
- Date: 2026-07-16
- Supersedes: ADR 0006

## Context

The product owner clarified that `agy-auth` exists specifically to supply account isolation that
Antigravity CLI does not officially expose. Requiring an official Google profile contract would make
the primary product goal unreachable.

An externally supplied reverse-engineering report describes a Linux SSH file-backed mode beneath the
effective home directory. That report is an untrusted observation, not project policy or sufficient
capability evidence. Earlier dedicated-user experiments also reused an existing identity despite a
fresh home and runtime, so environment isolation must be proven rather than inferred from filenames.

The product still excludes quota-driven rotation and using another person's credentials. Authorized
research may inspect private API behavior, OAuth state, tokens, credential storage, and keyrings to
discover the real client contract. Shipping an independent OAuth or backend implementation remains
a separate architecture decision; the current product still delegates login, refresh, and model
requests to `agy`.

## Decision

Permit a version-scoped, unofficial isolated-home provider for Linux SSH/headless use when black-box
tests demonstrate that separate managed environments remain separate. `agy-auth` may create
owner-only profile homes and runtime directories, clear the child environment, apply an reviewed
allowlist of home/XDG/PATH/locale variables, and launch the official executable directly with argv.

The first implementation is a non-interactive synthetic process primitive. It must prove distinct
homes receive distinct environment roots, inherited variables do not leak, unsafe directories fail
closed, and shell metacharacters are never evaluated. It does not enable `add`, `use`, `exec`, or
doctor capability claims.

Real-client enablement requires a later reviewed experiment with two operator-controlled test
accounts. The experiment may inspect and compare the client state necessary to establish isolation,
including credential storage and private logs. Raw sensitive artifacts must remain outside Git and
published evidence; the committed record contains only the derived contract and reproducible steps.

## Enablement gate

Linux isolated-home switching becomes `verified` only when all of the following are reproducible:

1. two owner-only homes complete login through the official client without token handling by
   `agy-auth`;
2. launching each home selects its expected dedicated account across process restart and token
   refresh;
3. a clean third home does not silently reuse either identity;
4. simultaneous processes cannot cross profile state;
5. upgrade or environment changes fail closed and mark evidence stale;
6. teardown removes only project-owned test homes and leaves official default state untouched.

Until then, `profileSwitching` remains false and `authStateMutation` remains false.

## Consequences

- Reverse-engineered black-box behavior may become a project-owned compatibility contract after
  review; it is never represented as Google-supported.
- Linux SSH/headless is the only candidate platform. Desktop keyrings, macOS, and Windows remain
  unsupported.
- Profiles isolate settings, cache, conversations, and other client state along with credentials;
  this costs disk space but avoids copying opaque authentication state.
- Every `agy` version change or unexpected identity reuse invalidates the capability evidence.
- The failed ADR 0006 experiments remain relevant warnings but no longer permanently prohibit a
  materially different, reviewed hypothesis.
