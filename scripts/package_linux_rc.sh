#!/usr/bin/env bash
set -euo pipefail

repo_root=$(git rev-parse --show-toplevel)
cd "$repo_root"

if ! git diff --quiet -- . || ! git diff --cached --quiet -- . || \
  [[ -n $(git ls-files --others --exclude-standard) ]]; then
  echo "local RC packaging requires a clean Git worktree" >&2
  exit 2
fi

target=${AGY_AUTH_RELEASE_TARGET:-x86_64-unknown-linux-gnu}
host=$(rustc -vV | sed -n 's/^host: //p')
if [[ "$target" != "$host" ]]; then
  echo "local RC packaging requires the host target ($host), got $target" >&2
  exit 2
fi
if [[ -e LICENSE || -e LICENSE-MIT || -e LICENSE-APACHE ]]; then
  echo "license detected; review this local-only script before using it for distribution" >&2
  exit 2
fi

version=$(cargo metadata --locked --format-version 1 --no-deps | python3 -c \
  'import json,sys; d=json.load(sys.stdin); print(next(p["version"] for p in d["packages"] if p["name"] == "agy-auth-cli"))')
revision=${AGY_AUTH_GIT_REVISION:-$(git rev-parse --verify HEAD)}
epoch=${SOURCE_DATE_EPOCH:-$(git show -s --format=%ct HEAD)}
name="agy-auth-${version}-${target}-local-rc"
output_dir=${1:-dist}
mkdir -p "$output_dir"
output_dir=$(cd "$output_dir" && pwd)
work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT
stage="$work/$name"
mkdir -p "$stage"

AGY_AUTH_GIT_REVISION="$revision" SOURCE_DATE_EPOCH="$epoch" \
  cargo build --release --locked --target "$target" --bin agy-auth
install -m 0755 "target/$target/release/agy-auth" "$stage/agy-auth"
install -m 0644 README.md "$stage/README.md"
python3 scripts/generate_sbom.py \
  --output "$stage/SBOM.spdx.json" \
  --revision "$revision" \
  --target "$target" \
  --source-date-epoch "$epoch"
cp "$stage/SBOM.spdx.json" "$output_dir/$name.spdx.json"
printf '%s\n' \
  'LOCAL RELEASE CANDIDATE — NOT FOR DISTRIBUTION' \
  'A project license has not been approved. Do not publish or redistribute this archive.' \
  > "$stage/NOT-FOR-DISTRIBUTION.txt"
printf 'version=%s\nrevision=%s\ntarget=%s\nsource_date_epoch=%s\n' \
  "$version" "$revision" "$target" "$epoch" > "$stage/RELEASE-METADATA.txt"

tar --sort=name --mtime="@$epoch" --owner=0 --group=0 --numeric-owner \
  -C "$work" -cf - "$name" | gzip -n > "$output_dir/$name.tar.gz"
(
  cd "$output_dir"
  sha256sum "$name.tar.gz" "$name.spdx.json" > "$name.sha256"
  sha256sum -c "$name.sha256"
)
printf 'Local release candidate: %s/%s.tar.gz\n' "$output_dir" "$name"
