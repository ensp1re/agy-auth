//! Capability-gated Antigravity CLI adapter boundary.

mod credential_envelope;

use agy_auth_app::{ClientDiagnostic, DoctorClientProbe};
use agy_auth_domain::ProviderKind;
#[cfg(feature = "experimental-profile-credentials")]
use agy_auth_process::{
    DiscoveredClient, InteractiveCompletion, IsolatedClientEnvironment, run_interactive_isolated,
    run_interactive_isolated_until_file,
};
use agy_auth_process::{DiscoveryError, OfficialClient, ProcessError, discover_client};
use std::ffi::OsString;
use std::path::PathBuf;
use std::time::Duration;

pub use credential_envelope::{
    ANTIGRAVITY_TOKEN_RELATIVE_PATH, ConsumerRefreshCredential, CredentialEnvelopeError,
    build_consumer_token_envelope, consumer_token_relative_path,
    extract_consumer_refresh_credential,
};

#[cfg(feature = "experimental-profile-credentials")]
use agy_auth_app::{
    CredentialEnvelopePort, CredentialMaterializationPlan, CredentialSessionClientPort,
    CredentialWorkflowError, ManagedProfileEnvironment, OpaqueSecretBytes,
};

/// Identify the provider implemented by this adapter.
#[must_use]
pub const fn provider_kind() -> ProviderKind {
    ProviderKind::AntigravityCli
}

/// Experimental adapter connecting the verified `agy` envelope to the application workflow.
#[cfg(feature = "experimental-profile-credentials")]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct AntigravityCredentialEnvelope;

/// Experimental real-client session adapter for verified Linux SSH profile homes.
#[cfg(feature = "experimental-profile-credentials")]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AntigravityInteractiveSession {
    client: DiscoveredClient,
    environment: IsolatedClientEnvironment,
}

#[cfg(feature = "experimental-profile-credentials")]
impl AntigravityInteractiveSession {
    /// Discover `agy` and bind it to one prepared profile environment.
    ///
    /// # Errors
    ///
    /// Returns a non-secret workflow error when discovery or environment validation fails.
    pub fn new(
        explicit_client: Option<&std::path::Path>,
        search_path: OsString,
        environment: &ManagedProfileEnvironment,
        terminal: Option<&str>,
    ) -> Result<Self, CredentialWorkflowError> {
        let client = discover_client(
            OfficialClient::AntigravityCli,
            explicit_client,
            Some(&search_path),
        )
        .map_err(|_| CredentialWorkflowError::ClientExecutionFailed)?;
        let isolated = IsolatedClientEnvironment::new(
            environment.home.clone(),
            environment.runtime_directory.clone(),
            search_path,
        )
        .map_err(|_| CredentialWorkflowError::ClientExecutionFailed)?
        .with_ssh_file_fallback();
        let isolated = match terminal {
            Some(terminal) => isolated
                .with_terminal(terminal)
                .map_err(|_| CredentialWorkflowError::ClientExecutionFailed)?,
            None => isolated,
        };
        Ok(Self {
            client,
            environment: isolated,
        })
    }

    /// Run official `agy` until it persists the verified credential path.
    ///
    /// # Errors
    ///
    /// Returns a stable workflow error when enrollment exits early, times out, or cannot be
    /// monitored safely.
    pub fn enroll_until_credential(&self) -> Result<(), CredentialWorkflowError> {
        match run_interactive_isolated_until_file(
            self.client.executable(),
            std::iter::empty::<&str>(),
            &self.environment,
            std::path::Path::new(ANTIGRAVITY_TOKEN_RELATIVE_PATH),
            Duration::from_secs(15 * 60),
        )
        .map_err(|_| CredentialWorkflowError::ClientExecutionFailed)?
        {
            InteractiveCompletion::FileCreated => Ok(()),
            InteractiveCompletion::Exited(_) => Err(CredentialWorkflowError::ClientExecutionFailed),
        }
    }
}

#[cfg(feature = "experimental-profile-credentials")]
impl CredentialSessionClientPort for AntigravityInteractiveSession {
    fn execute(&self, arguments: &[OsString]) -> Result<i32, CredentialWorkflowError> {
        let status =
            run_interactive_isolated(self.client.executable(), arguments, &self.environment)
                .map_err(|_| CredentialWorkflowError::ClientExecutionFailed)?;
        Ok(exit_status_code(status))
    }
}

#[cfg(feature = "experimental-profile-credentials")]
fn exit_status_code(status: std::process::ExitStatus) -> i32 {
    if let Some(code) = status.code() {
        return code;
    }
    #[cfg(unix)]
    {
        use std::os::unix::process::ExitStatusExt;
        if let Some(signal) = status.signal() {
            return 128_i32.saturating_add(signal);
        }
    }
    1
}

