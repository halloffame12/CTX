# @halloffame12/cli — ctx

The npm distribution of **ctx**, a codebase intelligence and context engine
for AI coding agents. This package is a thin launcher for the compiled Rust
binary, which is shipped in platform-specific optional dependencies:

```
@halloffame12/cli                (launcher: bin/ctx.js)
  ├── @halloffame12/cli-linux-x64
  ├── @halloffame12/cli-linux-arm64
  ├── @halloffame12/cli-darwin-x64
  ├── @halloffame12/cli-darwin-arm64
  ├── @halloffame12/cli-win32-x64
  └── @halloffame12/cli-win32-arm64
```

The binary is installed during `npm install`; nothing is downloaded at
runtime.

## Install

```bash
npm install -g @halloffame12/cli
ctx --version
```

## Usage without install

```bash
npx @halloffame12/cli --version
npx @halloffame12/cli context "add authentication"
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