//! Dependency-free domain types and invariants for `gemini-auth`.

/// Supported official-client provider families.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProviderKind {
    /// Google Gemini CLI using an isolated client home.
    GeminiCli,
    /// Antigravity CLI using a capability-gated file mode.
    AntigravityCli,
}

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
    use super::ErrorCode;

    #[test]
    fn error_codes_match_cli_contract() {
        assert_eq!(ErrorCode::Usage.exit_code(), 2);
        assert_eq!(ErrorCode::InternalInvariant.exit_code(), 11);
    }
}
