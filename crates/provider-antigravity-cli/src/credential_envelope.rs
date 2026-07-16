//! Version-scoped Antigravity CLI consumer credential envelope.

use serde::{Deserialize, Serialize};
use thiserror::Error;
use time::OffsetDateTime;

/// Home-relative path used by `agy 1.1.2` Linux SSH file-fallback storage.
pub const ANTIGRAVITY_TOKEN_RELATIVE_PATH: &str = ".gemini/antigravity-cli/antigravity-oauth-token";

const SUPPORTED_CLIENT_VERSION: &str = "1.1.2";
const MAX_ENVELOPE_BYTES: usize = 16 * 1024;
const MAX_REFRESH_TOKEN_BYTES: usize = 8 * 1024;
const PLACEHOLDER_ACCESS_TOKEN: &str = "agy-auth-expired-placeholder";
const EXPIRED_AT: &str = "2000-01-01T00:00:00Z";

/// A consumer refresh credential that cannot be formatted or serialized accidentally.
///
/// This type deliberately does not implement `Clone`, `Debug`, `Display`, `Serialize`, or equality.
pub struct ConsumerRefreshCredential {
    value: String,
}

impl ConsumerRefreshCredential {
    /// Validate an opaque UTF-8 refresh credential.
    ///
    /// # Errors
    ///
    /// Returns a non-secret error when the credential is empty, oversized, invalid UTF-8, or
    /// contains control characters.
    pub fn new(value: Vec<u8>) -> Result<Self, CredentialEnvelopeError> {
        if value.is_empty() || value.len() > MAX_REFRESH_TOKEN_BYTES {
            return Err(CredentialEnvelopeError::InvalidRefreshCredential);
        }
        let value = String::from_utf8(value)
            .map_err(|_| CredentialEnvelopeError::InvalidRefreshCredential)?;
        if value.chars().any(char::is_control) {
            return Err(CredentialEnvelopeError::InvalidRefreshCredential);
        }
        Ok(Self { value })
    }

    /// Consume the wrapper and return bytes for transfer to an approved secret store.
    #[must_use]
    pub fn into_secret_bytes(self) -> Vec<u8> {
        self.value.into_bytes()
    }

    fn expose(&self) -> &str {
        &self.value
    }
}

/// Serialize the minimal consumer token envelope accepted by `agy 1.1.2`.
///
/// The access token is an intentionally expired placeholder. The official client replaces it by
/// refreshing through the supplied credential.
///
/// # Errors
///
/// Returns a stable error for unsupported client versions or serialization failure.
pub fn build_consumer_token_envelope(
    client_version: &str,
    refresh_credential: &ConsumerRefreshCredential,
) -> Result<Vec<u8>, CredentialEnvelopeError> {
    require_supported_version(client_version)?;
    let document = EnvelopeDocument {
        auth_method: "consumer",
        token: TokenDocument {
            access_token: PLACEHOLDER_ACCESS_TOKEN,
            token_type: "Bearer",
            refresh_token: refresh_credential.expose(),
            expiry: EXPIRED_AT,
        },
    };
    serde_json::to_vec(&document).map_err(|_| CredentialEnvelopeError::SerializationFailed)
}

/// Extract the refresh credential from an envelope rewritten by the official client.
///
/// This parser is bounded and version-gated. It validates the surrounding fields so unrelated JSON
/// cannot be mistaken for an Antigravity consumer credential.
///
/// # Errors
///
/// Returns a stable non-secret error for unsupported versions, oversized or malformed JSON,
/// incompatible authentication fields, invalid expiry, or invalid refresh credential.
pub fn extract_consumer_refresh_credential(
    client_version: &str,
    envelope: &[u8],
) -> Result<ConsumerRefreshCredential, CredentialEnvelopeError> {
    require_supported_version(client_version)?;
    if envelope.is_empty() || envelope.len() > MAX_ENVELOPE_BYTES {
        return Err(CredentialEnvelopeError::InvalidEnvelopeSize);
    }
    let document: OwnedEnvelopeDocument =
        serde_json::from_slice(envelope).map_err(|_| CredentialEnvelopeError::MalformedEnvelope)?;
    if document.auth_method != "consumer" || document.token.token_type != "Bearer" {
        return Err(CredentialEnvelopeError::UnsupportedEnvelope);
    }
    if document.token.access_token.is_empty() {
        return Err(CredentialEnvelopeError::MalformedEnvelope);
    }
    OffsetDateTime::parse(
        &document.token.expiry,
        &time::format_description::well_known::Rfc3339,
    )
    .map_err(|_| CredentialEnvelopeError::InvalidExpiry)?;
    ConsumerRefreshCredential::new(document.token.refresh_token.into_bytes())
}

fn require_supported_version(client_version: &str) -> Result<(), CredentialEnvelopeError> {
    if client_version == SUPPORTED_CLIENT_VERSION {
        Ok(())
    } else {
        Err(CredentialEnvelopeError::UnsupportedClientVersion)
    }
}

#[derive(Serialize)]
struct EnvelopeDocument<'a> {
    auth_method: &'static str,
    token: TokenDocument<'a>,
}

