#!/usr/bin/env python3
"""Validate the non-secret official-client compatibility matrix."""

from __future__ import annotations

import json
from pathlib import Path


def main() -> int:
    path = Path("compatibility/antigravity-cli.json")
    document = json.loads(path.read_text(encoding="utf-8"))
    if document.get("schemaVersion") != 1:
        raise SystemExit("compatibility matrix must use schemaVersion 1")
    if document.get("provider") != "antigravity-cli":
        raise SystemExit("compatibility matrix provider is invalid")
    versions = document.get("versions")
    if not isinstance(versions, list) or not versions:
        raise SystemExit("compatibility matrix requires at least one version")
    required = {
        "version",
        "observedAt",
        "platform",
        "discovery",
        "versionProbe",
        "profileSwitching",
        "authStateMutation",
        "evidence",
        "reverifyWhen",
    }
    seen: set[tuple[str, str]] = set()
    for entry in versions:
        if not isinstance(entry, dict) or set(entry) != required:
            raise SystemExit("compatibility entry fields are invalid")
        key = (entry["version"], entry["platform"])
        if key in seen:
            raise SystemExit(f"duplicate compatibility entry: {key}")
        seen.add(key)
        state = (entry["profileSwitching"], entry["authStateMutation"])
        if state not in {("unsupported", "disabled"), ("verified", "verified")}:
            raise SystemExit(f"invalid compatibility capability state: {state}")
        if not Path(entry["evidence"]).is_file():
            raise SystemExit(f"compatibility evidence is missing: {entry['evidence']}")
    print(f"Compatibility matrix: passing ({len(versions)} entry)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
