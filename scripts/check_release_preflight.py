#!/usr/bin/env python3
"""Validate current version, release notes, packaging, and optional tag identity."""

from __future__ import annotations

import json
import subprocess
from pathlib import Path


def git(*args: str) -> str:
    return subprocess.run(
        ["git", *args],
        check=True,
        capture_output=True,
        text=True,
    ).stdout.strip()


def main() -> int:
    metadata = json.loads(
        subprocess.run(
            ["cargo", "metadata", "--locked", "--format-version", "1", "--no-deps"],
            check=True,
            capture_output=True,
            text=True,
        ).stdout
    )
    workspace = set(metadata["workspace_members"])
    versions = {
        package["version"] for package in metadata["packages"] if package["id"] in workspace
    }
    if len(versions) != 1:
        raise SystemExit(f"workspace versions must match: {sorted(versions)}")
    version = versions.pop()
    tag = f"v{version}"
    notes_path = Path(f"docs/releases/{tag}.md")
    if not notes_path.is_file():
        raise SystemExit(f"release notes are missing: {notes_path}")
    notes = notes_path.read_text(encoding="utf-8")
    required = (
        "x86_64-unknown-linux-gnu",
        "Antigravity CLI `1.1.2` and `1.1.3`",
        "reverse engineered",
        "GitHub Actions CI",
        "SHA-256",
        f"`{tag}`",
    )
    if any(fragment not in notes for fragment in required):
        raise SystemExit("release notes are missing required scope, risk, or verification details")
    packaging = Path("scripts/package_linux_rc.sh").read_text(encoding="utf-8")
    if 'name="agy-auth-${version}-${target}"' not in packaging:
        raise SystemExit("packaging artifact name is inconsistent with Cargo version")
    installer = Path("scripts/install.sh").read_text(encoding="utf-8")
    if f'VERSION="${{AGY_AUTH_VERSION:-{version}}}"' not in installer:
        raise SystemExit("installer default version is inconsistent with Cargo version")
    existing = git("tag", "--list", tag)
    if existing:
        target = git("rev-list", "-n", "1", tag)
        head = git("rev-parse", "HEAD")
        ancestor = subprocess.run(
            ["git", "merge-base", "--is-ancestor", target, head],
            check=False,
        ).returncode
        if ancestor != 0:
            raise SystemExit(f"existing release tag {tag} is not reachable from HEAD")
        state = "published tag reachable"
    else:
        state = "tag not created"
    print(f"Release preflight: passing ({tag}; {state})")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
