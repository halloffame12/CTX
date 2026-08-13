import type { Metadata } from "next";
import Link from "next/link";
import { H1, P, DocsShell, CommandTable } from "@/components/Sections";

export const metadata: Metadata = {
  title: "Documentation",
  description:
    "Documentation for ctx: install, commands, MCP server setup, and how the code graph works.",
};

export default function DocsPage() {
  return (
    <DocsShell>
      <H1>Documentation</H1>
      <P>
        ctx is a command-line tool and MCP server that answers one kind of
        question well: <em>which files does this task actually need?</em> It
        builds a local, deterministic index of your repository — symbols,
        files, and the dependency edges between them — and lets you (or an AI
        agent) query it over stdio.
      </P>

      <h2 className="mt-12 mb-4 text-xl font-semibold text-ink">Quick start</h2>
      <P>Install via npm (no Rust toolchain needed):</P>
      <pre className="ctx-scroll overflow-x-auto rounded-lg border border-line bg-surface p-4 font-mono text-[13px] leading-6 text-ink">
{`npm install -g ctxai-cli
ctx init
ctx context "what does the payment module do?"`}
      </pre>
      <P>
        Or from source with cargo:
      </P>
      <pre className="ctx-scroll overflow-x-auto rounded-lg border border-line bg-surface p-4 font-mono text-[13px] leading-6 text-ink">
{`cargo install ctxai-cli --locked
ctx init`}
      </pre>

      <h2 className="mt-12 mb-4 text-xl font-semibold text-ink">The core loop</h2>
      <CommandTable
        rows={[
          { cmd: "ctx init", desc: "Index the current directory into .ctx/. Incremental, so later runs are fast." },
          { cmd: "ctx doctor", desc: "Verify the index is healthy and current." },
          { cmd: "ctx context \"…\"", desc: "Build a relevance-ranked context package for a task description." },
          { cmd: "ctx mcp", desc: "Run the MCP server so an AI agent can query the graph." },
        ]}
      />

      <h2 className="mt-12 mb-4 text-xl font-semibold text-ink">Next steps</h2>
      <ul className="my-4 list-disc space-y-2 pl-5 text-ink-soft">
        <li>
          <Link href="/docs/install" className="font-semibold text-accent-deep hover:underline">Installation</Link> — npm, cargo, and prebuilt binaries
        </li>
        <li>
          <Link href="/docs/commands" className="font-semibold text-accent-deep hover:underline">Command reference</Link> — every command and flag
        </li>
        <li>
          <Link href="/docs/mcp" className="font-semibold text-accent-deep hover:underline">MCP server</Link> — wire ctx into Claude, Cursor, opencode, VS Code
        </li>
        <li>
          <Link href="/docs/architecture" className="font-semibold text-accent-deep hover:underline">How it works</Link> — the index, resolution, and ranking
        </li>
        <li>
          <Link href="/docs/faq" className="font-semibold text-accent-deep hover:underline">FAQ</Link> — privacy, languages, and limitations
        </li>
      </ul>
    </DocsShell>
  );
}