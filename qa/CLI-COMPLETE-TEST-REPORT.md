# CLI COMPLETE TEST + FIX — Final Report

**Version tested:** ctx 0.1.1 (`--version` → `ctx 0.1.1`)
**Date:** 2026-08-15
**Scope:** complete CLI test + fix cycle on `ctx` 0.1.1 (Rust, `ctxai-cli` crate).
**Binaries:** debug + release, built in container `ctxqa` (`cargo build`, `cargo build --release`); installed to `/usr/local/bin/ctx`.

---

## Summary

| Metric | Result |
| --- | --- |
| Commands discovered | 16 + `help` (`init, skeleton, search, symbol, deps, impact, context, changed, diff, schema, benchmark, watch, mcp, doctor, stats, version`) |
| Global flags | `-R/--root`, `-j/--json`, `-q/--quiet`, `-v/--verbose`, `--no-color`, `-h/--help`, `-V/--version` |
| CLI test cases run | **55** |
| PASS | **55** |
| FAIL | **0** |
| Unit tests | **21 pass** (debug) / **21 pass** (release) |
| Integration tests | **30 pass** (debug) / **30 pass** (release) |
| Skeleton tests | **9 pass** |
| `cargo fmt --check` | clean |
| `cargo clippy --all-targets` | 0 warnings / 0 errors |
| MCP e2e (stdio JSON-RPC) | verified: handshake, tools/list (11 tools), ctx_stats, ctx_diff, prompts/list, resources/list, unknown-method error |

### Fixes applied (2 code + 7 docs)

| # | Command / Artifact | Original failure | Root cause | Fix | Regression test | Retest |
| --- | --- | --- | --- | --- | --- | --- |
| F1 | `ctx diff <single-ref>` | `ctx diff main` showed the base branch's post-fork changes as `Removed` (opposite of documented behavior) | `src/git/diff.rs:45` used `base.unwrap_or("HEAD")` directly; single-ref base was never resolved to merge-base | Single-ref base now resolved to `git merge-base <base> HEAD`; user-typed label preserved for display; git ops use the resolved commit. Two-ref and no-arg behavior unchanged. Help text updated (`src/cli.rs`), MCP ctx_diff description updated (`src/mcp/tools.rs`) | `single_ref_diff_uses_merge_base_with_head` in `tests/integration.rs` (fork test: feature's own change reported, base's post-fork change excluded; two-ref diff still shows both) | PASS |
| F2 | `ctx mcp` in uninitialized dir | `ctx mcp` silently auto-created `.ctx/index.db` in any directory, contradicting the "point it at an indexed project" docs and masking misconfiguration | `src/cli.rs` Mcp arm used `Project::open(...).or_else(fallback)` which auto-initialized | Mcp arm now uses the same `open()` path as other read commands: refuses uninitialized roots with `error: no \`ctx\` index found at <dir>/.ctx — run \`ctx init\` first`; no `.ctx` created | (verified live in container, `verify-mcpfix.sh`) | PASS |
| F3 | README CLI reference | `Usage: ctx [OPTIONS] <COMMAND>` wrong (command is optional); `stats`/`version` missing; `ctx --version # ctx 0.1.0` stale | Doc drift vs 0.1.1 implementation | Usage line fixed to `[COMMAND]`; `stats` + `version` added; version example → 0.1.1; JSON-mode list + stats/version; new `ctx search` section (kind aliases `fn`/`const`/`alias`, limit clamp 1–500) | n/a (docs) | n/a |
| F4 | README MCP section | `ctx_stats` tool missing from tools table; `prompts/list` + `resources/list` omitted from protocol list; `ctx_diff` description stale | Doc drift | Added ctx_stats row (11 tools), added prompts/resources methods, updated ctx_diff description to match merge-base fix | n/a (docs) | n/a |
| F5 | `server.json` (MCP registry manifest) | `ctx_stats` absent; ctx_diff description stale | Same drift | Added ctx_stats entry; updated ctx_diff description | n/a (docs) | n/a |
| F6 | `packages/npm/cli/README.md` | Claimed "exposes 10 tools" and listed 10, omitting ctx_stats | Same drift | → 11 tools, ctx_stats added | n/a (docs) | n/a |
| F7 | website docs/commands page | `ctx stats` and `ctx version` sections missing; metadata description omitted them | Same drift | Added both sections + updated metadata description. Diff merge-base claim now matches implementation (kept) | website `npm run build` clean | PASS |
| F8 | website homepage | `jsonLd softwareVersion: "0.1.0"` stale | Same drift | → `"0.1.1"` | website build | PASS |
| F9 | README `ctx search --kind` etc. | (added) | — | see F3 | n/a | n/a |

