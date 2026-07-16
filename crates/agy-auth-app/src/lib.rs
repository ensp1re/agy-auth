//! Application-service boundary for use-case orchestration.

#[cfg(feature = "experimental-profile-credentials")]
mod credential_workflow;

use agy_auth_domain::{
    Profile, ProfileId, ProfileName, ProfileStatus, ProviderKind as DomainProviderKind,
    StorageLocator,
};
use serde::Serialize;
use std::ffi::OsString;
use std::path::PathBuf;

pub use agy_auth_domain::{ErrorCode, ProviderKind};
#[cfg(feature = "experimental-profile-credentials")]
pub use credential_workflow::{
    CredentialEnvelopePort, CredentialFilePort, CredentialMaterializationPlan,
    CredentialSessionClientPort, CredentialSessionLease, CredentialSessionLockPort,
    CredentialSessionOutcome, CredentialSessionPorts, CredentialWorkflowError, OpaqueSecretBytes,
    capture_refreshed_profile_credential, materialize_profile_credential,
    run_profile_credential_session,
};

/// Stable diagnostics schema version.
pub const DOCTOR_SCHEMA_VERSION: u32 = 1;

/// Project-owned filesystem roots used to launch one isolated official-client profile.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ManagedProfileEnvironment {
    /// Isolated effective home directory.
    pub home: PathBuf,
    /// Isolated XDG runtime directory.
    pub runtime_directory: PathBuf,
}

/// Stable failure categories for profile workflow adapters.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProfileWorkflowError {
    /// A new profile must begin in the pending state.
    InvalidInitialState,
    /// User-provided profile metadata violates a domain invariant.
    InvalidProfile,
    /// A profile must be ready before execution.
    ProfileNotReady,
    /// Non-secret profile metadata could not be reserved.
    CatalogReserveFailed,
    /// Profile metadata could not be promoted to ready.
    CatalogCommitFailed,
    /// Project-owned profile directories could not be prepared safely.
    HomeUnavailable,
    /// The official client's interactive login did not complete.
    ClientLoginFailed,
    /// The official client could not be launched for the profile.
    ClientExecFailed,
}

impl std::fmt::Display for ProfileWorkflowError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let message = match self {
            Self::InvalidInitialState => "profile must begin pending",
            Self::InvalidProfile => "profile metadata is invalid",
            Self::ProfileNotReady => "profile is not ready",
            Self::CatalogReserveFailed => "profile metadata reservation failed",
            Self::CatalogCommitFailed => "profile metadata commit failed",
            Self::HomeUnavailable => "managed profile home is unavailable",
            Self::ClientLoginFailed => "official client login failed",
            Self::ClientExecFailed => "official client execution failed",
        };
        formatter.write_str(message)
    }
}

impl std::error::Error for ProfileWorkflowError {}

/// Build pending non-secret metadata for a new isolated Antigravity profile.
///
/// # Errors
///
/// Returns [`ProfileWorkflowError::InvalidProfile`] when the name is invalid.
pub fn new_pending_profile(name: &str) -> Result<Profile, ProfileWorkflowError> {
    let id = ProfileId::new();
    let now = time::OffsetDateTime::now_utc();
    Ok(Profile {
        id,
        name: ProfileName::parse(name).map_err(|_| ProfileWorkflowError::InvalidProfile)?,
        provider: DomainProviderKind::AntigravityCli,
        storage: StorageLocator::isolated_home(&id.to_string())
            .map_err(|_| ProfileWorkflowError::InvalidProfile)?,
        account_hint: None,
        created_at: now,
        updated_at: now,
        client_version_at_capture: None,
        schema_fingerprint: None,
        status: ProfileStatus::Pending,
    })
}

/// Port for project-owned profile directory preparation.
pub trait ProfileHomePort {
    /// Create or validate the owner-only directories for a profile.
    ///
    /// # Errors
    ///
    /// Returns [`ProfileWorkflowError::HomeUnavailable`] when isolation cannot be guaranteed.
    fn prepare(
        &self,
        profile_id: ProfileId,
    ) -> Result<ManagedProfileEnvironment, ProfileWorkflowError>;
}

/// Port for atomic non-secret profile catalog transitions.
pub trait ProfileCatalogPort {
    /// Reserve pending metadata before invoking an interactive login.
    ///
    /// # Errors
    ///
    /// Returns a catalog failure when pending metadata cannot be persisted atomically.
    fn reserve(&self, profile: &Profile) -> Result<(), ProfileWorkflowError>;

