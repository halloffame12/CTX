# GIT-TEST-REPORT.md

Git integration test + fix for `ctx changed` / `ctx diff`.

## Summary

Built a fresh Git repo covering every change type, then compared `ctx changed`
against `git status --porcelain` and `ctx diff` against `git diff --name-status`
across all scenarios plus the extra states (empty repo, large diff, merge
conflict, detached HEAD, renames, staged/unstaged/mixed, binary, whitespace).

Final result: **36/36 comparison checks pass**, matching real git state.
Full Rust suite: **95 tests pass** (21 unit + 58 integration + 16 skeleton) in
debug and release; `cargo fmt --check` clean; `cargo clippy --all-targets` clean.

## What was tested

`tests/scripts/git-repo-scenarios.sh` builds the corpus; `tests/scripts/git-comparison-harness.py`
runs git + ctx and asserts equality. Ground truth is parsed ONLY from git
itself (`git status --porcelain=v1`, `git diff --name-status`, `git ls-files`),
so a PASS means ctx matches real git state.

Scenarios (`ctx changed --json` vs `git status`):
- new file (untracked)
- modified file (unstaged)
- staged + unstaged mixed on one file (MM)
- staged new file
- deleted file (unstaged)
- deleted file (staged)
- rename via `git mv` (staged)
- rename in worktree only (not staged)
- whitespace-only change
- binary file added
- large diff (100 new files)
- clean empty repo
- merge conflict state (UU)
- detached HEAD (twice, after a second commit)

Diff comparisons:
- `ctx diff <base>` (single-ref, worktree) vs `git diff --name-status <base>`
- `ctx diff <base> <head>` (two-ref) vs `git diff --name-status <base> <head>`
- `ctx changed --ref <ref>` vs `git diff --name-status <ref>` + untracked files

## Bugs found and fixed

### 1. `ctx diff <ref>` included untracked files (`git diff` does not)

Reproduce: create an untracked file, run `ctx diff HEAD`. It appeared as "A",
but `git diff HEAD` never reports untracked files (they are only visible to
`git status`).

Root cause: `changed_files(repo, Some(rev))` unconditionally appended
`git ls-files --others --exclude-standard` results as status "A", and the
single-ref diff path used that same function. This made `ctx diff` show files
that `git diff` does not.

Fix (`src/git/changed.rs`): added an `include_untracked: bool` parameter.
Status-like callers (`ctx changed`, `ctx context`, MCP context) pass `true`;
diff semantics (`symbol_diff`) pass `false`, so `ctx diff` matches `git diff`.

Regression tests:
- `single_ref_diff_excludes_untracked_files`
- `changed_files_untracked_flag_matches_git_diff_semantics`

### 2. Renamed files reported all symbols as Added

Reproduce: `git mv src/a.ts src/renamed.ts` (pure rename). `ctx diff HEAD`
reported every symbol as "Added" even though `git diff` shows `R100` with zero
content change. For a rename+edit it mislabeled the edited symbol as Added
instead of Modified.

Root cause: `ChangedFile` only carried the new path. `symbol_diff` read the old
source from the new path (`repo.show(base, "src/renamed.ts")`), which does not
exist at the base commit, so the old symbol set was empty and every symbol
looked new.

Fix:
- `ChangedFile` now has an `old_path: Option<String>` populated for renames
  (porcelain `R  old -> new` and diff `R100\told\tnew`, including the C copy
  case) in both `changed.rs` and the two-ref `diff_between` in `diff.rs`.
- `symbol_diff` reads the old source from `old_path` when present, so a pure
  rename reports no symbol changes and a rename+edit reports the edited symbol
  as Modified (verified against `git diff` semantics).

Regression tests:
- `renamed_staged_file_reports_rename_status` (old_path populated)
- `pure_rename_reports_no_symbol_changes`
- `rename_with_edit_reads_old_path_for_symbol_diff`

### 3. Pre-existing: single-ref diff now resolves base to merge-base with HEAD

While testing two-ref vs single-ref diffs, the single-ref path had no explicit
merge-base behavior. This change (resolving a single base ref to its
merge-base with HEAD so a single-ref diff shows only the current branch's
changes) was already covered by `single_ref_diff_uses_merge_base_with_head`.
It is documented in `src/git/diff.rs` and the MCP `ctx_diff` tool description.

## Harness notes

Two comparison failures during development were harness bugs, not ctx bugs:
- `.strip()` on `git status` output ate the leading space of the first
  porcelain line, truncating the first path char.
- `normalize_status` did not handle similarity scores (`R100`, `R099`).

Both were fixed in `tests/scripts/git-comparison-harness.py`; `ctx` needed no
change for those cases.

## Files changed

- `src/git/changed.rs` — `include_untracked` param; `old_path` on `ChangedFile`;
  porcelain + `--name-status` rename parsing.
- `src/git/diff.rs` — diff uses merge-base for single ref; `old_path` in
  `diff_between`; old source read from rename source path.
- `src/commands/context.rs`, `src/mcp/tools.rs` — updated `changed_files` calls;
  MCP `ctx_diff` description.
- `tests/integration.rs` — 5 new regression tests (total integration tests: 58).
- `tests/scripts/git-repo-scenarios.sh`, `tests/scripts/git-comparison-harness.py`
  — reproducible corpus + comparison harness.

## Verification commands

```
cargo test              # 95 pass
cargo test --release    # 95 pass
cargo fmt --check       # clean
cargo clippy --all-targets  # clean
python3 tests/scripts/git-comparison-harness.py  # 36/36 pass
```