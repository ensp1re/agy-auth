//! Storage adapters. This is the only crate permitted to persist opaque secrets.

mod credential_files;
mod profiles;
mod registry;

pub use credential_files::{CredentialFileError, OpaqueCredentialBytes, ProfileCredentialFiles};
pub use profiles::{ManagedProfileHomeError, ManagedProfileHomes};
pub use registry::{RegistryCatalog, RegistryDoctorProbe, RegistryFile, RegistryStoreError};
