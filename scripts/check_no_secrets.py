#!/usr/bin/env python3
"""Reject obvious credentials and real-provider token prefixes in tracked files."""

from __future__ import annotations

import re
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
PATTERNS = (
    re.compile(rb"-----BEGIN [A-Z0-9 ]*PRIVATE KEY-----"),
    re.compile(rb"\bAKIA[0-9A-Z]{16}\b"),
    re.compile(rb"\bgh[pousr]_[A-Za-z0-9]{36,}\b"),
    re.compile(rb"\b(?:ya29\.|1//)[A-Za-z0-9._-]+\b"),
)


def main() -> int:
    tracked = subprocess.run(
        ["git", "-C", str(ROOT), "ls-files", "-co", "--exclude-standard"],
        check=True,
        capture_output=True,
        text=True,
    ).stdout.splitlines()
    failures = []
    for relative in tracked:
        path = ROOT / relative
        if not path.is_file() or path.stat().st_size > 1_000_000:
            continue
        data = path.read_bytes()
        if any(pattern.search(data) for pattern in PATTERNS):
            failures.append(relative)
    if failures:
        print("Possible secret or forbidden real-token prefix:")
        print("\n".join(f"- {path}" for path in failures))
        return 1
    print(f"Secret policy scan: passing ({len(tracked)} files)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
