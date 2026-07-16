//! Project-owned managed profile directory lifecycle.

use agy_auth_app::{ManagedProfileEnvironment, ProfileHomePort, ProfileWorkflowError};
use agy_auth_domain::ProfileId;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use thiserror::Error;

/// Prepares and validates owner-only profile homes beneath one project data root.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ManagedProfileHomes {
    data_root: PathBuf,
}

impl ManagedProfileHomes {
    /// Configure an absolute project data root.
    ///
    /// The root's parent must already exist; preparation creates only project-owned descendants.
    ///
    /// # Errors
    ///
    /// Returns an error when the root is relative or has no parent.
    pub fn new(data_root: impl Into<PathBuf>) -> Result<Self, ManagedProfileHomeError> {
        let data_root = data_root.into();
        if !data_root.is_absolute() || data_root.parent().is_none() {
            return Err(ManagedProfileHomeError::InvalidRoot);
        }
        Ok(Self { data_root })
    }

    /// Create or validate the owner-only data root and profiles container.
    ///
    /// # Errors
    ///
    /// Returns an error when either directory cannot be established safely.
    pub fn initialize(&self) -> Result<(), ManagedProfileHomeError> {
        let parent = self
            .data_root
            .parent()
            .ok_or(ManagedProfileHomeError::InvalidRoot)?;
        validate_existing_directory(parent, false)?;
        ensure_secure_directory(&self.data_root)?;
        ensure_secure_directory(&self.data_root.join("profiles"))
    }

    /// Create or validate the profile's home and runtime directories.
    ///
    /// # Errors
    ///
    /// Returns an error for missing parents, links/non-directories, unsafe permissions or ownership,
    /// or filesystem failures.
    pub fn prepare(
        &self,
        profile_id: ProfileId,
    ) -> Result<ManagedProfileEnvironment, ManagedProfileHomeError> {
        self.initialize()?;
        let profiles = self.data_root.join("profiles");
        let profile_root = profiles.join(profile_id.to_string());
        ensure_secure_directory(&profile_root)?;
        let home = profile_root.join("home");
        let runtime_directory = profile_root.join("runtime");
        ensure_secure_directory(&home)?;
        ensure_secure_directory(&runtime_directory)?;
        Ok(ManagedProfileEnvironment {
            home,
            runtime_directory,
        })
    }

    /// Remove one project-owned managed profile directory during interrupted-import recovery.
    ///
    /// # Errors
    ///
    /// Returns an error when the managed root or target directory is unsafe or cannot be removed.
    pub fn remove(&self, profile_id: ProfileId) -> Result<(), ManagedProfileHomeError> {
        self.initialize()?;
        let profile_root = self.data_root.join("profiles").join(profile_id.to_string());
        match fs::symlink_metadata(&profile_root) {
            Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
            Err(error) => Err(error.into()),
            Ok(metadata) => {
                if metadata.file_type().is_symlink() || !metadata.is_dir() {
                    return Err(ManagedProfileHomeError::UnsafeType);
                }
                validate_platform_security(&metadata)?;
                fs::remove_dir_all(profile_root)?;
                Ok(())
            }
        }
    }
}

impl ProfileHomePort for ManagedProfileHomes {
    fn prepare(
        &self,
        profile_id: ProfileId,
    ) -> Result<ManagedProfileEnvironment, ProfileWorkflowError> {
        Self::prepare(self, profile_id).map_err(|_| ProfileWorkflowError::HomeUnavailable)
    }
}

fn ensure_secure_directory(path: &Path) -> Result<(), ManagedProfileHomeError> {
    match fs::symlink_metadata(path) {
        Ok(_) => validate_existing_directory(path, true),
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            create_owner_only_directory(path)?;
            validate_existing_directory(path, true)
        }
        Err(error) => Err(error.into()),
    }
}

fn validate_existing_directory(
    path: &Path,
    require_owner_only: bool,
) -> Result<(), ManagedProfileHomeError> {
    let metadata = fs::symlink_metadata(path)?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err(ManagedProfileHomeError::UnsafeType);
    }
    if require_owner_only {
        validate_platform_security(&metadata)?;
    }
    Ok(())
}

#[cfg(unix)]
fn create_owner_only_directory(path: &Path) -> io::Result<()> {
    use std::os::unix::fs::DirBuilderExt;

    let mut builder = fs::DirBuilder::new();
    builder.mode(0o700).create(path)
}

