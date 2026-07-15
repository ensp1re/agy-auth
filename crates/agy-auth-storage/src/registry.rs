use agy_auth_domain::{
    DomainError, Profile, ProfileId, ProfileName, ProfileStatus, ProviderKind, Registry,
    StorageLocator,
};
use fs2::FileExt;
use serde::{Deserialize, Serialize};
use std::fs::{self, File, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use thiserror::Error;
use time::{OffsetDateTime, format_description::well_known::Rfc3339};
use uuid::Uuid;

const CURRENT_SCHEMA_VERSION: u64 = 1;

/// Versioned registry file with locked, atomic replacement.
#[derive(Debug)]
pub struct RegistryFile {
    path: PathBuf,
    lock_path: PathBuf,
}

impl RegistryFile {
    /// Configure a registry path and its adjacent lock file.
    ///
    /// # Errors
    ///
    /// Returns [`RegistryStoreError::MissingParent`] when the path has no file name or parent.
    pub fn new(path: impl Into<PathBuf>) -> Result<Self, RegistryStoreError> {
        let path = path.into();
        let parent = path
            .parent()
            .ok_or(RegistryStoreError::MissingParent)?
            .to_owned();
        let file_name = path.file_name().ok_or(RegistryStoreError::MissingParent)?;
        let mut lock_name = file_name.to_os_string();
        lock_name.push(".lock");
        Ok(Self {
            path,
            lock_path: parent.join(lock_name),
        })
    }

    /// Load the current registry, returning an empty registry when no file exists.
    ///
    /// # Errors
    ///
    /// Returns an error for unsafe file types, I/O failures, malformed data, or invalid metadata.
    pub fn load(&self) -> Result<Registry, RegistryStoreError> {
        load_path(&self.path)
    }

    /// Replace the registry while holding an exclusive adjacent lock.
    ///
    /// # Errors
    ///
    /// Returns an error for invalid metadata, lock contention, unsafe paths, or durability failures.
    pub fn replace(&self, registry: &Registry) -> Result<(), RegistryStoreError> {
        registry.validate()?;
        let parent = self
            .path
            .parent()
            .ok_or(RegistryStoreError::MissingParent)?;
        fs::create_dir_all(parent)?;
        reject_unsafe_existing(&self.lock_path)?;
        let lock = OpenOptions::new()
            .create(true)
            .truncate(false)
            .read(true)
            .write(true)
            .open(&self.lock_path)?;
        lock.try_lock_exclusive()
            .map_err(|_| RegistryStoreError::Locked)?;
        let result = self.write_locked(registry);
        let _ = FileExt::unlock(&lock);
        result
    }

    fn write_locked(&self, registry: &Registry) -> Result<(), RegistryStoreError> {
        reject_unsafe_existing(&self.path)?;
        let parent = self
            .path
            .parent()
            .ok_or(RegistryStoreError::MissingParent)?;
        let document = RegistryDocument::from_domain(registry)?;
        let mut bytes = serde_json::to_vec_pretty(&document)?;
        bytes.push(b'\n');
        let temporary = parent.join(format!(".registry-{}.tmp", Uuid::new_v4()));
        let result = (|| {
            let mut options = OpenOptions::new();
            options.create_new(true).write(true);
            #[cfg(unix)]
            {
                use std::os::unix::fs::OpenOptionsExt;
                options.mode(0o600);
            }
            let mut file = options.open(&temporary)?;
            file.write_all(&bytes)?;
            file.sync_all()?;
            fs::rename(&temporary, &self.path)?;
            sync_directory(parent)?;
            Ok(())
        })();
        if result.is_err() {
            let _ = fs::remove_file(&temporary);
        }
        result
    }
}

fn load_path(path: &Path) -> Result<Registry, RegistryStoreError> {
    match fs::symlink_metadata(path) {
        Ok(metadata) if metadata.file_type().is_symlink() || !metadata.is_file() => {
            return Err(RegistryStoreError::UnsafeFileType);
        }
        Ok(_) => {}
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(Registry::default()),
        Err(error) => return Err(error.into()),
    }
    let bytes = fs::read(path)?;
    if bytes.len() > 1_000_000 {
        return Err(RegistryStoreError::Oversized);
    }
    let value: serde_json::Value = serde_json::from_slice(&bytes)?;
    let version = value
        .get("schemaVersion")
        .and_then(serde_json::Value::as_u64)
        .ok_or(RegistryStoreError::MissingSchemaVersion)?;
    match version {
        CURRENT_SCHEMA_VERSION => {
            let document: RegistryDocument = serde_json::from_value(value)?;
            document.into_domain()
        }
        unsupported => Err(RegistryStoreError::UnsupportedSchema(unsupported)),
    }
}

fn reject_unsafe_existing(path: &Path) -> Result<(), RegistryStoreError> {
    match fs::symlink_metadata(path) {
        Ok(metadata) if metadata.file_type().is_symlink() || !metadata.is_file() => {
            Err(RegistryStoreError::UnsafeFileType)
        }
        Ok(_) => Ok(()),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error.into()),
    }
}

