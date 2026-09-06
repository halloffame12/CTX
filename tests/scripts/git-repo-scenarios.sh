#!/bin/bash
# Build a git repo covering every scenario, leaving the repo in a state where
# the LAST scenario (mixed staged/unstaged + renames) is active for comparison.
# Each scenario is applied cumulatively; the harness snapshots git status and
# ctx output after each phase and compares.
export HOME=/root
set -e

REPO=$1
rm -rf "$REPO"
mkdir -p "$REPO/src" "$REPO/bin" "$REPO/docs"
cd "$REPO"

git init -q
git config user.email "qa@example.com"
git config user.name "QA"
git symbolic-ref HEAD refs/heads/main

# ---- initial committed state ----
cat > src/app.ts <<'EOF'
export function app(): string {
  return "app";
}
export function helper(): string {
  return "helper";
}
EOF
cat > src/models.ts <<'EOF'
export interface User {
  id: number;
  name: string;
}
export function getUser(id: number): User {
  return { id, name: "u" };
}
EOF
cat > src/utils.ts <<'EOF'
export const version = "1.0.0";
export function add(a: number, b: number): number {
  return a + b;
}
EOF
cat > README.md <<'EOF'
# Repo
Some docs.
EOF
printf '\x89PNG\r\n\x1a\n\x00\x00\x00\rIHDR\x00\x00\x00\x01\x00\x00\x00\x01\x08\x06\x00\x00\x00\x1f\x15\xc4\x89\x00\x00\x00\nIDATx\x9cc\x00\x01\x00\x00\x05\x00\x01\r\n\xb4\x00\x00\x00\x00IEND\xaeB`\x82' > bin/logo.png
git add -A
git commit -qm "initial"
echo "PHASE: baseline committed"
git status --porcelain=v1