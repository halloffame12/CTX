## Summary

<!-- What does this PR do, and why? Link the issue it closes (e.g. `Closes #123`). -->

## Type of change

- [ ] Bug fix
- [ ] New feature / command / MCP tool
- [ ] Docs / website
- [ ] Build, packaging, or release tooling
- [ ] Refactor / internal cleanup

## Checklist

- [ ] I read [CONTRIBUTING.md](../CONTRIBUTING.md) and the invariant list in
      [AGENTS.md](../AGENTS.md).
- [ ] I ran the quality gate locally:
      `cargo fmt --check`, `cargo clippy --all-targets --all-features -- -D warnings`,
      `cargo test --lib --test integration --test skeleton`.
- [ ] New public types/functions are documented; JSON and human output paths
      follow existing conventions.
- [ ] If behavior changed, docs/website that describe reality were updated and
      `npx tsc --noEmit` passes in `website/`.
- [ ] No `root.join()` on raw user input (path-security invariant preserved).

## Test plan

<!-- Commands run (CLI and/or MCP), expected vs actual output. For tests: which
fixture/additions and how they were verified. -->

## Notes for reviewers

<!-- Anything unusual, migration concerns, version bumps, follow-ups. -->