//! Atomic non-secret selected-profile metadata.

use serde::{Deserialize, Serialize};
use std::fs::{self, File, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use thiserror::Error;
use uuid::Uuid;

/// Owner-only store for the currently selected non-secret profile name.
#[derive(Debug)]
pub struct ActiveProfileStore {
    path: PathBuf,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ActiveProfileDocument {
    schema_version: u64,
    profile: String,
}

impl ActiveProfileStore {
    /// Bind to `active.json` beneath an owner-only project data root.
    ///
    /// # Errors
    ///
    /// Returns an error when the path has no parent.
    pub fn new(path: impl Into<PathBuf>) -> Result<Self, ActiveProfileStoreError> {
        let path = path.into();
        if path.parent().is_none() {
            return Err(ActiveProfileStoreError::InvalidPath);
        }
        Ok(Self { path })
    }

    /// Load the selected non-secret profile name.
    ///
    /// # Errors
    ///
    /// Returns an error for unsafe metadata or malformed state.
    pub fn load(&self) -> Result<Option<String>, ActiveProfileStoreError> {
        let bytes = match fs::read(&self.path) {
            Ok(bytes) => bytes,
            Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
            Err(error) => return Err(error.into()),
        };
        validate_file(&self.path)?;
        if bytes.len() > 4096 {
            return Err(ActiveProfileStoreError::InvalidDocument);
        }
        let document: ActiveProfileDocument =
            serde_json::from_slice(&bytes).map_err(|_| ActiveProfileStoreError::InvalidDocument)?;
        if document.schema_version != 1 || document.profile.is_empty() {
            return Err(ActiveProfileStoreError::InvalidDocument);
        }
        Ok(Some(document.profile))
    }

    /// Atomically persist the selected non-secret profile name.
    ///
    /// # Errors
    ///
    /// Returns an error when safe atomic persistence cannot be guaranteed.
    pub fn save(&self, profile: &str) -> Result<(), ActiveProfileStoreError> {
        if profile.is_empty() {
            return Err(ActiveProfileStoreError::InvalidDocument);
        }
        let parent = self
            .path
            .parent()
            .ok_or(ActiveProfileStoreError::InvalidPath)?;
        validate_directory(parent)?;
        if self.path.exists() {
            validate_file(&self.path)?;
        }
        let bytes = serde_json::to_vec(&ActiveProfileDocument {
            schema_version: 1,
            profile: profile.to_owned(),
        })
        .map_err(|_| ActiveProfileStoreError::InvalidDocument)?;
        let temporary = parent.join(format!(".active-{}.tmp", Uuid::new_v4()));
        let mut file = create_new(&temporary)?;
        let result = (|| {
            file.write_all(&bytes)?;
            file.sync_all()?;
            drop(file);
            fs::rename(&temporary, &self.path)?;
            File::open(parent)?.sync_all()
        })();
        if result.is_err() {
            let _ = fs::remove_file(temporary);
        }
        result.map_err(Into::into)
    }
}

fn validate_directory(path: &Path) -> Result<(), ActiveProfileStoreError> {
    let metadata = fs::symlink_metadata(path)?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err(ActiveProfileStoreError::UnsafeState);
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if metadata.permissions().mode() & 0o077 != 0 {
            return Err(ActiveProfileStoreError::UnsafeState);
        }
    }
    Ok(())
}

fn validate_file(path: &Path) -> Result<(), ActiveProfileStoreError> {
    let metadata = fs::symlink_metadata(path)?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Err(ActiveProfileStoreError::UnsafeState);
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::{MetadataExt, PermissionsExt};
        if metadata.permissions().mode() & 0o177 != 0 || metadata.nlink() != 1 {
            return Err(ActiveProfileStoreError::UnsafeState);
        }
    }
    Ok(())
}

#[cfg(unix)]
fn create_new(path: &Path) -> io::Result<File> {
    use std::os::unix::fs::OpenOptionsExt;
    OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .custom_flags(libc::O_NOFOLLOW)
        .open(path)
}

#[cfg(not(unix))]
fn create_new(path: &Path) -> io::Result<File> {
    OpenOptions::new().write(true).create_new(true).open(path)
}

/// Failure while loading or replacing selected-profile metadata.
#[derive(Debug, Error)]
pub enum ActiveProfileStoreError {
    /// The configured metadata path has no usable parent.
    #[error("active profile path is invalid")]
    InvalidPath,
    /// The metadata path, file type, links, or permissions are unsafe.
    #[error("active profile state is unsafe")]
    UnsafeState,
    /// The selected-profile document is malformed or unsupported.
    #[error("active profile document is invalid")]
    InvalidDocument,
    /// A filesystem operation failed.
    #[error("active profile storage failed")]
    Io(#[from] io::Error),
}
