//! Opt-in smoke test for a locally installed official client binary.

use agy_auth_process::{OfficialClient, discover_client};
use std::path::Path;
use std::time::Duration;

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
