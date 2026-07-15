# ADR 0002: Prefer Isolated Official-Client Homes

- Status: Superseded by ADR 0005
- Date: 2026-07-15

## Context

Copying OAuth token files couples the tool to private schema and refresh behavior. Gemini CLI officially documents `GEMINI_CLI_HOME`.

## Decision

Use one Gemini CLI home per profile and launch the official client with that home. Do not parse/copy Gemini OAuth tokens in the default workflow.

## Consequences

- Strong account isolation and resilience to token schema changes.
- Settings/history/cache are isolated too, using extra disk and requiring optional configuration templates.
- Users normally invoke `antigravity-auth exec <profile> -- gemini` or a shell alias.
- Antigravity requires a different fallback because no equivalent override is currently confirmed.
