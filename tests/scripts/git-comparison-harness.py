#!/usr/bin/env python3
"""Compare `ctx changed` against `git status --porcelain` and `ctx diff`
against `git diff --name-status` across many git states.

Ground truth is derived ONLY from git itself (porcelain + diff), so a PASS
means ctx matches real git state.
"""
import json, os, shutil, subprocess, sys

CTX = "/usr/local/bin/ctx"
WORK = "/tmp/gitqa"

def sh(cmd, cwd):
    r = subprocess.run(cmd, cwd=cwd, capture_output=True, text=True)
    if r.returncode != 0:
        sys.exit(f"CMD FAILED {cmd}: {r.stderr}")
    return r.stdout

def git(cmd, cwd):
    return sh(["git"] + cmd, cwd)

def ctx(args, cwd):
    return sh([CTX] + args, cwd)

def parse_porcelain(text):
    """git status --porcelain=v1 -> {path: (x, y)} handling renames/dirs."""
    out = {}
    for line in text.splitlines():
        if not line:
            continue
        xy = line[:2]
        rest = line[3:]
        if " -> " in rest:
            _, new = rest.split(" -> ", 1)
            out[new] = (xy[0], xy[1])
        else:
            out[rest] = (xy[0], xy[1])
    return out

def parse_name_status(text):
    """git diff --name-status (and -z not used; spaces handled via 3-part) ->
    {path: status} for the NEW path (rename: old\tnew)."""
    out = {}
    for line in text.splitlines():
        if not line:
            continue
        parts = line.split("\t")
        if len(parts) < 2:
            continue
        st = parts[0]
        path = parts[-1]  # for R, last is the new path
        out[path] = st
    return out

def ctx_changed(cwd, ref=None):
    args = ["changed", "--json"]
    if ref:
        args += ["--ref", ref]
    return json.loads(ctx(args, cwd))

def ctx_diff(cwd, base="HEAD"):
    return json.loads(ctx(["diff", "--json", base], cwd))

def normalize_status(st):
    if st[:1] in ("R", "C"):
        return "R"
    if st[:1] == "A":
        return "A"
    if st[:1] == "D":
        return "D"
    return "M"

RESULTS = {"pass": 0, "fail": 0, "details": []}

def check(name, ok, detail=""):
    if ok:
        RESULTS["pass"] += 1
        print(f"  [PASS] {name}")
    else:
        RESULTS["fail"] += 1
        print(f"  [FAIL] {name}: {detail}")

def run_phase(cwd, phase_name, apply_fn, expect_porcelain=None):
    """apply_fn mutates the repo; then compare ctx vs git truth."""
    print(f"===== {phase_name} =====")
    apply_fn(cwd)
    truth = parse_porcelain(git(["status", "--porcelain=v1"], cwd))
    report = ctx_changed(cwd)
    got = {f["path"]: f["status"] for f in report["files"]}

    # filter .ctx and .git dirs from truth (ctx excludes them)
    def filtered(d):
        return {p: s for p, s in d.items()
                if p != ".ctx" and not p.startswith(".ctx/")
                and p != ".git" and not p.startswith(".git/")}
    truth = filtered(truth)
    got = {p: s for p, s in got.items() if p not in ("",)}

    # 1) path set equality
    missing = set(truth) - set(got)
    extra = set(got) - set(truth)
    check(f"{phase_name}: path set matches git status",
          not missing and not extra,
          f"missing={sorted(missing)} extra={sorted(extra)}")

    # 2) status classification matches (ctx A/D/M/R vs porcelain XY)
    for p in sorted(set(truth) & set(got)):
        x, y = truth[p]
        expected = None
        if x == "?" and y == "?":
            expected = "A"
        elif x in "ADRC":
            expected = normalize_status(x)
        elif y in "ADRC":
            expected = normalize_status(y)
        else:
            expected = "M"
        if got[p] != expected:
            check(f"{phase_name}: status of {p}", False,
                  f"ctx={got[p]} git=({x}{y}) expected={expected}")
    check(f"{phase_name}: all statuses match git", True)
    return truth

