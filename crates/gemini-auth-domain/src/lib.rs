//! Domain types and invariants for `gemini-auth`.

use std::fmt;
use time::OffsetDateTime;
use unicode_normalization::UnicodeNormalization;
use uuid::Uuid;

/// Immutable identifier for a profile.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct ProfileId(Uuid);

impl ProfileId {
    /// Generate a random profile identifier.
    #[must_use]
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// Parse a canonical UUID profile identifier.
    ///
    /// # Errors
    ///
    /// Returns [`DomainError::InvalidProfileId`] when `value` is not a UUID.
    pub fn parse(value: &str) -> Result<Self, DomainError> {
        Uuid::parse_str(value)
            .map(Self)
            .map_err(|_| DomainError::InvalidProfileId)
    }
}

impl Default for ProfileId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for ProfileId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

/// Validated user-facing profile name and its comparison key.
#[derive(Clone, Debug, Eq)]
pub struct ProfileName {
    display: String,
    comparison_key: String,
}

impl ProfileName {
    /// Validate and normalize a profile name.
    ///
    /// # Errors
    ///
    /// Returns [`DomainError::InvalidProfileName`] for empty, oversized, control, or path-like names.
    pub fn parse(value: &str) -> Result<Self, DomainError> {
        let display = value.trim();
        if display.is_empty() || display.chars().count() > 64 {
            return Err(DomainError::InvalidProfileName);
        }
        if display
            .chars()
            .any(|character| character.is_control() || matches!(character, '/' | '\\'))
        {
            return Err(DomainError::InvalidProfileName);
        }
        let comparison_key = display
            .nfkc()
            .flat_map(char::to_lowercase)
            .collect::<String>();
        Ok(Self {
            display: display.to_owned(),
            comparison_key,
        })
    }

    /// Return the user-facing spelling.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.display
    }

    /// Return the normalized key used for uniqueness checks.
    #[must_use]
    pub fn comparison_key(&self) -> &str {
        &self.comparison_key
    }
}

impl PartialEq for ProfileName {
    fn eq(&self, other: &Self) -> bool {
        self.comparison_key == other.comparison_key
    }
}

/// Supported official-client provider families.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProviderKind {
    /// Google Gemini CLI using an isolated client home.
    GeminiCli,
    /// Antigravity CLI using a capability-gated file mode.
    AntigravityCli,
}

/// Non-secret reference to provider-managed profile storage.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum StorageLocator {
    /// Identifier of a managed isolated client home.
    IsolatedHome(String),
    /// Identifier of an opaque secret-store entry.
    SecretStoreEntry(String),
}

impl StorageLocator {
    /// Build a validated isolated-home locator.
    ///
    /// # Errors
    ///
    /// Returns [`DomainError::InvalidStorageLocator`] when the opaque identifier is unsafe.
    pub fn isolated_home(value: &str) -> Result<Self, DomainError> {
        validate_locator(value).map(|()| Self::IsolatedHome(value.to_owned()))
    }

    /// Build a validated secret-store locator.
    ///
    /// # Errors
    ///
    /// Returns [`DomainError::InvalidStorageLocator`] when the opaque identifier is unsafe.
    pub fn secret_store_entry(value: &str) -> Result<Self, DomainError> {
        validate_locator(value).map(|()| Self::SecretStoreEntry(value.to_owned()))
    }

    /// Return the opaque, non-path locator value.
    #[must_use]
    pub fn value(&self) -> &str {
        match self {
            Self::IsolatedHome(value) | Self::SecretStoreEntry(value) => value,
        }
    }
}

fn validate_locator(value: &str) -> Result<(), DomainError> {
    if value.is_empty()
        || value.len() > 128
        || value
            .chars()
            .any(|character| character.is_control() || matches!(character, '/' | '\\'))
    {
        return Err(DomainError::InvalidStorageLocator);
    }
    Ok(())
}

/// Lifecycle state of a profile registry entry.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProfileStatus {
    /// Profile is usable by its provider adapter.
    Ready,
    /// Profile was reserved but login did not complete.
    Pending,
    /// Profile requires user attention before use.
    Unavailable,
}

/// Non-secret profile metadata.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Profile {
    /// Immutable profile identifier.
    pub id: ProfileId,
    /// Validated user-facing name.
    pub name: ProfileName,
    /// Owning provider.
    pub provider: ProviderKind,
    /// Opaque reference to managed storage, never credential material.
    pub storage: StorageLocator,
    /// Optional pre-masked account hint.
    pub account_hint: Option<String>,
    /// Creation time.
    pub created_at: OffsetDateTime,
    /// Last metadata update time.
    pub updated_at: OffsetDateTime,
    /// Official client version observed during capture.
    pub client_version_at_capture: Option<String>,
    /// Non-secret schema fingerprint.
    pub schema_fingerprint: Option<String>,
    /// Current profile state.
    pub status: ProfileStatus,
}

