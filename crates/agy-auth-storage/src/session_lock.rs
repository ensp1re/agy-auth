//! Exclusive per-profile official-client session lease.

#![cfg_attr(not(unix), allow(clippy::unnecessary_wraps))]

use agy_auth_app::{CredentialSessionLease, CredentialSessionLockPort, CredentialWorkflowError};
use fs2::FileExt;
use std::fs::{self, File, OpenOptions};
use std::io;
use std::path::PathBuf;
use thiserror::Error;

const LOCK_FILENAME: &str = ".agy-auth-session.lock";

/// Exclusive session lock anchored in one owner-only profile runtime directory.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProfileSessionLock {
    path: PathBuf,
}

impl ProfileSessionLock {
    /// Bind a lock to one absolute owner-only runtime directory.
    ///
    /// # Errors
    ///
    /// Returns an error when the directory is relative, linked, unsafe, or unavailable.
    pub fn new(runtime_directory: impl Into<PathBuf>) -> Result<Self, ProfileSessionLockError> {
        let runtime_directory = runtime_directory.into();
        if !runtime_directory.is_absolute() {
            return Err(ProfileSessionLockError::InvalidRuntimeDirectory);
        }
        validate_runtime_directory(&runtime_directory)?;
        Ok(Self {
            path: runtime_directory.join(LOCK_FILENAME),
        })
    }

    fn acquire(&self) -> Result<ProfileSessionLease, ProfileSessionLockError> {
        let parent = self
            .path
            .parent()
            .ok_or(ProfileSessionLockError::InvalidRuntimeDirectory)?;
        validate_runtime_directory(parent)?;
        validate_existing_lock_path(&self.path)?;
        let file = open_lock_file(&self.path)?;
        validate_lock_metadata(&file.metadata()?)?;
        file.try_lock_exclusive().map_err(|error| {
            if error.kind() == io::ErrorKind::WouldBlock {
                ProfileSessionLockError::Locked
            } else {
                ProfileSessionLockError::Io(error)
            }
        })?;
        validate_lock_metadata(&file.metadata()?)?;
        Ok(ProfileSessionLease { file })
    }
}

impl CredentialSessionLockPort for ProfileSessionLock {
    type Lease<'a>
        = ProfileSessionLease
    where
        Self: 'a;

    fn try_acquire(&self) -> Result<Self::Lease<'_>, CredentialWorkflowError> {
        self.acquire().map_err(|error| match error {
            ProfileSessionLockError::Locked => CredentialWorkflowError::SessionBusy,
            ProfileSessionLockError::InvalidRuntimeDirectory
            | ProfileSessionLockError::UnsafeFileType
            | ProfileSessionLockError::UnsafePermissions
            | ProfileSessionLockError::UnsafeOwner
            | ProfileSessionLockError::UnsafeLinkCount
            | ProfileSessionLockError::Io(_) => CredentialWorkflowError::CredentialStorageFailed,
        })
    }
}

/// Held exclusive profile session lease.
#[derive(Debug)]
pub struct ProfileSessionLease {
    file: File,
}

impl CredentialSessionLease for ProfileSessionLease {}

impl Drop for ProfileSessionLease {
    fn drop(&mut self) {
        let _ = FileExt::unlock(&self.file);
    }
}

fn validate_existing_lock_path(path: &std::path::Path) -> Result<(), ProfileSessionLockError> {
    match fs::symlink_metadata(path) {
        Ok(metadata) => validate_lock_metadata(&metadata),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error.into()),
    }
}

fn validate_runtime_directory(path: &std::path::Path) -> Result<(), ProfileSessionLockError> {
    let metadata = fs::symlink_metadata(path)?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err(ProfileSessionLockError::UnsafeFileType);
    }
    validate_owner_and_mode(&metadata, 0o077)
}

fn validate_lock_metadata(metadata: &fs::Metadata) -> Result<(), ProfileSessionLockError> {
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Err(ProfileSessionLockError::UnsafeFileType);
    }
    validate_owner_and_mode(metadata, 0o177)?;
    validate_single_link(metadata)
}

#[cfg(unix)]
fn validate_owner_and_mode(
    metadata: &fs::Metadata,
    forbidden_mode: u32,
) -> Result<(), ProfileSessionLockError> {
    #[cfg(target_os = "linux")]
    use std::os::unix::fs::MetadataExt;
    use std::os::unix::fs::PermissionsExt;

    if metadata.permissions().mode() & forbidden_mode != 0 {
        return Err(ProfileSessionLockError::UnsafePermissions);
    }
    #[cfg(target_os = "linux")]
    {
        let process = fs::metadata("/proc/self")?;
        if metadata.uid() != process.uid() {
            return Err(ProfileSessionLockError::UnsafeOwner);
        }
    }
    Ok(())
}

