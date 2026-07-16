# macOS Keychain profile validation

## Status

macOS Keychain candidate awaiting two-account black-box validation.

## Scope

Validate `agy-auth` profile import, isolated enrollment, switching, refresh preservation, restart,
and rollback for Antigravity CLI `1.1.3` on Apple Silicon macOS.

The candidate contract is:

- generic-password service: `gemini`;
- generic-password account: `antigravity`;
- default official-client credential location: Apple login Keychain;
- Keychain value format: raw non-JSON refresh credential;
- isolated SSH-style homes: bounded file credential at
  `.gemini/antigravity-cli/antigravity-oauth-token`.

The candidate uses Apple Security.framework directly. Credential bytes are never passed through
command-line arguments or rendered in output.

## Existing evidence

On 2026-07-16, an operator running `agy 1.1.3` on `aarch64-apple-darwin` confirmed:

- `security find-generic-password -s gemini -a antigravity` succeeds;
- the default `~/.gemini/antigravity-cli/antigravity-oauth-token` file is absent;
- `agy-auth doctor` discovers the official client and reports a healthy empty registry.

Google's CLI troubleshooting documentation states that macOS uses Apple Keychain for session
tokens. Independent implementations identify the same `gemini` / `antigravity` generic-password
item and report file fallback for relocated homes. Those external observations are supporting
evidence only; they do not replace this project's black-box gates.

## Validation gate

Run a locally built candidate with:

```bash
export AGY_AUTH_MACOS_KEYCHAIN=1
```

The contract may be promoted only after all of these pass:

1. `agy-auth add personal` imports the currently active account without changing it.
2. `agy-auth login work` completes through official `agy` and creates a distinct ready profile.
3. `agy-auth switch personal`, followed by plain `agy`, shows the original account.
4. `agy-auth switch work`, followed by plain `agy`, shows the second account.
5. Both switches survive closing and restarting the terminal.
6. A normal authenticated request succeeds after each switch.
7. Token refresh does not contaminate the other profile.
8. An induced selected-metadata failure restores the original Keychain credential.
9. No credential value appears in stdout, stderr, process arguments, logs, registry metadata, or
   Git state.

Until this gate passes and an ADR records the evidence, normal macOS capability reporting remains
disabled and the candidate requires the explicit environment opt-in.
