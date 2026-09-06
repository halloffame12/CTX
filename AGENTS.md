# AGENTS.md — CTX contributor guide for AI coding agents

> Read this file before touching anything in this repository. It tells you
> what CTX is, how it is built, and the invariants you must not break.

## 1. What CTX is

**CTX (`ctx`) is a local, deterministic code graph for AI coding agents.**
It indexes a repository into SQLite (`.ctx/index.db`) — files, symbols,
dependency edges — and answers the questions agents actually ask:

- *where does this symbol live?* → `ctx search`, `ctx symbol`
- *what would break if I change it?* → `ctx impact`, `ctx deps`
- *which files does this task need?* → `ctx context` (ranked, scored, explained)
- *what changed?* → `ctx changed`, `ctx diff` (symbol-level, git-aware)
- *what does this file look like?* → `ctx skeleton` (signatures without bodies)

The same engine serves two surfaces: a **CLI** (`ctx …`) and an **MCP server**
over stdio (`ctx mcp`, 11 tools: `ctx_project`, `ctx_search`, `ctx_skeleton`,
`ctx_symbol`, `ctx_dependencies`, `ctx_dependents`, `ctx_impact`,
`ctx_context`, `ctx_changed`, `ctx_diff`, `ctx_stats`).

Non-goals (by design): no embeddings, no LLM calls, no network, no telemetry,
no type checking, no executing project code. Ranking is **name-based and
deterministic** — same repo + same query = same answer.

Supported languages (tree-sitter, 5 parsers): TypeScript, JavaScript, Python,
Rust, Go. Everything else is invisible to the index.

## 2. Repository layout

```text
src/
  main.rs            # entry; maps errors to exit codes (Usage→2, else→1)
  cli.rs             # clap CLI: 16 subcommands + global flags (-R, -j, -q, -v, --no-color)
  lib.rs             # library root (all modules are pub for tests/MCP reuse)
  commands/          # one file per subcommand: init, doctor, skeleton, search,
                     #   symbol, deps, impact, context, changed, diff, schema,
                     #   benchmark, watch, mcp, stats (+ shared Project/discover_root)
  context/           # ranking (tokenize, synonyms, IDF weights, reasons), builder, skeleton
  graph/             # SQLite database, symbols, dependencies, impact BFS
  indexing/          # scanner (gitignore-aware), SHA-256 hasher, incremental reindex
  parser/            # per-language tree-sitter extractors + import resolver
  git/               # changed/diff via the `git` binary (never libgit2)
  mcp/               # JSON-RPC 2.0 protocol, server loop, tool definitions
  config.rs          # .ctx/config.toml (index/context/watch sections + defaults)
  output.rs          # Term (human/JSON, color, quiet) + emit_json
  errors.rs          # CtxError (Usage, NotInitialized, PathOutsideRoot, …)
  lang.rs            # LanguageId (5 variants)
tests/
  integration.rs     # ~70 end-to-end tests (index → query → assert)
  skeleton.rs        # 17 golden skeleton tests
packages/npm/        # cli (launcher) + 6 platform packages (binaries staged at publish)
packaging/           # homebrew tap formula, scoop manifests, winget manifests
scripts/             # install.sh, install.ps1, release helpers
website/             # Next.js docs site (must describe reality — see §7)
server.json          # MCP registry metadata (name MUST be "ctx", version = release)
Dockerfile           # reproducible container build (used by evaluators)
glama.json           # Glama ownership claim (maintainers list — do not remove)
```

## 3. Build, test, verify (run all three before any PR)

```bash
cargo build
cargo test --lib --test integration --test skeleton   # 110 tests, must all pass
cargo clippy --all-targets --all-features -- -D warnings
cargo fmt --check
```

Toolchain: Rust **1.85+**, edition **2024**. SQLite is bundled (`rusqlite/bundled`,
needs a C toolchain). Windows MSVC release builds use **static CRT**
(`-C target-feature=+crt-static` in `release.yml`) so the `.exe` runs with no
VC++ Redistributable — do not remove that flag.

## 4. Invariants (breaking any of these is a release-blocking bug)

1. **stdout purity:** in `--json` mode stdout is JSON only — no ANSI, no logs,
   no progress. All diagnostics go to **stderr**. MCP startup banners go to
   stderr; stdout carries protocol frames only.
2. **Exit codes:** `0` success (including clean "no matches" reports);
   `1` runtime failure; `2` usage error (bad flags, invalid `--kind`, unknown
   subcommand). `ctx doctor` exits non-zero on unhealthy index (CI-gatable).
