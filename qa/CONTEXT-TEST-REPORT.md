# CONTEXT-TEST-REPORT.md

Context engine test + fix for `ctx context`.

## Summary

Built five synthetic full-stack repos (20, 100, 500, 2,000, 10,000 files) covering
15 realistic dev tasks (auth, billing, caching, workers, analytics, schema
changes, etc.), ran `ctx context "<task>"` against each, and scored precision /
recall / F1 / critical-recall / token-efficiency against a ground-truth set of
files per task. Then fixed the underlying engine failures and re-ran the whole
benchmark with the fixed binary.

Final result: **critical recall 0.931 -> 0.992** across 66 tasks that have a
critical file (50/66 fully recovered -> 65/66). Recall 0.709 -> 0.789, F1
0.507 -> 0.536, precision held flat (0.431 -> 0.429) while the previously-missed
critical files were recovered. Token efficiency stayed positive on critical
tasks (1.097 -> 0.884). Full Rust suite: **100 tests pass**; `cargo fmt --check`
clean; `cargo clippy --all-targets` clean.

## What was tested

`ctx-bench.py` runs `ctx context "<task>" --json` for 15 tasks x 5 repo sizes
(75 task-runs) and computes per-run metrics. Ground truth per repo/task is the
set of files the change plausibly touches. Before/after baselines:
`baseline_pre_tokenfix.json` (original engine) vs `after.json` (fixed engine).

Key metric definitions:
- precision / recall / F1 over selected files vs ground truth
- critical_recall: did every *critical* file (the one(s) a correct fix must touch)
  make it into the selected set — the metric that best predicts "the AI makes the
  right change"
- token_efficiency: how many tokens returned per relevant file found

## Bugs found and fixed

### 1. Naive substring matching pulled filler files into the top-25

Reproduce: `ctx context "Change User schema."` on the 10k repo. A warehouse
filler `WarehouseResult1004` scored 4.0 (path + symbol matched the letter
sequence "user" mid-word: "houser"+"esult"), flooded the package, and crowded
`src/modules/auth/auth.service.ts` and `src/config/env.ts` out of the top-25.

Root cause: symbol/path matching used raw `contains()` substring tests, so any
identifier containing the keyword's letters anywhere matched.

Fix (`src/context/ranking.rs`): added `word_tokens` (splits camelCase, digits,
and punctuation into tokens), a `TokenMatch` enum (None / Prefix / Extended /
Exact, `Ord`-derived), and `best_token_match`, then reworked `score_symbol`,
`symbol_reasons`, `file_reasons`, `path_keyword_bonus`, and `expand_keywords`
(synonym-group activation) to be token-aware. A keyword now only matches an
identifier when it aligns with a token (or a substantial prefix of one).

Regression test: `context_does_not_match_substrings_across_word_boundaries`.

### 2. Synonym groups over-activated on prefix collisions

Reproduce: `user` activated the `profile` group via member `userprofile`
(transitive); `authorization` prefix-matched `authApi` on the 4-char stem
"auth". Both pulled neutral files in.

Root cause: group activation and symbol matching used the same loose prefix rule.

Fix: group activation now uses `best_token_match` (Exact / substantial-prefix /
extended only), and `stem_match` requires a shared prefix of >= 5 chars
(`authenticate`/`authentication` = "authenticat" = 10 -> match; `login`/
`logistics` = "logi" = 4 -> rejected).

Regression tests: `stem_match_rejects_short_shared_runs`,
`synonym_expansion_does_not_add_irrelevant_terms`.

### 3. Import resolver never resolved dotted filenames (dependents graph broken)

Reproduce: `src/modules/auth/auth.service.ts` imports
`../../db/repositories/user.repository`, but the dependency was stored with
`target_file_id = NULL`, so `dependents_of(user.repository)` was empty and the
dependents-following pass could never add `auth.service.ts` to a "Change User
schema" package. Same for `password.service.ts`, `auth.routes.ts`,
`billing.routes.ts`.

Root cause: `probe()` in `src/parser/resolve.rs` skipped extension-probing when
`Path::extension()` was `Some(...)`. For a dotted stem like `user.repository`,
`extension()` returns `Some("repository")`, so `.ts` was never appended and the
file never resolved. (Only a known *source* extension should suppress probing.)

