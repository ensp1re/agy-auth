# Linux SSH Token Injection Observation: 2026-07-16

## Scope

- Platform: Ubuntu 24.04 family, Linux x86_64 VPS
- Client: `agy 1.1.2`
- Binary SHA-256: `70bf6eaf2e82fbb243db999b9c7c61fcf7f6e537f41980650eb2341ed84b24de`
- Storage mode: SSH file fallback beneath the effective home directory
- Operator authorization: inspect and temporarily copy authentication state on the operator's account

No credential value or reusable private artifact is committed in this record.

## Discovered file contract

With an SSH marker present and no Secret Service session, `agy` reads:

```text
$HOME/.gemini/antigravity-cli/antigravity-oauth-token
```

The observed file is a mode-`0600` JSON regular file owned by the effective user. Its version-scoped
shape is:

```json
{
  "auth_method": "consumer",
  "token": {
    "access_token": "<opaque string>",
    "token_type": "Bearer",
    "refresh_token": "<opaque string>",
    "expiry": "<RFC3339 timestamp>"
  }
}
```

The client binary retains the file-storage, token-refresh, keyring-fallback, and OAuth token-source
symbols that correspond to the runtime behavior.

## Full-record injection test

The existing record was copied without modification into a fresh mode-`0700` effective home. The
original record was not changed. The real client launched under `env -i`, the isolated `HOME`, a
synthetic loopback `SSH_CONNECTION`, and no D-Bus variables.

The copied access token was already expired. Runtime evidence showed that `agy`:

1. selected SSH file-based token storage;
2. loaded the expired record;
3. refreshed it through the stored refresh token;
4. authenticated the expected account without browser login;
5. persisted a replacement access token and later expiry in the isolated copy; and
6. reached `v1internal:loadCodeAssist`.

## Minimal-record injection test

A second fresh home received a newly serialized record containing the real refresh token, the fixed
schema above, a deliberately invalid placeholder access token, and an expiry in the year 2000. The
client again refreshed successfully, replaced the placeholder access token, authenticated without
starting browser login, and reached `loadCodeAssist`.

This proves that a saved refresh token plus the versioned JSON envelope is sufficient to restore a
consumer login for this installed client version. Copying a live access token is not required.

## Cleanup and impact

Both isolated homes, transcripts, refreshed token copies, logs, caches, and generated client state
were removed after deriving the booleans above. The default home token file was read but not modified.
No login, logout, prompt, or model request was initiated.

## Design consequence

Direct credential injection is technically possible for `agy 1.1.2` in Linux SSH file-fallback
mode. `agy-auth` can store a refresh credential per named profile and materialize the exact owner-only
JSON record before launching the official client. The official client remains responsible for token
refresh and backend requests.

Production enablement still requires two independently enrolled operator-controlled accounts,
encrypted or equivalently protected project-owned storage, atomic materialization and rollback,
running-process exclusion, refresh-token rotation capture, and upgrade invalidation.
