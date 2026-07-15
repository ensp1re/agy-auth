//! End-to-end contracts for the compile-time-gated in-process fake client.

#![cfg(all(feature = "experimental-fake-client", unix))]

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

fn data_root() -> std::path::PathBuf {
    let parent = std::env::temp_dir().join(format!(
        "agy-auth-experimental-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock after epoch")
            .as_nanos()
    ));
    fs::create_dir(&parent).expect("create parent");
    fs::set_permissions(&parent, fs::Permissions::from_mode(0o700)).expect("secure parent");
    parent.join("data")
}

#[test]
fn fake_add_and_exec_persist_only_ready_metadata_and_propagate_exit() {
    let root = data_root();
    let binary = env!("CARGO_BIN_EXE_agy-auth");
    let add = Command::new(binary)
        .args([
            "--data-dir",
            root.to_str().expect("utf8 path"),
            "experimental-add",
            "Work",
        ])
        .status()
        .expect("run fake add");
    assert!(add.success());

    let duplicate = Command::new(binary)
        .args([
            "--data-dir",
            root.to_str().expect("utf8 path"),
            "experimental-add",
            "work",
        ])
        .status()
        .expect("run duplicate add");
    assert_eq!(duplicate.code(), Some(3));

    let execute = Command::new(binary)
        .args([
            "--data-dir",
            root.to_str().expect("utf8 path"),
            "experimental-exec",
            "Work",
            "--fake-exit",
            "23",
            "--",
            "literal;not-shell",
        ])
        .status()
        .expect("run fake exec");
    assert_eq!(execute.code(), Some(23));

    let registry = fs::read_to_string(root.join("registry.json")).expect("read registry");
    assert!(registry.contains("\"status\": \"ready\""));
    assert!(registry.contains("TEST_FAKE_CLIENT_1.1.2"));
    assert!(!registry.contains("token"));
    assert_eq!(
        fs::metadata(&root)
            .expect("root metadata")
            .permissions()
            .mode()
            & 0o777,
        0o700
    );
    fs::remove_dir_all(root.parent().expect("parent")).expect("remove fixture");
}
