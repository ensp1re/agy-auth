# Agent instructions

`gemini-auth` is a local-only profile switcher for credentials created and used by official Google
clients. Preserve the hard boundary: never implement OAuth, backend API calls, quota-aware selection,
credential sharing, or automatic account rotation. Never print, log, commit, or retain real secrets.

## Start here

Run `python3 scripts/harness/context.py` before editing. Resolve any reported state conflict first.
Use `.harness/manifest.json` to locate canonical sources and commands; do not treat chat summaries or
generated context as project truth.

The approved product is in `docs/01-product-scope.md` and `docs/07-cli-specification.md`. Current
architecture is in `docs/04-architecture.md`, with decisions in `docs/adr/`. Security constraints in
`docs/08-security-policy.md` override convenience. The delivery sequence is in
`docs/10-delivery-roadmap.md`; unresolved research is tracked in `docs/12-open-questions.md`.

## Working contract

- Select work from `.harness/state/work.json`; only one item may be `active`.
- Keep `.harness/state/handoff.json` current before stopping or changing sessions.
- Preserve crate dependency direction: CLI -> application -> domain/ports; infrastructure and
  providers implement ports. Provider code must not mutate active credentials directly.
- Use synthetic credential fixtures only. Never use realistic token prefixes or copied local state.
- Prefer a small vertical change with its owning tests. Do not broaden the roadmap without approval.
- Run `python3 scripts/harness/validate.py` for harness and policy checks. Once Rust exists, also run
  the applicable Cargo checks documented in `docs/09-testing-strategy.md`.
- Review the complete diff before committing. Do not bypass hooks or weaken a failing check.

No client-specific adapter, hook, daemon, global configuration, or durable auto-memory is currently
approved. Nested instructions may add stable subtree rules but must not contradict this file.
