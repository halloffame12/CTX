# CTX — INIT + DOCTOR COMPLETE TEST REPORT

**Date:** 2026-08-15
**Binary under test:** `ctx 0.1.1` (release, rebuilt from `C:\startup\ctx` @ HEAD)
**Environment:** container `ctxqa` (Alpine/Rust), `/repo` bind-mounted to the host workspace
**Scope:** `ctx init` (16 scenarios / 20 checks), `ctx doctor` (10 scenarios), plus full existing regression suite.

---

## Result

| Area | PASS | FAIL | UNVERIFIED | BLOCKED |
|---|---|---|---|---|
| `ctx init` scenarios | 16 | 0 | 0 | 0 |
| `ctx init` checks | 20 | 0 | 0 | 0 |
| `ctx doctor` scenarios | 10 | 0 | 0 | 0 |
| Regression suite (debug) | 67 tests | 0 | 0 | 0 |
| Regression suite (release) | 67 tests | 0 | 0 | 0 |
| `cargo fmt --check` | clean | — | — | — |
| `cargo clippy --all-targets` | 0 warnings | 0 | — | — |
| MCP e2e (handshake, ctx_project, ctx_search) | 3/3 | 0 | — | — |

**Overall: ALL INIT + DOCTOR SCENARIOS PASS; ALL FIXES VERIFIED WITH REGRESSION TESTS.**

---

## 1. `ctx init` — 16 scenarios / 20 checks

| # | Scenario | Result |
|---|---|---|
| 1 | Empty directory → init creates `.ctx`, exits 0 | PASS |
| 2 | Inside a git repo → init works, git detected | PASS |
| 3 | Non-git directory → init works (no git) | PASS |
| 4 | Already initialized → "Index updated", state unchanged | PASS |
| 5 | Repeated init ×3 → symbols stable, no duplicate state | PASS |
| 6 | Nested subdir inside another repo → own `.ctx`, none at parent | PASS |
| 7 | Path with spaces → works | PASS |
| 8 | Unicode path (`prøjéct-ünïcode-日本語`) → works | PASS |
| 9 | Invalid path (regular file as dir) → non-zero exit, no `.ctx` created | PASS |
| 10 | Missing directory → creates the whole path, then inits | PASS |
| 11 | Read-only directory (non-root) → fails cleanly, `I/O error: Permission denied` | PASS |
| 12 | Partial init (empty `.ctx` dir) → creates config + db | PASS |
| 13 | Corrupt index (garbage) → plain init fails exit 1; `--force` recovers | PASS |
| 14 | Truncated db → plain init fails exit 1; `--force` recovers | PASS |
| 15 | Missing db → init creates it | PASS |
| 16 | Stale index → reindex picks up new symbol, idempotent | PASS |

### Bugs found & fixed (init)

1. **Raw SQLite crash on corrupt index (no remedy hint).** Plain `ctx init` on a corrupt `.ctx/index.db` printed a bare `error: SQLite error: file is not a database` with no way forward.
   - **Fix:** `src/commands/init.rs` — `corrupt_index_hint()` wraps the non-`--force` `run_index` error (both JSON and human paths): `"...appears corrupt; run 'ctx init --force' to rebuild it"`.
   - **Regression test:** `init_hints_force_on_corrupt_index` (also asserts `--force` recovers).

2. **Cryptic error when `.ctx` is a regular file.** `error: I/O error: File exists (os error 17)`.
   - **Fix:** `src/commands/init.rs` — explicit check at top of `cmd_init`: `"...exists but is not a directory — remove it and run 'ctx init' again"`.
   - **Regression test:** `init_errors_clearly_when_ctx_is_a_file`.

## 2. `ctx doctor` — 10 scenarios

| # | Scenario | Result |
|---|---|---|
| 1 | Healthy project → READY, exit 0 | PASS |
| 2 | Missing index (no `.ctx`) → NOT INITIALIZED, exit 1 | PASS |
| 3 | Corrupted index (garbage) → problem identified, exit 1 | PASS |
| 4 | Missing database (`.ctx` but no `index.db`) → NOT INITIALIZED, exit 1 | PASS |
| 5 | Corrupted db (truncated) → problem identified, exit 1 | PASS |
| 6 | Stale index → STALE, exit 1 | PASS |
| 7 | Missing configuration → no crash, still diagnoses | PASS |
| 8 | Invalid configuration → problem identified, exit 1 | PASS |
| 9 | Nonexistent project dir → exit 1 | PASS |
| 10 | Incomplete init (`.ctx` only, no db) → NOT INITIALIZED, exit 1 | PASS |

### Bugs found & fixed (doctor)