#[cfg(feature = "experimental-profile-credentials")]
impl CredentialEnvelopePort for AntigravityCredentialEnvelope {
    fn relative_path(&self, client_version: &str) -> Result<PathBuf, CredentialWorkflowError> {
        consumer_token_relative_path(client_version)
            .map(PathBuf::from)
            .map_err(map_credential_error)
    }

    fn build_plan(
        &self,
        client_version: &str,
        refresh_credential: &OpaqueSecretBytes,
    ) -> Result<CredentialMaterializationPlan, CredentialWorkflowError> {
        let refresh = ConsumerRefreshCredential::new(refresh_credential.expose_secret().to_vec())
            .map_err(map_credential_error)?;
        let envelope = build_consumer_token_envelope(client_version, &refresh)
            .map_err(map_credential_error)?;
        Ok(CredentialMaterializationPlan {
            relative_path: self.relative_path(client_version)?,
            envelope: OpaqueSecretBytes::new(envelope, 16 * 1024)?,
        })
    }

    fn extract_refresh(
        &self,
        client_version: &str,
        rewritten_envelope: OpaqueSecretBytes,
    ) -> Result<OpaqueSecretBytes, CredentialWorkflowError> {
        let refresh = extract_consumer_refresh_credential(
            client_version,
            &rewritten_envelope.into_secret_bytes(),
        )
        .map_err(map_credential_error)?;
        OpaqueSecretBytes::new(refresh.into_secret_bytes(), 8 * 1024)
    }
}

#[cfg(feature = "experimental-profile-credentials")]
fn map_credential_error(error: CredentialEnvelopeError) -> CredentialWorkflowError {
    match error {
        CredentialEnvelopeError::UnsupportedClientVersion => {
            CredentialWorkflowError::UnsupportedClientVersion
        }
        CredentialEnvelopeError::InvalidRefreshCredential
        | CredentialEnvelopeError::InvalidEnvelopeSize
        | CredentialEnvelopeError::MalformedEnvelope
        | CredentialEnvelopeError::UnsupportedEnvelope
        | CredentialEnvelopeError::InvalidExpiry
        | CredentialEnvelopeError::SerializationFailed => {
            CredentialWorkflowError::InvalidCredentialEnvelope
        }
    }
}

/// Authentication-free Antigravity executable diagnostics.
#[derive(Clone, Debug)]
pub struct AntigravityDoctorProbe {
    explicit_client: Option<PathBuf>,
    search_path: Option<OsString>,
    timeout: Duration,
}

impl AntigravityDoctorProbe {
    /// Configure deterministic discovery inputs.
    #[must_use]
    pub fn new(explicit_client: Option<PathBuf>, search_path: Option<OsString>) -> Self {
        Self {
            explicit_client,
            search_path,
            timeout: Duration::from_secs(5),
        }
    }
}

impl DoctorClientProbe for AntigravityDoctorProbe {
    fn probe(&self) -> ClientDiagnostic {
        let client = match discover_client(
            OfficialClient::AntigravityCli,
            self.explicit_client.as_deref(),
            self.search_path.as_deref(),
        ) {
            Ok(client) => client,
            Err(error) => {
                return ClientDiagnostic {
                    found: false,
                    version: None,
                    error_code: Some(discovery_code(&error).to_owned()),
                };
            }
        };
        match client.probe_version(self.timeout) {
            Ok(version) if !version.truncated() => ClientDiagnostic {
                found: true,
                version: Some(version.version().to_owned()),
                error_code: None,
            },
            Ok(_) => ClientDiagnostic {
                found: true,
                version: None,
                error_code: Some("client_version_truncated".to_owned()),
            },
            Err(error) => ClientDiagnostic {
                found: true,
                version: None,
                error_code: Some(process_code(&error).to_owned()),
            },
        }
    }
}

fn discovery_code(error: &DiscoveryError) -> &'static str {
    match error {
        DiscoveryError::MissingSearchPath => "client_search_path_missing",
        DiscoveryError::NotFound(_) => "client_not_found",
        DiscoveryError::RelativeOverride => "client_override_relative",
        DiscoveryError::UnsafeFileType => "client_unsafe_file_type",
        DiscoveryError::NotExecutable => "client_not_executable",
        DiscoveryError::Io(_) => "client_discovery_io",
    }
}

