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
cargo clippy --all-targets --all-features -- -D warnings
cargo fmt --check
```

## Finding an issue

New contributors are welcome to browse issues labelled
[good first issue](https://github.com/halloffame12/CTX/issues?q=is%3Aissue+is%3Aopen+label%3A%22good+first+issue%22).
Before you start work on anything, leave a comment saying you're taking it so
maintainers can avoid double-assignment. If an issue is not labelled, still
feel free to ask.

## Making a pull request

1. **Fork + branch.** Work on a feature branch (`feat/xyz`, `fix/xyz`).
2. **Match the quality gate** above before pushing: fmt, clippy (`-D warnings`,
   `--all-features`), and the full test suite must pass.
3. **Keep PRs focused.** One logical change per PR. If you're changing behavior,
   describe it and update any docs that describe reality.
4. **Review.** A maintainer will review; please respond to comments. CI runs
   the full matrix on your PR automatically.

### Commit style

Conventional commits, e.g. `feat(parser): add Go method support` or
`fix(git): resolve revision edge case`. Relevant scope tags include
`cli`, `mcp`, `parser`, `context`, `graph`, `git`, `indexing`, `website`,
`ci`, `packaging`, `docs`.

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
