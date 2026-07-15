#!/usr/bin/env python3
"""Run the fast, deterministic local quality gate."""

from __future__ import annotations

import subprocess

COMMANDS = (
    ("format", ["cargo", "fmt", "--all", "--", "--check"]),
    ("clippy", ["cargo", "clippy", "--workspace", "--all-targets", "--all-features", "--", "-D", "warnings"]),
    ("tests", ["cargo", "test", "--workspace", "--all-features"]),
    ("dependencies", ["python3", "scripts/check_dependencies.py"]),
    ("compatibility", ["python3", "scripts/check_compatibility.py"]),
    ("release metadata", ["python3", "scripts/check_release_metadata.py"]),
    ("secrets", ["python3", "scripts/check_no_secrets.py"]),
    ("harness", ["python3", "scripts/harness/validate.py"]),
)


def main() -> int:
    for name, command in COMMANDS:
        print(f"==> {name}", flush=True)
        result = subprocess.run(command, check=False)
        if result.returncode != 0:
            return result.returncode
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