fn process_code(error: &ProcessError) -> &'static str {
    match error {
        ProcessError::TimedOut => "client_version_timeout",
        ProcessError::ProbeFailed(_) => "client_version_failed",
        ProcessError::EmptyVersion => "client_version_empty",
        ProcessError::InvalidVersion => "client_version_invalid",
        ProcessError::UnsafeIsolation(_) => "client_isolation_unsafe",
        ProcessError::InvalidLimit
        | ProcessError::MissingPipe
        | ProcessError::ReaderPanicked
        | ProcessError::Io(_) => "client_version_io",
    }
}

#[cfg(all(test, unix))]
mod tests {
    use super::AntigravityDoctorProbe;
    use agy_auth_app::DoctorClientProbe;
    use std::fs;
    use std::io::Write;
    use std::os::unix::fs::PermissionsExt;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[cfg(feature = "experimental-profile-credentials")]
    use super::AntigravityInteractiveSession;
    #[cfg(feature = "experimental-profile-credentials")]
    use agy_auth_app::{CredentialSessionClientPort, ManagedProfileEnvironment};
    #[cfg(feature = "experimental-profile-credentials")]
    use std::ffi::OsString;

    #[test]
    fn probe_reports_bounded_version_without_authentication() {
        let directory = std::env::temp_dir().join(format!(
            "agy-auth-provider-doctor-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("clock after epoch")
                .as_nanos()
        ));
        fs::create_dir(&directory).expect("create fixture directory");
        let executable = directory.join("agy");
        let mut file = fs::File::create(&executable).expect("create fixture");
        file.write_all(b"#!/bin/sh\nprintf '1.1.2\\n'\n")
            .expect("write fixture");
        file.sync_all().expect("sync fixture");
        drop(file);
        let mut permissions = fs::metadata(&executable).expect("metadata").permissions();
        permissions.set_mode(0o700);
        fs::set_permissions(&executable, permissions).expect("set executable");

        let result = AntigravityDoctorProbe::new(Some(executable), None).probe();

        assert!(result.found);
        assert_eq!(result.version.as_deref(), Some("1.1.2"));
        assert_eq!(result.error_code, None);
        fs::remove_dir_all(directory).expect("remove fixture directory");
    }

    #[test]
    fn missing_client_has_stable_error_code() {
        let result = AntigravityDoctorProbe::new(
            Some(std::env::temp_dir().join("agy-auth-definitely-missing-client")),
            None,
        )
        .probe();

        assert!(!result.found);
        assert_eq!(result.error_code.as_deref(), Some("client_not_found"));
    }

    #[cfg(feature = "experimental-profile-credentials")]
    #[test]
    fn interactive_session_uses_direct_argv_and_ssh_profile_environment() {
        let directory = std::env::temp_dir().join(format!(
            "agy-auth-provider-session-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("clock after epoch")
                .as_nanos()
        ));
        fs::create_dir(&directory).expect("create fixture directory");
        let executable = directory.join("agy");
        let observation = directory.join("observation");
        let home = directory.join("home");
        let runtime = directory.join("runtime");
        for path in [&home, &runtime] {
            fs::create_dir(path).expect("create isolation directory");
            fs::set_permissions(path, fs::Permissions::from_mode(0o700))
                .expect("secure isolation directory");
        }
        let mut file = fs::File::create(&executable).expect("create fixture");
        file.write_all(
            b"#!/bin/sh\nprintf '%s|%s|%s|%s' \"$2\" \"$HOME\" \"${SSH_CONNECTION-unset}\" \"${USER-unset}\" > \"$1\"\nexit 29\n",
        )
        .expect("write fixture");
        file.sync_all().expect("sync fixture");
        drop(file);
        let mut permissions = fs::metadata(&executable).expect("metadata").permissions();
        permissions.set_mode(0o700);
        fs::set_permissions(&executable, permissions).expect("set executable");

        let session = AntigravityInteractiveSession::new(
            Some(&executable),
            OsString::from("/usr/bin:/bin"),
            &ManagedProfileEnvironment {
                home: home.clone(),
                runtime_directory: runtime,
            },
            Some("xterm-256color"),
        )
        .expect("session");
        let literal = "literal; shell syntax is data";
        let code = session
            .execute(&[
                observation.clone().into_os_string(),
                OsString::from(literal),
            ])
            .expect("execute fixture");

        assert_eq!(code, 29);
        assert_eq!(
            fs::read_to_string(&observation).expect("read observation"),
            format!(
                "{literal}|{}|127.0.0.1 40000 127.0.0.1 22|unset",
                home.display()
            )
        );
        assert!(!directory.join("shell syntax is data").exists());
        fs::remove_dir_all(directory).expect("remove fixture directory");
    }
}
