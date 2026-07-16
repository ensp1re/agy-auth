//! Protected opaque credential files beneath managed profile homes.

use std::fs::{self, File, OpenOptions};
use std::io::{self, Read, Write};
use std::path::{Component, Path, PathBuf};
use thiserror::Error;
use uuid::Uuid;

#[cfg(feature = "experimental-profile-credentials")]
use agy_auth_app::{CredentialFilePort, CredentialWorkflowError, OpaqueSecretBytes};

/// Opaque credential material that cannot be formatted or serialized accidentally.
///
/// The wrapper deliberately does not implement `Clone`, `Debug`, `Display`, `Serialize`, or
/// equality.
pub struct OpaqueCredentialBytes {
    value: Vec<u8>,
}

impl OpaqueCredentialBytes {
    /// Wrap non-empty bounded credential bytes.
    ///
    /// # Errors
    ///
    /// Returns a non-secret error when the value is empty or exceeds `maximum_bytes`.
    pub fn new(value: Vec<u8>, maximum_bytes: usize) -> Result<Self, CredentialFileError> {
        if maximum_bytes == 0 || value.is_empty() || value.len() > maximum_bytes {
            return Err(CredentialFileError::InvalidCredentialSize);
        }
        Ok(Self { value })
    }

    /// Consume the wrapper and return bytes for an approved provider parser or secret store.
    #[must_use]
    pub fn into_secret_bytes(self) -> Vec<u8> {
        self.value
    }

    fn expose(&self) -> &[u8] {
        &self.value
    }
}

/// Atomically materializes and rereads official-client credential files within one profile home.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProfileCredentialFiles {
    profile_home: PathBuf,
}

impl ProfileCredentialFiles {
    /// Bind the adapter to an absolute owner-only profile home.
    ///
    /// # Errors
    ///
    /// Returns an error when the home is relative, linked, not a directory, not owned by the current
    /// user, or accessible to group or other users.
    pub fn new(profile_home: impl Into<PathBuf>) -> Result<Self, CredentialFileError> {
        let profile_home = profile_home.into();
        if !profile_home.is_absolute() {
            return Err(CredentialFileError::InvalidProfileHome);
        }
        validate_secure_directory(&profile_home)?;
        Ok(Self { profile_home })
    }

    /// Atomically write opaque bytes at a safe home-relative path.
    ///
    /// Parent directories are created owner-only. The destination is never truncated in place.
    ///
    /// # Errors
    ///
    /// Returns a non-secret error for unsafe paths, links, hardlinks, ownership, permissions, file
    /// types, or filesystem failures.
    pub fn materialize(
        &self,
        relative_path: &Path,
        credential: &OpaqueCredentialBytes,
    ) -> Result<(), CredentialFileError> {
        validate_relative_path(relative_path)?;
        validate_secure_directory(&self.profile_home)?;
        let destination = self.profile_home.join(relative_path);
        let parent = destination
            .parent()
            .ok_or(CredentialFileError::InvalidRelativePath)?;
        ensure_secure_descendant_directories(&self.profile_home, parent)?;
        validate_existing_destination(&destination)?;

        let temporary = parent.join(format!(".agy-auth-{}.tmp", Uuid::new_v4()));
        let result = write_and_replace(&temporary, &destination, credential.expose(), parent);
        if result.is_err() {
            let _ = fs::remove_file(&temporary);
        }
        result?;
        validate_secure_regular_file(&destination).map(|_| ())
    }

    /// Read a bounded opaque credential file from a safe home-relative path.
    ///
    /// # Errors
    ///
    /// Returns a non-secret error for unsafe paths, links, hardlinks, ownership, permissions, file
    /// types, oversized content, or filesystem failures.
    pub fn read(
        &self,
        relative_path: &Path,
        maximum_bytes: usize,
    ) -> Result<OpaqueCredentialBytes, CredentialFileError> {
        if maximum_bytes == 0 {
            return Err(CredentialFileError::InvalidCredentialSize);
        }
        validate_relative_path(relative_path)?;
        validate_secure_directory(&self.profile_home)?;
        let path = self.profile_home.join(relative_path);
        validate_secure_ancestors(&self.profile_home, &path)?;
        let metadata = validate_secure_regular_file(&path)?;
        let length =
            usize::try_from(metadata.len()).map_err(|_| CredentialFileError::CredentialTooLarge)?;
        if length == 0 || length > maximum_bytes {
            return Err(CredentialFileError::CredentialTooLarge);
        }

        let mut file = open_read_no_follow(&path)?;
        let mut value = Vec::with_capacity(length);
        Read::by_ref(&mut file)
            .take(
                u64::try_from(maximum_bytes)
                    .unwrap_or(u64::MAX)
                    .saturating_add(1),
            )
            .read_to_end(&mut value)?;
        if value.len() > maximum_bytes {
            return Err(CredentialFileError::CredentialTooLarge);
        }
        let after = file.metadata()?;
        validate_regular_file_metadata(&after)?;
        OpaqueCredentialBytes::new(value, maximum_bytes)
    }
}

