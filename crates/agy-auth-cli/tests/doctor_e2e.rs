#![cfg(unix)]

//! End-to-end contract tests for diagnostics-only doctor output.

use serde_json::Value;
use std::fs;
use std::io::Write;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::time::{SystemTime, UNIX_EPOCH};

fn fixture() -> (PathBuf, PathBuf) {
    let root = std::env::temp_dir().join(format!(
        "agy-auth-doctor-e2e-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock after epoch")
            .as_nanos()
    ));
    fs::create_dir(&root).expect("create fixture root");
    fs::set_permissions(&root, fs::Permissions::from_mode(0o700)).expect("secure fixture root");
    let client = root.join("agy");
    let mut file = fs::File::create(&client).expect("create fake client");
    file.write_all(b"#!/bin/sh\nprintf '1.1.2\\n'\n")
        .expect("write fake client");
    file.sync_all().expect("sync fake client");
    drop(file);
    fs::set_permissions(&client, fs::Permissions::from_mode(0o700))
        .expect("make client executable");
    (root, client)
}

fn doctor(data_root: &Path, client: &Path, json: bool) -> Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_agy-auth"));
    if json {
        command.arg("--json");
    }
    command
        .arg("--data-dir")
        .arg(data_root)
        .arg("doctor")
        .arg("--client")
        .arg(client)
        .output()
        .expect("run doctor")
}

#[test]
fn json_contract_is_versioned_and_diagnostics_only() {
    let (root, client) = fixture();
    let data_root = root.join("absent-data");

    let output = doctor(&data_root, &client, true);

    assert!(output.status.success());
    let report: Value = serde_json::from_slice(&output.stdout).expect("valid JSON report");
    assert_eq!(report["schemaVersion"], 1);
    assert_eq!(report["tool"]["version"], env!("CARGO_PKG_VERSION"));
    assert_eq!(
        report["tool"]["gitRevision"].as_str().map(str::len),
        Some(40)
    );
    assert!(
        report["tool"]["target"]
            .as_str()
            .is_some_and(|value| !value.is_empty())
    );
    assert_eq!(report["client"]["version"], "1.1.2");
    assert_eq!(report["registry"]["dataDirectoryState"], "absent");
    assert_eq!(report["capabilities"]["profileSwitching"], false);
    assert_eq!(report["capabilities"]["authStateMutation"], false);
    assert_eq!(
        report["capabilities"]["reason"],
        if cfg!(target_os = "linux") {
            "verified_contract_not_enabled"
        } else {
            "no_verified_antigravity_profile_contract"
        }
    );
    assert!(!data_root.exists());
    fs::remove_dir_all(root).expect("remove fixture root");
}

#[test]
fn human_output_explains_unsupported_switching() {
    let (root, client) = fixture();

    let output = doctor(&root.join("absent-data"), &client, false);

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("UTF-8 output");
    assert!(stdout.contains("client: agy 1.1.2"));
    assert!(stdout.contains("profile switching: unsupported"));
    assert!(stdout.contains("authentication mutation: disabled"));
    assert!(stdout.contains(if cfg!(target_os = "linux") {
        "reason: verified_contract_not_enabled"
    } else {
        "reason: no_verified_antigravity_profile_contract"
    }));
    fs::remove_dir_all(root).expect("remove fixture root");
}

#[test]
fn missing_client_returns_exit_four() {
    let (root, _) = fixture();

    let output = doctor(&root.join("absent-data"), &root.join("missing-agy"), true);

    assert_eq!(output.status.code(), Some(4));
    let report: Value = serde_json::from_slice(&output.stdout).expect("valid JSON report");
    assert_eq!(report["client"]["errorCode"], "client_not_found");
    fs::remove_dir_all(root).expect("remove fixture root");
}

#[test]
fn unsafe_permissions_and_recovery_have_contract_exit_codes() {
    let (root, client) = fixture();
    let data_root = root.join("data");
    fs::create_dir(&data_root).expect("create data root");
    fs::set_permissions(&data_root, fs::Permissions::from_mode(0o750))
        .expect("set unsafe permissions");

    let unsafe_output = doctor(&data_root, &client, true);
    assert_eq!(unsafe_output.status.code(), Some(8));

    fs::set_permissions(&data_root, fs::Permissions::from_mode(0o700)).expect("secure data root");
    let transactions = data_root.join("transactions");
    fs::create_dir(&transactions).expect("create transaction directory");
    fs::write(transactions.join("synthetic-marker"), b"TEST").expect("write marker");

    let recovery_output = doctor(&data_root, &client, true);
    assert_eq!(recovery_output.status.code(), Some(10));
    let report: Value = serde_json::from_slice(&recovery_output.stdout).expect("valid JSON report");
    assert_eq!(report["registry"]["interruptedTransactions"], 1);
    fs::remove_dir_all(root).expect("remove fixture root");
}
