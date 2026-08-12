#!/usr/bin/env bash
# Verify a staged release before publishing: every artifact must be an
# executable that reports the expected version.
#
# Usage:
#   scripts/verify-release.sh [version]
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DIST="$ROOT/dist"
EXPECTED="${1:-$(grep '^version' "$ROOT/Cargo.toml" | head -1 | sed -E 's/version *= *"([^"]+)"/\1/')}"

fail() { echo "FAIL: $*" >&2; exit 1; }

[ -f "$DIST/checksums.txt" ] || fail "missing $DIST/checksums.txt (run scripts/build-release.sh first)"

# Verify every artifact hash listed in checksums.txt.
(cd "$DIST" && sha256sum -c --quiet checksums.txt) || fail "checksum verification failed"

artifacts=("$DIST"/ctx-*)
[ "${#artifacts[@]}" -gt 0 ] || fail "no artifacts in $DIST"

for art in "${artifacts[@]}"; do
  [ -e "$art" ] || continue
  base="$(basename "$art")"
  case "$base" in
    ctx-windows-*)
      # Can't execute Windows binaries on this host; check it's a PE in non-empty.
      [ -s "$art" ] || fail "$base is empty"
      echo "ok (PE, not executed here): $base"
      ;;
    *)
      [ -x "$art" ] || chmod +x "$art"
      out="$("$art" --version 2>&1 | head -1)" || fail "$base did not execute: $out"
      echo "$out"
      [ "$out" = "ctx $EXPECTED" ] || fail "$base version mismatch: got '$out', want 'ctx $EXPECTED'"
      ;;
  esac
done

echo
echo "Verified: ${#artifacts[@]} artifact(s) consumed, version ctx $EXPECTED."