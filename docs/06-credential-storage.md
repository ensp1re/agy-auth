# Credential and Storage Design

## Threat model

Assets:

- OAuth refresh tokens and access tokens.
- ID tokens and account identifiers.
- Employer/client separation.
- Integrity of official client configuration.
- Availability of login state.

Threats:

- Another local user reading files.
- Malware or a process already running as the same user.
- Accidental logging, shell history, crash dumps, or Git commits.
- Symlink/hardlink attacks against active credential paths.
- Concurrent client refresh overwriting a switch.
- Power loss between backup and rename.
- Malicious imported profile JSON.
- A future provider schema being misinterpreted.

Out of scope: protecting credentials from a fully compromised same-user session or root/administrator. The tool should reduce exposure, not claim impossible guarantees.

## Storage hierarchy

1. **Isolated official-client home**: preferred. The official client controls its own secret storage.
2. **OS keyring**: preferred for opaque captured credentials when a provider supports safe import/export and the platform keyring is available.
3. **Encrypted file vault**: opt-in fallback for headless systems without Secret Service.
4. **Plaintext mode-0600 vault**: not offered by default. A later expert-only mode would require loud warnings and explicit configuration.

Antigravity's active file is necessarily plaintext because that is the official client's verified SSH storage format. The stored inactive copy should still go to the keyring/encrypted vault.

## Secret-store interface

```rust
trait SecretStore {
    fn status(&self) -> Result<SecretStoreStatus>;
    fn put(&self, profile: ProfileId, secret: SecretBytes) -> Result<SecretRef>;
    fn get(&self, reference: &SecretRef) -> Result<SecretBytes>;
    fn delete(&self, reference: &SecretRef) -> Result<DeletionOutcome>;
}
```

The registry stores only `SecretRef`, never data. `SecretBytes` cannot implement `Display`, `Debug`, `Clone`, serialization, or equality that prints bytes. Hash comparisons operate through an audited helper.

## Keyring naming

- Service: `dev.agy-auth.credentials`
- Account: profile UUID, not email or display name.
- Payload: versioned envelope with provider kind and opaque bytes.

Do not enumerate the user's entire keyring. Maintain known references in the registry and access only those exact service/account entries.

## Encrypted fallback

Defer until after MVP. Requirements:

- Authenticated encryption; corruption and wrong passwords must be distinguishable only as generic unlock failure to callers.
- Password-derived key using Argon2id with per-vault random salt and reviewed parameters.
- Random nonce per secret.
- Header with format version, KDF parameters, cipher suite, and salt; all authenticated as associated data.
- Master password read from TTY, never a command-line argument or environment variable by default.
- Optional master key stored in OS keyring defeats the headless use case and should not be the default.
- Auto-lock after each command; no resident daemon.

Prefer a mature format/library such as `age` rather than designing a bespoke container.

## Atomic write protocol

For any JSON metadata or active credential file:

1. Reject symlink target using `symlink_metadata` and platform equivalents.
2. Open parent directory safely.
3. Create a random, exclusive temporary file in the same directory.
4. Write all bytes.
5. Apply final permissions before exposure.
6. Flush and `fsync` the file.
7. Rename atomically over destination.
8. `fsync` parent directory where supported.
9. Reopen without following links when the platform supports it; verify owner, permissions, size, and hash.

Never truncate the destination in place.

## Import/export

Import is optional for MVP; export is deferred.

Import requirements:

- default input is an isolated home produced by the official client;
- refuse world/group-readable credential files on Unix;
- limit file size before reading;
- reject symlinks, devices, sockets, and directories where a regular file is expected;
- parse only enough to identify supported schema, preserving original bytes;
- never refresh or validate the token over the network.

If export is added, output must be encrypted by default. Plaintext export requires `--unsafe-plaintext`, TTY confirmation, restrictive creation mode, and no overwrite without confirmation.

## Deletion semantics

Filesystem and keyring deletion cannot guarantee forensic erasure because of journaling, snapshots, SSD wear leveling, backups, and keyring implementation details. Documentation must say "remove reference and request deletion," not "securely erase."

## Backups and rollback retention

- Keep one rollback copy only during an incomplete transaction.
- Delete it after verified success unless `keepRollbackMinutes` is explicitly configured.
- Never include secrets in general registry backups.
- `doctor` identifies orphaned rollback entries by transaction ID and resolves them interactively.
