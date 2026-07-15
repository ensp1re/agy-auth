#!/usr/bin/env python3
"""Enforce the documented direct dependency direction between workspace crates."""

from __future__ import annotations

import json
import subprocess
import sys

ALLOWED = {
    "gemini-auth-domain": set(),
    "gemini-auth-app": {"gemini-auth-domain"},
    "gemini-auth-storage": {"gemini-auth-domain"},
    "provider-gemini-cli": {"gemini-auth-domain"},
    "provider-antigravity-cli": {"gemini-auth-domain"},
    "gemini-auth-cli": {"gemini-auth-app"},
    "gemini-auth-test-support": {"gemini-auth-domain"},
}


def main() -> int:
    result = subprocess.run(
        ["cargo", "metadata", "--format-version", "1", "--no-deps"],
        check=True,
        capture_output=True,
        text=True,
    )
    metadata = json.loads(result.stdout)
    workspace = set(metadata["workspace_members"])
    errors = []
    for package in metadata["packages"]:
        if package["id"] not in workspace or package["name"] not in ALLOWED:
            continue
        actual = {
            dependency["name"]
            for dependency in package["dependencies"]
            if dependency.get("path") is not None and dependency["kind"] is None
        }
        forbidden = actual - ALLOWED[package["name"]]
        if forbidden:
            errors.append(f"{package['name']} has forbidden dependencies: {sorted(forbidden)}")
    if errors:
        print("\n".join(errors), file=sys.stderr)
        return 1
    print("Workspace dependency direction: passing")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
