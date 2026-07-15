#!/usr/bin/env python3
"""Print bounded, deterministic startup context for antigravity-auth."""

from __future__ import annotations

import json
import subprocess
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]


def git(*args: str) -> str:
    result = subprocess.run(
        ["git", "-C", str(ROOT), *args],
        check=False,
        capture_output=True,
        text=True,
        timeout=10,
    )
    return result.stdout.strip() if result.returncode == 0 else "unavailable"


def git_ok(*args: str) -> bool:
    return subprocess.run(
        ["git", "-C", str(ROOT), *args],
        check=False,
        capture_output=True,
        text=True,
        timeout=10,
    ).returncode == 0


def is_closeout_commit(recorded: str, actual: str, worktree: str) -> bool:
    """Allow the single clean commit that necessarily contains its own handoff snapshot."""
    if worktree != "clean" or recorded in {"", "unavailable"} or actual == "unavailable":
        return False
    return git("rev-list", "--count", f"{recorded}..{actual}") == "1" and git_ok(
        "merge-base", "--is-ancestor", recorded, actual
    )


def load(relative: str) -> dict:
    return json.loads((ROOT / relative).read_text(encoding="utf-8"))


def main() -> int:
    work = load(".harness/state/work.json")
    handoff = load(".harness/state/handoff.json")
    branch = git("branch", "--show-current")
    head = git("rev-parse", "HEAD")
    status = git("status", "--short")
    actual_tree = "dirty" if status and status != "unavailable" else "clean"
    current = [item for item in work["items"] if item["status"] in {"active", "blocked"}]
    conflicts = []
    if len(current) != 1:
        conflicts.append(f"expected exactly one active or blocked item, found {len(current)}")
    if handoff["scope"] != [item["id"] for item in current]:
        conflicts.append("handoff scope differs from current work")
    if handoff["git"]["branch"] != branch:
        conflicts.append("handoff branch differs from Git")
    closeout = is_closeout_commit(handoff["git"]["head"], head, actual_tree)
    if handoff["git"]["head"] != head and not closeout:
        conflicts.append("handoff revision differs from Git; reconcile changed repository state")
    if handoff["git"]["worktree"] != actual_tree and not closeout:
        conflicts.append("handoff worktree state differs from Git")

    print(f"root: {ROOT}")
    print(f"git: {branch} {head[:12]} {actual_tree}")
    if current:
        item = current[0]
        print(f"current: {item['id']} ({item['status']}) — {item['title']}")
        print(f"source: {item['source']}")
    print(f"next: {handoff['nextActions'][0] if handoff['nextActions'] else 'none recorded'}")
    print(f"verification: {len(handoff['verification']['passed'])} passed, "
          f"{len(handoff['verification']['failed'])} failed, "
          f"{len(handoff['verification']['notRun'])} not run")
    if conflicts:
        print("state: CONFLICT")
        for conflict in conflicts:
            print(f"- {conflict}")
        return 1
    print("state: consistent")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
