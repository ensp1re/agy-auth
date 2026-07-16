#![cfg(all(feature = "experimental-real-profile-cli", unix))]
#![doc = "Synthetic end-to-end coverage for hidden real-profile CLI composition."]

use serde_json::Value;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

fn fixture() -> PathBuf {
    let path = std::env::temp_dir().join(format!(
        "agy-auth-real-profile-cli-{}-{}",
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

fn write_file(path: &Path, value: &[u8], mode: u32) {
    if let Some(parent) = path.parent() {
        let mut current = PathBuf::new();
        for component in parent.components() {
            current.push(component);
            if !current.exists() {
                fs::create_dir(&current).expect("create parent component");
            }
            if current.starts_with(std::env::temp_dir()) && current != std::env::temp_dir() {
                fs::set_permissions(&current, fs::Permissions::from_mode(0o700))
                    .expect("secure parent component");
            }
        }
    }
    fs::write(path, value).expect("write fixture");
    fs::set_permissions(path, fs::Permissions::from_mode(mode)).expect("set fixture mode");
}

#[test]
fn imports_secure_official_home_and_executes_direct_argv() {
    let root = fixture();
    let source_home = root.join("source");
    let data = root.join("data");
    let observation = root.join("observation");
    fs::create_dir(&source_home).expect("create source home");
    fs::set_permissions(&source_home, fs::Permissions::from_mode(0o700)).expect("secure source");
    write_file(
        &source_home.join(".gemini/antigravity-cli/antigravity-oauth-token"),
        br#"{"auth_method":"consumer","token":{"access_token":"synthetic-source-access","token_type":"Bearer","refresh_token":"synthetic-source-refresh","expiry":"2030-01-02T03:04:05Z"}}"#,
        0o600,
    );

    let client = root.join("agy");
    write_file(
        &client,
        b"#!/bin/sh\nif [ \"${1-}\" = \"--version\" ]; then printf '1.1.3\\n'; exit 0; fi\nprintf '%s|%s|%s' \"$1\" \"$2\" \"$HOME\" > \"$3\"\nexit 29\n",
        0o700,
    );
    let binary = env!("CARGO_BIN_EXE_agy-auth");
    let imported = Command::new(binary)
        .args(["--data-dir"])
        .arg(&data)
        .args(["experimental-import", "work", "--from-home"])
        .arg(&source_home)
        .arg("--client")
        .arg(&client)
        .status()
        .expect("run import");
    assert!(imported.success());

    let registry: Value =
        serde_json::from_slice(&fs::read(data.join("registry.json")).expect("read registry"))
            .expect("parse registry");
    let profile = &registry["profiles"][0];
    assert_eq!(profile["name"], "work");
    assert_eq!(profile["status"], "ready");
    assert_eq!(profile["clientVersionAtCapture"], "1.1.3");
    let profile_id = profile["id"].as_str().expect("profile id");
    let managed_home = data.join("profiles").join(profile_id).join("home");
    let managed_envelope: Value = serde_json::from_slice(
        &fs::read(managed_home.join(".gemini/antigravity-cli/antigravity-oauth-token"))
            .expect("read managed envelope"),
    )
    .expect("parse managed envelope");
    assert_eq!(
        managed_envelope["token"]["refresh_token"],
        "synthetic-source-refresh"
    );
    assert_eq!(
        managed_envelope["token"]["access_token"],
        "agy-auth-expired-placeholder"
    );

    let literal = "literal; shell syntax is data";
    let executed = Command::new(binary)
        .args(["--data-dir"])
        .arg(&data)
        .args(["experimental-real-exec", "work", "--client"])
        .arg(&client)
        .arg("--")
        .arg(literal)
        .arg("second")
        .arg(&observation)
        .status()
        .expect("run real exec");
    assert_eq!(executed.code(), Some(29));
    assert_eq!(
        fs::read_to_string(&observation).expect("read observation"),
        format!("{literal}|second|{}", managed_home.display())
    );
    assert!(!root.join("shell syntax is data").exists());

    write_file(
        &client,
        b"#!/bin/sh\nif [ \"${1-}\" = \"--version\" ]; then printf '1.1.2\\n'; exit 0; fi\nexit 0\n",
        0o700,
    );
    let mismatched = Command::new(binary)
        .args(["--data-dir"])
        .arg(&data)
        .args(["experimental-real-exec", "work", "--client"])
        .arg(&client)
        .status()
        .expect("run mismatched client");
    assert_eq!(mismatched.code(), Some(5));

    fs::remove_dir_all(root).expect("remove fixture");
}
