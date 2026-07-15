# Open Questions and Research Backlog

## Must resolve before MVP

1. Does current Gemini CLI scope every credential/keyring lookup to `GEMINI_CLI_HOME` on Linux, macOS, and Windows?
2. What is the least invasive official command sequence that establishes login success without making a model request?
3. Which authentication-overriding environment variables should `exec` warn about, and which may it safely unset only for the child process?
4. Does whole-home isolation unintentionally duplicate trusted-folder decisions or organization policy that should remain global?
5. How should a managed enterprise environment prohibit personal profiles through local policy?

## Must resolve before Antigravity support

1. Confirm file path and envelope across at least two Antigravity CLI versions and all three platforms in SSH/file mode.
2. Determine reliable running-process detection names and child relationships.
3. Confirm atomic replacement behavior when `agy` is stopped and when launched afterward.
4. Determine whether refreshed credentials preserve unknown JSON fields.
5. Decide whether storing the complete token envelope in an OS keyring exceeds platform item-size limits.
6. Establish a safe response when the active file is absent but a keyring token exists.

## Security research

- Windows DACL creation and verification without shelling out.
- macOS Keychain binary-signing ACL behavior across upgrades.
- Linux Secret Service availability under SSH and desktop sessions.
- Hardlink detection and replacement semantics.
- Swap/core-dump exposure and whether process-level mitigations are appropriate.
- Safe encrypted export format and password UX.

## Product questions

- Should `use` exist for Gemini CLI or should `exec` be the only supported mechanism?
- Should profile homes isolate all history/settings or share non-secret configuration through an explicit template?
- Should account hints be collected at all? Omitting them is more private; masked hints improve usability.
- Is `gemini-auth` too easily confused with an OAuth library? Alternative names could emphasize profiles, such as `gemini-profile`.
- What license best matches contributor expectations and patent concerns?

## Evidence that would change the design

- An official Google multi-account/profile command would make most switching logic unnecessary; integrate or defer to it.
- An official Antigravity home/profile override would replace token-file activation.
- Evidence that `GEMINI_CLI_HOME` does not isolate credentials would require a different supported strategy or narrower platform scope.
- Provider terms explicitly prohibiting local auth-state selection would require reevaluating or discontinuing affected support.

## Research discipline

Record each resolved item as an ADR or compatibility note with:

- date and client version;
- operating system;
- exact non-secret observation;
- reproduction procedure;
- confidence level;
- design consequence;
- expiration/reverification trigger.

Never attach real credential files, OAuth URLs, tokens, full emails, or private logs to issues.