    /// Mark a reserved profile ready with its observed official-client version.
    ///
    /// # Errors
    ///
    /// Returns a catalog failure when the ready transition cannot be persisted atomically.
    fn mark_ready(
        &self,
        profile_id: ProfileId,
        client_version: &str,
    ) -> Result<(), ProfileWorkflowError>;
}

/// Port that delegates authentication and execution to the official client.
pub trait ProfileClientPort {
    /// Run official interactive login and return the validated client version.
    ///
    /// # Errors
    ///
    /// Returns [`ProfileWorkflowError::ClientLoginFailed`] when login does not complete.
    fn login(
        &self,
        environment: &ManagedProfileEnvironment,
    ) -> Result<String, ProfileWorkflowError>;

    /// Run the official client with direct argv and return its process exit code.
    ///
    /// # Errors
    ///
    /// Returns [`ProfileWorkflowError::ClientExecFailed`] when the client cannot be launched.
    fn execute(
        &self,
        environment: &ManagedProfileEnvironment,
        arguments: &[OsString],
    ) -> Result<i32, ProfileWorkflowError>;
}

/// Reserve a pending profile, prepare its isolated home, delegate login, and commit readiness.
///
/// A failure after reservation deliberately leaves pending metadata for explicit recovery. The
/// application never reads or copies authentication state.
///
/// # Errors
///
/// Returns a stable workflow failure from validation or one of the three ports.
pub fn add_profile(
    profile: &Profile,
    catalog: &impl ProfileCatalogPort,
    homes: &impl ProfileHomePort,
    client: &impl ProfileClientPort,
) -> Result<(), ProfileWorkflowError> {
    if profile.status != ProfileStatus::Pending {
        return Err(ProfileWorkflowError::InvalidInitialState);
    }
    catalog.reserve(profile)?;
    let environment = homes.prepare(profile.id)?;
    let client_version = client.login(&environment)?;
    catalog.mark_ready(profile.id, &client_version)
}

/// Execute the official client inside a ready profile's isolated environment.
///
/// # Errors
///
/// Returns an error when the profile is not ready, its home is unsafe, or client launch fails.
pub fn exec_profile(
    profile: &Profile,
    arguments: &[OsString],
    homes: &impl ProfileHomePort,
    client: &impl ProfileClientPort,
) -> Result<i32, ProfileWorkflowError> {
    if profile.status != ProfileStatus::Ready {
        return Err(ProfileWorkflowError::ProfileNotReady);
    }
    let environment = homes.prepare(profile.id)?;
    client.execute(&environment, arguments)
}

/// Reproducible build identity without builder paths or environment values.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolDiagnostic {
    /// Package version.
    pub version: &'static str,
    /// Source revision, or `unknown` outside a Git build context.
    pub git_revision: &'static str,
    /// Rust compilation target triple.
    pub target: &'static str,
}

/// Safe official-client diagnostic result.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ClientDiagnostic {
    /// Whether a validated executable was found.
    pub found: bool,
    /// Bounded version output, when probing succeeded.
    pub version: Option<String>,
    /// Stable non-secret failure category.
    pub error_code: Option<String>,
}

/// Safe registry diagnostic result.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RegistryDiagnostic {
    /// Whether the non-secret registry parsed and validated.
    pub healthy: bool,
    /// Number of non-secret profile records, when healthy.
    pub profile_count: Option<usize>,
    /// Stable non-secret failure category.
    pub error_code: Option<String>,
    /// Data-root state: `absent`, `secure`, or `unsafe`.
    pub data_directory_state: &'static str,
    /// Whether owner checks passed, or `None` when absent/unavailable.
    pub owner_matches: Option<bool>,
    /// Whether permission checks passed, or `None` when absent/unavailable.
    pub permissions_secure: Option<bool>,
    /// Count of project-owned interrupted transaction markers.
    pub interrupted_transactions: usize,
}

/// Explicit capability-gate result.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CapabilityDiagnostic {
    /// Real profile switching is deliberately unavailable.
    pub profile_switching: bool,
    /// Authentication-state mutation is deliberately unavailable.
    pub auth_state_mutation: bool,
    /// Stable explanation suitable for users and automation.
    pub reason: &'static str,
}

/// Complete secret-safe doctor report.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DoctorReport {
    /// Report schema version.
    pub schema_version: u32,
    /// agy-auth build identity.
    pub tool: ToolDiagnostic,
    /// Provider identifier.
    pub provider: &'static str,
    /// Official-client status.
    pub client: ClientDiagnostic,
    /// Project-owned registry status.
    pub registry: RegistryDiagnostic,
    /// Supported capability status.
    pub capabilities: CapabilityDiagnostic,
}

