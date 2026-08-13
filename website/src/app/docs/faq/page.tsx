import type { Metadata } from "next";
import Link from "next/link";
import { H1, P, DocsShell, H2 } from "@/components/Sections";

export const metadata: Metadata = {
  title: "FAQ",
  description:
    "Frequently asked questions about ctx: privacy, supported languages, how it differs from other tools, and limitations.",
};

export default function FaqPage() {
  return (
    <DocsShell>
      <H1>FAQ</H1>

      <H2>Does ctx send my code anywhere?</H2>
      <P>
        No. The index lives in <span className="font-mono text-sm text-ink">.ctx/</span>{" "}
        inside your repository. ctx opens no network connections, sends no
        telemetry, and never executes project code. The MCP server runs as a
        child process over stdio — nothing leaves the machine.
      </P>

      <H2>Which languages are supported?</H2>
      <P>
        TypeScript/JavaScript, Python, Rust, and Go. Each is parsed with a
        tree-sitter grammar, and dependency resolution is language-aware.
      </P>

      <H2>Does it use embeddings or an LLM?</H2>
      <P>
        No. The graph is built with static analysis, and ranking is a
        deterministic scoring function. There are no API calls, no models, no
        keys — which is also why the results are reproducible.
      </P>

      <H2>How is this different from a grep or ripgrep wrapper?</H2>
      <P>
        grep finds lines; ctx knows structure. It knows that{" "}
        <span className="font-mono text-sm text-ink">create_user</span> is a
        function defined in one file and called from others, which files import
        which, and what a change would ripple into. Skeleton output gives an
        agent signatures and exports without bodies, and impact analysis walks
        real dependency edges instead of matching text.
      </P>

      <H2>Why a code graph instead of an embedding index?</H2>
      <P>
        Embeddings answer &quot;what is this about?&quot; — fuzzy semantic similarity.
        ctx answers &quot;where does this live, what calls it, what would break?&quot;
        which are exact questions with exact answers. For agents working on a
        codebase, exactness matters more than similarity.
      </P>

      <H2>Can it index a monorepo?</H2>
      <P>
        ctx indexes whatever directory you point it at, including the root of a
        monorepo. Use <span className="font-mono text-sm text-ink">-R DIR</span>{" "}
        to point any command at a specific project root.
      </P>

      <H2>Does it work with my editor / agent?</H2>
      <P>
        If your tool speaks MCP over stdio, it works. Claude Desktop, Cursor,
        opencode, VS Code (Cline, Roo), and most other MCP clients are covered
        in the{" "}
        <Link href="/docs/mcp" className="font-semibold text-accent-deep hover:underline">
          MCP guide
        </Link>
        .
      </P>

      <H2>What does the index cost at runtime?</H2>
      <P>
        The binary is a single static-ish Rust executable with no runtime
        dependencies. Initial indexing of a moderate repository takes about a
        second; incremental re-indexing is typically tens of milliseconds.
      </P>

      <H2>Can I use ctx in CI?</H2>
      <P>
        Yes. Every command supports <span className="font-mono text-sm text-ink">--json</span>,
        so <span className="font-mono text-sm text-ink">ctx diff</span> and{" "}
        <span className="font-mono text-sm text-ink">ctx impact</span> can feed
        checks, code review, or changelog generation.
      </P>
    </DocsShell>
  );
}