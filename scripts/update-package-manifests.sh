#!/usr/bin/env bash
# Regenerate release-time hashes in the winget + scoop manifests from
# dist/checksums.txt (produces resolvable manifests after a real release).
#
# Usage: scripts/update-package-manifests.sh
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
CHK="$ROOT/dist/checksums.txt"

[ -f "$CHK" ] || { echo "missing $CHK — run scripts/build-release.sh first" >&2; exit 1; }

sha() { grep -m1 "  $1$" "$CHK" | awk '{print $1}'; }

# ---- winget installer manifest -------------------------------------------------
W="$ROOT/packaging/winget/CTX.installer.yaml"
x64="$(sha ctx-windows-x86_64.exe)"
arm="$(sha ctx-windows-aarch64.exe)"
sed -i -E \
  -e 's/^(ReleaseDate: ).*/\1'"$(date -u +%Y-%m-%d)"'/' \
  -e "0,/(InstallerSha256: )[0-9a-f]{64}/s//\1$x64/" \
  -e "s/(InstallerSha256: )[0-9a-f]{64}/\1$arm/" \
  "$W"
echo "winget: CTX.installer.yaml updated"

# ---- scoop manifest ---------------------------------------------------------
S="$ROOT/packaging/scoop/ctx.json"
node - "$S" "$x64" "$arm" <<'EOF'
const fs = require("fs");
const [ , file, x64, arm ] = process.argv;
const p = JSON.parse(fs.readFileSync(file, "utf8"));
p.architecture["64bit"].hash = x64;
p.architecture.arm64.hash = arm;
fs.writeFileSync(file, JSON.stringify(p, null, 2) + "\n");
EOF
cp "$S" "$ROOT/packaging/scoop/bucket/ctx.json"
echo "scoop: ctx.json updated"