impl DoctorReport {
    /// Process exit code matching the CLI contract.
    #[must_use]
    pub fn exit_code(&self) -> u8 {
        if !self.client.found {
            4
        } else if self.registry.error_code.as_deref() == Some("transaction_recovery_required") {
            10
        } else if !self.registry.healthy {
            8
        } else {
            0
        }
    }
}

/// Port for a bounded, authentication-free official-client probe.
pub trait DoctorClientProbe {
    /// Discover and version-probe the official executable.
    fn probe(&self) -> ClientDiagnostic;
}

/// Port for validating project-owned non-secret registry state.
pub trait DoctorRegistryProbe {
    /// Load and validate registry metadata without mutation.
    fn probe(&self) -> RegistryDiagnostic;
}

/// Build a complete diagnostics report without authentication access.
#[must_use]
pub fn doctor(
    client: &impl DoctorClientProbe,
    registry: &impl DoctorRegistryProbe,
) -> DoctorReport {
    DoctorReport {
        schema_version: DOCTOR_SCHEMA_VERSION,
        tool: ToolDiagnostic {
            version: env!("CARGO_PKG_VERSION"),
            git_revision: env!("AGY_AUTH_GIT_REVISION"),
            target: env!("AGY_AUTH_BUILD_TARGET"),
        },
        provider: "antigravity-cli",
        client: client.probe(),
        registry: registry.probe(),
        capabilities: CapabilityDiagnostic {
            profile_switching: false,
            auth_state_mutation: false,
            reason: "no_supported_antigravity_profile_contract",
        },
    }
}

#[cfg(test)]
mod tests {
    use super::{
        ClientDiagnostic, DoctorClientProbe, DoctorRegistryProbe, ManagedProfileEnvironment,
        ProfileCatalogPort, ProfileClientPort, ProfileHomePort, ProfileWorkflowError,
        RegistryDiagnostic, add_profile, doctor, exec_profile,
    };
    use agy_auth_domain::{
        Profile, ProfileId, ProfileName, ProfileStatus, ProviderKind, StorageLocator,
    };
    use std::cell::RefCell;
    use std::ffi::OsString;
    use std::path::PathBuf;
    use time::OffsetDateTime;

    struct Client;

    impl DoctorClientProbe for Client {
        fn probe(&self) -> ClientDiagnostic {
            ClientDiagnostic {
                found: true,
                version: Some("1.1.2".to_owned()),
                error_code: None,
            }
        }
    }

    struct Registry;

    impl DoctorRegistryProbe for Registry {
        fn probe(&self) -> RegistryDiagnostic {
            RegistryDiagnostic {
                healthy: true,
                profile_count: Some(0),
                error_code: None,
                data_directory_state: "absent",
                owner_matches: None,
                permissions_secure: None,
                interrupted_transactions: 0,
            }
        }
    }

    #[test]
    fn report_is_explicitly_diagnostics_only() {
        let report = doctor(&Client, &Registry);
        assert_eq!(report.exit_code(), 0);
        assert!(!report.capabilities.profile_switching);
        assert!(!report.capabilities.auth_state_mutation);
        assert_eq!(report.client.version.as_deref(), Some("1.1.2"));
    }

    #[test]
    fn interrupted_transaction_uses_recovery_exit_code() {
        struct Interrupted;

        impl DoctorRegistryProbe for Interrupted {
            fn probe(&self) -> RegistryDiagnostic {
                RegistryDiagnostic {
                    healthy: false,
                    profile_count: None,
                    error_code: Some("transaction_recovery_required".to_owned()),
                    data_directory_state: "secure",
                    owner_matches: Some(true),
                    permissions_secure: Some(true),
                    interrupted_transactions: 1,
                }
            }
        }

        assert_eq!(doctor(&Client, &Interrupted).exit_code(), 10);
    }

    fn profile(status: ProfileStatus) -> Profile {
        let now = OffsetDateTime::UNIX_EPOCH;
        Profile {
            id: ProfileId::new(),
            name: ProfileName::parse("Work").expect("valid name"),
            provider: ProviderKind::AntigravityCli,
            storage: StorageLocator::isolated_home("synthetic-work-home").expect("valid locator"),
            account_hint: None,
            created_at: now,
            updated_at: now,
            client_version_at_capture: None,
            schema_fingerprint: None,
            status,
        }
    }

    #[derive(Default)]
    struct FakeCatalog {
        events: RefCell<Vec<String>>,
    }

    impl ProfileCatalogPort for FakeCatalog {
        fn reserve(&self, profile: &Profile) -> Result<(), ProfileWorkflowError> {
            self.events
                .borrow_mut()
                .push(format!("reserve:{}", profile.id));
            Ok(())
        }

