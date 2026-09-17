# CLI Command Inventory — ctx 0.1.1

Tested against: container `ctxqa` build of current HEAD (`/repo`), binary `ctx 0.1.1`
at `/usr/local/bin/ctx`. Host binary (scoop) is 0.1.0 (no `stats`, no `version` subcommand output).

Sources consulted:
- Binary: `ctx --help`, `ctx <cmd> --help` for all 16 commands
- Source: `src/cli.rs` (authoritative CLI surface)
- Docs: `README.md`, `website/src/app/docs/commands/page.tsx`,
  `website/src/app/docs/mcp/page.tsx`, `website/src/app/page.tsx`, `CHANGELOG.md`
- Reference site: https://ctx.sumitchauhan.me (not live — HTTP 000 at test time)

---

## 1. Top-level surface

```
Usage: ctx [OPTIONS] [COMMAND]        <-- actual binary. README says `ctx [OPTIONS] <COMMAND>` (command required) — MISMATCH, command is optional.
```

Global flags (appear on every subcommand — all verified in help):
| Flag | Meaning | Verified |
|---|---|---|
| `-R, --root <DIR>` | Project root (defaults to nearest dir containing .ctx) | yes |
| `-j, --json` | Machine-readable JSON instead of human text | yes |
| `-q, --quiet` | Suppress non-essential output | yes |
| `-v, --verbose` | Verbose diagnostics on stderr | yes |
| `--no-color` | Disable ANSI colors | yes |
| `-h, --help` | Print help | yes |
| `-V, --version` | Print version (top-level only) | yes |

No subcommand aliases exist (verified: no `alias`/`visible_alias` in `cli.rs`).
No CTX_* environment variables. Only env var affecting behavior: `CI` (`src/output.rs:20`,
`CI=true` or `CI=1` disables color). `--no-color` and `CI` both verified.

Exit codes (`src/main.rs`): usage errors → 2; all other errors → 1. Unknown subcommand → 2.
`ctx` with no command prints a welcome/overview and exits 0.

## 2. Command-by-command inventory

| # | Command | Args | Command-specific flags | JSON | MCP tool |
|---|---|---|---|---|---|
| 1 | `init` | `[PATH]` (default cwd) | `--force` | yes | n/a |
| 2 | `skeleton` | `<PATH>` | `--stats` | yes | ctx_skeleton |
| 3 | `search` | `<QUERY>` | `--kind <KIND>`, `--files`, `--limit <N>` (default 50) | yes | ctx_search |
| 4 | `symbol` | `<NAME>` | — | yes | ctx_symbol |
| 5 | `deps` | `<PATH>` | `--outgoing`, `--incoming` | yes | ctx_dependencies / ctx_dependents |
| 6 | `impact` | `<TARGET>` | `--depth <N>` (default 3) | yes | ctx_impact |
| 7 | `context` | `<TASK>` | `--include-bodies`, `--max-tokens <N>`, `--no-git` | yes | ctx_context |
| 8 | `changed` | — | `--ref <REF>`, `--sync` | yes | ctx_changed |
| 9 | `diff` | `[BASE] [HEAD]` | — | yes | ctx_diff |
| 10 | `schema` | — | — | yes | n/a |
| 11 | `benchmark` | — | — | yes | n/a |
| 12 | `watch` | — | — | yes | n/a |
| 13 | `mcp` | — | — | n/a | server itself |
| 14 | `doctor` | — | — | yes | n/a |
| 15 | `stats` | — | — | yes | ctx_stats |
| 16 | `version` | — | — | yes | n/a |

All 12 read/query commands accept `--json`; all verified to emit parseable JSON with no stderr
leakage. `--limit` clamps 1..500 (`search.rs:15`); `--depth` clamps 1..20 (`impact.rs`).

Symbol kinds (CLI `--kind` + MCP `kind` enum, 14): function, method, class, interface,
type, enum, constant, variable, struct, trait, module, field, constructor, impl.
CLI also accepts aliases `fn`→function, `const`→constant, `alias`→type (CHANGELOG-verified).

MCP server: 11 tools (initialize handshake, protocol 2025-06-18, capabilities
tools.listChanged:false). Tools: ctx_project, ctx_search, ctx_skeleton, ctx_symbol,
ctx_dependencies, ctx_dependents, ctx_impact, ctx_context, ctx_changed, ctx_diff, ctx_stats.

## 3. Classification

### DOCUMENTED + IMPLEMENTED
All 14 commands listed in README CLI reference (`init doctor skeleton search symbol deps
impact context changed diff schema benchmark watch mcp help`) plus `stats` and `version`
subcommands. All flags in README/website verified working:
- `ctx init sub/` nested project ✓, idempotent ✓, `--force` rebuild ✓
- `ctx search`/`--kind`/`--files`/`--limit` ✓ (limit clamps)
- `ctx symbol <name>` ✓
- `ctx deps --outgoing/--incoming` ✓ (both → Both direction)
- `ctx impact <target> --depth` ✓ (clamps)
- `ctx context <task>` with reasons, budget, `--include-bodies`, `--max-tokens`, `--no-git` ✓
- `ctx changed --ref`/`--sync` ✓
- `ctx diff [BASE] [HEAD]` ✓
- `ctx schema`, `ctx benchmark`, `ctx watch --json` (single-line JSON events) ✓
- `ctx doctor` (+`--json`) ✓, `--json` for all ✓
- MCP: initialize, ping, notifications/initialized, tools/list, tools/call ✓
- Config `max_tokens` default 12000 in `.ctx/config.toml`, exclude list (19 entries) ✓
- README documented examples (quick start block, docs pages) — all execute successfully

