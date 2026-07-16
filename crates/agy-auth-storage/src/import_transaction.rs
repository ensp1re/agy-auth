//! Non-secret interrupted-import journal and idempotent recovery.

use crate::{ManagedProfileHomes, RegistryCatalog};
use agy_auth_domain::{ProfileId, ProfileStatus};
use serde::{Deserialize, Serialize};
use std::fs::{self, File, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use thiserror::Error;
use uuid::Uuid;

const SCHEMA_VERSION: u64 = 1;

/// Durable import transition recorded without credential contents or user-facing names.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ImportTransactionStage {
    /// Marker exists before registry reservation.
    Started,
    /// Pending registry metadata is durable.
    Reserved,
    /// Managed credential materialization completed.
    Materialized,
    /// Ready registry metadata is durable; only marker cleanup remains.
    Ready,
}

/// Project-owned import journal rooted beneath one secure data directory.
#[derive(Clone, Debug)]
pub struct ImportTransactionJournal {
    data_root: PathBuf,
}

/// One active durable import transaction.
#[derive(Debug)]
pub struct ImportTransaction {
    path: PathBuf,
    document: ImportTransactionDocument,
}

/// Non-secret recovery result.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ImportRecoveryReport {
    /// Pending/absent imports removed.
    pub rolled_back: usize,
    /// Ready imports whose stale marker was removed.
    pub completed: usize,
}

impl ImportTransactionJournal {
    /// Bind the journal to an absolute data root with an existing parent.
    ///
    /// # Errors
    ///
    /// Returns an error when the root is relative or has no parent.
    pub fn new(data_root: impl Into<PathBuf>) -> Result<Self, ImportTransactionError> {
        let data_root = data_root.into();
        if !data_root.is_absolute() || data_root.parent().is_none() {
            return Err(ImportTransactionError::InvalidRoot);
        }
        Ok(Self { data_root })
    }

    /// Create the durable marker before any import mutation.
    ///
    /// # Errors
    ///
    /// Returns an error when the journal cannot be created and synced safely.
    pub fn begin(
        &self,
        profile_id: ProfileId,
    ) -> Result<ImportTransaction, ImportTransactionError> {
        let directory = self.ensure_directory()?;
        let document = ImportTransactionDocument {
            schema_version: SCHEMA_VERSION,
            transaction_id: Uuid::new_v4().to_string(),
            profile_id: profile_id.to_string(),
            stage: ImportTransactionStage::Started,
        };
        let path = directory.join(format!("{}.json", document.transaction_id));
        write_new_document(&path, &document)?;
        sync_directory(&directory)?;
        Ok(ImportTransaction { path, document })
    }

    /// Recover every well-formed interrupted import.
    ///
    /// # Errors
    ///
    /// Returns an error for malformed/unsafe markers or any cleanup that cannot complete safely.
    pub fn recover(
        &self,
        catalog: &RegistryCatalog,
        homes: &ManagedProfileHomes,
    ) -> Result<ImportRecoveryReport, ImportTransactionError> {
        let directory = self.ensure_directory()?;
        let mut report = ImportRecoveryReport::default();
        for entry in fs::read_dir(&directory)? {
            let entry = entry?;
            let path = entry.path();
            let metadata = fs::symlink_metadata(&path)?;
            if metadata.file_type().is_symlink() || !metadata.is_file() {
                return Err(ImportTransactionError::UnsafeMarker);
            }
            validate_mode(&metadata, 0o077)?;
            let bytes = fs::read(&path)?;
            if bytes.is_empty() || bytes.len() > 4096 {
                return Err(ImportTransactionError::InvalidMarker);
            }
            let document: ImportTransactionDocument = serde_json::from_slice(&bytes)
                .map_err(|_| ImportTransactionError::InvalidMarker)?;
            Uuid::parse_str(&document.transaction_id)
                .map_err(|_| ImportTransactionError::InvalidMarker)?;
            if document.schema_version != SCHEMA_VERSION
                || path.file_name().and_then(|value| value.to_str())
                    != Some(&format!("{}.json", document.transaction_id))
            {
                return Err(ImportTransactionError::InvalidMarker);
            }
            let profile_id = ProfileId::parse(&document.profile_id)
                .map_err(|_| ImportTransactionError::InvalidMarker)?;
            match (document.stage, catalog.profile_by_id(profile_id)?) {
                (_, Some(profile)) if profile.status == ProfileStatus::Ready => {
                    remove_marker(&path, &directory)?;
                    report.completed += 1;
                }
                (ImportTransactionStage::Ready, _) => {
                    return Err(ImportTransactionError::UnsafeProfileState);
                }
                (_, Some(profile)) if profile.status == ProfileStatus::Pending => {
                    homes.remove(profile_id)?;
                    catalog.remove_pending(profile_id)?;
                    remove_marker(&path, &directory)?;
                    report.rolled_back += 1;
                }
                (_, None) => {
                    homes.remove(profile_id)?;
                    remove_marker(&path, &directory)?;
                    report.rolled_back += 1;
                }
                (_, Some(_)) => return Err(ImportTransactionError::UnsafeProfileState),
            }
        }
        Ok(report)
    }