3. **Raw crash on corrupt db.** `ctx doctor` on garbage/truncated `index.db` died with `error: SQLite error: ...` and produced **no report at all** — exactly the failure mode a health check must never have.
   - **Fix:** `src/commands/doctor.rs` — `Database::open` + stats wrapped in a closure; failure → `database = Some(DatabaseStatus{healthy:false, ...})`, warning `"...unreadable or corrupted; run 'ctx init --force' to rebuild the index"`, status **CORRUPT**. Human printer shows `✗ .ctx database unreadable or corrupted` in the Index section. Exit stays 1 (Unhealthy).
   - **Regression tests:** `doctor_identifies_corrupt_database_without_crashing`, `doctor_identifies_truncated_database_without_crashing`.

4. **Raw crash on invalid config.** `doctor` on a syntactically broken `.ctx/config.toml` dumped a bare TOML parse error with no report.
   - **Fix:** `src/commands/doctor.rs` — `Config::load` result handled gracefully (`config_ok` flag); failure → warning `"could not read .../config.toml: <err>; using defaults"`, status **CONFIG**. Exit 1. Index stats still reported (they read the db, not the config).
   - **Regression test:** `doctor_reports_invalid_config_instead_of_crashing` (also asserts index still healthy).

5. **Missing config reported silently as READY.** `doctor` on a healthy index with a deleted config printed READY with no warning (fixed by warning even though READY remains correct: the index is the health subject).
   - **Fix:** warning `"...config.toml missing; using defaults — run 'ctx init' to rewrite it"`, status stays READY, exit 0.
   - **Regression test:** `doctor_warns_when_config_is_missing`.

6. **Nonexistent project dir gave a misleading recommendation.** It printed "NOT INITIALIZED — run `ctx init`" without noting the directory itself was gone (git discovery may itself fail there).
   - **Fix:** warning `"project directory does not exist: <path>"` emitted before git discovery.
   - **Regression test:** `doctor_warns_when_project_directory_missing`.

### New `ctx doctor` statuses (documented)
- `READY` — healthy, current (exit 0)
- `STALE` — db readable, symbols out of date (exit 1)
- `CORRUPT` — db unreadable/corrupted, rebuild via `ctx init --force` (exit 1)
- `CONFIG` — config invalid, fix `.ctx/config.toml` (exit 1)
- `NOT_INITIALIZED` — no `.ctx`/db (exit 1)

Precedence: `NOT_INITIALIZED > CORRUPT > CONFIG > READY/STALE`.

`doctor()` is now `pub` for testability; `cmd_doctor` exit behavior unchanged (non-READY → `CtxError::Unhealthy`, report on stdout, no `error:` prefix).

## 3. Regression suite

Added **7 new integration tests** (`tests/integration.rs`) covering every fix above. Full suite:

- Unit: 21 passed; Integration: 37 passed; Skeleton: 9 passed — **debug AND release**.
- `cargo fmt --check`: clean. `cargo clippy --all-targets`: 0 warnings/errors.

## 4. MCP e2e sanity (post-fix)

Handshake (`initialize`), `tools/call ctx_project`, `tools/call ctx_search` all return correct JSON; `ctx_project` reports root `/tmp/mat3`, 3 files, 5 symbols, 1 dep, git repo. No regressions from doctor/init changes.

## 5. Remaining limitations / UNVERIFIED

- **Live watch (`ctx watch`)** remains untested (no scenario; requires long-running subprocess + real file events). Not touched by this task.
- **Read-only-as-root** (container runs as root, so a read-only dir fails differently for root): read-only behavior was only verified as `nobody`. Not a defect.
- **`ctx init` hint vs. truly-foreign `index.db`**: the `--force` hint is gated on `.ctx/index.db` existing; a db whose open fails for non-corruption reasons still gets the hint text (acceptable over-hint).
- **Missing-config is warning-only, status READY, exit 0**: deliberate (index is the health subject). If you want config presence to be health-gating, that is a one-line status change — not done here to avoid changing documented exit semantics.
- Harness artifacts (`init-scen.sh`, `doctor-scen.sh`, etc.) live in the container `/tmp` and host temp dir; not committed.
- Report and all prior deliverables (incl. `CLI-COMPLETE-TEST-REPORT.md`) are **untracked** — nothing committed.

## Files changed (this task)
- `src/commands/doctor.rs` — graceful corrupt-db / invalid-config / missing-config / missing-dir handling + CORRUPT/CONFIG statuses (rewrite of `doctor()` + printer branches).
- `src/commands/init.rs` — `.ctx`-as-file check + `corrupt_index_hint` helper.
- `tests/integration.rs` — 7 new regression tests.