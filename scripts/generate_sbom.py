#!/usr/bin/env python3
"""Generate a deterministic SPDX 2.3 JSON SBOM from locked Cargo metadata."""

from __future__ import annotations

import argparse
import datetime as dt
import json
import re
import subprocess
import tomllib
from pathlib import Path


def spdx_id(value: str) -> str:
    return "SPDXRef-" + re.sub(r"[^A-Za-z0-9.-]", "-", value)


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--revision", required=True)
    parser.add_argument("--target", required=True)
    parser.add_argument("--source-date-epoch", type=int, required=True)
    args = parser.parse_args()
    if not re.fullmatch(r"[0-9a-f]{40}", args.revision):
        raise SystemExit("revision must be a full lowercase Git object ID")

    metadata = json.loads(
        subprocess.run(
            ["cargo", "metadata", "--locked", "--format-version", "1"],
            check=True,
            capture_output=True,
            text=True,
        ).stdout
    )
    lock = tomllib.loads(Path("Cargo.lock").read_text(encoding="utf-8"))
    checksums = {
        (package["name"], package["version"], package.get("source")): package.get("checksum")
        for package in lock["package"]
    }
    packages = []
    ids: dict[str, str] = {}
    for index, package in enumerate(sorted(metadata["packages"], key=lambda item: item["id"])):
        identifier = spdx_id(f"Package-{index}-{package['name']}-{package['version']}")
        ids[package["id"]] = identifier
        declared = package.get("license") or "NOASSERTION"
        entry = {
            "SPDXID": identifier,
            "name": package["name"],
            "versionInfo": package["version"],
            "downloadLocation": package.get("repository") or "NOASSERTION",
            "filesAnalyzed": False,
            "licenseConcluded": "NOASSERTION",
            "licenseDeclared": declared,
            "copyrightText": "NOASSERTION",
        }
        checksum = checksums.get((package["name"], package["version"], package.get("source")))
        if checksum:
            entry["checksums"] = [{"algorithm": "SHA256", "checksumValue": checksum}]
        packages.append(entry)

    relationships = []
    root = next(package for package in metadata["packages"] if package["name"] == "agy-auth-cli")
    relationships.append(
        {
            "spdxElementId": "SPDXRef-DOCUMENT",
            "relationshipType": "DESCRIBES",
            "relatedSpdxElement": ids[root["id"]],
        }
    )
    for node in metadata["resolve"]["nodes"]:
        for dependency in node["dependencies"]:
            relationships.append(
                {
                    "spdxElementId": ids[node["id"]],
                    "relationshipType": "DEPENDS_ON",
                    "relatedSpdxElement": ids[dependency],
                }
            )
    relationships.sort(key=lambda item: tuple(item.values()))
    created = dt.datetime.fromtimestamp(args.source_date_epoch, dt.UTC).strftime("%Y-%m-%dT%H:%M:%SZ")
    document = {
        "spdxVersion": "SPDX-2.3",
        "dataLicense": "CC0-1.0",
        "SPDXID": "SPDXRef-DOCUMENT",
        "name": f"agy-auth-{root['version']}-{args.target}",
        "documentNamespace": (
            f"https://github.com/ensp1re/agy-auth/sbom/{args.revision}/{args.target}"
        ),
        "creationInfo": {"created": created, "creators": ["Tool: agy-auth-generate-sbom"]},
        "packages": packages,
        "relationships": relationships,
    }
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(document, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    print(f"SPDX SBOM: {args.output} ({len(packages)} packages)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
