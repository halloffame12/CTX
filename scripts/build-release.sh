#!/usr/bin/env bash
# Build the full release matrix and emit named artifacts + checksums.
#
# Usage:
#   scripts/build-release.sh            # native build for this host
#   CROSS=true scripts/build-release.sh # cross-compile every platform (needs `cross`)
#
# Output: dist/<artifact>  +  dist/checksums.txt
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DIST="$ROOT/dist"
VERSION="$(grep '^version' "$ROOT/Cargo.toml" | head -1 | sed -E 's/version *= *"([^"]+)"/\1/')"
CROSS_BAY="${CROSS:-false}"

# target -> portable artifact name
declare -A NAMES=(
  [x86_64-unknown-linux-gnu]=ctx-linux-x86_64
  [aarch64-unknown-linux-gnu]=ctx-linux-aarch64
  [x86_64-apple-darwin]=ctx-macos-x86_64
  [aarch64-apple-darwin]=ctx-macos-aarch64
  [x86_64-pc-windows-msvc]=ctx-windows-x86_64.exe
  [aarch64-pc-windows-msvc]=ctx-windows-aarch64.exe
)

if [ "$CROSS_BAY" = "true" ]; then
  TARGETS=(x86_64-unknown-linux-gnu aarch64-unknown-linux-gnu \
           x86_64-apple-darwin aarch64-apple-darwin \
           x86_64-pc-windows-msvc aarch64-pc-windows-msvc)
else
  HOST="$(rustc -vV | sed -n 's/^host: //p')"
  TARGETS=("$HOST")
fi

echo "ctx v${VERSION} — building ${#TARGETS[@]} target(s)"
mkdir -p "$DIST"
rm -f "$DIST"/ctx-* "$DIST/checksums.txt"

for target in "${TARGETS[@]}"; do
  if [ "$CROSS_BAY" = "true" ]; then
    cross build --release --target "$target"
  else
    cargo build --release --target "$target"
  fi
  name="${NAMES[$target]}"
  bin="target/$target/release/ctx"
  [ -f "$bin.exe" ] && bin="$bin.exe"
  cp "$bin" "$DIST/$name"
  echo "built $name"
done

echo "generating checksums..."
(cd "$DIST" && sha256sum ./ctx-* > checksums.txt)
cat "$DIST/checksums.txt"

echo
echo "artifacts:"
ls -1 "$DIST"/ctx-*
echo "done."