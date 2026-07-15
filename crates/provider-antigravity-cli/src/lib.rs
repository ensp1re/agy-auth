//! Capability-gated Antigravity CLI adapter boundary.

use agy_auth_app::{ClientDiagnostic, DoctorClientProbe};
use agy_auth_domain::ProviderKind;
use agy_auth_process::{DiscoveryError, OfficialClient, ProcessError, discover_client};
use std::ffi::OsString;
use std::path::PathBuf;
use std::time::Duration;

/// Identify the provider implemented by this adapter.
#[must_use]
pub const fn provider_kind() -> ProviderKind {
    ProviderKind::AntigravityCli
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
}
