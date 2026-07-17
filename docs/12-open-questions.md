# Open Questions and Research Backlog

## Resolved for Linux SSH 1.1.2 and 1.1.3

The isolated-home and default-token activation contracts are approved by ADRs 0007 and 0008.

## Remaining compatibility research

1. Which minimal cleared environment makes Linux SSH homes independent for `agy 1.1.2`?
2. Which paths and keyring entries contain authentication state on Linux, Windows, and SSH?
3. Does an override isolate credentials, settings, workspace trust, conversations, and policy together?
4. How does `agy` behave if authentication state is switched while it or a child process is running?
5. Does refresh preserve unknown fields and remain associated with the selected account?
6. Which observation most reliably verifies the expected dedicated account while keeping sensitive
   captures local and out of Git?
7. What changes across `agy` versions, starting with the observed `1.1.2`?

## Product decisions

- Product, binary, crates, and repository use the approved `agy-auth` name.
- Whether an “auth” name is too easily confused with an OAuth library.
- Whether diagnostics-only is valuable if no supported switching contract exists.
- Whether legacy Gemini CLI enterprise compatibility warrants a separate optional provider.
- MIT license approved for distribution on July 16, 2026.

## Evidence format

Every resolved compatibility item records date, exact `agy` version, platform, storage mode,
reproduction steps, confidence, design consequence, and reverification trigger. Sensitive raw
artifacts may be used locally but must not be attached to the repository or public reports.