#[cfg(not(unix))]
fn create_owner_only_directory(path: &Path) -> io::Result<()> {
    fs::create_dir(path)
}

#[cfg(unix)]
fn validate_platform_security(metadata: &fs::Metadata) -> Result<(), ManagedProfileHomeError> {
    use std::os::unix::fs::PermissionsExt;

    if metadata.permissions().mode() & 0o077 != 0 {
        return Err(ManagedProfileHomeError::UnsafePermissions);
    }
    #[cfg(target_os = "linux")]
    {
        use std::os::unix::fs::MetadataExt;
        let process = fs::metadata("/proc/self")?;
        if metadata.uid() != process.uid() {
            return Err(ManagedProfileHomeError::UnsafeOwner);
        }
    }
    Ok(())
}

#[cfg(not(unix))]
fn validate_platform_security(_metadata: &fs::Metadata) -> Result<(), ManagedProfileHomeError> {
    Ok(())
}

/// Managed profile directory failure.
#[derive(Debug, Error)]
pub enum ManagedProfileHomeError {
    /// Data roots must be absolute and have an existing safe parent.
    #[error("managed profile data root is invalid")]
    InvalidRoot,
    /// A path is a link or another non-directory type.
    #[error("managed profile path has an unsafe type")]
    UnsafeType,
    /// A managed directory is accessible by group or other users.
    #[error("managed profile directory permissions are unsafe")]
    UnsafePermissions,
    /// A managed directory is not owned by the current Linux user.
    #[error("managed profile directory owner is unsafe")]
    UnsafeOwner,
    /// Filesystem operation failed.
    #[error("managed profile filesystem operation failed")]
    Io(#[from] io::Error),
}

#[cfg(all(test, unix))]
mod tests {
    use super::{ManagedProfileHomeError, ManagedProfileHomes};
    use agy_auth_domain::ProfileId;
    use std::fs;
    use std::os::unix::fs::{PermissionsExt, symlink};
    use std::path::{Path, PathBuf};
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};

    static SEQUENCE: AtomicU64 = AtomicU64::new(0);

    fn fixture() -> PathBuf {
        let path = std::env::temp_dir().join(format!(
            "agy-auth-profile-homes-{}-{}-{}",
            std::process::id(),
            SEQUENCE.fetch_add(1, Ordering::Relaxed),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("clock after Unix epoch")
                .as_nanos()
        ));
        fs::create_dir(&path).expect("create fixture");
        path
    }

    fn remove(path: &Path) {
        fs::remove_dir_all(path).expect("remove fixture");
    }

    #[test]
    fn creates_distinct_owner_only_profile_homes_idempotently() {
        let parent = fixture();
        let manager = ManagedProfileHomes::new(parent.join("data")).expect("manager");
        let work_id = ProfileId::new();
        let work = manager.prepare(work_id).expect("work home");
        let personal = manager.prepare(ProfileId::new()).expect("personal home");

        assert_ne!(work, personal);
        for path in [
            &parent.join("data"),
            &work.home,
            &work.runtime_directory,
            &personal.home,
            &personal.runtime_directory,
        ] {
            let metadata = fs::symlink_metadata(path).expect("directory metadata");
            assert!(metadata.is_dir());
            assert_eq!(metadata.permissions().mode() & 0o777, 0o700);
        }
        assert_eq!(manager.prepare(work_id).expect("repeat"), work);
        remove(&parent);
    }

    #[test]
    fn rejects_linked_and_group_accessible_managed_paths() {
        let parent = fixture();
        let target = parent.join("target");
        fs::create_dir(&target).expect("create target");
        fs::set_permissions(&target, fs::Permissions::from_mode(0o700)).expect("secure target");
        symlink(&target, parent.join("data")).expect("create data link");
        let linked = ManagedProfileHomes::new(parent.join("data")).expect("manager");
        assert!(matches!(
            linked.prepare(ProfileId::new()),
            Err(ManagedProfileHomeError::UnsafeType)
        ));
        fs::remove_file(parent.join("data")).expect("remove link");

        fs::create_dir(parent.join("data")).expect("create data");
        fs::set_permissions(parent.join("data"), fs::Permissions::from_mode(0o750))
            .expect("set unsafe mode");
        let unsafe_mode = ManagedProfileHomes::new(parent.join("data")).expect("manager");
        assert!(matches!(
            unsafe_mode.prepare(ProfileId::new()),
            Err(ManagedProfileHomeError::UnsafePermissions)
        ));
        remove(&parent);
    }
}
