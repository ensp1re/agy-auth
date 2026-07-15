# Linux Observation Record: 2026-07-15

## Scope

- Platform: Ubuntu 24.04 family, Linux x86_64 VPS
- Client: `agy 1.1.2`
- Intended boundary: disposable non-root user, fresh home, dedicated D-Bus, and GNOME Secret Service
- Secret or identity data retained: none

## Result

The attempt stopped during the unauthenticated-launch phase. The client silently identified an
existing signed-in account before the dedicated test-account flow ran. A model request was also
submitted interactively, contrary to the observation protocol. These facts prove that the attempted
process boundary was not isolated and invalidate all authentication-storage conclusions from the
run.

The launch overrode `HOME`, `XDG_RUNTIME_DIR`, and `PATH`, but inherited the remainder of the root
environment and retained the operator's working directory. No environment values or authentication
state were inspected, so the exact reuse mechanism is intentionally undetermined.

## Response and cleanup

- Terminated the disposable user's `agy` process.
- Did not invoke `/logout` because it could have removed the reused existing session.
- Did not inspect keyring entries, credential files, environment values, logs, or client state.
- Removed all remaining disposable-user processes, the user, its home, and its runtime directory.
- Retained installed system packages only; they contain no account state.

## Consequence

Do not repeat the inherited-environment launch. Any revised attempt requires a fresh approval and
must use an empty environment with an explicit allowlist, a dedicated working directory, a newly
created disposable user, and a newly verified isolated Secret Service session. Silent recognition of
any account before dedicated login is an immediate stop condition.
