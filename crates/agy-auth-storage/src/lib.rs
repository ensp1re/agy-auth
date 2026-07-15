//! Storage adapters. This is the only crate permitted to persist opaque secrets.

mod profiles;
mod registry;

pub use profiles::{ManagedProfileHomeError, ManagedProfileHomes};
pub use registry::{RegistryCatalog, RegistryDoctorProbe, RegistryFile, RegistryStoreError};
