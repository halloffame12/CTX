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
- `resolve_target` no longer matches wrong-language files for symbol targets
  (language-aware candidate filtering).
- `symbols`/`dependencies` counts in `ctx doctor` now come from live DB stats.
- Watch-mode rename/delete hygiene and clean shutdown verified.
