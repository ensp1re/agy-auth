#![cfg(unix)]
#![doc = "End-to-end coverage for public secret-safe profile listing."]

use serde_json::Value;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn lists_only_stable_non_secret_profile_metadata() {
    let root = std::env::temp_dir().join(format!(
        "agy-auth-list-e2e-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_nanos()
    ));
    let data = root.join("data");
    fs::create_dir_all(&data).expect("data");
    fs::set_permissions(&data, fs::Permissions::from_mode(0o700)).expect("secure data");
    fs::write(
        data.join("registry.json"),
        br#"{
          "schemaVersion": 1,
          "profiles": [{
            "id": "018f47be-7c2a-7b43-8c42-9c88536f46a1",
            "name": "work",
            "provider": "antigravity-cli",
            "storage": {"kind": "isolated-home", "locator": "synthetic-profile-home"},
            "accountHint": null,
            "createdAt": "2026-07-16T00:00:00Z",
            "updatedAt": "2026-07-16T00:00:01Z",
            "clientVersionAtCapture": "1.1.3",
            "schemaFingerprint": null,
            "status": "ready"
          }]
        }"#,
    )
    .expect("registry");
    fs::set_permissions(
        data.join("registry.json"),
        fs::Permissions::from_mode(0o600),
    )
    .expect("secure registry");

    let output = Command::new(env!("CARGO_BIN_EXE_agy-auth"))
        .args(["--json", "--data-dir"])
        .arg(&data)
        .arg("list")
        .output()
        .expect("list");
    assert!(output.status.success());
    let document: Value = serde_json::from_slice(&output.stdout).expect("JSON");
    assert_eq!(document["schemaVersion"], 1);
    assert_eq!(document["profiles"][0]["name"], "work");
    assert_eq!(document["profiles"][0]["status"], "ready");
    assert_eq!(document["profiles"][0]["clientVersion"], "1.1.3");
    assert!(document["profiles"][0].get("id").is_none());
    assert!(document["profiles"][0].get("storage").is_none());
    assert!(document["profiles"][0].get("accountHint").is_none());

    fs::remove_dir_all(root).expect("cleanup");
}
