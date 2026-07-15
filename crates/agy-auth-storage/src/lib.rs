//! Storage adapters. This is the only crate permitted to persist opaque secrets.

mod registry;

pub use registry::{RegistryDoctorProbe, RegistryFile, RegistryStoreError};
