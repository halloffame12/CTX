# ctxai — ctx

The npm distribution of **ctx**, a codebase intelligence and context engine
for AI coding agents. This package is a thin launcher for the compiled Rust
binary, which is shipped in platform-specific optional dependencies:

```
ctxai                (launcher: bin/ctx.js)
  ├── ctxai-linux-x64
  ├── ctxai-linux-arm64
  ├── ctxai-darwin-x64
  ├── ctxai-darwin-arm64
  ├── ctxai-win32-x64
  └── ctxai-win32-arm64
```

The binary is installed during `npm install`; nothing is downloaded at
runtime.

## Install

```bash
npm install -g ctxai
ctx --version
```

## Usage without install

```bash
npx ctxai --version
npx ctxai context "add authentication"
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