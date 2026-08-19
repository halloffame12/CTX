import type { Metadata } from "next";
import Link from "next/link";
import { H1, P, DocsShell, H2, CommandTable, Code, Note, Warn, Ul } from "@/components/Sections";

export const metadata: Metadata = {
  title: "Documentation",
  description:
    "Documentation for ctx: what it is and isn't, install, commands, configuration, MCP server setup, and how the code graph works.",
};

export default function DocsPage() {
  return (
    <DocsShell>
      <H1>Documentation</H1>
      <P>
        <Code>ctx</Code> is a command-line tool and MCP server that answers one
        kind of question well: <em>which files does this task actually need?</em>{" "}
        It builds a local index of your repository — files, symbols, and the
        dependency edges between them — and lets you (or an AI agent) query it
        over stdio.
      </P>

      <H2 id="what-it-is">What it is</H2>
      <Ul>
        <li>
          A deterministic, offline code graph. No embeddings, no API calls, no
          LLM in the loop — the same repository and the same query give the same
          answer.
        </li>
        <li>
          An incremental indexer. Only changed files are re-parsed on subsequent
          runs, so re-indexing a repo typically takes tens of milliseconds.
        </li>
        <li>
          A reference tool for agents: where does this symbol live, what imports
          what, what would break if I changed this, and which files does this
          task need.
        </li>
        <li>
          An MCP server over stdio that exposes the same capabilities to Claude
          Desktop, Cursor, opencode, VS Code, and any other MCP client.
        </li>
      </Ul>

      <H2 id="what-it-is-not">What it is not</H2>
      <Ul>
        <li>
          Not semantic search. <Code>ctx</Code> matches names and paths; it has
          no notion of what code <em>means</em>. A task like{" "}
          <Code>&quot;add Google OAuth&quot;</Code> is scored against symbol and
          file <em>names</em>, not code semantics.
        </li>
        <li>
          Not a full language server. It does not resolve types, follow macros,
          or understand generics deeply. It extracts definitions, references,
          and import edges.
        </li>
        <li>
          Not a replacement for grep for one-off searches. If you just need a
          string anywhere in a tree, <Code>grep</Code> / <Code>rg</Code> is the
          right tool.
        </li>
        <li>
          Not guaranteed to resolve every import. Unresolved imports are recorded
          as external/unknown edges rather than guessed.
        </li>
      </Ul>

      <H2 id="quickstart">Quick start</H2>
      <P>Install via npm (no Rust toolchain needed):</P>
      <pre className="ctx-scroll overflow-x-auto rounded-lg border border-line bg-surface p-4 font-mono text-[13px] leading-6 text-ink">
{`npm install -g ctxai-cli
ctx --version`}
      </pre>
      <P>Or from source with cargo:</P>
      <pre className="ctx-scroll overflow-x-auto rounded-lg border border-line bg-surface p-4 font-mono text-[13px] leading-6 text-ink">
{`cargo install ctxai-cli --locked
ctx --version`}
      </pre>
      <P>Then point it at a repository:</P>
      <pre className="ctx-scroll overflow-x-auto rounded-lg border border-line bg-surface p-4 font-mono text-[13px] leading-6 text-ink">
{`cd /path/to/your/project
ctx init
ctx doctor
ctx search "rate limit"`}
      </pre>

      <H2 id="core-loop">The core loop</H2>
      <CommandTable
        rows={[
          { cmd: "ctx init", desc: "Index the current directory into .ctx/. Incremental, so later runs are fast." },
          { cmd: "ctx doctor", desc: "Verify the index is healthy and current." },
          { cmd: "ctx context \"…\"", desc: "Build a relevance-ranked context package for a task description." },
          { cmd: "ctx mcp", desc: "Run the MCP server so an AI agent can query the graph." },
        ]}
      />

      <Warn>
        The index is a snapshot. Run <Code>ctx init</Code> (or keep{" "}
        <Code>ctx watch</Code> running) after adding, renaming, or deleting files
        so searches and agents see the current state.
      </Warn>

      <Note>
        Only TypeScript/JavaScript, Python, Rust, and Go are parsed. All other
        files are skipped by the scanner — see{" "}
        <Link href="/docs/architecture" className="font-semibold text-accent-deep hover:underline">
          How it works
        </Link>{" "}
        and the{" "}
        <Link href="/docs/faq" className="font-semibold text-accent-deep hover:underline">
          FAQ
        </Link>{" "}
        for details.
      </Note>

      <H2 id="next-steps">Next steps</H2>
      <Ul>
        <li>
          <Link href="/docs/install" className="font-semibold text-accent-deep hover:underline">Installation</Link> — npm, cargo, and prebuilt binaries
        </li>
        <li>
          <Link href="/docs/commands" className="font-semibold text-accent-deep hover:underline">Command reference</Link> — every command and flag
        </li>
        <li>
          <Link href="/docs/config" className="font-semibold text-accent-deep hover:underline">Configuration</Link> — the .ctx/config.toml file
        </li>
        <li>
          <Link href="/docs/mcp" className="font-semibold text-accent-deep hover:underline">MCP server</Link> — wire ctx into Claude, Cursor, opencode, VS Code
        </li>
        <li>
          <Link href="/docs/architecture" className="font-semibold text-accent-deep hover:underline">How it works</Link> — the index, resolution, and ranking
        </li>
        <li>
          <Link href="/docs/faq" className="font-semibold text-accent-deep hover:underline">FAQ &amp; limitations</Link> — privacy and honest limits
        </li>
      </Ul>
    </DocsShell>
  );
}