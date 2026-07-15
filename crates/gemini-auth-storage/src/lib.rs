//! Storage adapters. This is the only crate permitted to persist opaque secrets.

use gemini_auth_domain::ProviderKind;

/// Marker for the provider-scoped storage boundary.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct StorageScope(pub ProviderKind);