Fix: added `has_real_ext` (extension must be one of the probe extensions) and
append extensions with `format!("{cand}.{ext}")` so dotted stems resolve.

Regression test: `probes_dotted_filename_stems`.

### 4. Reverse/forward follow skipped already-scored files

Reproduce: `auth.service.ts` had a weak symbol hit (score 1.67) so it was in the
scored set; the dependents-following pass skipped files already present
(`if file_scores.contains_key(&tid) { continue; }`), so it never received the
reverse-follow boost and stayed below the top-25 cutoff even though it imports a
strongly-relevant file.

Fix (`src/context/builder.rs`): both follow passes now raise an already-scored
file's score to the follow value and append the reason instead of skipping it.

### 5. Forward/reverse follow expanded hubs' entire dependent tree

Reproduce: `Add analytics.` followed `analytics.api.ts` -> `client.ts` (a hub
imported by 5 files), then reverse-followed every importer of `client.ts`
(`auth.api.ts`, `user.api.ts`, `billing.api.ts`, ...) — flooding the package
with 25 unrelated files and dragging precision down.

Root cause: the follow passes iterated `file_scores` *after* earlier follow
additions, so a followed hub's dependents were expanded too.

Fix: both passes now expand only from the directly-relevant roots (symbol/path
matches >= 3.0) captured *before* any follow additions. Dependencies are still
included, but a dependency that is itself a hub no longer drags its whole
dependent tree in.

## Results (before = original engine, after = fixed engine)

Overall (75 task-runs, means):

| metric | before | after |
|---|---|---|
| precision | 0.431 | 0.429 |
| recall | 0.709 | 0.789 |
| F1 | 0.507 | 0.536 |
| critical recall (66 tasks with a critical file) | 0.931 | 0.992 |
| tasks with critR = 1.00 | 50/66 | 65/66 |
| token efficiency (critical tasks) | 1.097 | 0.884 |

Critical files recovered at repo10000 (the case that motivated each fix):

| task | critical file | critR before -> after |
|---|---|---|
| Add Google OAuth. | src/config/env.ts | 0.80 -> 1.00 |
| Add email verification. | src/db/repositories/user.repository.ts | 1.00 -> 1.00 |
| Change User schema. | src/modules/auth/auth.service.ts | 0.80 -> 1.00 |
| Add role-based authorization. | src/db/repositories/user.repository.ts | 0.67 -> 1.00 |
| Fix payment webhook. | src/modules/billing/billing.routes.ts | 0.67 -> 1.00 |

Across the four large repos (100/500/2k/10k) for those seven headline tasks:
critical recall 0.855 -> 1.000, recall 0.693 -> 0.908, F1 0.448 -> 0.545,
precision 0.343 -> 0.394 (precision *rose* once hub-flooding was capped).

The one remaining critR < 1.00 case is `repo20 Add role-based authorization.`
(critR 0.50, unchanged) — a 20-file repo where the generated ground-truth
critical file does not exist in the tree, so no engine change could recover it.

## Files changed

- `src/context/ranking.rs` — token-aware matching (`word_tokens`, `TokenMatch`,
  `best_token_match`, `stem_match` >= 5), token-aware `expand_keywords`;
  unit tests `stem_match_rejects_short_shared_runs`,
  `synonym_expansion_does_not_add_irrelevant_terms`.
- `src/context/builder.rs` — follow passes boost already-scored files; both
  passes expand only from directly-relevant roots (no hub-tree flooding).
- `src/parser/resolve.rs` — `probe()` dotted-filename extension probing
  (`has_real_ext`).
- `tests/integration.rs` — `context_does_not_match_substrings_across_word_boundaries`
  (total integration tests: 61).

## Verification commands

```
cargo test              # 100 pass (23 unit + 61 integration + 16 skeleton)
cargo fmt --check       # clean
cargo clippy --all-targets  # clean
# benchmark (container): python3 ctx-bench.py ; python3 checkcrit.py
#   before/after baselines in /tmp/ctxbench/baseline_pre_tokenfix.json, after.json
```