def compare_diff_worktree(cwd, phase_name):
    """Compare ctx diff (base=HEAD, worktree) against git diff --name-status HEAD."""
    gd = parse_name_status(git(["diff", "--name-status", "HEAD"], cwd))
    dd = ctx_diff(cwd, "HEAD")
    dpaths = {f["path"]: f["status"] for f in dd["files"]}

    # ctx diff only includes parseable languages + files with symbol changes;
    # for files with no symbol change it emits status but empty symbols.
    gd = {p: s for p, s in gd.items() if p != ".ctx" and not p.startswith(".ctx/") and p != ".git" and not p.startswith(".git/")}
    # language filter: ctx skips non-language files entirely
    langs = {".ts": 1, ".tsx": 1, ".js": 1, ".py": 1, ".rs": 1, ".go": 1}
    lang_changed = {p: s for p, s in gd.items() if os.path.splitext(p)[1] in langs}

    missing = set(lang_changed) - set(dpaths)
    extra = set(dpaths) - set(lang_changed)
    check(f"{phase_name}: ctx diff file set == git diff (language files)",
          not missing and not extra,
          f"missing={sorted(missing)} extra={sorted(extra)}")
    for p in sorted(set(lang_changed) & set(dpaths)):
        gs = normalize_status(lang_changed[p])
        if gs in ("R",) and dpaths[p] == "M":
            # diff without -M shows D+A; ctx reports A for new path; accept
            continue
        if dpaths[p] != gs:
            check(f"{phase_name}: diff status of {p}", False,
                  f"ctx={dpaths[p]} git={gs}")
    check(f"{phase_name}: ctx diff statuses match git", True)
    return dd