#[cfg(feature = "experimental-profile-credentials")]
impl CredentialFilePort for ProfileCredentialFiles {
    fn materialize(
        &self,
        relative_path: &Path,
        envelope: &OpaqueSecretBytes,
    ) -> Result<(), CredentialWorkflowError> {
        let credential = OpaqueCredentialBytes::new(
            envelope.expose_secret().to_vec(),
            envelope.expose_secret().len(),
        )
        .map_err(|_| CredentialWorkflowError::CredentialStorageFailed)?;
        Self::materialize(self, relative_path, &credential)
            .map_err(|_| CredentialWorkflowError::CredentialStorageFailed)
    }

    fn read(
        &self,
        relative_path: &Path,
        maximum_bytes: usize,
    ) -> Result<OpaqueSecretBytes, CredentialWorkflowError> {
        let credential = Self::read(self, relative_path, maximum_bytes)
            .map_err(|_| CredentialWorkflowError::CredentialStorageFailed)?;
        OpaqueSecretBytes::new(credential.into_secret_bytes(), maximum_bytes)
            .map_err(|_| CredentialWorkflowError::CredentialStorageFailed)
    }
}

fn validate_relative_path(path: &Path) -> Result<(), CredentialFileError> {
    if path.as_os_str().is_empty() || path.is_absolute() {
        return Err(CredentialFileError::InvalidRelativePath);
    }
    let mut count = 0_usize;
    for component in path.components() {
        match component {
            Component::Normal(value) if !value.is_empty() => count += 1,
            _ => return Err(CredentialFileError::InvalidRelativePath),
        }
    }
    if count == 0 || count > 16 {
        return Err(CredentialFileError::InvalidRelativePath);
    }
    Ok(())
}

fn ensure_secure_descendant_directories(
    profile_home: &Path,
    target: &Path,
) -> Result<(), CredentialFileError> {
    let relative = target
        .strip_prefix(profile_home)
        .map_err(|_| CredentialFileError::InvalidRelativePath)?;
    let mut current = profile_home.to_path_buf();
    for component in relative.components() {
        let Component::Normal(value) = component else {
            return Err(CredentialFileError::InvalidRelativePath);
        };
        current.push(value);
        match fs::symlink_metadata(&current) {
            Ok(_) => {
                validate_secure_directory(&current)?;
            }
            Err(error) if error.kind() == io::ErrorKind::NotFound => {
                create_owner_only_directory(&current)?;
                validate_secure_directory(&current)?;
            }
            Err(error) => return Err(error.into()),
        }
    }
    Ok(())
}

fn validate_secure_ancestors(
    profile_home: &Path,
    target: &Path,
) -> Result<(), CredentialFileError> {
    let parent = target
        .parent()
        .ok_or(CredentialFileError::InvalidRelativePath)?;
    let relative = parent
        .strip_prefix(profile_home)
        .map_err(|_| CredentialFileError::InvalidRelativePath)?;
    let mut current = profile_home.to_path_buf();
    validate_secure_directory(&current)?;
    for component in relative.components() {
        let Component::Normal(value) = component else {
            return Err(CredentialFileError::InvalidRelativePath);
        };
        current.push(value);
        validate_secure_directory(&current)?;
    }
    Ok(())
}

fn validate_existing_destination(path: &Path) -> Result<(), CredentialFileError> {
    match fs::symlink_metadata(path) {
        Ok(_) => {
            validate_secure_regular_file(path)?;
            Ok(())
        }
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error.into()),
    }
}

fn write_and_replace(
    temporary: &Path,
    destination: &Path,
    bytes: &[u8],
    parent: &Path,
) -> Result<(), CredentialFileError> {
    let mut file = create_owner_only_new_file(temporary)?;
    file.write_all(bytes)?;
    file.sync_all()?;
    drop(file);
    fs::rename(temporary, destination)?;
    sync_directory(parent)?;
    Ok(())
}

