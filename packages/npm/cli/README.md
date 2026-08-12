# @ctx/cli — ctx

The npm distribution of **ctx**, a codebase intelligence and context engine
for AI coding agents. This package is a thin launcher for the compiled Rust
binary, which is shipped in platform-specific optional dependencies:

```
@ctx/cli                (launcher: bin/ctx.js)
  ├── @ctx/cli-linux-x64
  ├── @ctx/cli-linux-arm64
  ├── @ctx/cli-darwin-x64
  ├── @ctx/cli-darwin-arm64
  ├── @ctx/cli-win32-x64
  └── @ctx/cli-win32-arm64
```

The binary is installed during `npm install`; nothing is downloaded at
runtime.

## Install

```bash
npm install -g @ctx/cli
ctx --version
```

## Usage without install

```bash
npx @ctx/cli --version
npx @ctx/cli context "add authentication"
```

## Using with an AI coding agent (MCP)

```bash
ctx mcp
```

Point your MCP client (opencode, Claude, Cursor, Copilot) at `ctx mcp` — no
Rust, cargo, or build tools required.

## Supported platforms

| OS | x64 | arm64 |
| --- | --- | --- |
| Linux | ✅ | ✅ |
| macOS | ✅ | ✅ |
| Windows | ✅ | ✅ |