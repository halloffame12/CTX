import type { Metadata } from "next";
import { H1, P, DocsShell, H2, Code, Note, Ul } from "@/components/Sections";

export const metadata: Metadata = {
  title: "MCP server",
  description:
    "Wire ctx into Claude Desktop, Cursor, opencode, VS Code, and any MCP client over stdio. Eleven read-only tools backed by a local index.",
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

const TOOLS = [
  ["ctx_project", "Project overview: root, git status, index counts."],
  ["ctx_search", "Find symbols or files by name, with kind and file filters."],
  ["ctx_skeleton", "Body-less structural skeleton of a file."],
  ["ctx_symbol", "Definition, kind, methods, references and dependencies of a symbol."],
  ["ctx_dependencies", "What a file imports."],
  ["ctx_dependents", "What imports a file."],
  ["ctx_impact", "What would break if a symbol or file changed."],
  ["ctx_context", "Relevance-ranked context package for a task."],
  ["ctx_changed", "Symbols changed in the working tree or since a ref."],
  ["ctx_diff", "Symbol-level diff between two refs."],
  ["ctx_stats", "Index statistics: files, symbols, dependencies, db size."],
];

export default function McpPage() {
  return (
    <DocsShell>
      <H1>MCP server</H1>
      <P>
        ctx speaks the Model Context Protocol over stdio — JSON-RPC 2.0, one
        message per line. There is no daemon and no network port. The client
        spawns <Code>ctx mcp</Code> as a child process, and all logs go to
        stderr so stdout stays protocol-clean.
      </P>

      <H2 id="global-install">Global install (Claude Desktop, opencode)</H2>
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

      <H2 id="npx">No install needed (npx)</H2>
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

      <H2 id="cursor">Cursor</H2>
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

      <H2 id="vscode">VS Code / Cline / Roo</H2>
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

      <H2 id="first-run">Prerequisites: index first</H2>
      <P>
        The tools read a local SQLite index. The server does not index on its
        own — point it at a project that has been indexed with{" "}
        <Code>ctx init</Code> first:
      </P>
      <CodeBlock
        title="build the index, then serve it"
        code={`cd /path/to/project
ctx init
ctx mcp -R /path/to/project`}
      />

      <H2 id="smoke-test">Smoke test over stdio</H2>
      <P>Confirm the handshake and tool list without an agent client:</P>
      <CodeBlock
        title="verify the handshake"
        code={`printf '%s\\n' '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}' \\
  '{"jsonrpc":"2.0","id":2,"method":"tools/list"}' \\
  | ctx mcp`}
      />
      <P>
        You should see an <Code>initialize</Code> response naming{" "}
        <Code>ctx</Code>, followed by the eleven tools below.
      </P>

      <H2 id="tools">The eleven tools</H2>
      <P>
        All tools are <strong>read-only</strong> — they read the index and
        never modify your repository:
      </P>
      <div className="my-5 overflow-x-auto rounded-lg border border-line">
        <table className="w-full border-collapse text-sm">
          <thead>
            <tr className="border-b border-line bg-paper text-left">
              <th className="px-4 py-2.5 font-mono text-xs font-semibold text-ink">Tool</th>
              <th className="px-4 py-2.5 text-xs font-semibold text-ink">Purpose</th>
            </tr>
          </thead>
          <tbody>
            {TOOLS.map(([name, desc]) => (
              <tr key={name} className="border-b border-line last:border-0">
                <td className="px-4 py-2.5 align-top font-mono text-[13px] whitespace-nowrap text-accent-deep">{name}</td>
                <td className="px-4 py-2.5 align-top leading-6 text-ink-soft">{desc}</td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>

      <H2 id="honest-notes">What the server can and cannot see</H2>
      <Ul>
        <li>
          The index is a snapshot. Re-run <Code>ctx init</Code> (or keep{" "}
          <Code>ctx watch</Code> running) after adding or renaming files so the
          tools see them.
        </li>
        <li>
          <Code>ctx_context</Code> ranking is name-based, not semantic. The
          agent should describe the task with the same words that appear in
          symbol and file names.
        </li>
        <li>
          Context following is limited to direct dependency and dependent edges
          from matching files. Symbols reachable only through a longer chain may
          not appear.
        </li>
        <li>
          Only TypeScript/JavaScript, Python, Rust, and Go are parsed. Other
          files are invisible to the tools.
        </li>
      </Ul>

      <H2 id="protocol">Protocol notes</H2>
      <P>
        The server negotiates the <Code>2025-06-18</Code> protocol version.
        Failures surface as <Code>isError: true</Code> in tool results rather
        than unhandled exceptions, so the client keeps running. Stdout is
        reserved for protocol frames — everything else goes to stderr.
      </P>

      <Note>
        The server reads source code only through its index. It does not execute
        project code, does not open network connections, and never uploads
        anything.
      </Note>
    </DocsShell>
  );
}