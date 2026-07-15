# Linux SSH File-Fallback Observation: 2026-07-16

## Scope

- Platform: Ubuntu 24.04 family, Linux x86_64 VPS
- Client: `agy 1.1.2`
- Binary SHA-256: `70bf6eaf2e82fbb243db999b9c7c61fcf7f6e537f41980650eb2341ed84b24de`
- Hypothesis: a synthetic SSH environment without D-Bus selects a home-local file fallback instead
  of the Secret Service path used in the earlier corrected retry
- Secret or identity data retained: none

The product owner explicitly authorized read-only reverse engineering of the installed client. This
observation sent no terminal input, invoked no login/logout command, and made no model request.

## Boundary

A new disposable non-root OS user received a fresh mode-0700 home, runtime directory, and working
directory. The copied client binary matched the recorded installed binary. The process used `env -i`
with only HOME, XDG home/runtime roots, PATH, locale, terminal type, and a synthetic loopback
`SSH_CONNECTION` marker. No D-Bus or Secret Service variable was supplied.

The client ran under a pseudo-terminal with stdin closed. A bounded eight-second timeout sent an
interrupt and then killed the process. Raw terminal output existed only in a temporary transcript;
automation derived booleans without printing it, then deleted it before cleanup.

## Result

- existing account identity visible: **false**;
- recognizable login/authentication screen visible: **false**;
- client process exit: forced termination after the bounded timeout (`137`);
- disposable-home entry count at bounded depth: 4 before, 37 after;
- remaining client processes after termination: 0;
- raw transcript retained: no.

This differs materially from both July 15 attempts, which silently displayed an existing account.
It supports—but does not prove—the hypothesis that SSH file-fallback selection avoids the reused
Secret Service identity. Because no login chooser was recognized, the observation is inconclusive
and does not satisfy any ADR 0007 enablement gate.

## Cleanup

The disposable process, OS user, home, runtime directory, copied binary, working directory, and raw
transcript were removed. A final existence check confirmed each was absent. No default operator
state was read or modified.

## Next bounded experiment

Repeat the same SSH/no-D-Bus boundary with a human at the terminal and a dedicated test account. Wait
long enough to identify the official login chooser, stop if any existing identity appears, and let
the account owner complete login only after a separate explicit approval. Record no raw output,
email, OAuth URL/code, token, log, or model interaction.
