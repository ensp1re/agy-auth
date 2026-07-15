# System Architecture

## Architectural style

Use a ports-and-adapters design with a small application core. Provider-specific knowledge belongs behind `ProviderAdapter`; persistence belongs behind `RegistryStore`, `SecretStore`, and `ActivationStore`. Commands orchestrate these ports but never parse provider tokens directly.

```text
┌─────────────────────────────────────────────────────────────┐
│ CLI presentation                                            │
│ clap args, prompts, tables, JSON output, exit codes          │
└────────────────────────────┬────────────────────────────────┘
                             │ Command DTOs
┌────────────────────────────▼────────────────────────────────┐
│ Application services                                       │
│ AddProfile, ActivateProfile, ExecProfile, Remove, Doctor    │
└──────────────┬─────────────────┬─────────────────┬──────────┘
               │                 │                 │
┌──────────────▼───────┐ ┌──────▼────────┐ ┌──────▼──────────┐
│ Provider adapters    │ │ Profile vault │ │ Transaction mgr │
│ Gemini / Antigravity │ │ metadata+secret│ │ lock/journal/RB │
└──────────────┬───────┘ └──────┬────────┘ └──────┬──────────┘
               │                 │                 │
┌──────────────▼─────────────────▼─────────────────▼──────────┐
│ Infrastructure                                             │
│ filesystem, OS keyring, process runner, clock, permissions  │
└─────────────────────────────────────────────────────────────┘
```

## Domain model

```rust
ProfileId(Uuid)
ProfileName(String)          // validated, unique case-folded name
ProviderKind                 // GeminiCli | AntigravityCli
Profile {
  id, name, provider, created_at, updated_at,
  storage_locator, account_hint, schema_fingerprint,
  client_version_at_capture, status
}
AccountHint { masked_email } // optional; never required for identity
ActiveSelection { provider, profile_id, activated_at }
CapabilitySet {
  isolated_home, capture_active_file, activate_file,
  os_keyring, exec, logout, process_detection
}
```

Do not use email as the primary key. Accounts can change email aliases and the same account may exist under multiple providers. `ProfileId` is immutable; `ProfileName` is user-facing.

## Provider adapter contract

```rust
trait ProviderAdapter {
    fn kind(&self) -> ProviderKind;
    fn discover(&self, ctx: &Context) -> Result<Discovery>;
    fn capabilities(&self, discovery: &Discovery) -> CapabilitySet;
    fn begin_login(&self, profile: &ProfileDraft, runner: &dyn ProcessRunner)
        -> Result<CapturedState>;
    fn validate_state(&self, state: &OpaqueSecret) -> Result<StateMetadata>;
    fn capture_active(&self, discovery: &Discovery) -> Result<CapturedState>;
    fn plan_activation(&self, profile: &Profile, discovery: &Discovery)
        -> Result<ActivationPlan>;
    fn verify_activation(&self, plan: &ActivationPlan) -> Result<Verification>;
}
```

Adapters return declarative `ActivationPlan` operations. They do not directly mutate the active filesystem. The transaction manager executes the plan, ensuring identical locking, backup, permissions, journaling, and rollback semantics across providers.

## Gemini CLI adapter

Preferred mode is not activation at all:

```text
gemini-auth exec personal -- gemini
  -> resolve profile
  -> set GEMINI_CLI_HOME=<profile home root>
  -> sanitize inherited auth-conflicting environment variables
  -> exec official binary
```

`add` creates the isolated home and launches Gemini CLI. Success is detected by presence of expected state and, if possible, an official-client command that reports authentication. The tool must not read access/refresh tokens merely to prove success.

Optional `use` can maintain a shell integration or active launcher configuration. It should not merge profile homes into `~/.gemini` in MVP.

## Antigravity CLI adapter

File-mode activation plan:

```text
1. Discover fixed active file and verify regular file/no symlink.
2. Detect running `agy`/Antigravity processes; stop unless absent.
3. Validate stored opaque JSON envelope without logging values.
4. Acquire global provider lock.
5. Hash current active bytes and save rollback copy in vault.
6. Write candidate to same-directory temporary file.
7. Set owner-only permissions/ACL.
8. fsync candidate; atomic rename over active path; fsync directory.
9. Re-read and compare bytes/hash/permissions.
10. Update active selection and commit journal.
11. Release lock.
```

If any post-write step fails, restore the rollback copy using the same atomic procedure. A crash leaves a journal that `gemini-auth doctor --repair` can reconcile.

## Activation transaction state machine

```text
Planned
  -> Locked
  -> BackupCreated
  -> CandidateWritten
  -> Installed
  -> Verified
  -> RegistryCommitted
  -> Complete

Any pre-Installed error -> discard candidate -> Aborted
Any post-Installed error -> restore backup -> RolledBack
Crash/unknown            -> NeedsRecovery
```

State transitions are monotonic and persisted before destructive steps. Re-running recovery is idempotent.

## Concurrency model

- One global registry lock prevents concurrent profile mutation.
- One provider activation lock prevents competing active-state changes.
- Read commands may proceed without the activation lock after taking a consistent registry snapshot.
- `exec` against isolated homes needs no global activation lock.
- Never hold a lock while waiting for browser login; reserve the profile name, release, authenticate, then reacquire to commit.

## Dependency direction

```text
cli -> application -> domain
                  -> ports
infrastructure implements ports
providers implement provider port using infrastructure abstractions
```

The domain crate must not depend on `clap`, OS keyring libraries, filesystem APIs, or provider JSON structures.

## Error taxonomy

- Usage/configuration error
- Profile not found/conflict
- Provider not installed
- Unsupported provider mode
- Client currently running
- Secret store unavailable/locked
- Credential schema unsupported/corrupt
- Permission/ownership unsafe
- Transaction conflict/recovery required
- Official login failed/cancelled
- Internal invariant failure

Every error has a stable machine code and a remediation message. Errors must not wrap raw secret payloads.
