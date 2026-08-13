#!/usr/bin/env bash
# Sync the version in every npm package.json + optionalDependencies from
# Cargo.toml (single source of truth). idempotent.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
VERSION="$(grep '^version' "$ROOT/Cargo.toml" | head -1 | sed -E 's/version *= *"([^"]+)"/\1/')"

pkg() { echo "$1"; } # placeholder for clarity

# meta ctxai
META="$ROOT/packages/npm/cli/package.json"
node - "$META" "$VERSION" <<'EOF'
const fs = require("fs");
const [ , file, version ] = process.argv;
const p = JSON.parse(fs.readFileSync(file, "utf8"));
p.version = version;
for (const k of Object.keys(p.optionalDependencies)) p.optionalDependencies[k] = version;
fs.writeFileSync(file, JSON.stringify(p, null, 2) + "\n");
EOF

# platform packages
for d in "$ROOT"/packages/npm/linux-x64 "$ROOT"/packages/npm/linux-arm64 \
         "$ROOT"/packages/npm/darwin-x64 "$ROOT"/packages/npm/darwin-arm64 \
         "$ROOT"/packages/npm/win32-x64 "$ROOT"/packages/npm/win32-arm64; do
  node - "$d/package.json" "$VERSION" <<'EOF'
const fs = require("fs");
const [ , file, version ] = process.argv;
const p = JSON.parse(fs.readFileSync(file, "utf8"));
p.version = version;
fs.writeFileSync(file, JSON.stringify(p, null, 2) + "\n");
EOF
done

echo "synced npm version to $VERSION"