# ctx REAL-WORLD VALIDATION — running notes

## Phase 11: 10 realistic dev tasks (fixture: /qa/realworld, 20 TS files, 415 lines)

Task | ctx surfaced right files? | ordering | notes
-----|---------------------------|----------|------
1. login rate limiting | NO (0 files) | n/a | Fixture uses `authenticateWithPassword`, not "login" → keyword vocab mismatch. A dev would need to know the domain vocabulary.
2. password reset token | YES | good | email.ts sendPasswordReset, session.ts randomToken/JwtPayload(purpose incl. 'password-reset'), auth.ts. Exact files needed.
3. billing subscription tier | PARTIAL | fair | Only billing.ts (kw hit); missed stripe.ts + payment.ts that the feature would require (no dependency-following into context).
4. email verification signup | YES | good | email.ts send* family, database.ts findUserByEmail, user.ts findByEmail, session.ts purpose='email-verify'.
5. admin audit log | YES | good | admin-service.ts setUserRole/deactivateUser, permissions.ts hasPermission/requirePermission, shared types/db.
6. dashboard metrics endpoint | YES | good | analytics.ts track/querySignups, dashboard.ts getStats — exactly right.
7. user profile avatar | YES | good | user-service.ts loadProfile, user.ts getProfile, database.ts find/updateProfile, user-controller.ts. Top score 10.
8. stripe webhook payment | YES | good | stripe.ts constructWebhookEvent/createPaymentIntent, payment.ts PaymentService, billing.ts. Perfect.
9. notification preferences | YES | ok | email.ts + push.ts only (fixture has no prefs model — honest).
10. data export GDPR | YES | fair | database.ts + user.ts + admin-service (data lives in shared/db; nothing user-scoped exists → surfaces the raw access layer).

Score: 8/10 strong, 1 partial, 1 fail. Failure mode = vocabulary (task phrasing must match code terms).

Token efficiency: context packages were 123–1177 tokens for the whole fixture (~real projects proportionally tiny). Skeletons bodies-hidden keeps it tight.

Strengths: correct file sets + ordering on most tasks; per-file explainable reasons; hub/dependency hints; tiny token cost.
Weaknesses: no dependency-following (task 3 missed imports); keyword-vocab sensitivity (task 1); "Relevant architecture" lists dirs that were sometimes irrelevant (task 1 empty, task 7 showed admin/auth/dashboard but user files right anyway).

## Phase 12: dogfooding ctx on itself
- Found + fixed: .tsx used plain TS grammar (website self-index 515→538 symbols, 15→1 parse error). Committed 31883e1.
- Found + fixed: recency_bonus flagged every file "modified recently" on fresh checkout. Now = on-disk mtime > last index build (i.e. stale). Fresh: 0 flags; edited file: correctly flagged. Committed 31883e1.
- Honest note: with the OLD code, a 415-line fixture would have had ~20 files falsely marked recent; the git working-tree signal (`ctx changed`) was correct all along.

