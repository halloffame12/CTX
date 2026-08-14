# Changelog

All notable changes to `ctx` are documented here.

## [Unreleased]

### Added
- `ctx init` — incremental code graph over a project (SQLite-backed).
- Symbol extraction via tree-sitter for Rust, TypeScript/JavaScript, Python and Go.
- Dependency resolution: relative imports, Python dotted modules, Rust `use`,
  Go imports; unresolved references are classified internal/external.
- `ctx search`, `ctx symbol`, `ctx deps` (incoming/outgoing), `ctx impact`
  (BFS change analysis), `ctx skeleton` (body-less structure).
- `ctx context` — relevance-ranked context packages with keyword, hub and
  recency scoring.
- `ctx changed` / `ctx diff` — git working-tree and ref-to-ref symbol diffs
  (delegates to the `git` binary; never writes to repos).
- `ctx watch` — notify-based incremental reindex.
- `ctx schema`, `ctx benchmark`.
- MCP server over stdio (`ctx mcp`) with 10 tools; JSON output everywhere via
  `--json`.
- `ctx stats` — index statistics (files, symbols, dependency edges, `index.db`
  size) with `--json` support.

## 0.2.0 (Phase 2)

### Added
- `ctx doctor` (+ `--json`) — project/health audit: git, language, framework,
  package manager, index freshness (stale file counts), SQLite health,
  parser support, READY/STALE status.
- Skeleton golden tests (`tests/skeleton.rs`) and expanded `tests/integration.rs`
  corpus: modified/deleted/renamed files, syntax-error tolerance, empty files,
  duplicate symbols/imports, incremental idempotence.
- CI: `cargo fmt --check`, `cargo clippy -D warnings`, tests and release builds
  across Linux/macOS/Windows; GitHub release workflow with cross-OS binaries
  and checksums.

### Changed
- `ctx impact` — UNKNOWN bucket for edges whose resolved dependency is absent
  or unresolved; `resolve_target` + resolved-target fields in JSON.
- `ctx context` — explainable per-file reasons, `--max-tokens` budget with
  honest overshoot reporting (`budget_exceeded` in JSON), working-tree
  git-change scoring bonus (`--no-git` disables), `target_symbol`.
- Safe path handling — `normalize_rel_path` now *rejects* any path that
  lexically escapes the project root (`PathOutsideRoot`) instead of silently
  clamping; applied at the CLI and MCP boundaries (top-1 traversal guard).
- Indexing — hard parse errors are captured and reported in `ctx init`'s
  summary (`parse_errors`) instead of being silently dropped; broken files
  still yield partial symbols.
- MCP — `ctx_context` accepts `max_tokens`; tool failures surface as
  `isError: true` results with a user-facing message instead of dying or
  returning empty.

### Fixed
- `.tsx` files are now parsed with the TSX (JSX-aware) grammar instead of plain
  TypeScript, so JSX components yield their real symbols instead of a single
  syntax-error symbol (website self-index: 515 → 538 symbols, 15 → 1 parse
  error).
- `ctx context` no longer flags every file as "modified recently" on a fresh
  checkout or clone. Recency is now measured against the actual on-disk mtime
  vs the last index build (i.e. files that would show as stale in
  `ctx doctor`), so only files edited since the last index get the bonus.
- `resolve_target` no longer matches wrong-language files for symbol targets
  (language-aware candidate filtering).
- `symbols`/`dependencies` counts in `ctx doctor` now come from live DB stats.
- Watch-mode rename/delete hygiene and clean shutdown verified.
- `ctx search --kind` now rejects invalid kinds (exit 2) and normalizes aliases
  (`fn` → `function`, `const` → `constant`, `alias` → `type`) instead of
  silently returning zero matches.
- `ctx init --force` now rebuilds the entire index from scratch rather than
  only resetting the default config.
- `ctx watch --json` emits single-line JSON events (`changed`/`deleted`/`error`)
  instead of ignoring the JSON flag.
- Crates.io package size — `Cargo.toml` `include` restricts the crate to
  source, tests and docs (948 files / 690KiB → 59 files / 80KiB compressed).
- README Scoop URL now points to the real bucket (`halloffame12/scoop-ctx`).
- Read commands (`search`, `skeleton`, `symbol`, `deps`, `impact`, `context`,
  `changed`, `diff`, `schema`, `benchmark`) now refuse a nonexistent or
  uninitialized root with a clear error (`no ctx index found — run ctx init
  first`) instead of silently creating an empty index under a mistyped `-R`
  path.
- `ctx init --force` now recovers from a corrupt/incompatible `index.db` by
  removing the database files before rebuilding, instead of failing on open.
