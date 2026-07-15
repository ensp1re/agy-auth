//! Capability-gated Gemini CLI adapter boundary.

use gemini_auth_domain::ProviderKind;

/// Identify the provider implemented by this adapter.
#[must_use]
pub const fn provider_kind() -> ProviderKind {
    ProviderKind::GeminiCli
}
