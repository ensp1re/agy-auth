#!/bin/sh
set -eu

VERSION="${AGY_AUTH_VERSION:-0.2.0-rc.2}"
REPOSITORY="${AGY_AUTH_REPOSITORY:-ensp1re/agy-auth}"
INSTALL_DIR="${AGY_AUTH_INSTALL_DIR:-${HOME}/.local/bin}"

case "$(uname -s):$(uname -m)" in
  Linux:x86_64|Linux:amd64)
    TARGET="x86_64-unknown-linux-gnu"
    ;;
  *)
    printf '%s\n' "agy-auth: unsupported installer platform; build from source instead" >&2
    exit 2
    ;;
esac

if ! command -v curl >/dev/null 2>&1; then
  printf '%s\n' "agy-auth: curl is required by the release installer" >&2
  exit 2
fi
if ! command -v sha256sum >/dev/null 2>&1; then
  printf '%s\n' "agy-auth: sha256sum is required by the release installer" >&2
  exit 2
fi

NAME="agy-auth-${VERSION}-${TARGET}"
if [ -n "${AGY_AUTH_DOWNLOAD_BASE:-}" ]; then
  BASE="$AGY_AUTH_DOWNLOAD_BASE"
  fetch() {
    curl -fLsS "$1" -o "$2"
  }
else
  BASE="https://github.com/${REPOSITORY}/releases/download/v${VERSION}"
  fetch() {
    if curl --proto '=https' --tlsv1.2 -fLsS "$1" -o "$2" 2>/dev/null; then
      return
    fi
    if command -v gh >/dev/null 2>&1; then
      gh release download "v${VERSION}" \
        --repo "$REPOSITORY" \
        --pattern "$(basename "$1")" \
        --output "$2"
      return
    fi
    printf '%s\n' "agy-auth: download failed; private repositories require authenticated gh CLI" >&2
    exit 2
  }
fi
WORK="$(mktemp -d)"
trap 'rm -rf "$WORK"' EXIT HUP INT TERM

fetch "$BASE/$NAME.tar.gz" "$WORK/$NAME.tar.gz"
fetch "$BASE/$NAME.sha256" "$WORK/$NAME.sha256"

EXPECTED="$(awk -v file="$NAME.tar.gz" '$2 == file { print $1 }' "$WORK/$NAME.sha256")"
if [ -z "$EXPECTED" ]; then
  printf '%s\n' "agy-auth: release checksum does not name the expected archive" >&2
  exit 2
fi
ACTUAL="$(sha256sum "$WORK/$NAME.tar.gz" | awk '{ print $1 }')"
if [ "$ACTUAL" != "$EXPECTED" ]; then
  printf '%s\n' "agy-auth: release archive checksum verification failed" >&2
  exit 2
fi

tar -xzf "$WORK/$NAME.tar.gz" -C "$WORK"
install -d -m 0700 "$INSTALL_DIR"
install -m 0755 "$WORK/$NAME/agy-auth" "$INSTALL_DIR/agy-auth"

printf 'Installed agy-auth %s to %s/agy-auth\n' "$VERSION" "$INSTALL_DIR"
case ":${PATH}:" in
  *":${INSTALL_DIR}:"*) ;;
  *) printf 'Add %s to PATH before running agy-auth.\n' "$INSTALL_DIR" ;;
esac
