//! Apple Keychain adapter for the versioned Antigravity CLI generic-password contract.

use crate::OpaqueCredentialBytes;
use security_framework::passwords::{get_generic_password, set_generic_password};
use thiserror::Error;

const SERVICE: &str = "gemini";
const ACCOUNT: &str = "antigravity";

/// Bounded opaque access to the official Antigravity CLI Keychain item.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct MacOsKeychainCredentialStore;

impl MacOsKeychainCredentialStore {
    /// Read the current official-client credential without rendering or logging it.
    ///
    /// # Errors
    ///
    /// Returns a non-secret error when Keychain access fails or the value is empty or oversized.
    pub fn read(&self, maximum_bytes: usize) -> Result<OpaqueCredentialBytes, MacOsKeychainError> {
        if maximum_bytes == 0 {
            return Err(MacOsKeychainError::InvalidCredential);
        }
        let value =
            get_generic_password(SERVICE, ACCOUNT).map_err(|_| MacOsKeychainError::Unavailable)?;
        if value.is_empty() || value.len() > maximum_bytes {
            return Err(MacOsKeychainError::InvalidCredential);
        }
        OpaqueCredentialBytes::new(value, maximum_bytes)
            .map_err(|_| MacOsKeychainError::InvalidCredential)
    }

    /// Create or atomically update the official-client Keychain credential.
    ///
    /// # Errors
    ///
    /// Returns a non-secret error when Keychain rejects the replacement.
    pub fn materialize(
        &self,
        credential: &OpaqueCredentialBytes,
    ) -> Result<(), MacOsKeychainError> {
        set_generic_password(SERVICE, ACCOUNT, credential.expose())
            .map_err(|_| MacOsKeychainError::Unavailable)
    }
}

/// Apple Keychain operation failure without credential-bearing context.
#[derive(Debug, Error)]
pub enum MacOsKeychainError {
    /// The expected generic-password item could not be accessed or updated.
    #[error("Antigravity Keychain credential is unavailable")]
    Unavailable,
    /// The retrieved credential is empty or exceeds the approved bound.
    #[error("Antigravity Keychain credential is invalid")]
    InvalidCredential,
}
