#!/usr/bin/env python3
"""Validate project-owned harness state and high-risk policy boundaries."""

from __future__ import annotations

import json
import os
import re
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
STATUSES = {"not_started", "active", "blocked", "passing"}
PLACEHOLDER = re.compile(r"__[A-Z][A-Z0-9_]*__|\[TODO(?::[^]]*)?]", re.IGNORECASE)
SECRETS = (
    re.compile(r"-----BEGIN [A-Z0-9 ]*PRIVATE KEY-----"),
    re.compile(r"\bAKIA[0-9A-Z]{16}\b"),
    re.compile(r"\bgh[pousr]_[A-Za-z0-9]{36,}\b"),
    re.compile(r"\b(?:ya29\.|1//)[A-Za-z0-9._-]+\b"),
)


def read_json(relative: str, errors: list[str]) -> dict:
    try:
        value = json.loads((ROOT / relative).read_text(encoding="utf-8"))
    except (OSError, UnicodeDecodeError, json.JSONDecodeError) as exc:
        errors.append(f"cannot read {relative}: {exc}")
        return {}
    if PLACEHOLDER.search(json.dumps(value)):
        errors.append(f"unresolved placeholder in {relative}")
    return value


def git(*args: str) -> str:
    result = subprocess.run(
        ["git", "-C", str(ROOT), *args], check=False, capture_output=True, text=True, timeout=10
    )
    return result.stdout.strip() if result.returncode == 0 else ""


def main() -> int:
    errors: list[str] = []
    manifest = read_json(".harness/manifest.json", errors)
    work = read_json(".harness/state/work.json", errors)
    handoff = read_json(".harness/state/handoff.json", errors)

    if manifest.get("status") != "operational":
        errors.append("manifest must be operational")
    sources = manifest.get("sources", {})
    referenced: list[str] = []
    for value in sources.values():
        referenced.extend(value if isinstance(value, list) else [value] if isinstance(value, str) else [])
    for relative in referenced:
        path = (ROOT / relative).resolve()
        if ROOT not in path.parents or not path.is_file():
            errors.append(f"missing or unsafe manifest source: {relative}")

    items = work.get("items", [])
    invalid = [item.get("id", "unknown") for item in items if item.get("status") not in STATUSES]
    if invalid:
        errors.append(f"invalid work status: {', '.join(invalid)}")
    active = [item for item in items if item.get("status") == "active"]
    current = [item for item in items if item.get("status") in {"active", "blocked"}]
    if not work.get("parallelMode") and len(active) > 1:
        errors.append("multiple active items without parallel mode")
    if not work.get("parallelMode") and len(current) > 1:
        errors.append("multiple current items without parallel mode")
    if handoff.get("scope") != [item.get("id") for item in current]:
        errors.append("handoff scope differs from current work")
    for item in items:
        if item.get("status") == "passing" and not item.get("verification"):
            errors.append(f"passing item lacks verification: {item.get('id')}")
        source = item.get("source")
        if source and not (ROOT / source).is_file():
            errors.append(f"work item source does not exist: {source}")

    # GitHub Actions checks out synthetic merge commits or detached revisions. Local validation
    # remains the authority for matching durable handoff state to a developer's working branch.
    if os.environ.get("GITHUB_ACTIONS") != "true" and handoff.get("git", {}).get(
        "branch"
    ) != git("branch", "--show-current"):
        errors.append("handoff branch differs from Git")

    tracked = git("ls-files").splitlines()
    for relative in tracked:
        path = ROOT / relative
        if not path.is_file() or path.stat().st_size > 1_000_000:
            continue
        try:
            text = path.read_text(encoding="utf-8")
        except UnicodeDecodeError:
            continue
        if any(pattern.search(text) for pattern in SECRETS):
            errors.append(f"possible secret or forbidden real-token prefix: {relative}")

    if errors:
        print(f"Harness check: {len(errors)} error(s)")
        for error in errors:
            print(f"- {error}")
        return 1
    print("Harness check: passing")
    print(f"- sources: {len(referenced)}")
    print(f"- work items: {len(items)} ({len(active)} active)")
    print(f"- tracked files scanned for secrets: {len(tracked)}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
