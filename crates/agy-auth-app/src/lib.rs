//! Application-service boundary for use-case orchestration.

use serde::Serialize;

pub use agy_auth_domain::{ErrorCode, ProviderKind};

/// Stable diagnostics schema version.
pub const DOCTOR_SCHEMA_VERSION: u32 = 1;

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
        ClientDiagnostic, DoctorClientProbe, DoctorRegistryProbe, RegistryDiagnostic, doctor,
    };

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
}
