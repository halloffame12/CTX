# Releasing ctx

The single source of truth for the version is `Cargo.toml`. Everything else is
derived from it during the release pipeline. **Never hand-edit versions in
multiple files.**

## Prerequisites

- `NPM_TOKEN` secret set in GitHub repo settings (an npm access token with
  publish rights for the `@ctx` scope).
- GitHub repo `halloffame12/CTX` with a `gh` authenticated environment.
- For winget submission: a Microsoft account + the `wingetcreate` CLI.

## Checklist

1. **Update CHANGELOG.md** with the changes for this release.
2. **Bump the version** in `Cargo.toml` only (e.g. `version = "0.2.0"`).
   - Optionally sync derived files: `bash scripts/npm-sync-version.sh`.
3. **Run the full quality gate:**
   ```bash
   cargo fmt --check
   cargo clippy --all-targets --all-features -- -D warnings
   cargo test --lib --test integration --test skeleton
   ```
4. **Commit** the changes (`feat: v0.2.0` or similar) and push to `main`.
   CI (`ci.yml`) runs fmt/clippy/tests/build across the 3 OS matrix.
5. **Tag and push:**
   ```bash
   git tag v0.2.0
   git push origin v0.2.0
   ```
   - `release.yml` runs: version-consistency gate → builds all 6 targets →
     checksums → GitHub Release with auto-generated notes.
6. **Verify release artifacts** locally (optional but recommended):
   ```bash
   gh release download v0.2.0 -D dist
   bash scripts/verify-release.sh 0.2.0
   ```
7. **Publish npm** (triggered automatically on `release` → `npm.yml`, or run
   manually via workflow_dispatch with the version):
   - publishes `@halloffame12/cli-linux-x64`, `-arm64`, `-darwin-x64`, `-arm64`,
     `-win32-x64`, `-arm64`, then `@halloffame12/cli` (all with provenance).
   - post-install job runs `npx @halloffame12/cli@<v> --version`.
8. **Update Homebrew formula:**
   ```bash
   bash scripts/update-homebrew.sh     # requires dist/checksums.txt
   ```
   Commit the new `packaging/homebrew/tap/Formula/ctx.rb` (and push the tap
   repo if you keep a separate `halloffame12/CTX` tap).
9. **Update WinGet + Scoop manifests:**
   ```bash
   bash scripts/update-package-manifests.sh
   ```
   - Commit `packaging/winget/*` and `packaging/scoop/*`.
   - Submit `packaging/winget/` to microsoft/winget-pkgs via `wingetcreate
     update --urls <url> --version <v>` (uses `checksums.txt`).
   - Publish `packaging/scoop/bucket/` to your scoop-bucket repo.
10. **Verify installations** (each needs its ecosystem installed):
    ```bash
    cargo install ctx-cli && ctx --version
    npm install -g @halloffame12/cli && ctx --version
    npx @halloffame12/cli --version
    brew install ctx && ctx --version          # after tap update
    winget install ctx && ctx --version        # after winget-pkgs merge
    scoop install ctx && ctx --version         # after bucket update
    curl -fsSL https://github.com/halloffame12/CTX/releases/latest/download/ctx-linux-x86_64 -o ctx && chmod +x ctx && ./ctx --version
    ```
11. **Publish release notes** — the release workflow generates them; confirm
    they include the install block and checksums reference.

> Note: brew/winget/scoop registries are NOT updated automatically by CI —
> they are human-review gates. Only claim a package is available after it has
> actually been published and verified.

## Version-consistency gate

`release.yml` and `npm.yml` each fail-fast if the tag / requested version does
not match `Cargo.toml`. `package-validation.yml` runs on every push/PR that
touches packaging and checks Cargo + npm + Homebrew + Winget + Scoop agree.