fn validate_secure_directory(path: &Path) -> Result<fs::Metadata, CredentialFileError> {
    let metadata = fs::symlink_metadata(path)?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err(CredentialFileError::UnsafeFileType);
    }
    validate_owner_and_mode(&metadata, 0o077)?;
    Ok(metadata)
}

fn validate_secure_regular_file(path: &Path) -> Result<fs::Metadata, CredentialFileError> {
    let metadata = fs::symlink_metadata(path)?;
    validate_regular_file_metadata(&metadata)?;
    Ok(metadata)
}

fn validate_regular_file_metadata(metadata: &fs::Metadata) -> Result<(), CredentialFileError> {
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Err(CredentialFileError::UnsafeFileType);
    }
    validate_owner_and_mode(metadata, 0o177)?;
    validate_single_link(metadata)
}

#[cfg(unix)]
fn validate_owner_and_mode(
    metadata: &fs::Metadata,
    forbidden_mode: u32,
) -> Result<(), CredentialFileError> {
    use std::os::unix::fs::{MetadataExt, PermissionsExt};

    if metadata.permissions().mode() & forbidden_mode != 0 {
        return Err(CredentialFileError::UnsafePermissions);
    }
    #[cfg(target_os = "linux")]
    {
        let process = fs::metadata("/proc/self")?;
        if metadata.uid() != process.uid() {
            return Err(CredentialFileError::UnsafeOwner);
        }
    }
    Ok(())
}

#[cfg(not(unix))]
fn validate_owner_and_mode(
    _metadata: &fs::Metadata,
    _forbidden_mode: u32,
) -> Result<(), CredentialFileError> {
    Ok(())
}

#[cfg(unix)]
fn validate_single_link(metadata: &fs::Metadata) -> Result<(), CredentialFileError> {
    use std::os::unix::fs::MetadataExt;

    if metadata.nlink() != 1 {
        return Err(CredentialFileError::UnsafeLinkCount);
    }
    Ok(())
}

#[cfg(not(unix))]
fn validate_single_link(_metadata: &fs::Metadata) -> Result<(), CredentialFileError> {
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
fn create_owner_only_new_file(path: &Path) -> io::Result<File> {
    use std::os::unix::fs::OpenOptionsExt;

    OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .custom_flags(libc::O_NOFOLLOW)
        .open(path)
}

#[cfg(not(unix))]
fn create_owner_only_new_file(path: &Path) -> io::Result<File> {
    OpenOptions::new().write(true).create_new(true).open(path)
}

#[cfg(unix)]
fn open_read_no_follow(path: &Path) -> io::Result<File> {
    use std::os::unix::fs::OpenOptionsExt;

    OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_NOFOLLOW)
        .open(path)
}

#[cfg(not(unix))]
fn open_read_no_follow(path: &Path) -> io::Result<File> {
    OpenOptions::new().read(true).open(path)
}

fn sync_directory(path: &Path) -> Result<(), CredentialFileError> {
    #[cfg(unix)]
    {
        File::open(path)?.sync_all()?;
    }
    #[cfg(not(unix))]
    {
        let _ = path;
    }
    Ok(())
}

/// Protected credential-file failure without secret-bearing context.
#[derive(Debug, Error)]
pub enum CredentialFileError {
    /// The configured profile home is not a safe absolute directory.
    #[error("invalid profile home")]
    InvalidProfileHome,
    /// The credential destination is not a bounded normal relative path.
    #[error("invalid credential relative path")]
    InvalidRelativePath,
    /// Credential bytes are empty, oversized, or configured with an invalid bound.
    #[error("invalid credential size")]
    InvalidCredentialSize,
    /// A credential file exceeds the requested read bound.
    #[error("credential file is too large")]
    CredentialTooLarge,
    /// A path is a symlink or has an unexpected file type.
    #[error("credential path has an unsafe file type")]
    UnsafeFileType,
    /// A path is accessible to group or other users.
    #[error("credential path permissions are unsafe")]
    UnsafePermissions,
    /// A path is not owned by the current user.
    #[error("credential path owner is unsafe")]
    UnsafeOwner,
    /// A credential file has more than one hardlink.
    #[error("credential file link count is unsafe")]
    UnsafeLinkCount,
    /// Filesystem operation failed.
    #[error("credential file operation failed")]
    Io(#[from] io::Error),
}

#[cfg(all(test, unix))]
mod tests {
    use super::{CredentialFileError, OpaqueCredentialBytes, ProfileCredentialFiles};
    use std::fs;
    use std::os::unix::fs::{PermissionsExt, symlink};
    use std::path::{Path, PathBuf};
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};