    fn ensure_directory(&self) -> Result<PathBuf, ImportTransactionError> {
        let homes = ManagedProfileHomes::new(&self.data_root)?;
        homes.initialize()?;
        let directory = self.data_root.join("transactions");
        match fs::symlink_metadata(&directory) {
            Ok(metadata) => {
                if metadata.file_type().is_symlink() || !metadata.is_dir() {
                    return Err(ImportTransactionError::UnsafeMarker);
                }
                validate_mode(&metadata, 0o077)?;
            }
            Err(error) if error.kind() == io::ErrorKind::NotFound => {
                create_owner_only_directory(&directory)?;
            }
            Err(error) => return Err(error.into()),
        }
        Ok(directory)
    }
}

impl ImportTransaction {
    /// Durably advance one import stage.
    ///
    /// # Errors
    ///
    /// Returns an error for an out-of-order stage or failed durable marker replacement.
    pub fn advance(&mut self, stage: ImportTransactionStage) -> Result<(), ImportTransactionError> {
        let valid = matches!(
            (self.document.stage, stage),
            (
                ImportTransactionStage::Started,
                ImportTransactionStage::Reserved
            ) | (
                ImportTransactionStage::Reserved,
                ImportTransactionStage::Materialized
            ) | (
                ImportTransactionStage::Materialized,
                ImportTransactionStage::Ready
            )
        );
        if !valid {
            return Err(ImportTransactionError::InvalidTransition);
        }
        self.document.stage = stage;
        replace_document(&self.path, &self.document)
    }

