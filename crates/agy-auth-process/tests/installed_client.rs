//! Opt-in smoke test for a locally installed official client binary.

#![cfg(unix)]

use agy_auth_process::{IsolatedClientEnvironment, OfficialClient, discover_client};
use std::ffi::OsString;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

fn isolated_directory(label: &str) -> PathBuf {
    let path = std::env::temp_dir().join(format!(
        "agy-auth-installed-{label}-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock after Unix epoch")
            .as_nanos()
    ));
    fs::create_dir(&path).expect("create isolated directory");
    fs::set_permissions(&path, fs::Permissions::from_mode(0o700))
        .expect("secure isolated directory");
    path
}

#[test]
#[ignore = "requires AGY_AUTH_TEST_AGY pointing to an installed agy executable"]
fn discovers_and_probes_installed_agy_without_authentication() {
    let path = std::env::var_os("AGY_AUTH_TEST_AGY").expect("AGY_AUTH_TEST_AGY must be set");
    let client = discover_client(OfficialClient::AntigravityCli, Some(Path::new(&path)), None)
        .expect("discover installed agy");
    let version = client
        .probe_version(Duration::from_secs(5))
        .expect("run agy --version");
    assert!(!version.version().is_empty());
    assert!(!version.truncated());
    if let Ok(expected) = std::env::var("AGY_AUTH_TEST_AGY_VERSION") {
        assert_eq!(version.version(), expected);
    }
}

#[test]
#[ignore = "requires AGY_AUTH_TEST_AGY pointing to an installed agy executable"]
fn probes_installed_agy_in_two_empty_isolated_homes() {
    let path = std::env::var_os("AGY_AUTH_TEST_AGY").expect("AGY_AUTH_TEST_AGY must be set");
    let client = discover_client(OfficialClient::AntigravityCli, Some(Path::new(&path)), None)
        .expect("discover installed agy");
    let parent = isolated_directory("parent");
    let expected = std::env::var("AGY_AUTH_TEST_AGY_VERSION").ok();
    let mut versions = Vec::new();

    for label in ["work", "personal"] {
        let home = parent.join(format!("{label}-home"));
        let runtime = parent.join(format!("{label}-runtime"));
        for directory in [&home, &runtime] {
            fs::create_dir(directory).expect("create profile directory");
            fs::set_permissions(directory, fs::Permissions::from_mode(0o700))
                .expect("secure profile directory");
        }
        let environment = IsolatedClientEnvironment::new(
            home.clone(),
            runtime.clone(),
            OsString::from("/usr/bin:/bin"),
        )
        .expect("isolated environment");
        let version = client
            .probe_version_isolated(&environment, Duration::from_secs(5))
            .expect("run isolated agy --version");
        if let Some(expected) = &expected {
            assert_eq!(version.version(), expected);
        }
        assert!(fs::read_dir(&home).expect("read home").next().is_none());
        assert!(
            fs::read_dir(&runtime)
                .expect("read runtime")
                .next()
                .is_none()
        );
        versions.push(version.version().to_owned());
    }

    assert_eq!(versions[0], versions[1]);
    fs::remove_dir_all(parent).expect("remove isolated directories");
}