        fn mark_ready(
            &self,
            profile_id: ProfileId,
            client_version: &str,
        ) -> Result<(), ProfileWorkflowError> {
            self.events
                .borrow_mut()
                .push(format!("ready:{profile_id}:{client_version}"));
            Ok(())
        }
    }

    struct FakeHomes {
        events: RefCell<Vec<String>>,
    }

    impl ProfileHomePort for FakeHomes {
        fn prepare(
            &self,
            profile_id: ProfileId,
        ) -> Result<ManagedProfileEnvironment, ProfileWorkflowError> {
            self.events
                .borrow_mut()
                .push(format!("prepare:{profile_id}"));
            Ok(ManagedProfileEnvironment {
                home: PathBuf::from("/synthetic/home"),
                runtime_directory: PathBuf::from("/synthetic/runtime"),
            })
        }
    }

    #[derive(Default)]
    struct FakeClient {
        login_homes: RefCell<Vec<PathBuf>>,
        exec_arguments: RefCell<Vec<Vec<OsString>>>,
    }

    impl ProfileClientPort for FakeClient {
        fn login(
            &self,
            environment: &ManagedProfileEnvironment,
        ) -> Result<String, ProfileWorkflowError> {
            self.login_homes.borrow_mut().push(environment.home.clone());
            Ok("1.1.2".to_owned())
        }

        fn execute(
            &self,
            _environment: &ManagedProfileEnvironment,
            arguments: &[OsString],
        ) -> Result<i32, ProfileWorkflowError> {
            self.exec_arguments.borrow_mut().push(arguments.to_vec());
            Ok(23)
        }
    }

    struct FailingLoginClient;

    impl ProfileClientPort for FailingLoginClient {
        fn login(
            &self,
            _environment: &ManagedProfileEnvironment,
        ) -> Result<String, ProfileWorkflowError> {
            Err(ProfileWorkflowError::ClientLoginFailed)
        }

        fn execute(
            &self,
            _environment: &ManagedProfileEnvironment,
            _arguments: &[OsString],
        ) -> Result<i32, ProfileWorkflowError> {
            unreachable!("execution is not part of this test")
        }
    }

    #[test]
    fn add_reserves_before_login_and_commits_observed_version() {
        let profile = profile(ProfileStatus::Pending);
        let catalog = FakeCatalog::default();
        let homes = FakeHomes {
            events: RefCell::new(Vec::new()),
        };
        let client = FakeClient::default();

        add_profile(&profile, &catalog, &homes, &client).expect("add profile");

        assert_eq!(catalog.events.borrow().len(), 2);
        assert_eq!(homes.events.borrow().len(), 1);
        assert_eq!(
            client.login_homes.borrow().as_slice(),
            [PathBuf::from("/synthetic/home")]
        );
        assert!(catalog.events.borrow()[0].starts_with("reserve:"));
        assert!(catalog.events.borrow()[1].ends_with(":1.1.2"));
    }

    #[test]
    fn exec_requires_ready_profile_and_preserves_literal_argv() {
        let homes = FakeHomes {
            events: RefCell::new(Vec::new()),
        };
        let client = FakeClient::default();
        let arguments = [
            OsString::from("literal; not shell"),
            OsString::from("--flag"),
        ];

        let exit = exec_profile(&profile(ProfileStatus::Ready), &arguments, &homes, &client)
            .expect("execute ready profile");
        assert_eq!(exit, 23);
        assert_eq!(
            client.exec_arguments.borrow().as_slice(),
            [arguments.to_vec()]
        );

        let error = exec_profile(
            &profile(ProfileStatus::Pending),
            &arguments,
            &homes,
            &client,
        )
        .expect_err("pending profile must not execute");
        assert_eq!(error, ProfileWorkflowError::ProfileNotReady);
    }

    #[test]
    fn failed_login_leaves_pending_reservation_without_ready_commit() {
        let profile = profile(ProfileStatus::Pending);
        let catalog = FakeCatalog::default();
        let homes = FakeHomes {
            events: RefCell::new(Vec::new()),
        };

        let error = add_profile(&profile, &catalog, &homes, &FailingLoginClient)
            .expect_err("login must fail");

        assert_eq!(error, ProfileWorkflowError::ClientLoginFailed);
        assert_eq!(catalog.events.borrow().len(), 1);
        assert!(catalog.events.borrow()[0].starts_with("reserve:"));
        assert_eq!(homes.events.borrow().len(), 1);
    }
}