    /// Remove the marker after the ready registry commit.
    ///
    /// # Errors
    ///
    /// Returns an error unless the transaction reached ready or marker cleanup cannot be synced.
    pub fn complete(self) -> Result<(), ImportTransactionError> {
        if self.document.stage != ImportTransactionStage::Ready {
            return Err(ImportTransactionError::InvalidTransition);
        }
        let directory = self
            .path
            .parent()
            .ok_or(ImportTransactionError::InvalidRoot)?;
        remove_marker(&self.path, directory)
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ImportTransactionDocument {
    schema_version: u64,
    transaction_id: String,
    profile_id: String,
    stage: ImportTransactionStage,
}

fn write_new_document(
    path: &Path,
    document: &ImportTransactionDocument,
) -> Result<(), ImportTransactionError> {
    let mut options = OpenOptions::new();
    options.create_new(true).write(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let mut file = options.open(path)?;
    let bytes = serde_json::to_vec(document)?;
    file.write_all(&bytes)?;
    file.sync_all()?;
    Ok(())
}

fn replace_document(
    path: &Path,
    document: &ImportTransactionDocument,
) -> Result<(), ImportTransactionError> {
    let directory = path.parent().ok_or(ImportTransactionError::InvalidRoot)?;
    let temporary = directory.join(format!(".{}.tmp", Uuid::new_v4()));
    write_new_document(&temporary, document)?;
    fs::rename(&temporary, path)?;
    sync_directory(directory)
}

fn remove_marker(path: &Path, directory: &Path) -> Result<(), ImportTransactionError> {
    fs::remove_file(path)?;
    sync_directory(directory)
}

#[cfg(unix)]
fn create_owner_only_directory(path: &Path) -> io::Result<()> {
    use std::os::unix::fs::DirBuilderExt;
    fs::DirBuilder::new().mode(0o700).create(path)
}

#[cfg(not(unix))]
fn create_owner_only_directory(path: &Path) -> io::Result<()> {
    fs::create_dir(path)
}

#[cfg(unix)]
fn validate_mode(metadata: &fs::Metadata, forbidden: u32) -> Result<(), ImportTransactionError> {
    use std::os::unix::fs::{MetadataExt, PermissionsExt};
    if metadata.permissions().mode() & forbidden != 0 {
        return Err(ImportTransactionError::UnsafeMarker);
    }
    if metadata.is_file() && metadata.nlink() != 1 {
        return Err(ImportTransactionError::UnsafeMarker);
    }
    #[cfg(target_os = "linux")]
    {
        if metadata.uid() != fs::metadata("/proc/self")?.uid() {
            return Err(ImportTransactionError::UnsafeMarker);
        }
    }
    Ok(())
}

#[cfg(not(unix))]
fn validate_mode(_metadata: &fs::Metadata, _forbidden: u32) -> Result<(), ImportTransactionError> {
    Ok(())
}

#[cfg(unix)]
fn sync_directory(path: &Path) -> Result<(), ImportTransactionError> {
    File::open(path)?.sync_all()?;
    Ok(())
}

#[cfg(not(unix))]
fn sync_directory(_path: &Path) -> Result<(), ImportTransactionError> {
    Ok(())
}

/// Import journal or recovery failure without secret-bearing context.
#[derive(Debug, Error)]
pub enum ImportTransactionError {
    /// Data root is not an absolute project-owned location.
    #[error("import transaction root is invalid")]
    InvalidRoot,
    /// Marker or journal directory has an unsafe type, owner, or mode.
    #[error("import transaction marker is unsafe")]
    UnsafeMarker,
    /// Marker schema, size, identifier, or JSON is invalid.
    #[error("import transaction marker is invalid")]
    InvalidMarker,
    /// Journal stages were advanced out of order.
    #[error("import transaction transition is invalid")]
    InvalidTransition,
    /// Recovery encountered metadata that cannot be rolled back safely.
    #[error("profile state cannot be recovered automatically")]
    UnsafeProfileState,
    /// Filesystem operation failed.
    #[error("import transaction filesystem operation failed")]
    Io(#[from] io::Error),
    /// Marker serialization failed.
    #[error("import transaction serialization failed")]
    Json(#[from] serde_json::Error),
    /// Managed home cleanup failed closed.
    #[error("managed profile recovery failed")]
    Homes(#[from] crate::ManagedProfileHomeError),
    /// Registry cleanup failed closed.
    #[error("registry recovery failed")]
    Registry(#[from] crate::RegistryStoreError),
}

#[cfg(all(test, unix))]
mod tests {
    use super::{ImportTransactionJournal, ImportTransactionStage};
    use crate::{ManagedProfileHomes, RegistryCatalog};
    use agy_auth_app::{ProfileCatalogPort, new_pending_profile};
    use std::fs;
    use std::os::unix::fs::PermissionsExt;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn fixture() -> std::path::PathBuf {
        let root = std::env::temp_dir().join(format!(
            "agy-auth-import-recovery-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("clock")
                .as_nanos()
        ));
        fs::create_dir(&root).expect("fixture");
        fs::set_permissions(&root, fs::Permissions::from_mode(0o700)).expect("secure fixture");
        root
    }

    #[test]
    fn rolls_back_pending_import_and_is_idempotent() {
        let root = fixture();
        let data = root.join("data");
        let homes = ManagedProfileHomes::new(&data).expect("homes");
        let catalog = RegistryCatalog::new(data.join("registry.json")).expect("catalog");
        let journal = ImportTransactionJournal::new(&data).expect("journal");
        let profile = new_pending_profile("synthetic-work").expect("profile");
        let mut transaction = journal.begin(profile.id).expect("begin");
        catalog.reserve(&profile).expect("reserve");
        transaction
            .advance(ImportTransactionStage::Reserved)
            .expect("reserved");
        let environment = homes.prepare(profile.id).expect("prepare");
        fs::write(environment.home.join("synthetic-state"), b"synthetic").expect("state");
        transaction
            .advance(ImportTransactionStage::Materialized)
            .expect("materialized");
        drop(transaction);

        let report = journal.recover(&catalog, &homes).expect("recover");
        assert_eq!(report.rolled_back, 1);
        assert_eq!(report.completed, 0);
        assert!(!environment.home.exists());
        assert!(
            catalog
                .profile_by_id(profile.id)
                .expect("registry")
                .is_none()
        );
        assert_eq!(
            journal.recover(&catalog, &homes).expect("repeat"),
            super::ImportRecoveryReport::default()
        );
        fs::remove_dir_all(root).expect("cleanup");
    }

    #[test]
    fn preserves_ready_profile_when_marker_cleanup_was_interrupted() {
        let root = fixture();
        let data = root.join("data");
        let homes = ManagedProfileHomes::new(&data).expect("homes");
        let catalog = RegistryCatalog::new(data.join("registry.json")).expect("catalog");
        let journal = ImportTransactionJournal::new(&data).expect("journal");
        let profile = new_pending_profile("synthetic-personal").expect("profile");
        let mut transaction = journal.begin(profile.id).expect("begin");
        catalog.reserve(&profile).expect("reserve");
        transaction
            .advance(ImportTransactionStage::Reserved)
            .expect("reserved");
        let environment = homes.prepare(profile.id).expect("prepare");
        transaction
            .advance(ImportTransactionStage::Materialized)
            .expect("materialized");
        catalog
            .mark_ready(profile.id, "TEST_CLIENT_1")
            .expect("ready");
        transaction
            .advance(ImportTransactionStage::Ready)
            .expect("ready marker");
        drop(transaction);

        let report = journal.recover(&catalog, &homes).expect("recover");
        assert_eq!(report.rolled_back, 0);
        assert_eq!(report.completed, 1);
        assert!(environment.home.exists());
        assert!(
            catalog
                .profile_by_id(profile.id)
                .expect("registry")
                .is_some_and(|value| value.status == agy_auth_domain::ProfileStatus::Ready)
        );
        fs::remove_dir_all(root).expect("cleanup");
    }
}
