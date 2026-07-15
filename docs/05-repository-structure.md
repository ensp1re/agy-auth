# Repository and Folder Structure

## Proposed workspace

```text
agy-auth/
├── Cargo.toml                    # workspace members, shared lint/profile config
├── Cargo.lock                    # committed for executable reproducibility
├── README.md
├── LICENSE
├── SECURITY.md
├── CONTRIBUTING.md
├── deny.toml                     # cargo-deny policy
├── rust-toolchain.toml           # pinned stable toolchain
├── crates/
│   ├── agy-auth-cli/
│   │   └── src/
│   │       ├── main.rs
│   │       ├── args.rs
│   │       ├── output.rs
│   │       ├── prompts.rs
│   │       └── exit_codes.rs
│   ├── agy-auth-app/
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── add_profile.rs
│   │       ├── activate.rs
│   │       ├── exec_profile.rs
│   │       ├── list_profiles.rs
│   │       ├── remove_profile.rs
│   │       ├── import_export.rs
│   │       └── doctor.rs
│   ├── agy-auth-domain/
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── profile.rs
│   │       ├── provider.rs
│   │       ├── capability.rs
│   │       ├── transaction.rs
│   │       └── error.rs
│   ├── agy-auth-storage/
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── registry.rs
│   │       ├── secret_store.rs
│   │       ├── keyring.rs
│   │       ├── file_vault.rs
│   │       ├── atomic_file.rs
│   │       ├── lock.rs
│   │       ├── journal.rs
│   │       └── permissions.rs
│   ├── agy-auth-process/
│   │   └── src/
│   │       └── lib.rs
│   ├── provider-antigravity-cli/
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── discovery.rs
│   │       ├── file_mode.rs
│   │       ├── schema.rs
│   │       └── process_guard.rs
│   └── agy-auth-test-support/
│       └── src/
│           ├── lib.rs
│           ├── fake_keyring.rs
│           ├── fake_provider.rs
│           ├── fixtures.rs
│           └── fault_injection.rs
├── tests/
│   ├── cli/
│   ├── transactions/
│   ├── migrations/
│   └── fixtures/
│       └── synthetic-antigravity/
├── docs/
│   ├── adr/
│   └── ... planning and user documentation
├── scripts/
│   ├── check-no-secrets.sh
│   ├── generate-completions.sh
│   └── package-release.sh
├── completions/                  # generated bash/zsh/fish/PowerShell
├── man/
└── .github/
    ├── workflows/
    │   ├── ci.yml
    │   ├── release.yml
    │   └── security.yml
    ├── dependabot.yml
    └── ISSUE_TEMPLATE/
```

## Why multiple crates

The split enforces dependency direction at compile time and keeps secrets away from presentation code. It also lets adapters be reviewed independently. Avoid splitting more finely: each crate should represent a genuine policy or platform boundary.

## Runtime data layout

Use platform data/config conventions through the `directories` crate.

Unix-style illustration:

```text
~/.config/agy-auth/
├── config.json                   # non-secret user preferences
└── policy.json                   # optional admin/local restrictions

~/.local/share/agy-auth/
├── registry.json                 # non-secret profile metadata
├── active.json                   # selected profiles by provider
├── profiles/                     # only after a verified Antigravity isolation contract
│   └── <profile-uuid>/
│       ├── home/                 # effective HOME and XDG descendants owned by official agy
│       └── runtime/              # isolated XDG_RUNTIME_DIR
├── vault/                        # encrypted/file fallback only
│   └── <profile-uuid>.secret
├── rollback/                     # short-lived encrypted/0600 backups
└── transactions/
    └── <transaction-uuid>.json

~/.cache/agy-auth/
└── diagnostics/                  # opt-in and redacted
```

On macOS use Application Support/Preferences/Caches conventions; on Windows use LocalAppData. Do not hardcode Unix paths in application services.

## Registry schema sketch

```json
{
  "schemaVersion": 1,
  "profiles": [
    {
      "id": "uuid",
      "name": "personal",
      "provider": "antigravity-cli",
      "storage": { "kind": "isolated-home", "locator": "uuid" },
      "accountHint": "a***@example.com",
      "createdAt": "RFC3339",
      "updatedAt": "RFC3339",
      "clientVersionAtCapture": "optional",
      "schemaFingerprint": "optional hash",
      "status": "ready"
    }
  ]
}
```

No access token, refresh token, ID token, API key, OAuth code, full email, or command output belongs in this file.
Profile UUIDs, never user-provided names, derive managed directory paths.

## Module ownership rules

- CLI crate owns wording and display, not behavior.
- Application crate owns use-case order and authorization prompts.
- Domain crate owns invariants and state transitions.
- Storage crate is the only code allowed to persist opaque secrets.
- Provider crates know paths and schemas but cannot choose other profiles.
- Test-support crate may generate synthetic tokens only; real token-shaped prefixes are forbidden.
