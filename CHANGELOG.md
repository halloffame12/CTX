# Changelog

All notable changes to `ctx` are documented here.

## 0.1.2

### Fixed
- Symbol references are now symbol-level, not file-level: a symbol's references
  list only the files that actually import that symbol (verified for Rust,
  Python, Go, TypeScript and JavaScript). Previously, `Message` referenced every
  importer of its containing module (`P2-1`).
- TypeScript/JavaScript named imports now record their imported symbols. The
  tree-sitter-typescript grammar marks no `import_clause` field, so
  `child_by_field_name("import_clause")` silently returned `None` and every
  JS/TS import lost its symbol — degrading all JS/TS symbol references to
  file-level. The parser now locates the clause node by kind (`P2-5`).
- `ctx impact` resolves an ambiguous symbol to its production definition when
  one exists, instead of the test double (`P3-1`).
- A file that fails to parse now yields a bounded, declaration-only skeleton
  (80 lines / 12 KiB) instead of dumping the whole source (`P2-2`).
- `ctx context` caps follow-only files (dependencies/dependents of relevant
  files with no direct signal) at 6, so a hub can no longer flood the package
  or starve a genuinely-needed direct match (`P2-3`).
- `ctx changed` now reports only the symbols that actually changed (added /
  modified / removed with status), instead of every symbol in a changed file
  (`P2-4`).
- `token_estimate` renamed to `is_estimate` across the codebase (`P3-2`).

### Tests
- 109 tests green: 23 unit + 69 integration + 17 skeleton, including new
  regression tests for every fix above.

## 0.1.1

### Added
- `ctx stats` — index statistics (files, symbols, dependency edges, `index.db`
  size) with `--json` support; also exposed as the `ctx_stats` MCP tool.
- `ctx context` expands task keywords through synonym groups and prefix-matches
  against symbol names, and follows one hop of a strongly-relevant file's
  imports into the package (see Fixed below).

### Fixed
- `.tsx` files parsed with the TSX (JSX-aware) grammar instead of plain
  TypeScript, yielding real JSX symbols instead of a syntax-error symbol.
- `ctx context` recency measured against on-disk mtime vs the last index build
  — no more false "modified recently" flags on a fresh checkout.
- `ctx context` surfaces the right files for inflected task vocabulary
  ("login" → `authenticate`, "authentication" → `authenticateWithPassword`).
- `ctx context` no longer prints an empty "Relevant architecture" section when
  nothing matched.
- `resolve_target` language-aware candidate filtering; `ctx init --force` full
  rebuild and corrupt-`index.db` recovery; read commands refuse uninitialized
  roots; crates.io package slimmed to 59 files / 80 KiB.

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
  size) with `--json` support; also exposed as the `ctx_stats` MCP tool.

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
- `ctx context` now prefix-matches keywords against symbol names, so task
  vocabulary inflections ("authentication" vs `authenticateWithPassword`)
  still surface the right files instead of returning nothing.
- `ctx context` expands task keywords through synonym groups ("login" also
  matches code that says `authenticate`, "avatar" also matches `picture`), so
  task phrasing no longer has to match code vocabulary exactly.
- `ctx context` follows one hop of a strongly-relevant file's imports and
  includes those dependencies in the package ("add a subscription tier" now
  surfaces billing.ts *and* the stripe/payment clients it uses), with an
  explicit `imported by a relevant file (dependency)` reason.
- `ctx context` no longer prints an empty "Relevant architecture" section when
  nothing matched the task.
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
