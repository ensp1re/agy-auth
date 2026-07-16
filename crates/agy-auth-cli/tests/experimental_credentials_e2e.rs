#![cfg(all(feature = "experimental-profile-credentials", unix))]
#![doc = "Synthetic end-to-end coverage for the experimental credential workflow."]

use agy_auth_app::{
    OpaqueSecretBytes, capture_refreshed_profile_credential, materialize_profile_credential,
};
use agy_auth_storage::{OpaqueCredentialBytes, ProfileCredentialFiles};
use provider_antigravity_cli::{ANTIGRAVITY_TOKEN_RELATIVE_PATH, AntigravityCredentialEnvelope};
use serde_json::Value;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

fn fixture() -> PathBuf {
    let path = std::env::temp_dir().join(format!(
        "agy-auth-experimental-credential-e2e-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock after Unix epoch")
            .as_nanos()
    ));
    fs::create_dir(&path).expect("create fixture");
    fs::set_permissions(&path, fs::Permissions::from_mode(0o700)).expect("secure fixture");
    path
}

#[test]
fn provider_and_storage_round_trip_synthetic_refresh_credential() {
    let home = fixture();
    let files = ProfileCredentialFiles::new(&home).expect("protected profile files");
    let provider = AntigravityCredentialEnvelope;
    let refresh = OpaqueSecretBytes::new(b"synthetic-initial-refresh".to_vec(), 4096)
        .expect("synthetic refresh");

    materialize_profile_credential("1.1.2", &refresh, &provider, &files)
        .expect("materialize provider envelope");

    let relative = Path::new(ANTIGRAVITY_TOKEN_RELATIVE_PATH);
    let materialized = files.read(relative, 16 * 1024).expect("read envelope");
    let document: Value =
        serde_json::from_slice(&materialized.into_secret_bytes()).expect("parse envelope");
    assert_eq!(document["auth_method"], "consumer");
    assert_eq!(
        document["token"]["refresh_token"],
        "synthetic-initial-refresh"
    );

    let rewritten = OpaqueCredentialBytes::new(
        br#"{
            "auth_method":"consumer",
            "token":{
                "access_token":"synthetic-refreshed-access",
                "token_type":"Bearer",
                "refresh_token":"synthetic-rotated-refresh",
                "expiry":"2030-01-02T03:04:05Z"
            }
        }"#
        .to_vec(),
        16 * 1024,
    )
    .expect("synthetic rewrite");
    files
        .materialize(relative, &rewritten)
        .expect("simulate official rewrite");

    let captured = capture_refreshed_profile_credential("1.1.2", &provider, &files, 16 * 1024)
        .expect("capture rotated refresh");
    assert_eq!(captured.into_secret_bytes(), b"synthetic-rotated-refresh");

    fs::remove_dir_all(home).expect("remove fixture");
}