impl Profile {
    /// Validate fields whose safety depends on their combination.
    ///
    /// # Errors
    ///
    /// Returns a domain error for unmasked account hints or inconsistent timestamps.
    pub fn validate(&self) -> Result<(), DomainError> {
        if self.updated_at < self.created_at {
            return Err(DomainError::InvalidTimestampOrder);
        }
        if let Some(hint) = &self.account_hint {
            if hint.len() > 128 || hint.chars().any(char::is_control) || !hint.contains('*') {
                return Err(DomainError::UnmaskedAccountHint);
            }
        }
        Ok(())
    }
}

/// Validated collection of profile metadata.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Registry {
    profiles: Vec<Profile>,
}

impl Registry {
    /// Build a registry and enforce all collection invariants.
    ///
    /// # Errors
    ///
    /// Returns a domain error when a profile is invalid or IDs or normalized names collide.
    pub fn new(profiles: Vec<Profile>) -> Result<Self, DomainError> {
        let registry = Self { profiles };
        registry.validate()?;
        Ok(registry)
    }

    /// Return profiles in stable registry order.
    #[must_use]
    pub fn profiles(&self) -> &[Profile] {
        &self.profiles
    }

    /// Add a profile after enforcing identity and name uniqueness.
    ///
    /// # Errors
    ///
    /// Returns a domain error when the profile is invalid or its ID or normalized name collides.
    pub fn add(&mut self, profile: Profile) -> Result<(), DomainError> {
        profile.validate()?;
        if self
            .profiles
            .iter()
            .any(|existing| existing.id == profile.id)
        {
            return Err(DomainError::DuplicateProfileId);
        }
        if self
            .profiles
            .iter()
            .any(|existing| existing.name == profile.name)
        {
            return Err(DomainError::DuplicateProfileName);
        }
        self.profiles.push(profile);
        Ok(())
    }

    /// Recheck every registry invariant.
    ///
    /// # Errors
    ///
    /// Returns a domain error when a profile is invalid or IDs or normalized names collide.
    pub fn validate(&self) -> Result<(), DomainError> {
        let mut ids = std::collections::HashSet::new();
        let mut names = std::collections::HashSet::new();
        for profile in &self.profiles {
            profile.validate()?;
            if !ids.insert(profile.id) {
                return Err(DomainError::DuplicateProfileId);
            }
            if !names.insert(profile.name.comparison_key()) {
                return Err(DomainError::DuplicateProfileName);
            }
        }
        Ok(())
    }
}

/// Domain invariant failure without secret-bearing values.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DomainError {
    /// Profile identifier is not a UUID.
    InvalidProfileId,
    /// Profile name is empty, too long, or path-like.
    InvalidProfileName,
    /// Storage locator is empty, too long, or path-like.
    InvalidStorageLocator,
    /// Account hint was not demonstrably masked.
    UnmaskedAccountHint,
    /// Update time precedes creation time.
    InvalidTimestampOrder,
    /// Two profiles share an immutable identifier.
    DuplicateProfileId,
    /// Two profile names compare equal after normalization.
    DuplicateProfileName,
}

impl fmt::Display for DomainError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::InvalidProfileId => "invalid profile identifier",
            Self::InvalidProfileName => "invalid profile name",
            Self::InvalidStorageLocator => "invalid storage locator",
            Self::UnmaskedAccountHint => "account hint is not masked",
            Self::InvalidTimestampOrder => "profile timestamps are inconsistent",
            Self::DuplicateProfileId => "duplicate profile identifier",
            Self::DuplicateProfileName => "duplicate normalized profile name",
        };
        formatter.write_str(message)
    }
}

impl std::error::Error for DomainError {}

/// Stable application error categories mapped to the documented CLI exit codes.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum ErrorCode {
    /// Invalid CLI usage.
    Usage = 2,
    /// Requested profile is absent or conflicts with another profile.
    ProfileConflict = 3,
    /// The official provider client is unavailable.
    ProviderUnavailable = 4,
    /// The installed provider uses an unsupported storage mode.
    UnsupportedProviderMode = 5,
    /// Official authentication was cancelled or failed.
    AuthenticationFailed = 6,
    /// The configured secret store is unavailable or locked.
    SecretStoreUnavailable = 7,
    /// State or filesystem permissions are unsafe.
    UnsafeState = 8,
    /// Another client or operation conflicts with this operation.
    ConcurrencyConflict = 9,
    /// An interrupted transaction requires recovery.
    RecoveryRequired = 10,
    /// A program invariant failed.
    InternalInvariant = 11,
}

impl ErrorCode {
    /// Return the stable process exit code.
    #[must_use]
    pub const fn exit_code(self) -> u8 {
        self as u8
    }
}

#[cfg(test)]
mod tests {
    use super::{ErrorCode, ProfileName};

    #[test]
    fn error_codes_match_cli_contract() {
        assert_eq!(ErrorCode::Usage.exit_code(), 2);
        assert_eq!(ErrorCode::InternalInvariant.exit_code(), 11);
    }

    #[test]
    fn profile_names_compare_by_normalized_case() {
        let composed = ProfileName::parse("Work").expect("valid name");
        let lowercase = ProfileName::parse("work").expect("valid name");
        assert_eq!(composed, lowercase);
    }

    #[test]
    fn profile_names_reject_paths_and_controls() {
        assert!(ProfileName::parse("../work").is_err());
        assert!(ProfileName::parse("work\nname").is_err());
    }
}