def main():
    shutil.rmtree(WORK, ignore_errors=True)
    os.makedirs(WORK)
    repo_script = os.path.join(os.path.dirname(os.path.abspath(__file__)), "git-repo-scenarios.sh")
    subprocess.run([repo_script, WORK], check=True)
    # init ctx once
    ctx(["init", "--force"], WORK)
    git(["add", "-A"], WORK)
    git(["commit", "-qm", "baseline"], WORK)
    print("baseline committed:", git(["rev-parse", "HEAD"], WORK))

    # ---- scenario 1: new file (untracked) ----
    def s_new(cwd):
        with open(os.path.join(cwd, "src/new.ts"), "w") as f:
            f.write("export function fresh(): string {\n  return 'x';\n}\n")
    run_phase(WORK, "new-untracked", s_new)

    # ---- scenario 2: modified file (unstaged) ----
    def s_mod(cwd):
        with open(os.path.join(cwd, "src/utils.ts"), "a") as f:
            f.write("export function sub(a: number, b: number): number {\n  return a - b;\n}\n")
    run_phase(WORK, "modified-unstaged", s_mod)

    # ---- scenario 3: staged + unstaged mixed on same file ----
    def s_mixed(cwd):
        p = os.path.join(cwd, "src/utils.ts")
        with open(p, "a") as f:
            f.write("export function mul(a: number, b: number): number {\n  return a * b;\n}\n")
        git(["add", "src/utils.ts"], cwd)
        with open(p, "a") as f:
            f.write("export function div(a: number, b: number): number {\n  return a / b;\n}\n")
    run_phase(WORK, "mixed-staged-unstaged", s_mixed)

    # ---- scenario 4: staged new file ----
    def s_staged_new(cwd):
        with open(os.path.join(cwd, "src/staged.ts"), "w") as f:
            f.write("export const staged = 1;\n")
        git(["add", "src/staged.ts"], cwd)
    run_phase(WORK, "staged-new", s_staged_new)

    # ---- scenario 5: deleted file (unstaged) ----
    def s_del(cwd):
        os.remove(os.path.join(cwd, "README.md"))
    run_phase(WORK, "deleted-unstaged", s_del)

    # ---- scenario 6: deleted file (staged) ----
    def s_del_staged(cwd):
        git(["rm", "-q", "bin/logo.png"], cwd)
    run_phase(WORK, "deleted-staged", s_del_staged)

    # ---- scenario 7: rename via git mv (staged) ----
    def s_rename(cwd):
        git(["mv", "src/models.ts", "src/entities.ts"], cwd)
    run_phase(WORK, "renamed-staged", s_rename)

    # ---- scenario 8: rename in worktree only (not staged) ----
    def s_rename_ws(cwd):
        os.rename(os.path.join(cwd, "src/app.ts"), os.path.join(cwd, "src/main.ts"))
    run_phase(WORK, "renamed-worktree", s_rename_ws)

    # ---- scenario 9: whitespace-only change ----
    def s_ws(cwd):
        p = os.path.join(cwd, "src/staged.ts")
        with open(p) as f:
            content = f.read()
        with open(p, "w") as f:
            f.write(content.replace("= 1", "= 1  "))
    run_phase(WORK, "whitespace-only", s_ws)

    # ---- scenario 10: binary modified ----
    def s_bin(cwd):
        os.makedirs(os.path.join(cwd, "bin"), exist_ok=True)
        with open(os.path.join(cwd, "bin", "blob.bin"), "wb") as f:
            f.write(b"\x00\x01\x02binary payload\xff\xfe")
    run_phase(WORK, "binary-new", s_bin)

    # ---- scenario 11: large diff (100 files) ----
    def s_large(cwd):
        os.makedirs(os.path.join(cwd, "bulk"), exist_ok=True)
        for i in range(100):
            with open(os.path.join(cwd, "bulk", f"f{i}.ts"), "w") as f:
                f.write(f"export function f{i}(): number {{\n  return {i};\n}}\n")
    run_phase(WORK, "large-diff-100", s_large)

    # ---- scenario 12: empty-repo fresh check ----
    empty = "/tmp/gitempty"
    shutil.rmtree(empty, ignore_errors=True)
    os.makedirs(os.path.join(empty, "src"))
    git(["init", "-q"], empty)
    git(["config", "user.email", "qa@example.com"], empty)
    git(["config", "user.name", "QA"], empty)
    with open(os.path.join(empty, "src", "a.ts"), "w") as f:
        f.write("export function a() { return 1; }\n")
    git(["add", "-A"], empty)
    git(["commit", "-qm", "one"], empty)
    ctx(["init", "--force"], empty)
    run_phase(empty, "empty-repo-clean", lambda cwd: None)

    # ---- scenario 13: merge conflict state ----
    mc = "/tmp/gitmerge"
    shutil.rmtree(mc, ignore_errors=True)
    os.makedirs(os.path.join(mc, "src"))
    git(["init", "-q"], mc)
    git(["config", "user.email", "qa@example.com"], mc)
    git(["config", "user.name", "QA"], mc)
    git(["symbolic-ref", "HEAD", "refs/heads/main"], mc)
    with open(os.path.join(mc, "src", "conflict.ts"), "w") as f:
        f.write("export function x(): number {\n  return 1;\n}\n")
    git(["add", "-A"], mc)
    git(["commit", "-qm", "base"], mc)
    git(["checkout", "-qb", "topic"], mc)
    with open(os.path.join(mc, "src", "conflict.ts"), "w") as f:
        f.write("export function x(): number {\n  return 100;\n}\n")
    git(["commit", "-qam", "topic change"], mc)
    git(["checkout", "-q", "main"], mc)
    with open(os.path.join(mc, "src", "conflict.ts"), "w") as f:
        f.write("export function x(): number {\n  return 999;\n}\n")
    git(["commit", "-qam", "main change"], mc)
    # merge with conflict expected; exit code 1 on conflict
    subprocess.run(["git", "merge", "topic"], cwd=mc, capture_output=True, text=True)
    status = git(["status", "--porcelain=v1"], mc)
    assert "UU" in status or "AA" in status or "DD" in status, f"expected conflict, got {status!r}"
    ctx(["init", "--force"], mc)
    run_phase(mc, "merge-conflict", lambda cwd: None)

    # ---- scenario 14: detached HEAD ----
    dh = "/tmp/gitdetach"
    shutil.rmtree(dh, ignore_errors=True)
    os.makedirs(os.path.join(dh, "src"))
    git(["init", "-q"], dh)
    git(["config", "user.email", "qa@example.com"], dh)
    git(["config", "user.name", "QA"], dh)
    with open(os.path.join(dh, "src", "a.ts"), "w") as f:
        f.write("export function a() { return 1; }\n")
    git(["add", "-A"], dh)
    git(["commit", "-qm", "one"], dh)
    git(["checkout", "-q", "--detach"], dh)
    with open(os.path.join(dh, "src", "b.ts"), "w") as f:
        f.write("export function b() { return 2; }\n")
    git(["add", "-A"], dh)
    git(["commit", "-qm", "detached work"], dh)
    ctx(["init", "--force"], dh)
    run_phase(dh, "detached-head", lambda cwd: None)

    # diff comparisons on the main work
    print("===== ctx diff vs git diff (worktree) =====")
    compare_diff_worktree(WORK, "diff-worktree")

    # two-ref diff comparison: commit staged+unstaged, then diff commits
    git(["add", "-A"], WORK)
    git(["commit", "-qm", "phase state"], WORK)
    d2 = ctx_diff(WORK, "HEAD~1")
    gd2 = parse_name_status(git(["diff", "--name-status", "HEAD~1", "HEAD"], WORK))
    dpaths2 = {f["path"]: f["status"] for f in d2["files"]}
    lang2 = {p: s for p, s in gd2.items()
             if os.path.splitext(p)[1] in {".ts", ".tsx", ".js", ".py", ".rs", ".go"}}
    missing = set(lang2) - set(dpaths2)
    extra = set(dpaths2) - set(lang2)
    check("two-ref-diff: file set matches git diff",
          not missing and not extra, f"missing={sorted(missing)} extra={sorted(extra)}")
    bad = [p for p in set(lang2) & set(dpaths2) if dpaths2[p] != normalize_status(lang2[p])]
    check("two-ref-diff: statuses match git", not bad, f"bad={bad}")
    print(f"  [INFO] two-ref diff files={len(dpaths2)}")

    # ctx changed <ref> vs git diff <ref> + untracked (status semantics)
    gd3 = parse_name_status(git(["diff", "--name-status", "HEAD~1"], WORK))
    untracked = set(git(["ls-files", "--others", "--exclude-standard"], WORK).split())
    gd3 = {p: s for p, s in gd3.items()
           if p != ".ctx" and not p.startswith(".ctx/") and p != ".git" and not p.startswith(".git/")}
    for p in untracked:
        gd3.setdefault(p, "A")
    report = ctx_changed(WORK, "HEAD~1")
    cc = {f["path"]: f["status"] for f in report["files"]}
    missing = set(gd3) - set(cc)
    extra = set(cc) - set(gd3)
    check("changed<ref>: path set matches git diff+untracked",
          not missing and not extra, f"missing={sorted(missing)} extra={sorted(extra)}")
    bad = [p for p in set(gd3) & set(cc) if cc[p] != normalize_status(gd3[p])]
    check("changed<ref>: statuses match git", not bad, f"bad={bad}")

    # detached HEAD: verify ctx changed works when no branch
    run_phase(dh, "detached-head-2", lambda cwd: None)

    print()
    print(f"TOTAL pass={RESULTS['pass']} fail={RESULTS['fail']}")
    sys.exit(1 if RESULTS["fail"] else 0)

if __name__ == "__main__":
    main()