#[cfg(not(unix))]
fn validate_owner_and_mode(
    _metadata: &fs::Metadata,
    _forbidden_mode: u32,
) -> Result<(), ProfileSessionLockError> {
    Ok(())
}

#[cfg(unix)]
fn validate_single_link(metadata: &fs::Metadata) -> Result<(), ProfileSessionLockError> {
    use std::os::unix::fs::MetadataExt;

    if metadata.nlink() != 1 {
        return Err(ProfileSessionLockError::UnsafeLinkCount);
    }
    Ok(())
}

#[cfg(not(unix))]
fn validate_single_link(_metadata: &fs::Metadata) -> Result<(), ProfileSessionLockError> {
    Ok(())
}

#[cfg(unix)]
fn open_lock_file(path: &std::path::Path) -> io::Result<File> {
    use std::os::unix::fs::OpenOptionsExt;

    OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .mode(0o600)
        .custom_flags(libc::O_NOFOLLOW)
        .open(path)
}

#[cfg(not(unix))]
fn open_lock_file(path: &std::path::Path) -> io::Result<File> {
    OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(path)
}

/// Profile session lock failure without secret-bearing context.
#[derive(Debug, Error)]
pub enum ProfileSessionLockError {
    /// The runtime directory is not a safe absolute directory.
    #[error("invalid profile runtime directory")]
    InvalidRuntimeDirectory,
    /// The runtime or lock path has an unsafe type.
    #[error("profile session lock path has an unsafe type")]
    UnsafeFileType,
    /// The runtime or lock path is accessible to group or other users.
    #[error("profile session lock permissions are unsafe")]
    UnsafePermissions,
    /// The runtime or lock path is not owned by the current user.
    #[error("profile session lock owner is unsafe")]
    UnsafeOwner,
    /// The lock file has multiple hardlinks.
    #[error("profile session lock link count is unsafe")]
    UnsafeLinkCount,
    /// Another session already holds the lock.
    #[error("profile session is already locked")]
    Locked,
    /// Filesystem operation failed.
    #[error("profile session lock operation failed")]
    Io(#[from] io::Error),
}

#[cfg(all(test, unix))]
mod tests {
    use super::{ProfileSessionLock, ProfileSessionLockError};
    use agy_auth_app::{CredentialSessionLockPort, CredentialWorkflowError};
    use std::fs;
    use std::os::unix::fs::{PermissionsExt, symlink};
    use std::path::{Path, PathBuf};
    use std::time::{SystemTime, UNIX_EPOCH};

    fn fixture() -> PathBuf {
        let path = std::env::temp_dir().join(format!(
            "agy-auth-session-lock-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("clock after epoch")
                .as_nanos()
        ));
        fs::create_dir(&path).expect("create fixture");
        fs::set_permissions(&path, fs::Permissions::from_mode(0o700)).expect("secure fixture");
        path
    }

    fn remove(path: &Path) {
        fs::remove_dir_all(path).expect("remove fixture");
    }

    #[test]
    fn excludes_a_second_session_and_releases_on_drop() {
        let runtime = fixture();
        let lock = ProfileSessionLock::new(&runtime).expect("lock");
        let lease = lock.try_acquire().expect("first lease");
        assert!(matches!(
            lock.try_acquire(),
            Err(CredentialWorkflowError::SessionBusy)
        ));
        drop(lease);
        lock.try_acquire().expect("lease after drop");
        remove(&runtime);
    }

    #[test]
    fn rejects_unsafe_runtime_and_lock_paths() {
        let runtime = fixture();
        fs::set_permissions(&runtime, fs::Permissions::from_mode(0o750)).expect("unsafe runtime");
        assert!(matches!(
            ProfileSessionLock::new(&runtime),
            Err(ProfileSessionLockError::UnsafePermissions)
        ));
        fs::set_permissions(&runtime, fs::Permissions::from_mode(0o700)).expect("restore runtime");

        let target = runtime.join("target");
        fs::write(&target, b"synthetic").expect("target");
        fs::set_permissions(&target, fs::Permissions::from_mode(0o600)).expect("secure target");
        symlink(&target, runtime.join(super::LOCK_FILENAME)).expect("lock symlink");
        let lock = ProfileSessionLock::new(&runtime).expect("lock");
        assert!(matches!(
            lock.try_acquire(),
            Err(CredentialWorkflowError::CredentialStorageFailed)
        ));
        remove(&runtime);
    }

    #[test]
    fn rejects_hardlinked_lock_file() {
        let runtime = fixture();
        let lock = ProfileSessionLock::new(&runtime).expect("lock");
        drop(lock.try_acquire().expect("create lock file"));
        fs::hard_link(
            runtime.join(super::LOCK_FILENAME),
            runtime.join("linked-lock"),
        )
        .expect("hardlink lock");
        assert!(matches!(
            lock.try_acquire(),
            Err(CredentialWorkflowError::CredentialStorageFailed)
        ));
        remove(&runtime);
    }
}
