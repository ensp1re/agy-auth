//! Experimental orchestration for versioned profile credential materialization.

use std::path::{Path, PathBuf};

/// Opaque secret bytes that cannot be formatted or serialized accidentally.
///
/// This type deliberately does not implement `Clone`, `Debug`, `Display`, `Serialize`, or equality.
pub struct OpaqueSecretBytes {
    value: Vec<u8>,
}

impl OpaqueSecretBytes {
    /// Wrap non-empty bounded secret bytes.
    ///
    /// # Errors
    ///
    /// Returns a non-secret error when the value is empty, oversized, or the bound is zero.
    pub fn new(value: Vec<u8>, maximum_bytes: usize) -> Result<Self, CredentialWorkflowError> {
        if maximum_bytes == 0 || value.is_empty() || value.len() > maximum_bytes {
            return Err(CredentialWorkflowError::InvalidSecretSize);
        }
        Ok(Self { value })
    }

    /// Borrow bytes for an approved provider or storage adapter.
    #[must_use]
    pub fn expose_secret(&self) -> &[u8] {
        &self.value
    }

    /// Consume the wrapper and return bytes for transfer between approved adapters.
    #[must_use]
    pub fn into_secret_bytes(self) -> Vec<u8> {
        self.value
    }
}

/// Provider-produced destination and opaque official-client envelope.
pub struct CredentialMaterializationPlan {
    /// Safe home-relative destination owned by the provider contract.
    pub relative_path: PathBuf,
    /// Opaque bytes to persist through the credential-file port.
    pub envelope: OpaqueSecretBytes,
}

/// Port for versioned provider envelope construction and extraction.
pub trait CredentialEnvelopePort {
    /// Return the safe home-relative destination for a supported client version.
    ///
    /// # Errors
    ///
    /// Returns an unsupported-version error when no contract is approved.
    fn relative_path(&self, client_version: &str) -> Result<PathBuf, CredentialWorkflowError>;

    /// Produce the official-client file destination and minimal credential envelope.
    ///
    /// # Errors
    ///
    /// Returns a stable non-secret workflow error for unsupported versions or invalid credentials.
    fn build_plan(
        &self,
        client_version: &str,
        refresh_credential: &OpaqueSecretBytes,
    ) -> Result<CredentialMaterializationPlan, CredentialWorkflowError>;

    /// Extract the current refresh credential from an official-client rewrite.
    ///
    /// # Errors
    ///
    /// Returns a stable non-secret workflow error for unsupported or malformed envelopes.
    fn extract_refresh(
        &self,
        client_version: &str,
        rewritten_envelope: OpaqueSecretBytes,
    ) -> Result<OpaqueSecretBytes, CredentialWorkflowError>;
}

/// Port for protected profile-home credential files.
pub trait CredentialFilePort {
    /// Atomically materialize opaque bytes at a safe profile-home-relative destination.
    ///
    /// # Errors
    ///
    /// Returns a stable non-secret workflow error when storage safety cannot be guaranteed.
    fn materialize(
        &self,
        relative_path: &Path,
        envelope: &OpaqueSecretBytes,
    ) -> Result<(), CredentialWorkflowError>;

    /// Reread bounded opaque bytes after the official client may have refreshed them.
    ///
    /// # Errors
    ///
    /// Returns a stable non-secret workflow error for unsafe or unavailable storage.
    fn read(
        &self,
        relative_path: &Path,
        maximum_bytes: usize,
    ) -> Result<OpaqueSecretBytes, CredentialWorkflowError>;
}

/// Build a versioned provider envelope and atomically persist it in one protected profile home.
///
/// # Errors
///
/// Returns a stable provider or storage workflow error.
pub fn materialize_profile_credential(
    client_version: &str,
    refresh_credential: &OpaqueSecretBytes,
    envelope_port: &impl CredentialEnvelopePort,
    file_port: &impl CredentialFilePort,
) -> Result<(), CredentialWorkflowError> {
    let plan = envelope_port.build_plan(client_version, refresh_credential)?;
    file_port.materialize(&plan.relative_path, &plan.envelope)
}

/// Reread the official-client envelope and extract its current refresh credential.
///
/// This captures refresh-token rotation without exposing the access token to the application.
///
/// # Errors
///
/// Returns a stable provider or storage workflow error.
pub fn capture_refreshed_profile_credential(
    client_version: &str,
    envelope_port: &impl CredentialEnvelopePort,
    file_port: &impl CredentialFilePort,
    maximum_envelope_bytes: usize,
) -> Result<OpaqueSecretBytes, CredentialWorkflowError> {
    let destination = envelope_port.relative_path(client_version)?;
    let rewritten = file_port.read(&destination, maximum_envelope_bytes)?;
    envelope_port.extract_refresh(client_version, rewritten)
}

/// Experimental credential workflow failure without secret-bearing context.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CredentialWorkflowError {
    /// Secret bytes are empty, oversized, or use an invalid bound.
    InvalidSecretSize,
    /// The official client version has no approved provider contract.
    UnsupportedClientVersion,
    /// Provider envelope construction or parsing failed.
    InvalidCredentialEnvelope,
    /// Protected profile storage could not safely complete the operation.
    CredentialStorageFailed,
}

