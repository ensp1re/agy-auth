//! Storage adapters. This is the only crate permitted to persist opaque secrets.

mod active_selection;
mod credential_files;
#[cfg(feature = "experimental-profile-credentials")]
mod import_transaction;
#[cfg(target_os = "macos")]
mod macos_keychain;
mod profiles;
mod registry;
#[cfg(feature = "experimental-profile-credentials")]
mod session_lock;

pub use active_selection::{ActiveProfileStore, ActiveProfileStoreError};
pub use credential_files::{
    CredentialFileError, OfficialCredentialSourceFiles, OpaqueCredentialBytes,
    ProfileCredentialFiles,
};
#[cfg(feature = "experimental-profile-credentials")]
pub use import_transaction::{
    ImportRecoveryReport, ImportTransaction, ImportTransactionError, ImportTransactionJournal,
    ImportTransactionStage,
};
#[cfg(target_os = "macos")]
pub use macos_keychain::{MacOsKeychainCredentialStore, MacOsKeychainError};
pub use profiles::{ManagedProfileHomeError, ManagedProfileHomes};
pub use registry::{RegistryCatalog, RegistryDoctorProbe, RegistryFile, RegistryStoreError};
#[cfg(feature = "experimental-profile-credentials")]
pub use session_lock::{ProfileSessionLease, ProfileSessionLock, ProfileSessionLockError};
