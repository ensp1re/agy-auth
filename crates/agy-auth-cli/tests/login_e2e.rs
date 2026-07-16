#![doc = "Synthetic end-to-end coverage for official-client delegated profile enrollment."]
#![cfg(all(feature = "profile-cli", unix))]

use serde_json::Value;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn login_delegates_to_client_and_registers_resulting_profile() {
    let root = std::env::temp_dir().join(format!(
        "agy-auth-login-e2e-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_nanos()
    ));
    fs::create_dir(&root).expect("root");
    fs::set_permissions(&root, fs::Permissions::from_mode(0o700)).expect("secure root");
    let client = root.join("agy");
    fs::write(
        &client,
        b"#!/bin/sh\nif [ \"${1-}\" = \"--version\" ]; then printf '1.1.3\\n'; exit 0; fi\numask 022\nmkdir -p \"$HOME/.gemini/antigravity-cli/cache\" \"$HOME/.gemini/antigravity-cli/log\"\nprintf '%s' \"applyAuthResult: email='synthetic-user@example.invalid'\" > \"$HOME/.gemini/antigravity-cli/log/cli-test.log\"\nprintf '%s' '{\"auth_method\":\"consumer\",\"token\":{\"access_token\":\"synthetic-login-access\",\"token_type\":\"Bearer\",\"refresh_token\":\"synthetic-login-refresh\",\"expiry\":\"2030-01-02T03:04:05Z\"}}' > \"$HOME/.gemini/antigravity-cli/antigravity-oauth-token\"\nchmod 600 \"$HOME/.gemini/antigravity-cli/antigravity-oauth-token\"\nsleep 1\nprintf '%s' '{\"consumerOnboardingComplete\":true,\"enterpriseOnboardingComplete\":false,\"onboardingComplete\":true}' > \"$HOME/.gemini/antigravity-cli/cache/onboarding.json\"\nsleep 2\ntouch \"$HOME/unwanted-next-action\"\n",
    )
    .expect("client");
    fs::set_permissions(&client, fs::Permissions::from_mode(0o700)).expect("executable");
    let data = root.join("data");
    let status = Command::new(env!("CARGO_BIN_EXE_agy-auth"))
        .args(["--data-dir"])
        .arg(&data)
        .args(["login", "--client"])
        .arg(&client)
        .status()
        .expect("login");
    assert!(status.success());

    let registry: Value =
        serde_json::from_slice(&fs::read(data.join("registry.json")).expect("registry"))
            .expect("JSON");
    assert_eq!(registry["profiles"][0]["name"], "profile1");
    assert_eq!(registry["profiles"][0]["status"], "ready");
    assert_eq!(registry["profiles"][0]["clientVersionAtCapture"], "1.1.3");
    assert_eq!(
        registry["profiles"][0]["accountHint"],
        "syn***@example.invalid"
    );
    assert_eq!(
        fs::read_dir(data.join("transactions"))
            .expect("transactions")
            .count(),
        0
    );
    let profile_id = registry["profiles"][0]["id"].as_str().expect("profile id");
    assert!(
        !data
            .join("profiles")
            .join(profile_id)
            .join("home")
            .join("unwanted-next-action")
            .exists()
    );
    fs::remove_dir_all(root).expect("cleanup");
}
