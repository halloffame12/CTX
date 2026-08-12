#!/usr/bin/env bash
# Stage the built release binaries into the npm platform packages.
#
# Usage:
#   scripts/stage-npm-binaries.sh          # copy from dist/ (after build-release.sh)
#   DIST=/path/to/dist scripts/stage-npm-binaries.sh
#
# Each platform package's bin/ctx(/.exe) is replaced with the matching
# artifact. Run npm-sync-version.sh first.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DIST="${DIST:-$ROOT/dist}"
NPM="$ROOT/packages/npm"

stage() { # artifact-name  dest-file
  [ -f "$DIST/$1" ] || { echo "missing $DIST/$1" >&2; exit 1; }
  mkdir -p "$(dirname "$2")"
  cp "$DIST/$1" "$2"
  chmod +x "$2" || true
  echo "staged $1 -> ${2#"$ROOT"/}"
}

stage ctx-linux-x86_64   "$NPM/linux-x64/bin/ctx"
stage ctx-linux-aarch64  "$NPM/linux-arm64/bin/ctx"
stage ctx-macos-x86_64   "$NPM/darwin-x64/bin/ctx"
stage ctx-macos-aarch64  "$NPM/darwin-arm64/bin/ctx"
stage ctx-windows-x86_64.exe "$NPM/win32-x64/bin/ctx.exe"
stage ctx-windows-aarch64.exe "$NPM/win32-arm64/bin/ctx.exe"

echo "all platform binaries staged."