3. **Path security:** any user path lexically escaping the project root
   (`../../etc/passwd`, including `..\..` Windows forms) is rejected with
   `PathOutsideRoot` — including through MCP tools. Never `root.join()` raw input.
4. **Determinism:** ranking/search/skeleton output must be byte-identical for
   identical inputs (tests assert this). No timestamps, no hash-map ordering in output.
5. **No guessing:** unresolvable imports become `external`/`unknown` edges —
   never fabricated internal edges. Impact analysis surfaces UNKNOWN bucket.
6. **Graceful degradation:** malformed source yields a bounded declaration-only
   skeleton (never body leaks, never aborts the index). Empty/oversized
   (>2 MB)/hidden/unicode/space-containing paths must not crash indexing.
7. **Error messages** must name the resource + reason + action
   (e.g. ``no `ctx` index found at <path>/.ctx — run `ctx init` first``).

## 5. Ranking engine notes (read before touching `context/`)

- `tokenize()` drops English glue words via STOP — but **code verbs
  (`add`, `create`, `make`, `use`, `implement`, …) must NEVER be stop words**
  (regression tested). Generic terms are damped by IDF weights, not deleted.
- `SYNONYM_GROUPS` maps task vocabulary (`login`→`authenticate`,
  `addition`→`add`). Add inflections as groups, never as special cases.
- Hub files get a bonus, but their dependents are capped so hubs can't flood.
- Every suggested file must carry machine-checkable `reasons`.

## 6. Versioning, packaging, release (single source of truth: `Cargo.toml`)

- `Cargo.toml` `version` is canonical. These MUST match it before any tag:
  all 7 `packages/npm/*/package.json` (incl. `optionalDependencies`),
  `server.json`, website `softwareVersion`/badges/install URLs, README examples.
- `release.yml` enforces this with a `verify-version` gate (tag == Cargo ==
  npm packages) and now runs a **functional smoke test** (init → search →
  doctor on a fixture). Do not weaken either gate.
- `packaging/*` (homebrew/scoop/winget) pins release-asset URLs **+ SHA-256
  from `checksums.txt`** — these can only be filled AFTER the GitHub release
  exists, so they update in a follow-up commit, plus the live
  `homebrew-ctx` / `scoop-ctx` tap repos.
- `npm.yml` auto-publishes all 7 packages on GitHub-release-published (if the
  trigger ever misses, `gh workflow run npm.yml --ref main -f version=X.Y.Z`).
- There is **no crates.io release** and **no Winget submission** — docs say so
  explicitly. Do not re-add `cargo install ctxai-cli` / `winget install`
  instructions without actually publishing those channels first.

## 7. Docs = reality (website truth rule)

`website/` MUST describe actual behavior. After any behavior change, update:
`website/src/app/page.tsx` (hero stats/facts/install), `docs/commands`,
`docs/install`, `docs/faq`, `docs/mcp`, `docs/config`, `docs/architecture`.
Verify with `npx tsc --noEmit` in `website/`. Never document a command, flag,
or install path you have not executed.

## 8. Glama (external listing — read-only from here)

- Listing: `glama.ai/mcp/servers/halloffame12/CTX`. Score tracks: Glama
  release, coherence, tool quality, maintenance, license, README, usage,
  `glama.json`, author verification, related servers.
- **No rebuild API and no rebuild button exist.** Glama auto-syncs git history
  and re-runs its build/introspect/score pipeline on every new commit (queue
  delay is normal). A Dockerfile failure like `failed to resolve source
  metadata for docker.io/…` with no repo logs is Glama/Docker-Hub infra flake —
  retrigger with any new commit.
- Owner-only actions (sessions, not API): claim ownership (auto via GitHub
  sign-in for personal repos), suggest related servers, Inspector trial call
  (registers "active usage"), profile completion, discussions.

## 9. Known gotchas from production debugging

- `npx` invocations each create fresh cache dirs and re-download (~10 MB);
  slowness/hangs of `npx -y ctxai-cli …` on a machine usually mean cache or
  network flake, not a ctx bug — verify with the locally built
  `./target/debug/ctx.exe` first.
- PowerShell 5.1 `| Out-String | Set-Content` pipelines can swallow native
  output; prefer direct invocation and byte-size checks when diagnosing
  "empty output" reports.
- `ctx doctor` STALE is normal in a working tree (exit 1 by design for CI).
- `benchmark` performs a real index pass and can refresh a stale `index.db`.
- `watch` runs until killed; test it with a background job + file touch.
- `cargo test --bin ctx` may be blocked by Windows Application Control
  (os error 4551) — lib/integration/skeleton suites cover behavior.
