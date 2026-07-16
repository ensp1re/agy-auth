#![cfg(all(feature = "profile-cli", target_os = "linux"))]
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

fn create_and_recover_conflicting_import(
    binary: &str,
    data: &Path,
    source_home: &Path,
    client: &Path,
) {
    let interrupted = Command::new(binary)
        .args(["--data-dir"])
        .arg(data)
        .args(["add", "work", "--from-home"])
        .arg(source_home)
        .arg("--client")
        .arg(client)
        .status()
        .expect("run conflicting import");
    assert_eq!(interrupted.code(), Some(3));
    assert_eq!(
        fs::read_dir(data.join("transactions"))
            .expect("transaction directory")
            .count(),
        1
    );
    let recovered = Command::new(binary)
        .args(["--data-dir"])
        .arg(data)
        .arg("recover")
        .status()
        .expect("recover import");
    assert!(recovered.success());
    assert_eq!(
        fs::read_dir(data.join("transactions"))
            .expect("transaction directory")
            .count(),
        0
    );
}

fn set_and_verify_masked_hint(binary: &str, data: &Path) {
    let hinted = Command::new(binary)
        .args(["--data-dir"])
        .arg(data)
        .args(["hint", "work", "w***@example.invalid"])
        .status()
        .expect("set hint");
    assert!(hinted.success());
    let hinted_listing = Command::new(binary)
        .args(["--json", "--data-dir"])
        .arg(data)
        .arg("list")
        .output()
        .expect("list hinted profile");
    let hinted_document: Value =
        serde_json::from_slice(&hinted_listing.stdout).expect("parse hinted listing");
    assert_eq!(
        hinted_document["profiles"][0]["accountHint"],
        "w***@example.invalid"
    );
}

fn switch_and_verify_default(binary: &str, data: &Path, home: &Path, client: &Path) {
    let switched = Command::new(binary)
        .env("HOME", home)
        .args(["--data-dir"])
        .arg(data)
        .args(["switch", "work", "--client"])
        .arg(client)
        .status()
        .expect("switch default account");
    assert!(switched.success());
    let active: Value =
        serde_json::from_slice(&fs::read(data.join("active.json")).expect("active selection"))
            .expect("active JSON");
    assert_eq!(active["profile"], "work");
    let default_envelope: Value = serde_json::from_slice(
        &fs::read(home.join(".gemini/antigravity-cli/antigravity-oauth-token"))
            .expect("default envelope"),
    )
    .expect("default envelope JSON");
    assert_eq!(
        default_envelope["token"]["refresh_token"],
        "synthetic-source-refresh"
    );
    assert_eq!(
        default_envelope["token"]["access_token"],
        "agy-auth-expired-placeholder"
    );
    let selected_listing = Command::new(binary)
        .args(["--json", "--data-dir"])
        .arg(data)
        .arg("list")
        .output()
        .expect("list selected profile");
    let selected: Value =
        serde_json::from_slice(&selected_listing.stdout).expect("selected listing JSON");
    assert_eq!(selected["profiles"][0]["selected"], true);
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
        .args(["add", "work", "--from-home"])
        .arg(&source_home)
        .arg("--client")
        .arg(&client)
        .status()
        .expect("run import");
    assert!(imported.success());

    create_and_recover_conflicting_import(binary, &data, &source_home, &client);

    let listed = Command::new(binary)
        .args(["--json", "--data-dir"])
        .arg(&data)
        .arg("list")
        .output()
        .expect("list profiles");
    assert!(listed.status.success());
    let listing: Value = serde_json::from_slice(&listed.stdout).expect("parse listing");
    assert_eq!(listing["schemaVersion"], 1);
    assert_eq!(listing["profiles"][0]["name"], "work");
    assert_eq!(listing["profiles"][0]["status"], "ready");
    assert_eq!(listing["profiles"][0]["clientVersion"], "1.1.3");
    assert!(listing["profiles"][0]["accountHint"].is_null());
    assert!(listing["profiles"][0].get("id").is_none());
    assert!(listing["profiles"][0].get("storage").is_none());

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
        .args(["exec", "work", "--client"])
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

    switch_and_verify_default(binary, &data, &source_home, &client);

    set_and_verify_masked_hint(binary, &data);

    write_file(
        &client,
        b"#!/bin/sh\nif [ \"${1-}\" = \"--version\" ]; then printf '1.1.2\\n'; exit 0; fi\nexit 0\n",
        0o700,
    );
    let mismatched = Command::new(binary)
        .args(["--data-dir"])
        .arg(&data)
        .args(["exec", "work", "--client"])
        .arg(&client)
        .status()
        .expect("run mismatched client");
    assert_eq!(mismatched.code(), Some(5));

    fs::remove_dir_all(root).expect("remove fixture");
}
