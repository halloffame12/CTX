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
   - publishes `ctxai-linux-x64`, `-arm64`, `-darwin-x64`, `-arm64`,
     `-win32-x64`, `-arm64`, then `ctxai-cli` (all with provenance).
   - post-install job runs `npx ctxai-cli@<v> --version`.
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
    cargo install ctxai-cli && ctx --version
    npm install -g ctxai-cli && ctx --version
    npx ctxai-cli --version
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

## Code signing

As of `0.1.6` release binaries are **not** code-signed. That is acceptable to
users on stock Linux, macOS (with the standard Gatekeeper "verify developer"
prompt) and Windows (with Smart App Control off), but it **blocks** users on
Windows with Smart App Control / WDAC enforced and adds friction on hardened
macOS machines. Before signing can ship, the maintainer must provision
credentials; this is tracked separately from releases.

Planned signing (each step is a CI job in `release.yml`, gated on secrets
existing so an unsigned release still ships):

1. **Windows (Authenticode)** — use [Azure Trusted Signing] (low cost, no
   hardware token, CI-friendly). Additions to `release.yml`:
   - secrets: `AZURE_TENANT_ID`, `AZURE_CLIENT_ID`, `AZURE_CLIENT_SECRET`,
     `AZURE_TRUSTED_SIGNING_ACCOUNT`, `AZURE_TRUSTED_SIGNING_CERT_PROFILE`.
   - after `Stage artifact`, run Trusted Signing's `SignClient` (or
     `osslsigncode sign`) against `ctx-windows-*.exe`; timestamp using the
     configured RFC 3161 URL.
2. **macOS (Developer ID + notarization)** — requires an Apple Developer
   account. Additions:
   - secrets: `APPLE_CERTIFICATE` (base64 `.p12` and
     `APPLE_CERTIFICATE_PASSWORD`), `APPLE_TEAM_ID`.
   - `codesign --options runtime` each `ctx-macos-*` binary, then
     `xcrun notarytool submit` and `stapler` (the Apple Notary GitHub Action
     can be used).
3. **Verify** — extend `scripts/verify-release.sh` to confirm signatures
   (`Get-AuthenticodeSignature` / `codesign -dv`). Update installer scripts to
   log signed vs unsigned status, and update `SECURITY.md` accordingly.

Until the secrets exist, releases remain unsigned; the signing jobs are written
with `if:` conditions on the presence of the relevant secret so the pipeline
never blocks on them.

[Azure Trusted Signing]: https://learn.microsoft.com/azure/trusted-signing/