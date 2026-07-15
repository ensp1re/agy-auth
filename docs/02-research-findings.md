# Research Findings

Research date: 2026-07-15.

## Comparable project: `Loongphy/codex-auth`

[`codex-auth`](https://github.com/Loongphy/codex-auth) establishes the relevant user experience:

- named accounts;
- official client login as the acquisition path;
- list, switch, remove, alias, import, and export operations;
- an account registry separate from active authentication state;
- optional usage inspection;
- compatibility across multiple client surfaces.

Useful lessons:

1. Keep aliases and display metadata separate from raw auth documents.
2. Make import/export explicit rather than an incidental file copy.
3. Treat each client version as a compatibility boundary.
4. Provide a local-only mode when an online metadata endpoint is unnecessary.

Features deliberately not copied:

- live usage/rate-limit polling;
- account rotation or seamless fallback;
- generic plaintext credential export by default;
- direct provider API requests using subscription OAuth tokens.

These omissions keep `gemini-auth` focused on local manual switching.

## Gemini CLI

### Confirmed official behavior

Google's [configuration documentation](https://github.com/google-gemini/gemini-cli/blob/main/docs/reference/configuration.md) documents `GEMINI_CLI_HOME`. The value changes the root used for Gemini CLI user-level configuration and storage; the client creates `.gemini` below that root.

This provides a stable isolation primitive:

```text
~/.local/share/gemini-auth/homes/<profile>/
└── .gemini/
    ├── settings.json
    ├── oauth_creds.json   # depending on current client/storage mode
    └── ... official client state
```

The official [authentication documentation](https://github.com/google-gemini/gemini-cli/blob/main/docs/get-started/authentication.mdx) confirms that cached authentication is reused in headless mode. Historical source/issues show OAuth state at `~/.gemini/oauth_creds.json`, with fields such as access token, refresh token, scopes, token type, ID token, and expiry. That path must be treated as version-dependent rather than a permanent contract.

### Design consequence

For Gemini CLI, do not parse or copy OAuth tokens in the default path. Create a profile-specific `GEMINI_CLI_HOME`, run the official `gemini` binary against it, and let Gemini CLI own authentication, encryption/keyring choice, schema changes, and refresh behavior.

This also isolates settings, history, trusted folders, and cache. Users may optionally choose credential-only switching later, but whole-home isolation is the safer default.

### Policy warning

Gemini CLI's official [FAQ](https://github.com/google-gemini/gemini-cli/blob/main/docs/resources/faq.md) explicitly warns against third-party software harvesting or piggybacking on Gemini CLI OAuth to access backend services. `gemini-auth` must not make backend requests. It only selects the local state used by the official client.

## Antigravity CLI

### Confirmed local artifact

The installed Antigravity CLI in the research environment uses:

```text
~/.gemini/antigravity-cli/antigravity-oauth-token
```

The file is mode `0600` and has the structural shape:

```json
{
  "auth_method": "consumer",
  "token": {
    "access_token": "<secret>",
    "token_type": "Bearer",
    "refresh_token": "<secret>",
    "expiry": "<timestamp>"
  }
}
```

The executable retains functions named `NewCLITokenStorage`, `cliFileTokenStorage.SaveToken`, `LoadToken`, `RemoveToken`, and `shouldBypassKeyring`. Runtime logs state that file storage is used because an SSH session was detected. On a later run, the same stored refresh credential produced a new one-hour access token.

### Design consequence

- File-mode support can capture and restore the complete JSON document.
- Schema validation must allow unknown fields but require the known envelope and string types.
- The active file must remain `0600` on Unix.
- Antigravity must be stopped during activation because it can refresh and rewrite the file.
- The project must not perform OAuth refresh itself; Antigravity remains the sole network client.

### Desktop uncertainty

Antigravity tries a system keyring outside SSH/container/WSL fallback conditions. The keyring service/account identifiers and interaction with desktop/Electron state are not public. Direct keyring support is deferred. The diagnostic command may report "keyring-backed Antigravity detected; switching unsupported" but must not guess identifiers or scan all secrets.

## Existing Gemini/Antigravity third-party projects

- [`opencode-antigravity-auth`](https://github.com/NoeFabris/opencode-antigravity-auth) supports OAuth, multiple accounts, and quota rotation inside OpenCode. Its README itself warns about policy and ban risk. Its scope is intentionally broader and riskier than this project.
- [`Antigravity-Manager`](https://github.com/lbjlaq/Antigravity-Manager) exposes OpenAI/Anthropic/Gemini-compatible proxy endpoints and performs protocol translation. It is not an architectural template for this project.
- [`antigravity-claude-proxy`](https://github.com/badrisnarayanan/antigravity-claude-proxy) converts Antigravity access for other clients. Again, this is explicitly out of scope.

The main lesson is negative: account management tends to expand into pooling and proxying. The module boundaries and contribution policy must prevent that scope drift.

## Uncertainties requiring implementation-time probes

1. Whether current Gemini CLI always honors `GEMINI_CLI_HOME` for OS-keyring credential lookup or uses a global keyring service independent of home.
2. Whether Gemini CLI provides a noninteractive logout command suitable for tests.
3. Antigravity behavior when its token file is replaced while no process is running.
4. Whether Antigravity exposes a future supported profile/home override.
5. Windows ACL behavior for both active files and the project vault.
6. macOS Keychain prompts when official clients run from profile-specific homes.

These are acceptance tests, not reasons to embed private backend behavior.
