#!/usr/bin/env python3
"""Validate version, notes, artifact naming, and tag readiness for rc.1."""

from __future__ import annotations

import json
import subprocess
from pathlib import Path


EXPECTED_VERSION = "0.1.0-rc.1"
EXPECTED_TAG = f"v{EXPECTED_VERSION}"


def main() -> int:
    metadata = json.loads(
        subprocess.run(
            ["cargo", "metadata", "--locked", "--format-version", "1", "--no-deps"],
            check=True,
            capture_output=True,
            text=True,
        ).stdout
    )
    versions = {
        package["version"]
        for package in metadata["packages"]
        if package["id"] in set(metadata["workspace_members"])
    }
    if versions != {EXPECTED_VERSION}:
        raise SystemExit(f"workspace versions must all be {EXPECTED_VERSION}: {sorted(versions)}")
    notes_path = Path(f"docs/releases/{EXPECTED_TAG}.md")
    notes = notes_path.read_text(encoding="utf-8")
    required = (
        "diagnostics-only",
        "Profile switching is unsupported",
        "authentication-state mutation is disabled",
        "x86_64-unknown-linux-gnu",
        "Hosted CI: deferred",
        f"Git tag will be `{EXPECTED_TAG}`",
    )
    if any(fragment not in notes for fragment in required):
        raise SystemExit("release notes are missing required scope or risk disclosures")
    packaging = Path("scripts/package_linux_rc.sh").read_text(encoding="utf-8")
    if 'name="agy-auth-${version}-${target}"' not in packaging:
        raise SystemExit("packaging artifact name is inconsistent with Cargo version")
    existing = subprocess.run(
        ["git", "tag", "--list", EXPECTED_TAG],
        check=True,
        capture_output=True,
        text=True,
    ).stdout.strip()
    if existing:
        raise SystemExit(f"release candidate tag already exists: {EXPECTED_TAG}")
    print(f"Release preflight: passing ({EXPECTED_TAG} ready, not created)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
