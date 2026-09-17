# Security Policy

`ctx` is a local-first, offline tool: it reads source files, writes a SQLite
index under `.ctx/`, and spawns `git` for change queries. It **never** executes
project code, makes network calls, or sends telemetry. That keeps the attack
surface small, but we still take reports seriously.

## Supported versions

| Version | Supported |
| --- | --- |
| latest release | :white_check_mark: |
| previous release | partial, best effort |
| older | :x: |

## Reporting a vulnerability

**Please do not open a public GitHub issue for security problems.** Report
them privately:

- **Email:** open a private disclosure via GitHub's
  [Security Advisories](https://github.com/halloffame12/CTX/security/advisories/new)
  (preferred), or email the maintainers at the address listed on the profile.

What to include:

- The `ctx --version` output and your OS (Windows/macOS/Linux).
- Steps to reproduce, ideally with a minimal repo or file set.
- Expected vs actual behavior.
- Whether the issue involves path resolution, index parsing, or the MCP server.

You will receive an acknowledgement within 3 business days and a status update
on a fix timeline. We ask that you allow a 90-day coordinated-disclosure window
before publicizing.

## Known hardening notes

- **Path security** is a core invariant: any user-supplied path that lexically
  escapes the project root is rejected (`PathOutsideRoot`), including `..\..`
  Windows forms and through all MCP tools. Never `root.join()` raw input —
  contributing code must preserve this.
- **Binary signing:** as of `0.1.6`, Windows and macOS release binaries are
  not code-signed. On Windows with Smart App Control / WDAC policy enforced,
  unsigned binaries are blocked by the OS. See
  [RELEASE.md](RELEASE.md#code-signing) for the signing plan. Until signed
  builds ship, users on SAC/WDAC-enforced systems should run from source or
  add an explicit exception.
- **Supply chain:** npm packages are published with provenance; release
  artifacts carry SHA-256 `checksums.txt`. Package versions are cross-checked
  against `Cargo.toml` by `package-validation.yml` on every push.

## Reporting non-security bugs

Use the regular [bug report template](.github/ISSUE_TEMPLATE/bug_report.yml).