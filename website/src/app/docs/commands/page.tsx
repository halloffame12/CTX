import type { Metadata } from "next";
import { H1, P, DocsShell, H2, CommandTable, Note } from "@/components/Sections";

export const metadata: Metadata = {
  title: "Command reference",
  description:
    "Every ctx command and flag: init, search, symbol, deps, impact, context, changed, diff, mcp, doctor, stats, version and more.",
};

const GLOBAL = [
  { cmd: "-R, --root DIR", desc: "Project root (defaults to the nearest directory containing .ctx)" },
  { cmd: "-j, --json", desc: "Emit machine-readable JSON instead of human text" },
  { cmd: "-q, --quiet", desc: "Suppress non-essential output" },
  { cmd: "-v, --verbose", desc: "Verbose diagnostics on stderr" },
  { cmd: "--no-color", desc: "Disable ANSI colors" },
];

export default function CommandsPage() {
  return (
    <DocsShell>
      <H1>Command reference</H1>
      <P>
        All commands accept the same global flags. Paths can be project-relative
        or absolute. Run <span className="font-mono text-sm text-ink">ctx &lt;command&gt; --help</span> for
        details on any command.
      </P>

      <H2>Global flags</H2>
      <CommandTable rows={GLOBAL} />

      <H2>ctx init</H2>
      <P>Create <span className="font-mono text-sm text-ink">.ctx</span>, write a default config, and index the project.</P>
      <pre className="ctx-scroll overflow-x-auto rounded-lg border border-line bg-surface p-4 font-mono text-[13px] leading-6 text-ink">
{`ctx init                 # index the current directory
ctx init src/            # index a specific directory
ctx init --force         # rebuild index + config even if they exist`}
      </pre>

      <H2>ctx search</H2>
      <P>Search the graph for symbols or files.</P>
      <pre className="ctx-scroll overflow-x-auto rounded-lg border border-line bg-surface p-4 font-mono text-[13px] leading-6 text-ink">
{`ctx search "rate limit"          # symbols whose name matches
ctx search --files "auth"         # file paths instead of symbols
ctx search --kind struct "user"   # restrict to a symbol kind
ctx search "token" --limit 20     # cap the number of results`}
      </pre>

      <H2>ctx symbol</H2>
      <P>Details about a symbol: definition, references, and dependencies.</P>
      <pre className="ctx-scroll overflow-x-auto rounded-lg border border-line bg-surface p-4 font-mono text-[13px] leading-6 text-ink">
{`ctx symbol UserService.updateUser`}
      </pre>

      <H2>ctx deps</H2>
      <P>Show what a file imports and what imports it.</P>
      <pre className="ctx-scroll overflow-x-auto rounded-lg border border-line bg-surface p-4 font-mono text-[13px] leading-6 text-ink">
{`ctx deps src/models/user.py       # both directions
ctx deps --outgoing src/main.rs    # imports only
ctx deps --incoming src/api.rs     # dependents only`}
      </pre>

      <H2>ctx impact</H2>
      <P>Analyze the impact of changing a symbol or file.</P>
      <pre className="ctx-scroll overflow-x-auto rounded-lg border border-line bg-surface p-4 font-mono text-[13px] leading-6 text-ink">
{`ctx impact UserService.updateUser --depth 5
ctx impact src/worker.rs`}
      </pre>
      <P>
        Traversal is a cycle-safe breadth-first search over the inverted graph,
        grouped into direct, indirect, test-file, and unknown buckets. Depth is
        clamped to 1–20.
      </P>

      <H2>ctx context</H2>
      <P>Build a relevance-ranked context package for a task.</P>
      <pre className="ctx-scroll overflow-x-auto rounded-lg border border-line bg-surface p-4 font-mono text-[13px] leading-6 text-ink">
{`ctx context "add Google OAuth"
ctx context "fix the payment retry bug" --include-bodies
ctx context "add rate limiting" --max-tokens 12000
ctx context "refactor the CLI" --no-git`}
      </pre>

      <H2>ctx changed</H2>
      <P>Show symbols changed in the working tree or between refs.</P>
      <pre className="ctx-scroll overflow-x-auto rounded-lg border border-line bg-surface p-4 font-mono text-[13px] leading-6 text-ink">
{`ctx changed                 # working tree vs HEAD
ctx changed --ref main       # diff against main
ctx changed --sync           # update the graph before comparing`}
      </pre>

      <H2>ctx diff</H2>
      <P>Semantic diff of symbols between two git refs.</P>
      <pre className="ctx-scroll overflow-x-auto rounded-lg border border-line bg-surface p-4 font-mono text-[13px] leading-6 text-ink">
{`ctx diff HEAD~3 HEAD
ctx diff main`}
      </pre>
      <P>
        The base defaults to the merge-base with HEAD, so{" "}
        <span className="font-mono text-sm text-ink">ctx diff main</span> shows
        everything your branch changed, not everything main did since the fork.
      </P>

      <H2>ctx mcp</H2>
      <P>Run the Model Context Protocol server over stdio.</P>
      <pre className="ctx-scroll overflow-x-auto rounded-lg border border-line bg-surface p-4 font-mono text-[13px] leading-6 text-ink">
{`ctx mcp -R /path/to/project`}
      </pre>

      <H2>ctx doctor</H2>
      <P>Inspect the project and report the health of the index.</P>
      <pre className="ctx-scroll overflow-x-auto rounded-lg border border-line bg-surface p-4 font-mono text-[13px] leading-6 text-ink">
{`ctx doctor
ctx doctor --json`}
      </pre>

      <H2>ctx stats</H2>
      <P>Show index statistics: files, symbols, dependency edges, and the size of index.db.</P>
      <pre className="ctx-scroll overflow-x-auto rounded-lg border border-line bg-surface p-4 font-mono text-[13px] leading-6 text-ink">
{`ctx stats
ctx stats --json`}
      </pre>

      <H2>ctx version</H2>
      <P>Print version information.</P>
      <pre className="ctx-scroll overflow-x-auto rounded-lg border border-line bg-surface p-4 font-mono text-[13px] leading-6 text-ink">
{`ctx version
ctx version --json`}
      </pre>

      <H2>ctx skeleton</H2>
      <P>Show a body-less structural skeleton of a source file.</P>
      <pre className="ctx-scroll overflow-x-auto rounded-lg border border-line bg-surface p-4 font-mono text-[13px] leading-6 text-ink">
{`ctx skeleton src/models.py
ctx skeleton src/models.py --stats`}
      </pre>

      <H2>ctx schema</H2>
      <P>Print the SQLite graph schema.</P>
      <pre className="ctx-scroll overflow-x-auto rounded-lg border border-line bg-surface p-4 font-mono text-[13px] leading-6 text-ink">
{`ctx schema`}
      </pre>

      <H2>ctx benchmark</H2>
      <P>Re-run an index pass and print incremental timing.</P>
      <pre className="ctx-scroll overflow-x-auto rounded-lg border border-line bg-surface p-4 font-mono text-[13px] leading-6 text-ink">
{`ctx benchmark`}
      </pre>

      <H2>ctx watch</H2>
      <P>Watch the project and keep the graph in sync.</P>
      <pre className="ctx-scroll overflow-x-auto rounded-lg border border-line bg-surface p-4 font-mono text-[13px] leading-6 text-ink">
{`ctx watch`}
      </pre>

      <Note>
        Running <span className="font-mono text-sm text-ink">ctx</span> with no
        arguments prints a quick overview of the tool.
      </Note>
    </DocsShell>
  );
}