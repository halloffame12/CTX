import type { Metadata } from "next";
import { H1, P, DocsShell, H2, Note } from "@/components/Sections";

export const metadata: Metadata = {
  title: "MCP server",
  description:
    "Wire ctx into Claude Desktop, Cursor, opencode, VS Code, and any MCP client over stdio.",
};

function CodeBlock({ title, code }: { title: string; code: string }) {
  return (
    <div className="my-5 overflow-hidden rounded-lg border border-line bg-surface">
      <div className="border-b border-line px-4 py-2">
        <span className="font-mono text-xs text-ink-faint">{title}</span>
      </div>
      <pre className="ctx-scroll overflow-x-auto p-4 font-mono text-[13px] leading-6 text-ink">
        <code>{code}</code>
      </pre>
    </div>
  );
}

export default function McpPage() {
  return (
    <DocsShell>
      <H1>MCP server</H1>
      <P>
        ctx speaks the Model Context Protocol over stdio — JSON-RPC 2.0, one
        message per line. There is no daemon and no network port. The client
        spawns <span className="font-mono text-sm text-ink">ctx mcp</span> as a
        child process, and all logs go to stderr so stdout stays protocol-clean.
      </P>

      <H2>Global install (Claude Desktop, opencode)</H2>
      <CodeBlock
        title="opencode / Claude Desktop"
        code={`{
  "mcpServers": {
    "ctx": {
      "command": "ctx",
      "args": ["mcp", "-R", "/path/to/project"]
    }
  }
}`}
      />

      <H2>No install needed (npx)</H2>
      <P>
        If you don&apos;t want a global install, npx will fetch the package on
        first use:
      </P>
      <CodeBlock
        title="opencode / Claude Desktop"
        code={`{
  "mcpServers": {
    "ctx": {
      "command": "npx",
      "args": ["-y", "ctxai-cli", "mcp", "-R", "/path/to/project"]
    }
  }
}`}
      />

      <H2>Cursor</H2>
      <CodeBlock
        title=".cursor/mcp.json"
        code={`{
  "mcpServers": {
    "ctx": {
      "command": "npx",
      "args": ["-y", "ctxai-cli", "mcp", "-R", "/path/to/project"]
    }
  }
}`}
      />

      <H2>VS Code / Cline / Roo</H2>
      <CodeBlock
        title="mcp.json"
        code={`{
  "mcpServers": {
    "ctx": {
      "command": "npx",
      "args": ["-y", "ctxai-cli", "mcp", "-R", "/path/to/project"]
    }
  }
}`}
      />

      <H2>The ten tools</H2>
      <P>
        Once connected, the agent can call:
      </P>
      <div className="my-5 overflow-hidden rounded-lg border border-line">
        <table className="w-full border-collapse text-sm">
          <thead>
            <tr className="border-b border-line bg-paper text-left">
              <th className="px-4 py-2.5 font-mono text-xs font-semibold text-ink">Tool</th>
              <th className="px-4 py-2.5 text-xs font-semibold text-ink">Purpose</th>
            </tr>
          </thead>
          <tbody>
            {[
              ["ctx_project", "Project overview: root, git status, index counts."],
              ["ctx_search", "Find symbols or files by name, with kind and file filters."],
              ["ctx_skeleton", "Body-less structural skeleton of a file."],
              ["ctx_symbol", "Definition, references, and dependencies of a symbol."],
              ["ctx_dependencies", "What a file imports."],
              ["ctx_dependents", "What imports a file."],
              ["ctx_impact", "What would break if a symbol or file changed."],
              ["ctx_context", "Relevance-ranked context package for a task."],
              ["ctx_changed", "Symbols changed in the working tree or since a ref."],
              ["ctx_diff", "Symbol-level diff between two refs."],
            ].map(([name, desc]) => (
              <tr key={name} className="border-b border-line last:border-0">
                <td className="px-4 py-2.5 align-top font-mono text-[13px] text-accent-deep">{name}</td>
                <td className="px-4 py-2.5 align-top leading-6 text-ink-soft">{desc}</td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>

      <H2>Protocol notes</H2>
      <P>
        The server negotiates the{" "}
        <span className="font-mono text-sm text-ink">2025-06-18</span> protocol
        version. Failures surface as{" "}
        <span className="font-mono text-sm text-ink">isError: true</span> in tool
        results rather than unhandled exceptions, so the client keeps running.
        Stdout is reserved for protocol frames — everything else goes to stderr.
      </P>

      <Note>
        The server reads source code only through its index. It does not execute
        project code, does not open network connections, and never uploads
        anything.
      </Note>
    </DocsShell>
  );
}