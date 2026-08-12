# Contributing to ctx

Thanks for helping out. Here's how to work on this codebase.

## Setup

Rust 1.85+ (edition 2024). On Windows use a GNU toolchain:

```powershell
scoop install mingw
$env:PATH = "$env:USERPROFILE\scoop\apps\mingw\current\bin;" + $env:PATH
```

## Commands

```bash
cargo build                     # dev build
cargo test --lib --test integration --test skeleton   # unit + integration + golden
cargo clippy --all-targets -- -D warnings
cargo fmt --check
```

## Layout

- `src/parser/` — language extractors and the import resolver. Add a language
  by implementing `LanguageParser` (traits.rs) and registering it in
  `parser/mod.rs` + `lang.rs`.
- `src/graph/` — the SQLite schema and queries. Schema changes bump
  `SCHEMA_VERSION` and add a migration in `database.rs`.
- `src/context/` — skeleton generation, scoring, and context package assembly.
- `src/git/` — git integration (spawns `git`; never mutates repos).
- `src/mcp/` — JSON-RPC server and the tool registry. New tools are added in
  `tools.rs` (definition + implementation) and wired in the CLI in `cli.rs`.

## Conventions

- Every public type and function is documented.
- Commands return `CtxResult<()>` and emit human text via `Term`; JSON mode is
  a flag, not a separate code path.
- Keep the parser layer independent from the graph layer — parsers return
  plain `ParsedFile`/`Symbol`/`Dependency` values.
- No external code executes user input; `git` commands only read.
- Run `cargo clippy` and `cargo fmt` before opening a PR.

## Testing

Integration tests in `tests/integration.rs` build throwaway fixture projects
in the OS temp dir. Skeleton golden tests live in `tests/skeleton.rs` and pin
the exact deterministic output per language. Unit tests live in
`#[cfg(test)]` modules next to the code.

## Note for Windows

`cargo test --bin ctx` may be blocked by Application Control policy
(`os error 4551`). Use `cargo test --lib --test integration` instead; CI runs
the same commands.
