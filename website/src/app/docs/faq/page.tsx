import type { Metadata } from "next";
import Link from "next/link";
import { H1, P, DocsShell, H2, Code, Ul } from "@/components/Sections";

export const metadata: Metadata = {
  title: "FAQ & limitations",
  description:
    "Frequently asked questions about ctx: privacy, supported languages, how it differs from other tools, and honest limitations.",
};

export default function FaqPage() {
  return (
    <DocsShell>
      <H1>FAQ &amp; limitations</H1>

      <H2 id="privacy">Does ctx send my code anywhere?</H2>
      <P>
        No. The index lives in <Code>.ctx/</Code> inside your repository. ctx
        opens no network connections, sends no telemetry, and never executes
        project code. The MCP server runs as a child process over stdio —
        nothing leaves the machine.
      </P>

      <H2 id="languages">Which languages are supported?</H2>
      <P>
        TypeScript/JavaScript, Python, Rust, and Go. Each is parsed with a
        tree-sitter grammar, and dependency resolution is language-aware. Any
        other file type is skipped by the scanner.
      </P>

      <H2 id="embeddings">Does it use embeddings or an LLM?</H2>
      <P>
        No. The graph is built with static analysis, and ranking is a
        deterministic scoring function over names and paths. There are no API
        calls, no models, no keys — which is also why the results are
        reproducible.
      </P>

      <H2 id="grep">How is this different from a grep or ripgrep wrapper?</H2>
      <P>
        grep finds lines; ctx knows structure. It knows that{" "}
        <Code>create_user</Code> is a function defined in one file and called
        from others, which files import which, and what a change would ripple
        into. Skeleton output gives an agent signatures and exports without
        bodies, and impact analysis walks real dependency edges instead of
        matching text.
      </P>

      <H2 id="semantic">Why a code graph instead of an embedding index?</H2>
      <P>
        Embeddings answer &quot;what is this about?&quot; — fuzzy semantic
        similarity. ctx answers &quot;where does this live, what calls it, what
        would break?&quot; which are exact questions with exact answers. For
        agents working on a codebase, exactness matters more than similarity.
        The trade-off is that ctx cannot find semantically similar code that
        shares no names.
      </P>

      <H2 id="monorepo">Can it index a monorepo?</H2>
      <P>
        ctx indexes whatever directory you point it at, including the root of a
        monorepo. Use <Code>-R DIR</Code> to point any command at a specific
        project root.
      </P>

      <H2 id="agents">Does it work with my editor / agent?</H2>
      <P>
        If your tool speaks MCP over stdio, it works. Claude Desktop, Cursor,
        opencode, VS Code (Cline, Roo), and most other MCP clients are covered
        in the{" "}
        <Link href="/docs/mcp" className="font-semibold text-accent-deep hover:underline">
          MCP guide
        </Link>
        .
      </P>

      <H2 id="cost">What does the index cost at runtime?</H2>
      <P>
        The binary is a single static-ish Rust executable with no runtime
        dependencies. As a concrete example, indexing this project&apos;s own
        repository (≈ 75 source files, ≈ 870 symbols) takes about 90 ms; a
        re-index of an unchanged tree is typically in the tens of milliseconds.
        Your mileage depends on repository size and hardware.
      </P>

      <H2 id="ci">Can I use ctx in CI?</H2>
      <P>
        Yes. Every command supports <Code>--json</Code>, and{" "}
        <Code>ctx doctor</Code> exits non-zero on an unhealthy index — so{" "}
        <Code>ctx diff</Code>, <Code>ctx impact</Code>, and{" "}
        <Code>ctx doctor</Code> can feed checks, code review, or changelog
        generation.
      </P>

      <H2 id="limitations">Honest limitations</H2>
      <Ul>
        <li>
          <strong>Four languages.</strong> TS/JS, Python, Rust, Go. Everything
          else is invisible.
        </li>
        <li>
          <strong>Name-based, not semantic.</strong> Search and context ranking
          match words; they cannot match concepts.
        </li>
        <li>
          <strong>No type checking.</strong> ctx is not a language server. It
          does not resolve types or verify that code compiles.
        </li>
        <li>
          <strong>Context following is bounded.</strong>{" "}
          <Code>ctx context</Code> follows direct dependency and dependent edges
          from matching files and caps dependents of hub files. Code reachable
          only through longer chains may be omitted — this is by design, to keep
          the package small, not a guarantee of completeness.
        </li>
        <li>
          <strong>Unresolved imports are flagged, not guessed.</strong> Dynamic
          or exotic import patterns may end up in the unknown bucket.
        </li>
        <li>
          <strong>The index is a snapshot.</strong> It only knows what was on
          disk at the last <Code>ctx init</Code> or what <Code>ctx watch</Code>{" "}
          has seen.
        </li>
        <li>
          <strong>MCP runs on a spawn, not a daemon.</strong> There is no long
          running service and no shared cache between clients beyond the SQLite
          file.
        </li>
      </Ul>
    </DocsShell>
  );
}