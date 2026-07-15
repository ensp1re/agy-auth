#!/usr/bin/env python3
"""Validate repository license and release metadata consistency."""

from __future__ import annotations

import json
import subprocess
from pathlib import Path


def main() -> int:
    license_text = Path("LICENSE").read_text(encoding="utf-8")
    required_fragments = (
        "MIT License",
        "Copyright (c) 2026 Enspire",
        "Permission is hereby granted, free of charge",
        'THE SOFTWARE IS PROVIDED "AS IS"',
    )
    if any(fragment not in license_text for fragment in required_fragments):
        raise SystemExit("LICENSE does not contain the approved MIT terms and identity")
    metadata = json.loads(
        subprocess.run(
            ["cargo", "metadata", "--locked", "--format-version", "1", "--no-deps"],
            check=True,
            capture_output=True,
            text=True,
        ).stdout
    )
    workspace = set(metadata["workspace_members"])
    missing = [
        package["name"]
        for package in metadata["packages"]
        if package["id"] in workspace and package.get("license") != "MIT"
    ]
    if missing:
        raise SystemExit(f"workspace packages missing MIT metadata: {sorted(missing)}")
    if "[MIT License](LICENSE)" not in Path("README.md").read_text(encoding="utf-8"):
        raise SystemExit("README does not link the MIT license")
    print(f"Release metadata: passing ({len(workspace)} MIT workspace packages)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