---

## Test matrix (55 CLI cases)

All run against a fresh git project with a realistic Python package (`src/pkg/{a,b,__init__}.py`, `b.py` uses `from .a import ...`) in container `ctxqa`.

| Command | Cases | Result |
| --- | --- | --- |
| `init` | human, `--json` | PASS |
| `doctor` | human, `--json` | PASS |
| `search` | query, `--json`, `--kind function`, `--kind fn` (alias), `--kind bogus` (rejected, exit 2), `--files`, `--limit 500` (max clamp), `--limit 0` (clamped) | PASS |
| `symbol` | found, `--json`, missing (informational message, exit 0) | PASS |
| `deps` | relative-import resolve, `--json`, `--incoming`, `--outgoing` | PASS |
| `impact` | human, `--json`, `--depth 5`, `--depth 20` (max clamp) | PASS |
| `context` | human, `--json`, `--max-tokens`, `--include-bodies` | PASS |
| `changed` | clean, dirty (worktree), `--json`, `--ref HEAD`, `--sync` | PASS |
| `diff` | no args, same-ref, `--json`, single-ref (merge-base) | PASS |
| `schema` | human, `--json` | PASS |
| `benchmark` | human, `--json` | PASS |
| `stats` | human, `--json` | PASS |
| `version` | `version`, `version --json`, `--version` | PASS |
| `help` | `help`, `help search`, no-args overview | PASS |
| `mcp` | uninitialized dir refuses (exit 1, no `.ctx` created); e2e handshake below | PASS |
| errors | missing arg (exit 2), `-R` nonexistent (exit 1), unknown command (exit 2), no index (exit 1) | PASS |
| `skeleton` | human, `--stats` | PASS |

## MCP e2e (stdio JSON-RPC, live)

- `initialize` → capabilities + serverInfo `{name:"ctx", version:"0.1.1"}`, protocolVersion 2025-06-18
- `tools/list` → **11 tools** (`ctx_project, ctx_search, ctx_skeleton, ctx_symbol, ctx_dependencies, ctx_dependents, ctx_impact, ctx_context, ctx_changed, ctx_diff, ctx_stats`); input schemas show `limit {maximum:500, minimum:1}`, `depth {maximum:20}`, `max_tokens {maximum:100000, minimum:128}`
- `tools/call ctx_stats` → valid JSON (`files:3, symbols:5, dependencies:1, db_size:45056, root`)
- `tools/call ctx_diff` (base=main, head=main) → valid empty diff; description documents merge-base behavior
- `prompts/list` → `{"prompts":[]}` (implemented, empty)
- `resources/list` → `{"resources":[]}` (implemented, empty)
- unknown method → `{"error":{"code":-32601,"message":"method not found: bogus/method"}}`
- uninitialized dir → server refuses to start with clear error, no `.ctx` created

## Notes / non-issues investigated

- `ctx symbol <missing>` and `ctx search <no-match>` exit 0 with an informational message (not an error) — consistent design, matches "no match is not a failure" convention; JSON returns `[]`.
- `deps` on a bare Python import (`from a import ...`) resolves via project-root probing (strict Python absolute-import semantics) → correctly `External`/absent when the module isn't at root. Relative imports (`from .a import ...`) resolve to the sibling file. Not a bug; the test that initially "failed" used an unrealistic layout.
- `watch` requires an interactive TTY / blocking loop; covered structurally (source review + `--help`) rather than a blocking live run.
- Documentation claims for merge-base (`ctx diff main` shows only your branch's changes) now match implementation (F1) — the one functional doc bug found is **fixed**, not just documented.

## Deliverables

- `CLI-COMMAND-INVENTORY.md` — per-command inventory: args, flags, defaults, JSON, MCP tool mapping, documented-example table, classification, doc locations.
- This report — fixes, regression results, test matrix, MCP verification.

## Distribution status (unchanged from inventory; not in scope of fixes)

- npm `ctxai-cli` 0.1.1, crates.io `ctxai-cli` 0.1.1 — current.
- Homebrew tap & Scoop bucket stale at 0.1.0; winget PR #417409 open; `ctx.dev` HTTP 000 (DNS). Not addressed here.