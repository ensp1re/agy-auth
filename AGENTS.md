# Agent instructions

`agy-auth` is a capability-gated local profile manager for Google Antigravity CLI (`agy`).
Its purpose is deliberate switching between accounts controlled by the operator, without `/logout`
or repeated login. Prefer the smallest version-scoped integration that leaves model requests and
token refresh with the official client.

## Start here

Run `python3 scripts/harness/context.py` before editing. Resolve any reported state conflict first.
Use `.harness/manifest.json` to locate canonical sources and commands; do not treat chat summaries or
generated context as project truth.

The approved product is in `docs/01-product-scope.md` and `docs/07-cli-specification.md`. Current
architecture is in `docs/04-architecture.md`, with decisions in `docs/adr/`. Security constraints in
`docs/08-security-policy.md` override convenience. The delivery sequence is in
`docs/10-delivery-roadmap.md`; unresolved research is tracked in `docs/12-open-questions.md`.

## Authority and change gates

- Current user instructions authorize work only within approved product scope. A request to research
  a mechanism does not by itself authorize shipping it.
- Product-scope changes require updates to the owning specification and an accepted ADR before
  implementation. In particular, an independent OAuth enrollment flow remains gated until those
  sources explicitly approve its client identity, redirect flow, scopes, storage, and support policy.
- Reverse-engineered behavior is always version- and platform-scoped. Record the exact `agy` version,
  evidence, failure behavior, and reverification trigger.
- Real profile commands remain disabled until their capability gates pass. Experimental code must be
  non-default, visibly labeled, and unable to affect the production path accidentally.

## Permitted work

- Read public documentation, source code, binary metadata, symbols, process behavior, files,
  keyrings, logs, OAuth exchanges, and network behavior when necessary to discover the client
  contract.
- During explicitly authorized research, inspect and temporarily copy authentication state belonging
  to the operator. Use owner-only temporary storage, preserve the original, and clean up disposable
  artifacts after deriving the required evidence.
- Implement synthetic fixtures, parsers, serializers, atomic storage, isolated homes, launchers,
  rollback, diagnostics, and capability gates that are already approved by specifications and ADRs.
- Persist derived schemas, compatibility findings, and reproducible procedures without persisting
  reusable credentials or private conversation content.

## Hard prohibitions

- Never request, capture, or store a Google password.
- Never import, use, share, synchronize, or rotate credentials belonging to another person.
- Never call model, quota, entitlement, or private Antigravity backends from the product unless a
  later product decision explicitly changes that boundary.
- Never add quota-aware selection, automatic fallback, load balancing, fingerprint spoofing, a proxy,
  or a credential service.
- Never commit reusable tokens, OAuth codes, cookies, private keys, credential files, environment
  dumps, or private client logs. Routine output must not expose them.
- Never interpret authorization to inspect local state as authorization to publish it, revoke it,
  log out an account, or mutate unrelated default-client state.

## Working contract

- Select work from `.harness/state/work.json`; only one item may be `active`.
- Keep `.harness/state/handoff.json` current before stopping or changing sessions.
- Preserve crate dependency direction: CLI -> application -> domain/ports; infrastructure and
  providers implement ports. Provider code must not mutate active credentials directly.
- Tests and committed fixtures use obviously synthetic credentials only. Real state is permitted only
  in an authorized local research procedure and must remain outside Git.
- Prefer a small vertical change with its owning tests. Do not broaden the roadmap without approval.
- Real authentication-state mutation remains disabled until versioned `agy` capability evidence is
  approved in an ADR. ADR 0007 permits version-scoped isolated-home and credential-envelope research;
  production enablement still requires its remaining multi-account, concurrency, rollback, refresh,
  and upgrade gates.
- Run `python3 scripts/check.py` for the complete local gate. Use
  `python3 scripts/harness/validate.py` only for a narrow harness-state diagnosis.
- Review the complete diff before committing. Do not bypass hooks or weaken a failing check.

No client-specific adapter, hook, daemon, global configuration, or durable auto-memory is currently
approved. Nested instructions may add stable subtree rules but must not contradict this file.
