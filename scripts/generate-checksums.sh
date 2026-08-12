#!/usr/bin/env bash
# Generate SHA-256 checksums for the current dist/ artifacts.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DIST="$ROOT/dist"

[ -d "$DIST" ] || { echo "no dist/ directory — run scripts/build-release.sh first" >&2; exit 1; }

(cd "$DIST" && rm -f checksums.txt && sha256sum ./ctx-* > checksums.txt)
cat "$DIST/checksums.txt"
echo "wrote $DIST/checksums.txt"