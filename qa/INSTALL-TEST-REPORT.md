# ctx v0.1.1 — Production Installation Test Report

Date: 2026-08-15
Scope: Every installation method documented on https://ctx.sumitchauhan.me (website /docs/install) and README.md lines 52–63.
Constraint honored: no product code was modified during testing.
Test platforms: Linux x86_64 container (node v20.19.2, npm 9.2.0, cargo 1.97.1); Windows host (node v24.16.0, npm 11.13.0, cargo 1.97.0, winget v1.29.280, scoop 0.5.3, git 2.54.0).

## Status legend
PASS = works exactly as documented, correct version.
PARTIAL = works but with a caveat (stale version / extra step).
FAIL = documented command does not work.
BLOCKED = external dependency (domain/DNS or upstream registry) prevents testing.
UNVERIFIED = no hardware available to test.

## Results table

| Method | Command | Expected | Actual | Exit | Time | Status |
|---|---|---|---|---|---|---|
| npm (Linux) | `npm install -g ctxai-cli` | ctx 0.1.1 | ctx 0.1.1, clean upgrade 0.1.0→0.1.1, idempotent, uninstall clean | 0 | ~2–4s | PASS |
| npm (Windows) | `npm install -g ctxai-cli` | ctx 0.1.1 | 0.1.1 installed to AppData\Roaming\npm; `ctx.cmd --version` → 0.1.1 | 0 | 4.4s | PASS |
| npx (Linux) | `npx -y ctxai-cli --version` | ctx 0.1.1 | ctx 0.1.1; `--help` OK | 0 | ~3s | PASS |
| npx (Windows) | `npx -y ctxai-cli --version` | ctx 0.1.1 | ctx 0.1.1 | 0 | 3.7s | PASS |
| cargo (Linux) | `cargo install ctxai-cli --locked` | ctx 0.1.1 | 0.1.1 at /usr/local/cargo/bin/ctx; reinstall idempotent ("Ignored ... already installed") | 0 | 48s | PASS |
| cargo (Windows) | `cargo install ctxai-cli --locked` | ctx 0.1.1 | 0.1.1 at %USERPROFILE%\.cargo\bin\ctx.exe | 0 | 97.5s | PASS |
| GitHub release binary | download `ctx-windows-x86_64.exe` + verify `checksums.txt` | SHA256 match, runs | SHA256 `d77f4b0b…` matches; runs, help exit 0 | 0 | 6.3s dl | PASS |
| install.sh (ctx.dev) | `curl -fsSL https://ctx.dev/install.sh \| sh` | install | **HTTP 000 — ctx.dev not live** (fallback raw.githubusercontent worked: PASS) | 22 | – | BLOCKED |
| install.sh (raw fallback) | `sh install.sh` from raw.githubusercontent | install + checksum | Installs to ~/.ctx/bin/ctx-linux-x86_64, SHA256 `edec4734…` verified, 0.1.1; re-run → "already installed" exit 0; `CTX_VERSION=v9.9.9` → curl 404, exit 22, old binary intact; missing curl/wget → clean exit 1 "could not resolve the latest release version"; missing sha256sum → clean exit 1 checksum error; unsupported OS (uname=plan9) → clean exit 1 "unsupported OS" | 0 | ~10s | PASS |
| install.ps1 (ctx.dev) | `irm https://ctx.dev/install.ps1 \| iex` | install | **HTTP 000 — ctx.dev not live** (fallback raw.githubusercontent worked: PASS) | – | – | BLOCKED |
| install.ps1 (raw fallback) | `powershell -File install.ps1` | install + PATH | Installs 0.1.1 to $CTX_INSTALL_DIR\bin, adds user PATH, "✓ ctx 0.1.1"; re-run → "already installed" exit 0 | 0 | 8.4s | PASS |
| Homebrew | `brew tap halloffame12/CTX && brew install ctx` | ctx 0.1.1 | **FAILS out of the box**: "Error: Refusing to load formula halloffame12/ctx/ctx from untrusted tap halloffame12/ctx. Run `brew trust --formula halloffame12/ctx/ctx` or `brew trust halloffame12/ctx` to trust it." After `brew trust halloffame12/ctx`, install succeeds but serves **stale 0.1.0** (live tap formula is still 0.1.0) | 0 (post-trust) | – | FAIL |
| Winget | `winget install halloffame12.CTX` | ctx 0.1.1 | "No package found matching input criteria." (PR microsoft/winget-pkgs#417409 still OPEN) | -1978335212 | ~2.6s | FAIL |
| Scoop | `scoop bucket add ctx https://github.com/halloffame12/scoop-ctx && scoop install ctx` | ctx 0.1.1 | Installs + shim works, but serves **stale 0.1.0**; `scoop update ctx` → "0.1.0 (latest version)" (live bucket is still 0.1.0); hash verified | 0 | ~5s | PARTIAL |
| Smoke test | `ctx init` + `ctx doctor` | index built, READY | init exit 0 (14ms, 1 file indexed); doctor exit 0 READY, all 5 parsers OK; `ctx mcp --help` exit 0 | 0 | <1s | PASS |

## Edge cases

| Scenario | Result | Status |
|---|---|---|
| npm invalid version `ctxai-cli@99.99.99` | ETARGET "No matching version found", exit 1, nothing broken | PASS |
| npx invalid version | ETARGET, clean failure | PASS |
| cargo invalid version `--version 99.99.99` | "could not find `ctxai-cli` ... with version `=99.99.99`", exit 101 | PASS |
| install.sh invalid version `CTX_VERSION=v9.9.9` | curl 404, exit 22, existing binary intact | PASS |
| Interrupted cargo install (SIGKILL at 3s) | no binary left, no broken state | PASS |
| Interrupted install.sh (SIGKILL) | install dir contains only `bin/` (0 binaries) + orphaned `.install.tmp.*` dir | PARTIAL (minor) |
| Interrupted install.ps1 (kill job) | install dir + orphaned tmp dir, no partial exe in bin | PARTIAL (minor) |
| Double install (npm / cargo / install.sh / install.ps1) | idempotent, exit 0, "already installed" | PASS |
| Upgrade 0.1.0 → 0.1.1 (npm) | works, `ctx --version` → 0.1.1 | PASS |
| Missing dependency (install.sh, no curl/wget) | clean exit 1 + clear message | PASS |
| Missing checksum tool (sha256sum masked) | clean exit 1 checksum error | PASS |
| Unsupported OS (install.sh, uname=plan9) | clean exit 1 "unsupported OS: plan9 (linux and macOS only — use install.ps1 on Windows)" | PASS |
| Checksum/tamper verification | installed binary SHA256 matches release checksums.txt (`edec4734…`) | PASS |
| Release binary checksum | Windows exe SHA256 `d77f4b0b…` matches checksums.txt | PASS |

## Release blockers (documented methods that fail today)

1. **Winget — FAIL.** Package `halloffame12.CTX` is not in the winget repository. PR microsoft/winget-pkgs#417409 (v0.1.1 manifests) is still open. Manifest files at `packaging/winget/` (hashes `6ee688d9…`, `b3f7bbbe…`).
2. **Homebrew — FAIL as documented.** Live tap `halloffame12/homebrew-ctx` main branch still ships **v0.1.0** (repo copy `packaging/homebrew/tap/Formula/ctx.rb` is 0.1.1). Additionally, modern Homebrew refuses untrusted-tap formulas, so the documented two-liner errors with a `brew trust` prompt — the docs do not mention this step.
3. **Scoop — PARTIAL.** Live bucket `halloffame12/scoop-ctx` `bucket/ctx.json` still ships **v0.1.0** (repo copy `packaging/scoop/bucket/ctx.json` is 0.1.1). Scoop users silently get an outdated binary.
4. **ctx.dev — BLOCKED.** `https://ctx.dev` resolves to HTTP 000 (domain not live). The README documents `curl -fsSL https://ctx.dev/install.sh | sh` / `irm https://ctx.dev/install.ps1 | iex` — these fail. README already documents a raw.githubusercontent fallback, which works.

## Findings / notes

- **PATH shadowing (Windows).** With scoop ctx installed, `ctx` resolves to the scoop shim (0.1.0) even after npm/cargo install 0.1.1, because `...\scoop\shims` precedes both `%USERPROFILE%\.cargo\bin` and `%APPDATA%\npm` in PATH. A user who installs via multiple methods can end up running the older scoop binary while believing they have 0.1.1. Direct invocation of each method's binary works (verified 0.1.1).
- **Orphaned temp dirs on hard-kill.** Both install.sh and install.ps1 leave an orphaned `.install.tmp.*` directory if the process is SIGKILLed mid-download (the EXIT trap / finally block cannot run). No partial binary is ever written to the final location (downloads are atomic into tmp, then moved). Low severity.
- **Idempotency is consistent** across npm, cargo, install.sh, install.ps1 — all exit 0 on re-run with "already installed".
- **Interrupted/uninstall never leaves a broken `ctx` binary** (cargo: nothing installed; npm/install.sh/install.ps1: old binary preserved).

## Not testable in this environment (UNVERIFIED)

- macOS (x64 + arm64) installs — no Mac available. Binaries exist in the release; checksums present.
- Linux arm64 / Windows arm64 prebuilt binaries — no such hardware.
- Homebrew on a real Mac (linuxbrew tested on Linux; the untrusted-tap behavior is Homebrew-version-dependent, not OS-specific).
- Winget/Scoop/Homebrew on a completely pristine machine (host/container already had prior ctx state; baseline cleaned/restored after tests).

## Recommended fixes (product docs/repo, not the app binary)

1. Merge winget PR #417409, then re-verify `winget install halloffame12.CTX`.
2. Push 0.1.1 formula to live `homebrew-ctx` tap; add `brew trust halloffame12/ctx` to the README (or document `brew tap halloffame12/CTX` then install).
3. Push 0.1.1 manifest to live `scoop-ctx` bucket.
4. Bring `ctx.dev` live, or update README to point the primary install commands at the raw.githubusercontent fallback.
