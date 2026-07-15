//! Capability-gated Gemini CLI adapter boundary.

use gemini_auth_domain::{ProfileId, ProviderKind};
use gemini_auth_process::{DiscoveredClient, DiscoveryError, OfficialClient, discover_client};
use std::ffi::{OsStr, OsString};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Authentication-related variables reviewed for removal from a Gemini profile child only.
pub const AUTH_CONFLICTING_ENVIRONMENT: &[&str] = &[
    "GEMINI_API_KEY",
    "GOOGLE_API_KEY",
    "GOOGLE_APPLICATION_CREDENTIALS",
];

/// Identify the provider implemented by this adapter.
#[must_use]
pub const fn provider_kind() -> ProviderKind {
    ProviderKind::GeminiCli
}

/// Discovered Gemini CLI with isolation capability state.
#[derive(Clone, Debug)]
pub struct GeminiCliAdapter {
    client: DiscoveredClient,
    isolation: IsolationEvidence,
}

impl GeminiCliAdapter {
    /// Discover Gemini CLI without reading client state or assuming home isolation.
    ///
    /// # Errors
    ///
    /// Returns a discovery error when no safe executable can be found.
    pub fn discover(
        explicit: Option<&Path>,
        search_path: Option<&OsStr>,
    ) -> Result<Self, DiscoveryError> {
        Ok(Self {
            client: discover_client(OfficialClient::GeminiCli, explicit, search_path)?,
            isolation: IsolationEvidence::unverified(),
        })
    }

    /// Return the discovered executable.
    #[must_use]
    pub fn executable(&self) -> &Path {
        self.client.executable()
    }

    /// Report whether versioned evidence currently permits isolated-home execution.
    #[must_use]
    pub const fn isolated_home_verified(&self) -> bool {
        self.isolation.verified
    }

    /// Build a child-only isolated-home execution plan.
    ///
    /// # Errors
    ///
    /// Returns an error while isolation is unverified or when the managed home path is unsafe.
    pub fn plan_exec<I, S>(
        &self,
        data_root: &Path,
        profile_id: ProfileId,
        arguments: I,
    ) -> Result<GeminiExecPlan, GeminiAdapterError>
    where
        I: IntoIterator<Item = S>,
        S: Into<OsString>,
    {
        if !self.isolation.verified {
            return Err(GeminiAdapterError::IsolationUnverified);
        }
        let profile_home = prepare_profile_home(data_root, profile_id)?;
        Ok(GeminiExecPlan {
            executable: self.client.executable().to_owned(),
            arguments: arguments.into_iter().map(Into::into).collect(),
            profile_home,
        })
    }

    #[cfg(test)]
    fn with_synthetic_verified_isolation(mut self) -> Self {
        self.isolation = IsolationEvidence { verified: true };
        self
    }
}

/// Private capability evidence; production currently exposes only the unverified state.
#[derive(Clone, Copy, Debug)]
struct IsolationEvidence {
    verified: bool,
}

impl IsolationEvidence {
    const fn unverified() -> Self {
        Self { verified: false }
    }
}

/// Direct executable plan with child-only environment changes.
pub struct GeminiExecPlan {
    executable: PathBuf,
    arguments: Vec<OsString>,
    profile_home: PathBuf,
}

impl GeminiExecPlan {
    /// Return the validated official executable.
    #[must_use]
    pub fn executable(&self) -> &Path {
        &self.executable
    }

    /// Return the managed profile home.
    #[must_use]
    pub fn profile_home(&self) -> &Path {
        &self.profile_home
    }

    /// Return argv values without shell serialization.
    #[must_use]
    pub fn arguments(&self) -> &[OsString] {
        &self.arguments
    }

    /// Return names proposed for removal from the child environment.
    #[must_use]
    pub const fn removed_environment_names(&self) -> &'static [&'static str] {
        AUTH_CONFLICTING_ENVIRONMENT
    }

    /// Construct a child command. This does not modify the parent process environment.
    #[must_use]
    pub fn command(&self) -> Command {
        let mut command = Command::new(&self.executable);
        command.args(&self.arguments);
        command.env("GEMINI_CLI_HOME", &self.profile_home);
        for name in AUTH_CONFLICTING_ENVIRONMENT {
            command.env_remove(name);
        }
        command
    }
}

fn prepare_profile_home(
    data_root: &Path,
    profile_id: ProfileId,
) -> Result<PathBuf, GeminiAdapterError> {
    if !data_root.is_absolute() {
        return Err(GeminiAdapterError::RelativeDataRoot);
    }
    ensure_directory(data_root)?;
    let homes = data_root.join("homes");
    ensure_directory(&homes)?;
    let profile_home = homes.join(profile_id.to_string());
    ensure_directory(&profile_home)?;
    if !profile_home.starts_with(&homes) {
        return Err(GeminiAdapterError::PathEscape);
    }
    Ok(profile_home)
}

fn ensure_directory(path: &Path) -> Result<(), GeminiAdapterError> {
    match fs::symlink_metadata(path) {
        Ok(metadata) if metadata.file_type().is_symlink() || !metadata.is_dir() => {
            return Err(GeminiAdapterError::UnsafeDirectory);
        }
        Ok(_) => {}
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            let mut builder = fs::DirBuilder::new();
            builder.recursive(false);
            #[cfg(unix)]
            {
                use std::os::unix::fs::DirBuilderExt;
                builder.mode(0o700);
            }
            builder.create(path)?;
        }
        Err(error) => return Err(error.into()),
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let metadata = fs::metadata(path)?;
        if metadata.permissions().mode() & 0o077 != 0 {
            return Err(GeminiAdapterError::UnsafePermissions);
        }
    }
    Ok(())
}

