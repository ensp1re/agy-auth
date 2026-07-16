//! Storage adapters. This is the only crate permitted to persist opaque secrets.

mod credential_files;
#[cfg(feature = "experimental-profile-credentials")]
mod import_transaction;
mod profiles;
mod registry;
#[cfg(feature = "experimental-profile-credentials")]
mod session_lock;

pub use credential_files::{CredentialFileError, OpaqueCredentialBytes, ProfileCredentialFiles};
#[cfg(feature = "experimental-profile-credentials")]
pub use import_transaction::{
    ImportRecoveryReport, ImportTransaction, ImportTransactionError, ImportTransactionJournal,
    ImportTransactionStage,
};
pub use profiles::{ManagedProfileHomeError, ManagedProfileHomes};
pub use registry::{RegistryCatalog, RegistryDoctorProbe, RegistryFile, RegistryStoreError};
#[cfg(feature = "experimental-profile-credentials")]
pub use session_lock::{ProfileSessionLease, ProfileSessionLock, ProfileSessionLockError};
