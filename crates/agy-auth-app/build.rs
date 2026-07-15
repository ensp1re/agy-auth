//! Embed reproducible, non-secret build identity for diagnostics.

use std::process::Command;

fn main() {
    println!("cargo:rerun-if-env-changed=AGY_AUTH_GIT_REVISION");
    println!("cargo:rerun-if-changed=../../.git/HEAD");
    if let Ok(head) = std::fs::read_to_string("../../.git/HEAD") {
        if let Some(reference) = head.trim().strip_prefix("ref: ") {
            println!("cargo:rerun-if-changed=../../.git/{reference}");
        }
    }
    let target = std::env::var("TARGET").unwrap_or_else(|_| "unknown-target".to_owned());
    let revision = std::env::var("AGY_AUTH_GIT_REVISION")
        .ok()
        .filter(|value| valid_revision(value))
        .or_else(git_revision)
        .unwrap_or_else(|| "unknown".to_owned());
    println!("cargo:rustc-env=AGY_AUTH_BUILD_TARGET={target}");
    println!("cargo:rustc-env=AGY_AUTH_GIT_REVISION={revision}");
}

fn git_revision() -> Option<String> {
    let output = Command::new("git")
        .args(["rev-parse", "--verify", "HEAD"])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let value = String::from_utf8(output.stdout).ok()?;
    let value = value.trim();
    valid_revision(value).then(|| value.to_owned())
}

fn valid_revision(value: &str) -> bool {
    value.len() == 40 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}