/// Fail-closed Gemini adapter error without environment values or client output.
#[derive(Debug)]
pub enum GeminiAdapterError {
    /// Real isolated-home behavior has not been verified.
    IsolationUnverified,
    /// Managed data root must be absolute.
    RelativeDataRoot,
    /// Managed path would escape its home root.
    PathEscape,
    /// Existing managed path is a link or not a directory.
    UnsafeDirectory,
    /// Existing managed directory permits group or other access.
    UnsafePermissions,
    /// Directory metadata or creation failed.
    Io(io::Error),
}

impl From<io::Error> for GeminiAdapterError {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}

impl std::fmt::Display for GeminiAdapterError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::IsolationUnverified => {
                formatter.write_str("Gemini CLI home isolation is unverified")
            }
            Self::RelativeDataRoot => formatter.write_str("managed data root must be absolute"),
            Self::PathEscape => formatter.write_str("managed profile home escaped its root"),
            Self::UnsafeDirectory => formatter.write_str("managed profile home path is unsafe"),
            Self::UnsafePermissions => {
                formatter.write_str("managed profile home permissions are unsafe")
            }
            Self::Io(_) => formatter.write_str("managed profile home operation failed"),
        }
    }
}

impl std::error::Error for GeminiAdapterError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(error) => Some(error),
            _ => None,
        }
    }
}

#[cfg(all(test, unix))]
mod tests {
    use super::{AUTH_CONFLICTING_ENVIRONMENT, GeminiAdapterError, GeminiCliAdapter};
    use gemini_auth_domain::ProfileId;
    use std::fs;
    use std::os::unix::fs::PermissionsExt;
    use std::path::{Path, PathBuf};
    use std::sync::atomic::{AtomicU64, Ordering};

    static FIXTURE_SEQUENCE: AtomicU64 = AtomicU64::new(0);

    fn fixture() -> (PathBuf, PathBuf) {
        let root = std::env::temp_dir().join(format!(
            "gemini-auth-gemini-adapter-{}-{}",
            std::process::id(),
            FIXTURE_SEQUENCE.fetch_add(1, Ordering::Relaxed)
        ));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir(&root).expect("create fixture root");
        fs::set_permissions(&root, fs::Permissions::from_mode(0o700)).expect("secure root");
        let executable = root.join("gemini");
        fs::write(
            &executable,
            "#!/bin/sh\nprintf '%s\\n' \"$GEMINI_CLI_HOME\" \"${GEMINI_API_KEY-unset}\" \"$1\"\n",
        )
        .expect("write fake Gemini CLI");
        fs::set_permissions(&executable, fs::Permissions::from_mode(0o700))
            .expect("make executable");
        (root, executable)
    }

    fn adapter(executable: &Path) -> GeminiCliAdapter {
        GeminiCliAdapter::discover(Some(executable), None)
            .expect("discover fake Gemini CLI")
            .with_synthetic_verified_isolation()
    }

    #[test]
    fn production_discovery_keeps_isolation_unverified() {
        let (root, executable) = fixture();
        let adapter = GeminiCliAdapter::discover(Some(&executable), None).expect("discover client");
        assert!(!adapter.isolated_home_verified());
        assert!(matches!(
            adapter.plan_exec(&root, ProfileId::new(), ["--version"]),
            Err(GeminiAdapterError::IsolationUnverified)
        ));
        fs::remove_dir_all(root).expect("remove fixture");
    }

    #[test]
    fn synthetic_plan_is_contained_and_owner_only() {
        let (root, executable) = fixture();
        let profile_id = ProfileId::new();
        let plan = adapter(&executable)
            .plan_exec(&root, profile_id, ["value; not-a-shell"])
            .expect("build plan");
        assert!(plan.profile_home().starts_with(root.join("homes")));
        assert_eq!(plan.arguments(), ["value; not-a-shell"]);
        let mode = fs::metadata(plan.profile_home())
            .expect("profile home metadata")
            .permissions()
            .mode()
            & 0o777;
        assert_eq!(mode, 0o700);
        fs::remove_dir_all(root).expect("remove fixture");
    }

    #[test]
    fn command_changes_only_child_environment() {
        let (root, executable) = fixture();
        let original = std::env::var_os("GEMINI_API_KEY");
        let plan = adapter(&executable)
            .plan_exec(&root, ProfileId::new(), ["literal;argument"])
            .expect("build plan");
        let output = plan.command().output().expect("run fake Gemini CLI");
        assert!(output.status.success());
        let lines = String::from_utf8(output.stdout).expect("synthetic UTF-8 output");
        let mut lines = lines.lines();
        assert_eq!(
            lines.next(),
            Some(plan.profile_home().to_string_lossy().as_ref())
        );
        assert_eq!(lines.next(), Some("unset"));
        assert_eq!(lines.next(), Some("literal;argument"));
        assert_eq!(std::env::var_os("GEMINI_API_KEY"), original);
        assert_eq!(
            plan.removed_environment_names(),
            AUTH_CONFLICTING_ENVIRONMENT
        );
        fs::remove_dir_all(root).expect("remove fixture");
    }

    #[cfg(unix)]
    #[test]
    fn symlinked_managed_home_fails_closed() {
        use std::os::unix::fs::symlink;
        let (root, executable) = fixture();
        let homes = root.join("homes");
        fs::create_dir(&homes).expect("create homes");
        fs::set_permissions(&homes, fs::Permissions::from_mode(0o700)).expect("secure homes");
        let profile_id = ProfileId::new();
        symlink(&root, homes.join(profile_id.to_string())).expect("create profile symlink");
        assert!(matches!(
            adapter(&executable).plan_exec(&root, profile_id, ["--version"]),
            Err(GeminiAdapterError::UnsafeDirectory)
        ));
        fs::remove_dir_all(root).expect("remove fixture");
    }
}
