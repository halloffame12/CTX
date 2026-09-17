# ctx — Real-World Value Test / Dogfooding Report
**Date:** 2026-08-14 · **Version tested:** 0.1.0 · **Method:** first-principles evaluation treating ctx as an unknown tool discovered by a real developer + AI agent today.

## 1. Executive verdict

**ctx works, is honest, and is worth using — with the caveat that its value scales with codebase size and agentic workflows, and it still has a real vocabulary gap.**

Rating by dimension (1–5):

| Dimension | Score | Notes |
|---|---|---|
| Onboarding / first-run experience | 5 | `--help` clear, `init` ~37ms, `doctor` useful |
| Context ranking quality | 4 | Right files first on 8/10 realistic tasks |
| Information density / token cost | 4 | Skeletons 46% smaller than source; context ~550–2800 tokens |
| MCP / agent experience | 4 | 11 tools, all verified working, graceful errors |
| Git integration | 5 | `changed`/`diff` correct, working-tree aware, read-only |
| Security | 5 | All path-traversal attempts rejected with clear errors |
| Speed | 5 | init 86ms, context 58ms, search 57ms on fixture |
| Robustness | 5 | 43 tests, clippy/fmt clean, found+fixed real bugs via dogfooding |
| Vocabulary sensitivity | 3 | "login" vs "authenticate" still a miss (fixed inflections only) |

**Overall: 4.4 / 5. SHIP IT — but publish an honest caveat about vocabulary sensitivity, and consider a synonym layer as a follow-up.**

> **Post-campaign hardening (this pass):** the two top weaknesses from the
> verdict — vocabulary sensitivity and missing dependency-following — were
> directly addressed and are now fixed:
> - **Synonym layer:** task keywords expand through synonym groups, so "login"
>   now surfaces `authenticateWithPassword`, "avatar" surfaces `picture`, etc.
>   (verified live; was 0 results before).
> - **One-hop dependency following:** "add a subscription tier" now surfaces
>   billing.ts *and* the stripe.ts/payment.ts clients it imports, with an
>   explicit `imported by a relevant file (dependency)` reason.
> - **Empty-architecture polish:** nonsense tasks no longer print an empty
>   "Relevant architecture" section.
> Result: context ranking is now 10/10 on the earlier failure cases; 48 tests
> (was 43), clippy/fmt clean, committed as `edea92e`.

## 2. What was actually tested

- **Onboarding (Ph 1-3):** fresh `--help`/`doctor`/`init`. Clear, fast, self-explanatory.
- **Real tasks (Ph 11):** 10 realistic dev tasks on a 20-file / 415-line TypeScript fixture.
  - 8/10 surfaced exactly the right files in the right order (password reset → email.ts/session.ts/auth.ts; stripe webhook → stripe.ts/payment.ts/billing.ts; avatar → user-service/user/types/db).
  - 1 partial (subscription tier: only billing.ts; missed stripe.ts/payment.ts — no dependency-following into context).
  - 1 fail (login rate limiting: 0 files — fixture says `authenticate`, task said `login`).
- **With vs without ctx (Ph 4-9):** OAuth task: with ctx surfaced 2 files (oauth.ts, config.ts), missing session.ts/auth.ts/user-controller.ts. rg without ctx found 5 files and the human read 5 files (~250 lines) to understand the flow.
- **AI agent experiments (Ph 6-7):** WITHOUT ctx: 20 tool calls, 13 full file reads, ~500 lines, plan produced. WITH ctx: 32 calls (16 ctx invocations), 12 full reads, ~570 lines. **Honest finding: on a 415-line repo, ctx added calls without reducing exploration — on real repos the savings scale up, on toy repos it's overhead.**
- **Dogfooding on ctx itself (Ph 12-13):** found and fixed 2 real bugs; a sub-agent implemented a new `ctx stats` command using ctx as its exploration tool (read only 4 files fully).

## 3. Bugs found & fixed (this campaign)

| Fix | Commit | Impact |
|---|---|---|
| `.tsx` files parsed with plain TS grammar (no JSX) | 31883e1 | self-index: 515→538 symbols, 15→1 parse error |
| Recency bonus flagged every file "modified recently" on fresh checkout | 31883e1 | now = on-disk mtime newer than last index (honest) |
| `ctx stats` command didn't exist | 245d05f | new feature (files/symbols/deps/db-size, human + JSON) |
| `ctx stats` not exposed via MCP | 460dd53 | MCP now has 11 tools |
| Keyword-vocab inflections missed ("authentication" → authenticateWithPassword) | 194d535 | prefix-matching (≥4 shared chars) added |

Also fixed earlier in this QA effort (already released): search `--kind` validation, crate bloat, `init --force` rebuild, watch NDJSON, `init --force` corrupt-DB recovery, read-commands refuse nonexistent roots.

## 4. Honest weaknesses that remain

1. **True synonyms beyond the built-in groups:** prefix-matching + synonym
   groups cover common inflections and vocabulary, but a niche term not in the
   table still misses. A configurable user synonym map or embeddings would be
   the next step.
2. **One-hop dependency following only:** context follows one level of a
   strongly-relevant file's imports. Deep transitive chains are not followed
   (by design — keeps the package small).
3. **Overhead on tiny repos:** for an agent on a <500-line project, ctx
   invocations add calls without net savings. Its value is on larger,
   unfamiliar codebases.
4. **Install friction:** distribution via npm/winget/scoop/manifests is set up
   and PRs are open, but no tagged release binary is published to crates.io yet.
   Host Windows Application Control blocked direct execution of the freshly
   built binary — container is the canonical test env.

## 5. Killer features (from a developer's POV)

1. **`ctx skeleton`** — unique, high-density, body-less structure. The single most useful command.
2. **Relevance-ranked `ctx context` with explainable reasons** — each suggestion says WHY.
3. **Semantic git `changed`/`diff`** — symbol-level diffs, not line diffs.
4. **Single-binary MCP server (11 tools)** — zero-install agent integration.
5. **`ctx impact`** — who breaks if I change this.

## 6. Final recommendation

**Ship.** The 30-phase test produced 3 new features/fixes and 2 product-quality fixes beyond the original QA scope, and this hardening pass closed the two biggest remaining gaps (vocabulary + dependency-following). The tool is genuinely useful and the dogfooding loop works. Remaining follow-ups: a configurable synonym map, verifying a tagged crates.io release, and merging the distribution PRs (winget #417409 ready-to-merge, awesome-mcp-servers #12088 MERGEABLE, mcp.so #3545 updated to 11 tools).

— *Recorded live during the campaign; all measurements in qa-notes.md; all fixes committed to main.*