#[cfg(unix)]
fn sync_directory(path: &Path) -> Result<(), RegistryStoreError> {
    File::open(path)?.sync_all()?;
    Ok(())
}

#[cfg(not(unix))]
fn sync_directory(_path: &Path) -> Result<(), RegistryStoreError> {
    Ok(())
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct RegistryDocument {
    schema_version: u64,
    profiles: Vec<ProfileDocument>,
}

impl RegistryDocument {
    fn from_domain(registry: &Registry) -> Result<Self, RegistryStoreError> {
        Ok(Self {
            schema_version: CURRENT_SCHEMA_VERSION,
            profiles: registry
                .profiles()
                .iter()
                .map(ProfileDocument::from_domain)
                .collect::<Result<_, _>>()?,
        })
    }

    fn into_domain(self) -> Result<Registry, RegistryStoreError> {
        if self.schema_version != CURRENT_SCHEMA_VERSION {
            return Err(RegistryStoreError::UnsupportedSchema(self.schema_version));
        }
        Registry::new(
            self.profiles
                .into_iter()
                .map(ProfileDocument::into_domain)
                .collect::<Result<_, _>>()?,
        )
        .map_err(Into::into)
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ProfileDocument {
    id: String,
    name: String,
    provider: ProviderDocument,
    storage: StorageDocument,
    account_hint: Option<String>,
    created_at: String,
    updated_at: String,
    client_version_at_capture: Option<String>,
    schema_fingerprint: Option<String>,
    status: StatusDocument,
}

impl ProfileDocument {
    fn from_domain(profile: &Profile) -> Result<Self, RegistryStoreError> {
        let (kind, locator) = match &profile.storage {
            StorageLocator::IsolatedHome(value) => (StorageKindDocument::IsolatedHome, value),
            StorageLocator::SecretStoreEntry(value) => {
                (StorageKindDocument::SecretStoreEntry, value)
            }
        };
        Ok(Self {
            id: profile.id.to_string(),
            name: profile.name.as_str().to_owned(),
            provider: profile.provider.into(),
            storage: StorageDocument {
                kind,
                locator: locator.to_owned(),
            },
            account_hint: profile.account_hint.clone(),
            created_at: profile.created_at.format(&Rfc3339)?,
            updated_at: profile.updated_at.format(&Rfc3339)?,
            client_version_at_capture: profile.client_version_at_capture.clone(),
            schema_fingerprint: profile.schema_fingerprint.clone(),
            status: profile.status.into(),
        })
    }

    fn into_domain(self) -> Result<Profile, RegistryStoreError> {
        let storage = match self.storage.kind {
            StorageKindDocument::IsolatedHome => {
                StorageLocator::isolated_home(&self.storage.locator)?
            }
            StorageKindDocument::SecretStoreEntry => {
                StorageLocator::secret_store_entry(&self.storage.locator)?
            }
        };
        let profile = Profile {
            id: ProfileId::parse(&self.id)?,
            name: ProfileName::parse(&self.name)?,
            provider: self.provider.into(),
            storage,
            account_hint: self.account_hint,
            created_at: OffsetDateTime::parse(&self.created_at, &Rfc3339)?,
            updated_at: OffsetDateTime::parse(&self.updated_at, &Rfc3339)?,
            client_version_at_capture: self.client_version_at_capture,
            schema_fingerprint: self.schema_fingerprint,
            status: self.status.into(),
        };
        profile.validate()?;
        Ok(profile)
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
enum ProviderDocument {
    GeminiCli,
    AntigravityCli,
}

impl From<ProviderKind> for ProviderDocument {
    fn from(value: ProviderKind) -> Self {
        match value {
            ProviderKind::GeminiCli => Self::GeminiCli,
            ProviderKind::AntigravityCli => Self::AntigravityCli,
        }
    }
}

impl From<ProviderDocument> for ProviderKind {
    fn from(value: ProviderDocument) -> Self {
        match value {
            ProviderDocument::GeminiCli => Self::GeminiCli,
            ProviderDocument::AntigravityCli => Self::AntigravityCli,
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct StorageDocument {
    kind: StorageKindDocument,
    locator: String,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
enum StorageKindDocument {
    IsolatedHome,
    SecretStoreEntry,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
enum StatusDocument {
    Ready,
    Pending,
    Unavailable,
}

impl From<ProfileStatus> for StatusDocument {
    fn from(value: ProfileStatus) -> Self {
        match value {
            ProfileStatus::Ready => Self::Ready,
            ProfileStatus::Pending => Self::Pending,
            ProfileStatus::Unavailable => Self::Unavailable,
        }
    }
}

impl From<StatusDocument> for ProfileStatus {
    fn from(value: StatusDocument) -> Self {
        match value {
            StatusDocument::Ready => Self::Ready,
            StatusDocument::Pending => Self::Pending,
            StatusDocument::Unavailable => Self::Unavailable,
        }
    }
}

/// Failure to validate, read, lock, migrate, or atomically write registry metadata.
#[derive(Debug, Error)]
pub enum RegistryStoreError {
    /// Registry path has no usable parent directory.
    #[error("registry path has no parent directory")]
    MissingParent,
    /// Existing registry or lock path is not a regular file.
    #[error("registry path is not a safe regular file")]
    UnsafeFileType,
    /// Another writer holds the mutation lock.
    #[error("registry is locked by another mutation")]
    Locked,
    /// Registry exceeds the bounded input size.
    #[error("registry exceeds the maximum supported size")]
    Oversized,
    /// Registry omits its schema version.
    #[error("registry is missing schemaVersion")]
    MissingSchemaVersion,
    /// Registry schema cannot be migrated by this version.
    #[error("unsupported registry schema version {0}")]
    UnsupportedSchema(u64),
    /// Domain metadata violates an invariant.
    #[error("registry metadata violates a domain invariant")]
    Domain(#[from] DomainError),
    /// Filesystem operation failed without exposing file contents.
    #[error("registry filesystem operation failed: {0}")]
    Io(#[from] io::Error),
    /// Registry JSON is malformed or contains unsupported fields.
    #[error("registry JSON is invalid: {0}")]
    Json(#[from] serde_json::Error),
    /// Timestamp is not RFC 3339.
    #[error("registry timestamp is invalid")]
    TimestampParse(#[from] time::error::Parse),
    /// Timestamp could not be rendered as RFC 3339.
    #[error("registry timestamp cannot be formatted")]
    TimestampFormat(#[from] time::error::Format),
}

#[cfg(test)]
mod tests {
    use super::{RegistryFile, RegistryStoreError};
    use agy_auth_domain::{
        Profile, ProfileId, ProfileName, ProfileStatus, ProviderKind, Registry, StorageLocator,
    };
    use fs2::FileExt;
    use std::fs;
    use time::OffsetDateTime;

    fn profile(name: &str) -> Profile {
        let now = OffsetDateTime::from_unix_timestamp(1_700_000_000).expect("valid timestamp");
        Profile {
            id: ProfileId::new(),
            name: ProfileName::parse(name).expect("valid profile name"),
            provider: ProviderKind::GeminiCli,
            storage: StorageLocator::isolated_home("synthetic-profile-home")
                .expect("valid locator"),
            account_hint: Some("t***@example.invalid".to_owned()),
            created_at: now,
            updated_at: now,
            client_version_at_capture: Some("TEST_CLIENT_VERSION".to_owned()),
            schema_fingerprint: Some("TEST_SCHEMA_FINGERPRINT".to_owned()),
            status: ProfileStatus::Ready,
        }
    }

    fn temporary_directory() -> std::path::PathBuf {
        let path = std::env::temp_dir().join(format!("agy-auth-test-{}", uuid::Uuid::new_v4()));
        fs::create_dir(&path).expect("create test directory");
        path
    }

    #[test]
    fn registry_round_trip_contains_metadata_only() {
        let directory = temporary_directory();
        let path = directory.join("registry.json");
        let store = RegistryFile::new(&path).expect("valid path");
        let registry = Registry::new(vec![profile("Personal")]).expect("valid registry");
        store.replace(&registry).expect("write registry");
        assert_eq!(store.load().expect("load registry"), registry);
        let raw = fs::read_to_string(&path).expect("read test registry");
        assert!(!raw.contains("accessToken"));
        assert!(!raw.contains("refreshToken"));
        assert!(!raw.contains("taylor@example"));
        fs::remove_dir_all(directory).expect("remove test directory");
    }

    #[test]
    fn duplicate_normalized_names_fail_closed() {
        let result = Registry::new(vec![profile("Work"), profile("work")]);
        assert!(result.is_err());
    }

    #[test]
    fn unsupported_schema_fails_closed() {
        let directory = temporary_directory();
        let path = directory.join("registry.json");
        fs::write(&path, br#"{"schemaVersion":2,"profiles":[]}"#).expect("write fixture");
        let store = RegistryFile::new(&path).expect("valid path");
        assert!(matches!(
            store.load(),
            Err(RegistryStoreError::UnsupportedSchema(2))
        ));
        fs::remove_dir_all(directory).expect("remove test directory");
    }

    #[test]
    fn malformed_json_fails_closed() {
        let directory = temporary_directory();
        let path = directory.join("registry.json");
        fs::write(&path, b"not-json").expect("write fixture");
        let store = RegistryFile::new(&path).expect("valid path");
        assert!(matches!(store.load(), Err(RegistryStoreError::Json(_))));
        fs::remove_dir_all(directory).expect("remove test directory");
    }

    #[test]
    fn held_mutation_lock_fails_without_writing() {
        let directory = temporary_directory();
        let path = directory.join("registry.json");
        let lock_path = directory.join("registry.json.lock");
        let lock = fs::OpenOptions::new()
            .create(true)
            .truncate(false)
            .read(true)
            .write(true)
            .open(&lock_path)
            .expect("open lock");
        lock.try_lock_exclusive().expect("hold lock");
        let store = RegistryFile::new(&path).expect("valid path");
        let registry = Registry::new(vec![profile("Personal")]).expect("valid registry");
        assert!(matches!(
            store.replace(&registry),
            Err(RegistryStoreError::Locked)
        ));
        assert!(!path.exists());
        FileExt::unlock(&lock).expect("unlock");
        fs::remove_dir_all(directory).expect("remove test directory");
    }

    #[test]
    fn repeated_round_trips_preserve_invariants() {
        let directory = temporary_directory();
        let path = directory.join("registry.json");
        let store = RegistryFile::new(&path).expect("valid path");
        for index in 0..32 {
            let registry =
                Registry::new(vec![profile(&format!("profile-{index}"))]).expect("valid registry");
            store.replace(&registry).expect("replace registry");
            assert_eq!(store.load().expect("load registry"), registry);
        }
        fs::remove_dir_all(directory).expect("remove test directory");
    }

    #[cfg(unix)]
    #[test]
    fn registry_is_owner_read_write_only() {
        use std::os::unix::fs::PermissionsExt;
        let directory = temporary_directory();
        let path = directory.join("registry.json");
        let store = RegistryFile::new(&path).expect("valid path");
        store.replace(&Registry::default()).expect("write registry");
        let mode = fs::metadata(&path)
            .expect("registry metadata")
            .permissions()
            .mode()
            & 0o777;
        assert_eq!(mode, 0o600);
        fs::remove_dir_all(directory).expect("remove test directory");
    }

    #[cfg(unix)]
    #[test]
    fn symlink_registry_fails_closed() {
        use std::os::unix::fs::symlink;
        let directory = temporary_directory();
        let target = directory.join("target.json");
        let path = directory.join("registry.json");
        fs::write(&target, br#"{"schemaVersion":1,"profiles":[]}"#).expect("write target");
        symlink(&target, &path).expect("create symlink");
        let store = RegistryFile::new(&path).expect("valid path");
        assert!(matches!(
            store.load(),
            Err(RegistryStoreError::UnsafeFileType)
        ));
        fs::remove_dir_all(directory).expect("remove test directory");
    }

    #[cfg(unix)]
    #[test]
    fn dangling_symlink_registry_fails_closed() {
        use std::os::unix::fs::symlink;
        let directory = temporary_directory();
        let path = directory.join("registry.json");
        symlink(directory.join("missing.json"), &path).expect("create dangling symlink");
        let store = RegistryFile::new(&path).expect("valid path");
        assert!(matches!(
            store.load(),
            Err(RegistryStoreError::UnsafeFileType)
        ));
        fs::remove_dir_all(directory).expect("remove test directory");
    }
}
