# ctxai-cli — ctx

The npm distribution of **ctx**, a codebase intelligence and context engine
for AI coding agents. This package is a thin launcher for the compiled Rust
binary, which is shipped in platform-specific optional dependencies:

```
ctxai-cli                (launcher: bin/ctx.js)
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
npm install -g ctxai-cli
ctx --version
```

## Usage without install

```bash
npx ctxai-cli --version
npx ctxai-cli context "add authentication"
```

## Using with an AI coding agent (MCP)

```bash
ctx mcp                      # already installed globally
npx ctxai-cli mcp            # no install needed at all
```

Point your MCP client at `ctx mcp` — no Rust, cargo, or build tools required.
The server speaks MCP over stdio and exposes 11 tools (`ctx_project`,
`ctx_search`, `ctx_skeleton`, `ctx_symbol`, `ctx_dependencies`,
`ctx_dependents`, `ctx_impact`, `ctx_context`, `ctx_changed`, `ctx_diff`,
`ctx_stats`).

### Claude / Claude Desktop

`.claude.json` or Claude Desktop → Settings → MCP servers:

```json
{
  "mcpServers": {
    "ctx": {
      "command": "npx",
      "args": ["-y", "ctxai-cli", "mcp", "-R", "/absolute/path/to/project"]
    }
  }
}
```

### Cursor

Cursor → Settings → MCP → Add new MCP server → Type: `command`:

```json
{
  "command": "npx",
  "args": ["-y", "ctxai-cli", "mcp", "-R", "/absolute/path/to/project"]
}
```

### opencode

In opencode's MCP configuration:

```json
{
  "mcpServers": {
    "ctx": {
      "command": "npx",
      "args": ["-y", "ctxai-cli", "mcp", "-R", "/absolute/path/to/project"]
    }
  }
}
```

### VS Code (Cline / Roo / Continue)

Add a new MCP server of type `stdio`:

```
command: npx
args: -y ctxai-cli mcp -R /absolute/path/to/project
```

> **Tip:** omit `-R` and `ctx` will auto-detect the project root from the
> working directory. Use `ctx -R <root> mcp` when your editor launches the
> server from a different directory.

## Supported platforms

| OS | x64 | arm64 |
| --- | --- | --- |
| Linux | ✅ | ✅ |
| macOS | ✅ | ✅ |
| Windows | ✅ | ✅ |