    static SEQUENCE: AtomicU64 = AtomicU64::new(0);
    const RELATIVE: &str = ".gemini/antigravity-cli/antigravity-oauth-token";

    fn fixture() -> PathBuf {
        let path = std::env::temp_dir().join(format!(
            "agy-auth-credential-files-{}-{}-{}",
            std::process::id(),
            SEQUENCE.fetch_add(1, Ordering::Relaxed),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("clock after Unix epoch")
                .as_nanos()
        ));
        fs::create_dir(&path).expect("create fixture");
        fs::set_permissions(&path, fs::Permissions::from_mode(0o700)).expect("secure fixture");
        path
    }

    fn synthetic(value: &[u8]) -> OpaqueCredentialBytes {
        OpaqueCredentialBytes::new(value.to_vec(), 4096).expect("synthetic credential")
    }

    fn remove(path: &Path) {
        fs::remove_dir_all(path).expect("remove fixture");
    }

    #[test]
    fn atomically_materializes_and_rereads_owner_only_credentials() {
        let home = fixture();
        let files = ProfileCredentialFiles::new(&home).expect("credential files");
        let relative = Path::new(RELATIVE);

        files
            .materialize(relative, &synthetic(b"synthetic-first"))
            .expect("first materialization");
        files
            .materialize(relative, &synthetic(b"synthetic-second"))
            .expect("atomic replacement");
        let reread = files.read(relative, 4096).expect("bounded reread");

        assert_eq!(reread.into_secret_bytes(), b"synthetic-second");
        let destination = home.join(relative);
        assert_eq!(
            fs::metadata(&destination)
                .expect("credential metadata")
                .permissions()
                .mode()
                & 0o777,
            0o600
        );
        for directory in [home.join(".gemini"), home.join(".gemini/antigravity-cli")] {
            assert_eq!(
                fs::metadata(directory)
                    .expect("directory metadata")
                    .permissions()
                    .mode()
                    & 0o777,
                0o700
            );
        }
        let temporary_count = fs::read_dir(destination.parent().expect("parent"))
            .expect("read parent")
            .filter_map(Result::ok)
            .filter(|entry| {
                entry
                    .file_name()
                    .to_string_lossy()
                    .starts_with(".agy-auth-")
            })
            .count();
        assert_eq!(temporary_count, 0);
        remove(&home);
    }

    #[test]
    fn rejects_escaping_paths_and_linked_ancestors() {
        let home = fixture();
        let files = ProfileCredentialFiles::new(&home).expect("credential files");
        assert!(matches!(
            files.materialize(Path::new("../credential"), &synthetic(b"synthetic")),
            Err(CredentialFileError::InvalidRelativePath)
        ));
        assert!(matches!(
            files.materialize(Path::new("/tmp/credential"), &synthetic(b"synthetic")),
            Err(CredentialFileError::InvalidRelativePath)
        ));

        let outside = fixture();
        symlink(&outside, home.join(".gemini")).expect("linked ancestor");
        assert!(matches!(
            files.materialize(Path::new(RELATIVE), &synthetic(b"synthetic")),
            Err(CredentialFileError::UnsafeFileType)
        ));
        remove(&home);
        remove(&outside);
    }

    #[test]
    fn rejects_unsafe_destination_permissions_hardlinks_and_oversized_reads() {
        let home = fixture();
        let files = ProfileCredentialFiles::new(&home).expect("credential files");
        let relative = Path::new(RELATIVE);
        files
            .materialize(relative, &synthetic(b"synthetic-existing"))
            .expect("materialize");
        let destination = home.join(relative);

        fs::set_permissions(&destination, fs::Permissions::from_mode(0o640))
            .expect("unsafe permissions");
        assert!(matches!(
            files.read(relative, 4096),
            Err(CredentialFileError::UnsafePermissions)
        ));
        fs::set_permissions(&destination, fs::Permissions::from_mode(0o600))
            .expect("restore permissions");

        let linked = home.join(".gemini/antigravity-cli/linked-copy");
        fs::hard_link(&destination, &linked).expect("hardlink");
        assert!(matches!(
            files.read(relative, 4096),
            Err(CredentialFileError::UnsafeLinkCount)
        ));
        fs::remove_file(&linked).expect("remove hardlink");

        assert!(matches!(
            files.read(relative, 4),
            Err(CredentialFileError::CredentialTooLarge)
        ));
        remove(&home);
    }
}
