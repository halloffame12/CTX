# QA archive

Historical QA campaign reports, inventory notes and captured tool output from
the development of `ctx`. These documents were produced during feature
development and are archived here for record — they are **not** maintained
docs.

For current, maintained information see:

- [README.md](../README.md) — features, install, CLI reference
- [docs/architecture](https://halloffame12.github.io/CTX/docs/architecture) — internal design
- [RELEASE.md](../RELEASE.md) — release checklist
- [AGENTS.md](../AGENTS.md) — contributor/invariant guide for AI agents

## Contents

- `*-TEST-REPORT.md` — end-to-end verification reports for CLI commands,
  init/doctor, search/symbol, skeleton, git/diff, dependencies, and full
  regression passes.
- `*COMMAND-INVENTORY.md` — per-command inventory (args, flags, JSON output,
  MCP tool mapping).
- `qa-notes.md`, `qa-report.md` — live QA campaign notes and summary.
- `clippy_output.txt` — captured clippy diagnostics from one campaign.
- `checksums.txt` — checksums captured during local development. The live
  release checksums are generated per release as `dist/checksums.txt`.

## Why this is not in the repo root

A clean root keeps the repository approachable for new contributors. Every file
here is tracked in git history, so nothing is lost by archiving.