### DOCUMENTED + BROKEN
1. **`ctx diff` base default.** CLI help: `[BASE] Base ref (default: git merge-base with
   HEAD)`. Website commands page: "base defaults to the merge-base with HEAD, so `ctx diff
   main` shows everything your branch changed, not everything main did since the fork."
   **Implementation (`src/git/diff.rs:45`): `base.unwrap_or("HEAD")` — default is HEAD,
   NOT merge-base.** Reproduced: in a clean repo where `main` advanced past the fork point,
   `ctx diff main` (single arg) shows main's `mainOnly` change as **Removed** — the exact
   opposite of the website's claim. The MCP ctx_diff tool description ("Default base is
   HEAD.") matches the code but contradicts CLI help + website.
2. **`ctx context` budget overshoot wording.** Not broken — CHANGELOG claims
   `budget_exceeded` honest reporting; verified with `--max-tokens 50` → `budget_exceeded:
   True`, `omitted_files: 1`. (kept as a positive confirmation)
3. **README `ctx --version` example stale**: README says `ctx --version  # ctx 0.1.0` but
   binary reports 0.1.1. Website homepage JSON-LD `softwareVersion: "0.1.0"` also stale.

### IMPLEMENTED + UNDOCUMENTED
1. `ctx stats` command + `ctx_stats` MCP tool — absent from README CLI reference, absent
   from website commands page (only in CHANGELOG 0.1.1, website mcp page, homepage).
2. `ctx version` subcommand (JSON: `{"name":"ctx","version":"0.1.1"}`) — absent from README
   CLI reference and website commands page (README mentions `ctx version` in prose only).
3. MCP `prompts/list` and `resources/list` methods — implemented (return empty lists) but
   not listed in README protocol line (`initialize, ping, notifications/initialized,
   tools/list, tools/call`).
4. MCP silent auto-init: running `ctx mcp` in a NON-initialized directory silently creates
   an empty `.ctx/index.db` (no config.toml). Website says "point the server at a project
   that has been indexed first" — no mention of auto-create behavior.
5. Kind aliases (`fn`, `const`, `alias`) only documented in CHANGELOG, not CLI help or README.
6. `--limit`/`--depth` clamping rules only in CHANGELOG, not README.
7. `ctx init` auto-writes `.gitignore` with `.ctx/` (seen in `ctx changed` output as
   `[added] .gitignore`) — not documented.

### DOCUMENTED + MISSING (in docs, not code)
1. `stats` and `version` commands missing from README CLI reference block and website
   commands page (implemented in code).
2. `ctx_stats` missing from README MCP tools table (README lists 9 rows / 10 tools;
   ctx_stats present on website mcp page + homepage). Server exposes 11 tools.

## 4. Doc inconsistencies (exact locations)
- README `Usage: ctx [OPTIONS] <COMMAND>` vs binary `Usage: ctx [OPTIONS] [COMMAND]`
  (README.md, CLI reference block).
- README `ctx --version   # ctx 0.1.0` (README.md ~L98) — binary reports 0.1.1.
- Homepage `softwareVersion: "0.1.0"` (website/src/app/page.tsx:33) — stale.
- `ctx diff` merge-base claim: src/git/diff.rs:45 vs CLI help text vs website commands
  page (docs/commands/page.tsx) vs MCP ctx_diff description.
- README CLI reference and website commands page omit `stats` / `version`.
- README MCP tools table omits `ctx_stats`.
- README MCP protocol list omits `prompts/list`, `resources/list`.
- `ctx mcp` auto-init behavior undocumented.

## 5. Documented examples recorded for later testing
(All executed during this session — full outputs in QA notes. Rerun commands for regression:)

README quick start:
- `npm install -g ctxai-cli` / `npx ctxai-cli --help` / `cargo install ctx` /
  `brew install halloffame12/ctx/ctx` / scoop `ctx` (distribution — see INSTALL-TEST-REPORT.md)
- `ctx init`
- `ctx search --kind struct "user"`
- `ctx symbol UserService`
- `ctx deps src/models.py`
- `ctx impact UserService.updateUser --depth 5 --json`
- `ctx context "add pagination to the users list" --json`
- `ctx changed` / `ctx changed --ref main`
- `ctx diff main..feature`
- `ctx schema`, `ctx benchmark`
- `ctx skeleton src/app.ts --stats`
- `ctx watch` (verify single-line JSON events with `--json`)
- `ctx doctor`
- `ctx stats`

Website commands page:
- `ctx init src/` (subdir init)
- `ctx search --kind function`, `ctx search --files`
- `ctx impact --depth 2`
- `ctx context --max-tokens 4000 --include-bodies`
- `ctx diff main` (merge-base claim — known BROKEN)
- `ctx mcp` config snippets for opencode / Claude

MCP (README + website):
- opencode `plugin` config, Claude desktop config
- printf initialize + tools/list smoke test
- ctx_impact with symbol/path/neither, ctx_search kind, ctx_skeleton with_stats,
  ctx_changed ref, ctx_context max_tokens

## 6. Summary of findings
- 16 commands + help; all work. JSON everywhere. Clamps and exit codes consistent.
- 1 functional doc bug: `ctx diff` merge-base default (code uses HEAD).
- 2 stale version strings (README example, homepage JSON-LD).
- Usage-line mismatch (required vs optional command).
- 2 commands (`stats`, `version`) + 1 MCP tool (`ctx_stats`) implemented but missing from
  the two primary CLI reference documents.
- 2 MCP protocol methods implemented but undocumented.
- 1 undocumented behavioral edge: `ctx mcp` auto-creates an empty index in uninitialized dirs.

No changes made to the product during this testing session.