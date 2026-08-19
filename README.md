# ctx

**Codebase intelligence and context engine for AI coding agents.**

Documentation: **[https://halloffame12.github.io/CTX](https://halloffame12.github.io/CTX)** · [docs overview](https://halloffame12.github.io/CTX/docs)

`ctx` builds a queryable code graph for a project — files, symbols, signatures
and imports — and turns it into compact, relevance-ranked context for AI tools
and for humans. It speaks **MCP** over stdio, so coding agents (Claude, Cursor,
Copilot, opencode and any MCP client) can call it directly.

```
ctx init                      index the project
ctx doctor                    inspect the project + index health
ctx search "create_user"      find symbols
ctx deps src/app.py           what a file imports (and what imports it)
ctx impact create_user        who would break if I change this?
ctx context "add OAuth"       build a ranked context package for a task
ctx changed                   files & symbols changed since HEAD
ctx diff                      semantic diff of symbols between refs
ctx skeleton src/models.py    body-less structural skeleton
ctx watch                     keep the graph in sync while editing
ctx mcp                       run the MCP server over stdio
```

## Why does it exist?

AI coding tools get the codebase wrong in predictable ways: they hallucinate
file paths, read whole directories into context, miss ripple effects, and waste
tokens. `ctx` addresses that with:

- **Incremental code graph** — a SQLite database (`.ctx/index.db`) of files,
  symbols and dependency edges, updated only where files changed.
- **Honest dependency resolution** — relative imports, Python dotted modules,
  Rust `use` paths and Go imports resolve to real files *or are marked
  external/unresolved*. `ctx` never fabricates internal edges it can't prove.
- **Skeletons over dumps** — structural context (signatures, types, exports)
  without bodies, so a model sees the shape of a codebase in a fraction of the
  tokens.
- **Explainable ranked context** — keyword + hub + recency + path + git scoring
  picks the handful of files that matter for a task, and tells you *why*.
- **Impact analysis** — cycle-safe BFS over the dependency graph, grouped into
  direct / indirect / tests / unknown buckets.
- **Git-aware change tracking** — symbol-level diffs between refs, not just
  file status.

`ctx` is **local, fast, private and offline**: nothing is sent anywhere, no
code is executed, no telemetry.

## Install ctx

The `ctx` command is one native, compiled Rust binary — install it through
whichever ecosystem you already use:

| Method | Command |
| --- | --- |
| **macOS / Linux** (Homebrew) | `brew tap halloffame12/CTX && brew install ctx` |
| **Windows** (Winget) | `winget install halloffame12.CTX` |
| **Windows** (Scoop) | `scoop bucket add ctx https://github.com/halloffame12/scoop-ctx && scoop install ctx` |
| **Node.js** (npm) | `npm install -g ctxai-cli` |
| **Run without installing** | `npx ctxai-cli --version` |
| **Rust** (cargo) | `cargo install ctxai-cli` |
| **Direct** | `curl -fsSL https://ctx.dev/install.sh \| sh` (Unix) or `irm https://ctx.dev/install.ps1 \| iex` (Windows) — or grab a binary from [GitHub Releases](https://github.com/halloffame12/CTX/releases) |

> The installer scripts currently live in the repository (`scripts/install.sh`,
> `scripts/install.ps1`). Until `ctx.dev` is live you can run them directly:
>
> ```bash
> curl -fsSL https://raw.githubusercontent.com/halloffame12/CTX/main/scripts/install.sh | sh
> ```
>
> ```powershell
> irm https://raw.githubusercontent.com/halloffame12/CTX/main/scripts/install.ps1 | iex
> ```

Every method installs the **same** binary, verified against SHA-256 checksums
published with each release. No Rust toolchain is required except for
`cargo install`.

### Building from source

Requires Rust 1.85+ (edition 2024). On Windows, a GNU toolchain is recommended
for the bundled SQLite build:

```powershell
scoop install mingw
$env:PATH = "$env:USERPROFILE\scoop\apps\mingw\current\bin;" + $env:PATH
```

```bash
cargo build --release
cargo install --path .        # installs `ctx` to PATH
```

Version is read from the git tag / Cargo.toml:

```bash
ctx --version   # ctx 0.1.2
ctx version     # same
```

## Quick start

```bash
cd your-project
ctx init                     # creates .ctx/config.toml + indexes the project
ctx doctor                   # verify the index is current
ctx search "migrate"         # find functions/methods matching
ctx deps src/lib.rs          # what this file imports / what imports it
ctx impact create_user       # change analysis for a symbol or path
ctx context "add pagination" # context package for a task (JSON with --json)
```

## Reasoning about how much context `ctx` saves

Without `ctx`, an agent trying to understand a change typically has a human
paste a few files:

```
AI reads:
  12,000 lines of source
  ~100,000 tokens

With ctx:
  AI receives an architecture tree
  relevant symbols + signatures
  dependency / impact map
  skeletons of the files that matter
  target implementations only when relevant
```

These numbers are **illustrative**, not benchmarks — run `ctx benchmark` for
real timings on your own repository.

## CLI reference

```
Usage: ctx [OPTIONS] [COMMAND]

Commands:
  init        Create .ctx, write a default config and index the project
  doctor      Inspect the project and report the health of the ctx index
  skeleton    Show a body-less structural skeleton of a source file
  search      Search the graph for symbols or files
  symbol      Details about a symbol: definition, references, dependencies
  deps        Show what a file imports and what imports it
  impact      Analyze impact of changing a symbol or file
  context     Build a relevance-ranked context package for a task
  changed     Show symbols changed in the working tree or between refs
  diff        Semantic diff of symbols between two git refs
  schema      Print the SQLite graph schema
  benchmark   Re-run an index pass and print incremental timing
  watch       Watch the project and keep the graph in sync
  mcp         Run the Model Context Protocol server over stdio
  stats       Show index statistics (files, symbols, dependencies, db size)
  version     Print version information
  help        Print this message or the help of the given subcommand(s)

Options:
  -R, --root <DIR>  Project root (defaults to the nearest directory containing .ctx)
  -j, --json        Emit machine-readable JSON instead of human text
  -q, --quiet       Suppress non-essential output
  -v, --verbose     Enable verbose diagnostics on stderr
      --no-color    Disable ANSI colors
  -h, --help        Print help
  -V, --version     Print version
```

Per-command help always available:

```bash
ctx context --help
ctx impact --help
```

### `ctx context` — the flagship

```bash
ctx context "add Google OAuth authentication"
```

Scoring is deterministic and explainable. Every suggested file carries the
reasons it was selected:

```
Suggested files:
  src/auth/providers.ts  (score 0.91, ~180 tokens)
      + exact symbol match `OAuthProvider`
      + path matches keyword `auth`
      + imported by 4 files (hub)
      + modified in working tree
```

Budgeting (token counts are **estimates** — the heuristic is bytes/4):

```bash
ctx context "add OAuth" --max-tokens 12000
```

```
Context budget: 7,842 / 12,000 tokens (estimate)
Omitted: 13 lower-relevance files
```

If the budget cannot hold the top files, `budget_exceeded: true` is reported in
JSON rather than silently truncating. Working-tree git changes get a small
scoring bonus automatically (`--no-git` disables it).

### `ctx impact`

```bash
ctx impact UserService.updateUser --depth 5 --json
```

Results are grouped:

```
Direct dependents
Indirect dependents
Tests
UNKNOWN (unresolvable imports in affected files)
Potentially affected: N files / M symbols
```

Traversal is BFS with per-node cycle protection, bounded by `--depth`.

### `ctx doctor`

```bash
ctx doctor          # human-readable
ctx doctor --json   # machine-readable
```

Reports git presence, detected languages, framework, package manager, index
freshness (files changed on disk since indexing), SQLite health and parser
support — with a final `Status: READY / STALE / NOT INITIALIZED`.

### `ctx search`

```bash
ctx search "user"                    # case-insensitive name match
ctx search --kind struct "user"      # filter by symbol kind
ctx search --kind function User      # kind aliases: fn, const, alias
ctx search --files "src/auth"        # search file paths instead
ctx search "user" --limit 20         # default 50, clamped to 1–500
```

Symbol kinds: `function, method, class, interface, type, enum, constant,
variable, struct, trait, module, field, constructor, impl` — with aliases
`fn` → function, `const` → constant, `alias` → type. An invalid kind is
rejected (exit 2).

### `ctx stats` / `ctx version`

```bash
ctx stats            # files, symbols, dependency edges, index.db size
ctx stats --json
ctx version          # ctx 0.1.2
ctx version --json   # {"name":"ctx","version":"0.1.2"}
```

## JSON mode

Every query command has deterministic JSON output:

```bash
ctx search User --json
ctx symbol UserService --json
ctx deps src/user.ts --json
ctx impact UserService --json
ctx context "add OAuth" --json
ctx changed --json
ctx diff --json
ctx doctor --json
ctx init --json
ctx benchmark --json
ctx schema --json
ctx stats --json
ctx version --json
```

Invariants: **stdout is JSON only** — no ANSI codes, no decorations, no
progress bars. Diagnostics go to stderr. Errors exit non-zero with a message on
stderr.

## MCP server

```bash
ctx mcp                      # speaks MCP over stdio
ctx -R /path/to/project mcp  # or target a project root explicitly
npx ctxai-cli mcp            # no install required
```

The server implements JSON-RPC 2.0 over line-delimited stdio:

- `initialize`, `ping`, `notifications/initialized`, `tools/list`, `tools/call`
- `prompts/list` and `resources/list` are served (empty by default)
- unknown methods → `-32601`, invalid JSON → `-32700`, tool errors surface as
  `isError: true` results
- **stdout carries protocol messages only**; all logs go to stderr

Tools exposed:

| Tool | Purpose |
| --- | --- |
| `ctx_project` | project overview (root, git, counts) |
| `ctx_search` | symbol / file search (with `kind` + `files` filters) |
| `ctx_skeleton` | body-less structural skeleton of a file |
| `ctx_symbol` | definition, methods, references, deps of a symbol |
| `ctx_dependencies` / `ctx_dependents` | outgoing / incoming imports |
| `ctx_impact` | change-impact analysis (`symbol` or `path`, `depth`) |
| `ctx_context` | ranked context package (`task`, `include_bodies`, `max_tokens`) |
| `ctx_changed` | files & symbols changed since a ref |
| `ctx_diff` | symbol-level diff between refs (single base resolves to its merge-base with HEAD) |
| `ctx_stats` | index statistics (files, symbols, dependencies, db size) |

### opencode

Add `ctx` as an MCP server (see opencode's MCP configuration docs):

```json
{
  "mcpServers": {
    "ctx": {
      "command": "npx",
      "args": ["-y", "ctxai-cli", "mcp", "-R", "/absolute/path/to/project"]
    }
  }
}
```

### Claude / Claude Desktop

```json
{
  "mcpServers": {
    "ctx": {
      "command": "npx",
      "args": ["-y", "ctxai-cli", "mcp", "-R", "/absolute/path/to/project"]
    }
  }
}
```

### Cursor

Settings → MCP → Add → type `command`, then:

```json
{
  "command": "npx",
  "args": ["-y", "ctxai-cli", "mcp", "-R", "/absolute/path/to/project"]
}
```

### VS Code (Cline / Roo / Continue)

Add an MCP server of type `stdio`:

```
command: npx
args: -y ctxai-cli mcp -R /absolute/path/to/project
```

> `npx` form requires no global install. If `ctx` is already on your PATH,
> replace `"command": "npx", "args": ["-y", "ctxai-cli", ...]` with
> `"command": "ctx", "args": ["mcp", ...]`.
```

## Supported languages

| Language | Files | Symbols | Dependencies | Skeleton |
| --- | --- | --- | --- | --- |
| TypeScript | ts, tsx, mts, cts | functions, methods, classes, interfaces, enums, constants, fields, modules, type aliases | `import` / `require` / dynamic `import()` / re-exports; `./`, `../`, `@/` aliases, bare specifiers | bodies elided, docs kept |
| JavaScript | js, jsx, mjs, cjs | same as TS | same as TS | bodies elided |
| Python | py, pyi, pyw | functions, methods, classes, constants, module-level vars | `import x`, `from x import y`, relative imports | bodies elided |
| Rust | rs | functions, structs, enums, traits, impl blocks, methods, consts | `use` — `crate::`, `self::`, `super::` | impl/fn bodies elided, struct/enum kept |
| Go | go | functions, methods, types, interfaces, consts | package `import` paths | fn/method bodies elided, types kept |

Partially-broken files degrade gracefully: tree-sitter recovers, valid symbols
are still extracted, and the file is reported in `parse_errors` for the
`ctx init` summary instead of aborting the index.

## Security

- Indexing **never** executes project code or evaluates scripts.
- Paths are validated before use: any user-supplied path that lexically escapes
  the project root is rejected (`path is outside the project root`), including
  through MCP (e.g. `../../../../etc/passwd`).
- No telemetry, no network calls, no external AI APIs. The engine is fully
  local and offline.

## Privacy

`ctx` reads source files, writes a local SQLite index under `.ctx/`, and spawns
`git` for change queries. Nothing leaves the machine.

## Configuration

`.ctx/config.toml` is written on `ctx init`:

```toml
[index]
exclude = ["node_modules", "target", ".git", ".ctx", "dist", "build", "vendor"]

[context]
max_tokens = 12000
max_files = 25
include_bodies = false

[watch]
enabled = true
debounce_ms = 200
```

## Architecture

```
src/
  parser/     tree-sitter extractors (Rust, TypeScript/JS, Python, Go) + resolver
  graph/      SQLite code graph: database, symbols, dependencies, impact
  indexing/   scanning (gitignore-aware), hashing, incremental reindex
  context/    skeletons, ranking, context package builder
  git/        diff & changed tracking via the `git` binary
  mcp/        JSON-RPC server, tools, protocol types
  commands/   CLI + MCP command implementations
```

## Development

```bash
cargo build
cargo test --lib --test integration --test skeleton   # unit + integration + golden
cargo clippy --all-targets --all-features -- -D warnings
cargo fmt --check
```

GitHub Actions CI runs fmt, clippy (`-D warnings`, all targets/features), tests
and release builds on Linux / macOS / Windows. The release workflow publishes
prebuilt binaries (6 targets) with a `checksums.txt` for each tag, and the
npm workflow publishes the `ctxai-cli` meta package + 6 platform packages with
provenance. `package-validation.yml` keeps all packaging in sync with
`Cargo.toml`. See [RELEASE.md](RELEASE.md) for the full release checklist.

Packaging lives in `packaging/` (Homebrew tap, Winget, Scoop) and `packages/`
(npm). Release tooling is in `scripts/` (`build-release.sh`,
`verify-release.sh`, `generate-checksums.sh`, `install.sh`, `install.ps1`,
`update-homebrew.sh`, `update-package-manifests.sh`).

> Note: `cargo test --bin ctx` may be blocked by Windows Application Control
> policy on some machines (os error 4551); the library, integration and golden
> suites above cover the behavior.

## Roadmap

- Serverside of `ctx doctor`: expose diagnostics via MCP.
- Column/method-level references and more static symbol-reference resolution.
- Config-file reading for Node/TypeScript path aliases (tsconfig `paths`).
- Cargo / Go workspace awareness for module-boundary imports.
- `ctx context` incremental "focus files" (files you've asked the agent to
  touch) to steer ranking.

## License

MIT — see [LICENSE](LICENSE).