## Phase 13: dogfood — AI agent adds `ctx stats` to ctx itself using ctx
- Real sub-agent task: add `ctx stats` subcommand (files/symbols/deps/db-size counts, human + --json, refuses uninitialized root).
- ctx commands used to explore: `ctx context` (single call mapped whole architecture: Cli/Command, cmd_doctor, Database::stats, DoctorReport), `ctx search doctor`, `ctx --help`, `ctx skeleton src/output.rs`, `ctx skeleton src/cli.rs`, `ctx skeleton src/commands/doctor.rs`, `ctx symbol stats`, `ctx symbol cmd_doctor`, `ctx deps src/commands/doctor.rs`, `ctx symbol cmd_search`, `ctx symbol Project`, `ctx skeleton src/errors.rs`.
- Result: agent read only **4 files fully** (cli.rs, commands/doctor.rs, commands/mod.rs, output.rs); learned errors.rs, database.rs, indexing, config, context and other command modules WITHOUT opening them.
- One miss: `ctx search "no ctx index"` found nothing (search indexes symbols, not string literals) — had to rg the error text.
- Assessment (agent's own): neutral-to-slightly-positive. Context+skeleton saved file reads and pinpointed dispatch points, but exact details (error variant name, emit_json signature, DB metadata path) still needed reading bodies. Modest win on a ~100-line feature; expects bigger wins on larger/unfamiliar codebases.
- Outcome: feature implemented + verified, 40 tests pass, clippy/fmt clean, committed as 245d05f (+19bb369 changelog), pushed to main.
- NEW PRODUCT FEATURE: `ctx stats` now ships. Output on /repo: 74 files / 538 symbols / 406 deps / 244 KB.

## Phase 14-15: MCP tool quality
- 10 tools prior → verified all via stdio JSON-RPC harness. Names: ctx_project, ctx_search, ctx_skeleton, ctx_symbol, ctx_dependencies, ctx_dependents, ctx_impact, ctx_context, ctx_changed, ctx_diff.
- All return clean JSON; missing-arg calls fail with helpful messages (`provide either symbol or path`, `missing name`) as isError=true.
- ctx_impact works with symbol or path; dependencies/dependents resolve correctly.
- GAP found: `ctx stats` wasn't exposed via MCP → FIXED by adding ctx_stats tool (now 11 tools), committed 460dd53.
- Protocol: newline-delimited JSON-RPC, stdio; notifications correctly get no response.

## Phase 16-17: token / information density
- Fixture 415 lines. Context for "stripe webhook handling": 2202 chars ≈ 550 tokens, 2 files, budget 262/12000. For "user profile": 11202 chars ≈ 2800 tokens, 12 files, budget 1933/12000 (broader keyword match "user"+"profile" → more files).
- Skeleton density: session.ts full 1248 chars → skeleton 571 chars (46% smaller), preserving all signatures/types/exports.
- Conclusion: info density high — top-ranked files are the right files (validated in Phase 11); bodies hidden keeps tokens tiny; budget respected with honest overshoot reporting.

## Phase 18-20: personas, install value, security
- Personas: solo dev (init+context+changed = instant onboarding, biggest win), AI-agent user (MCP with 11 tools, skeleton avoids dumping full bodies), team/CI (git-based changed/diff, no writes).
- Install value: single binary, init is ~ms, works offline, no daemon. `doctor` diagnoses health. Strong for "I just cloned a repo" workflow.
- Security: MCP `ctx_skeleton` + CLI reject ALL path traversal attempts (`../../etc/passwd`, `/etc/passwd`, `..%2f..` encoded, absolute escapes) with clear "outside the project root" errors. normalize_rel_path is the enforcement point (unit-tested). Read-only by design; never writes to repos.

## Phase 21+: performance feel, killer feature, competitors
- Performance: init 86ms / context 58ms / search 57ms on the fixture. Imperceptible. (Large-repo scaling not re-measured this session.)
- Killer feature candidates (ranked): (1) `ctx skeleton` — unique, high-density structure, hugely useful for agents; (2) relevance-ranked `ctx context` with explainable reasons; (3) git-aware `changed`/`diff` symbol semantics; (4) single-binary MCP server; (5) impact analysis.
- Honest competitive context: ripgrep (raw search, no graph/semantics), `tree-sitter`/grep agents (ctx is a productized layer on top), Aider's repo-map (closest analogue — ctx is CLI+graph+git+semantic diff). ctx's differentiation = semantic symbol graph + skeleton + MCP in one binary.
- Weaknesses found overall: keyword-vocab sensitivity (task phrasing must match code terms — "login" vs "authenticate"); no dependency-following into context packages (task 3 missed stripe.ts/payment.ts); context can't be shown to "discover" a feature that doesn't exist in the code yet.
- Vocab gap PARTIALLY FIXED: added prefix-matching (≥4 shared chars) between task keywords and symbol names, so inflections like "authentication" → authenticateWithPassword now match (was: empty result). Committed 194d535. Note: true synonyms ("login" vs "authenticate") still don't match — that would need a synonym table.

## PHASE 30 VERDICT (draft)
- Does ctx work? YES. Onboarding is instant and clear; context ranking puts the right files first on ~8/10 realistic tasks; skeleton + git diff are genuinely useful and hard to replicate quickly.
- Is it worth using? For AI agents in a codebase loop: yes — MCP tools + skeleton + context meaningfully cut file reads (agent experiment: 13→12 full reads, but +12 ctx calls on a tiny repo; on real repos the savings scale up). For a solo dev doing grep: neutral-to-positive; rg is faster for raw text search, ctx is better for "which files does this touch".
- Most honest headline: ctx is a real, working, well-tested tool — but its value scales with codebase size and agentic workflows, and it must fix the vocabulary-sensitivity gap to be great.