impl std::fmt::Display for CredentialWorkflowError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let message = match self {
            Self::InvalidSecretSize => "invalid secret size",
            Self::UnsupportedClientVersion => "unsupported client version",
            Self::InvalidCredentialEnvelope => "invalid credential envelope",
            Self::CredentialStorageFailed => "credential storage operation failed",
        };
        formatter.write_str(message)
    }
}

impl std::error::Error for CredentialWorkflowError {}

#[cfg(test)]
mod tests {
    use super::{
        CredentialEnvelopePort, CredentialFilePort, CredentialMaterializationPlan,
        CredentialWorkflowError, OpaqueSecretBytes, capture_refreshed_profile_credential,
        materialize_profile_credential,
    };
    use std::cell::RefCell;
    use std::path::{Path, PathBuf};

    const DESTINATION: &str = ".synthetic/client/credential";

    struct FakeEnvelope;

    impl CredentialEnvelopePort for FakeEnvelope {
        fn relative_path(&self, client_version: &str) -> Result<PathBuf, CredentialWorkflowError> {
            if client_version != "TEST_CLIENT_1" {
                return Err(CredentialWorkflowError::UnsupportedClientVersion);
            }
            Ok(PathBuf::from(DESTINATION))
        }

        fn build_plan(
            &self,
            client_version: &str,
            refresh_credential: &OpaqueSecretBytes,
        ) -> Result<CredentialMaterializationPlan, CredentialWorkflowError> {
            if client_version != "TEST_CLIENT_1" {
                return Err(CredentialWorkflowError::UnsupportedClientVersion);
            }
            let mut envelope = b"synthetic-envelope:".to_vec();
            envelope.extend_from_slice(refresh_credential.expose_secret());
            Ok(CredentialMaterializationPlan {
                relative_path: self.relative_path(client_version)?,
                envelope: OpaqueSecretBytes::new(envelope, 4096)?,
            })
        }

        fn extract_refresh(
            &self,
            client_version: &str,
            rewritten_envelope: OpaqueSecretBytes,
        ) -> Result<OpaqueSecretBytes, CredentialWorkflowError> {
            if client_version != "TEST_CLIENT_1" {
                return Err(CredentialWorkflowError::UnsupportedClientVersion);
            }
            let bytes = rewritten_envelope.into_secret_bytes();
            let refresh = bytes
                .strip_prefix(b"synthetic-rewritten:")
                .ok_or(CredentialWorkflowError::InvalidCredentialEnvelope)?;
            OpaqueSecretBytes::new(refresh.to_vec(), 4096)
        }
    }

    #[derive(Default)]
    struct FakeFiles {
        events: RefCell<Vec<String>>,
        rewritten: RefCell<Option<Vec<u8>>>,
    }

    impl CredentialFilePort for FakeFiles {
        fn materialize(
            &self,
            relative_path: &Path,
            envelope: &OpaqueSecretBytes,
        ) -> Result<(), CredentialWorkflowError> {
            self.events.borrow_mut().push(format!(
                "write:{}:{}",
                relative_path.display(),
                envelope.expose_secret().len()
            ));
            Ok(())
        }

        fn read(
            &self,
            relative_path: &Path,
            maximum_bytes: usize,
        ) -> Result<OpaqueSecretBytes, CredentialWorkflowError> {
            self.events
                .borrow_mut()
                .push(format!("read:{}:{maximum_bytes}", relative_path.display()));
            let value = self
                .rewritten
                .borrow_mut()
                .take()
                .ok_or(CredentialWorkflowError::CredentialStorageFailed)?;
            OpaqueSecretBytes::new(value, maximum_bytes)
        }
    }

    #[test]
    fn materializes_provider_plan_through_storage_port() {
        let files = FakeFiles::default();
        let refresh = OpaqueSecretBytes::new(b"synthetic-refresh".to_vec(), 4096).expect("refresh");

        materialize_profile_credential("TEST_CLIENT_1", &refresh, &FakeEnvelope, &files)
            .expect("materialize");

        assert_eq!(
            files.events.into_inner(),
            vec!["write:.synthetic/client/credential:36"]
        );
    }

    #[test]
    fn captures_rotated_refresh_without_returning_access_state() {
        let files = FakeFiles {
            rewritten: RefCell::new(Some(b"synthetic-rewritten:synthetic-rotated".to_vec())),
            ..FakeFiles::default()
        };

        let refresh =
            capture_refreshed_profile_credential("TEST_CLIENT_1", &FakeEnvelope, &files, 4096)
                .expect("capture refresh");

        assert_eq!(refresh.into_secret_bytes(), b"synthetic-rotated");
        assert_eq!(
            files.events.into_inner(),
            vec!["read:.synthetic/client/credential:4096"]
        );
    }

    #[test]
    fn unsupported_version_stops_before_storage() {
        let files = FakeFiles::default();
        let refresh = OpaqueSecretBytes::new(b"synthetic-refresh".to_vec(), 4096).expect("refresh");

        assert!(matches!(
            materialize_profile_credential("TEST_CLIENT_2", &refresh, &FakeEnvelope, &files),
            Err(CredentialWorkflowError::UnsupportedClientVersion)
        ));
        assert!(files.events.into_inner().is_empty());
    }
}