#[derive(Serialize)]
struct TokenDocument<'a> {
    access_token: &'static str,
    token_type: &'static str,
    refresh_token: &'a str,
    expiry: &'static str,
}

#[derive(Deserialize)]
struct OwnedEnvelopeDocument {
    auth_method: String,
    token: OwnedTokenDocument,
}

#[derive(Deserialize)]
struct OwnedTokenDocument {
    access_token: String,
    token_type: String,
    refresh_token: String,
    expiry: String,
}

/// Versioned credential-envelope failure without secret-bearing context.
#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub enum CredentialEnvelopeError {
    /// The installed client version has no approved envelope contract.
    #[error("unsupported Antigravity client version")]
    UnsupportedClientVersion,
    /// The refresh credential violates bounded opaque-value requirements.
    #[error("invalid consumer refresh credential")]
    InvalidRefreshCredential,
    /// The envelope is empty or exceeds the parser limit.
    #[error("invalid credential envelope size")]
    InvalidEnvelopeSize,
    /// The envelope is not valid expected JSON.
    #[error("malformed credential envelope")]
    MalformedEnvelope,
    /// The authentication method or token type is unsupported.
    #[error("unsupported credential envelope")]
    UnsupportedEnvelope,
    /// The expiry is not an RFC 3339 timestamp.
    #[error("invalid credential envelope expiry")]
    InvalidExpiry,
    /// Serialization failed without exposing source values.
    #[error("credential envelope serialization failed")]
    SerializationFailed,
}

#[cfg(test)]
mod tests {
    use super::{
        ANTIGRAVITY_TOKEN_RELATIVE_PATH, ConsumerRefreshCredential, CredentialEnvelopeError,
        MAX_ENVELOPE_BYTES, build_consumer_token_envelope, extract_consumer_refresh_credential,
    };
    use serde_json::Value;

    const SYNTHETIC_REFRESH: &[u8] = b"synthetic-refresh-credential-for-tests";

    #[test]
    fn builds_exact_versioned_minimal_envelope() {
        let credential =
            ConsumerRefreshCredential::new(SYNTHETIC_REFRESH.to_vec()).expect("credential");
        let encoded =
            build_consumer_token_envelope("1.1.2", &credential).expect("supported envelope");
        let document: Value = serde_json::from_slice(&encoded).expect("JSON");

        assert_eq!(
            ANTIGRAVITY_TOKEN_RELATIVE_PATH,
            ".gemini/antigravity-cli/antigravity-oauth-token"
        );
        assert_eq!(document["auth_method"], "consumer");
        assert_eq!(
            document["token"]["access_token"],
            "agy-auth-expired-placeholder"
        );
        assert_eq!(document["token"]["token_type"], "Bearer");
        assert_eq!(
            document["token"]["refresh_token"],
            "synthetic-refresh-credential-for-tests"
        );
        assert_eq!(document["token"]["expiry"], "2000-01-01T00:00:00Z");
    }

    #[test]
    fn extracts_rotated_refresh_credential_from_official_rewrite() {
        let rewritten = br#"{
            "auth_method":"consumer",
            "token":{
                "access_token":"synthetic-current-access",
                "token_type":"Bearer",
                "refresh_token":"synthetic-rotated-refresh",
                "expiry":"2030-01-02T03:04:05Z"
            }
        }"#;

        let credential =
            extract_consumer_refresh_credential("1.1.2", rewritten).expect("extract credential");

        assert_eq!(credential.into_secret_bytes(), b"synthetic-rotated-refresh");
    }

    #[test]
    fn rejects_unsupported_versions_and_incompatible_documents() {
        let credential =
            ConsumerRefreshCredential::new(SYNTHETIC_REFRESH.to_vec()).expect("credential");
        assert_eq!(
            build_consumer_token_envelope("1.1.3", &credential),
            Err(CredentialEnvelopeError::UnsupportedClientVersion)
        );

        let wrong_method = br#"{
            "auth_method":"enterprise",
            "token":{
                "access_token":"synthetic-access",
                "token_type":"Bearer",
                "refresh_token":"synthetic-refresh",
                "expiry":"2030-01-02T03:04:05Z"
            }
        }"#;
        assert!(matches!(
            extract_consumer_refresh_credential("1.1.2", wrong_method),
            Err(CredentialEnvelopeError::UnsupportedEnvelope)
        ));
    }

    #[test]
    fn rejects_malformed_oversized_and_secret_like_control_input() {
        assert!(matches!(
            ConsumerRefreshCredential::new(b"synthetic\ncredential".to_vec()),
            Err(CredentialEnvelopeError::InvalidRefreshCredential)
        ));
        assert!(matches!(
            extract_consumer_refresh_credential("1.1.2", b"not-json"),
            Err(CredentialEnvelopeError::MalformedEnvelope)
        ));
        let oversized = vec![b'x'; MAX_ENVELOPE_BYTES + 1];
        assert!(matches!(
            extract_consumer_refresh_credential("1.1.2", &oversized),
            Err(CredentialEnvelopeError::InvalidEnvelopeSize)
        ));
    }
}
