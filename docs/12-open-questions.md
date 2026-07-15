# Open Questions and Research Backlog

## Must resolve before real profile mutation

1. Does `agy` expose an official account, profile, or home-isolation command?
2. Which paths and keyring entries contain authentication state on Linux, macOS, Windows, and SSH?
3. Does an override isolate credentials, settings, workspace trust, conversations, and policy together?
4. How does `agy` behave if authentication state is switched while it or a child process is running?
5. Does refresh preserve unknown fields and remain associated with the selected account?
6. Which non-authenticated command can verify login success without making a model request?
7. What changes across `agy` versions, starting with the observed `1.1.2`?

## Product decisions

- Final product/repository name: proposed `antigravity-auth`; repository rename waits for review.
- Whether an “auth” name is too easily confused with an OAuth library.
- Whether diagnostics-only is valuable if no supported switching contract exists.
- Whether legacy Gemini CLI enterprise compatibility warrants a separate optional provider.
- License choice before distribution.

## Evidence format

Every resolved compatibility item records date, exact `agy` version, platform, storage mode,
non-secret reproduction steps, confidence, design consequence, and reverification trigger. Never
attach credentials, OAuth URLs, full emails, private logs, or environment dumps.
