//! Capability-gated Antigravity CLI adapter boundary.

use agy_auth_domain::ProviderKind;

/// Identify the provider implemented by this adapter.
#[must_use]
pub const fn provider_kind() -> ProviderKind {
    ProviderKind::AntigravityCli
}
