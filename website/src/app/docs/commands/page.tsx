import type { Metadata } from "next";
import type { ReactNode } from "react";
import { H1, P, DocsShell, H2, Code, Note, Warn } from "@/components/Sections";

export const metadata: Metadata = {
  title: "Command reference",
  description:
    "Every ctx command and flag, matching the real CLI: init, search, symbol, deps, impact, context, changed, diff, skeleton, mcp, doctor, stats, version, schema, benchmark, watch.",
};

const GLOBAL = [
  { flag: "-R, --root <DIR>", desc: "Project root (defaults to the nearest directory containing .ctx)" },
  { flag: "-j, --json", desc: "Emit machine-readable JSON instead of human text" },
  { flag: "-q, --quiet", desc: "Suppress non-essential output" },
  { flag: "-v, --verbose", desc: "Enable verbose diagnostics on stderr" },
  { flag: "--no-color", desc: "Disable ANSI colors" },
  { flag: "-h, --help", desc: "Print help" },
  { flag: "-V, --version", desc: "Print version" },
];

function FlagTable({ rows }: { rows: { flag: string; desc: string }[] }) {
  return (
    <div className="my-5 overflow-x-auto rounded-lg border border-line">
      <table className="w-full border-collapse text-sm">
        <tbody>
          {rows.map((r) => (
            <tr key={r.flag} className="border-b border-line last:border-0">
              <td className="px-4 py-2 align-top font-mono text-[13px] whitespace-nowrap text-accent-deep">{r.flag}</td>
              <td className="px-4 py-2 align-top leading-6 text-ink-soft">{r.desc}</td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}

function Command({
  name,
  usage,
  summary,
  children,
  flags,
}: {
  name: string;
  usage: string;
  summary: string;
  children?: ReactNode;
  flags?: { flag: string; desc: string }[];
}) {
  return (
    <>
      <H2 id={name}>
        <Code>ctx {name}</Code>
      </H2>
      <P>{summary}</P>
      <pre className="ctx-scroll overflow-x-auto rounded-lg border border-line bg-surface p-4 font-mono text-[13px] leading-6 text-ink">
        {usage}
      </pre>
      {children}
      {flags && <FlagTable rows={flags} />}
    </>
  );
}

export default function CommandsPage() {
  return (
    <DocsShell>
      <H1>Command reference</H1>
      <P>
        Every command accepts the same global flags listed below. Paths can be
        project-relative or absolute. This page is generated to match the real{" "}
        <Code>ctx &lt;command&gt; --help</Code> output — run that yourself to
        confirm the exact behavior on your build.
      </P>

      <H2 id="global-flags">Global flags</H2>
      <FlagTable rows={GLOBAL} />

      <Command
        name="init"
        usage={`ctx init [OPTIONS] [PATH]`}
        summary="Create .ctx, write a default config, and index the project."
        flags={[
          { flag: "[PATH]", desc: "Directory to initialize (default: current directory)" },
          { flag: "--force", desc: "Recreate the index and config even if they exist" },
        ]}
      >
        <pre className="ctx-scroll overflow-x-auto rounded-lg border border-line bg-surface p-4 font-mono text-[13px] leading-6 text-ink">
{`ctx init            # index the current directory
ctx init src/       # index a specific directory
ctx init --force    # rebuild index + config even if they exist`}
        </pre>
      </Command>

      <Command
        name="search"
        usage={`ctx search [OPTIONS] <QUERY>`}
        summary="Search the graph for symbols or files."
        flags={[
          { flag: "<QUERY>", desc: "Case-insensitive name query" },
          { flag: "--kind <KIND>", desc: "Restrict to a symbol kind (fn, const, alias, struct, trait, interface, type, enum, class, method, module, field, constructor, impl…)" },
          { flag: "--files", desc: "Search file paths instead of symbols" },
          { flag: "--limit <LIMIT>", desc: "Maximum number of results [default: 50]" },
        ]}
      >
        <pre className="ctx-scroll overflow-x-auto rounded-lg border border-line bg-surface p-4 font-mono text-[13px] leading-6 text-ink">
{`ctx search "rate limit"         # symbols whose name matches
ctx search --files "auth"        # file paths instead of symbols
ctx search --kind struct "user"  # restrict to a symbol kind
ctx search "token" --limit 20    # cap the number of results`}
        </pre>
        <Note>
          Kind aliases: <Code>fn</Code> → function, <Code>const</Code> →
          constant, <Code>alias</Code> → type. An invalid kind is rejected.
        </Note>
      </Command>

      <Command
        name="symbol"
        usage={`ctx symbol <NAME>`}
        summary="Details about a symbol: definition, references, dependencies."
      >
        <pre className="ctx-scroll overflow-x-auto rounded-lg border border-line bg-surface p-4 font-mono text-[13px] leading-6 text-ink">
{`ctx symbol UserService.updateUser`}
        </pre>
        <P>
          Accepts a bare name (<Code>updateUser</Code>) or a qualified name (
          <Code>UserService.updateUser</Code>). References are symbol-level, not
          just file-level, for parsed languages.
        </P>
      </Command>

      <Command
        name="deps"
        usage={`ctx deps [OPTIONS] <PATH>`}
        summary="Show what a file imports and what imports it."
        flags={[
          { flag: "--outgoing", desc: "Only show outgoing dependencies" },
          { flag: "--incoming", desc: "Only show incoming dependents" },
        ]}
      >
        <pre className="ctx-scroll overflow-x-auto rounded-lg border border-line bg-surface p-4 font-mono text-[13px] leading-6 text-ink">
{`ctx deps src/models/user.py       # both directions
ctx deps --outgoing src/main.rs    # imports only
ctx deps --incoming src/api.rs     # dependents only`}
        </pre>
      </Command>

      <Command
        name="impact"
        usage={`ctx impact [OPTIONS] <TARGET>`}
        summary="Analyze the impact of changing a symbol or file."
        flags={[
          { flag: "<TARGET>", desc: "Symbol name or file path to change" },
          { flag: "--depth <DEPTH>", desc: "How deep to traverse dependent graphs [default: 3]" },
        ]}
      >
        <pre className="ctx-scroll overflow-x-auto rounded-lg border border-line bg-surface p-4 font-mono text-[13px] leading-6 text-ink">
{`ctx impact UserService.updateUser --depth 5
ctx impact src/worker.rs`}
        </pre>
        <P>
          Traversal is a cycle-safe breadth-first search over the inverted graph,
          grouped into direct, indirect, test-file, and unknown buckets. When a
          name exists as both a production symbol and a test double, the
          production definition is preferred.
        </P>
      </Command>

      <Command
        name="context"
        usage={`ctx context [OPTIONS] <TASK>`}
        summary="Build a relevance-ranked context package for a task."
        flags={[
          { flag: "<TASK>", desc: "Natural-language description of the task" },
          { flag: "--include-bodies", desc: "Include full function/type bodies in the suggested context" },
          { flag: "--max-tokens <N>", desc: "Token budget for the suggested context (overrides config; default 12000)" },
          { flag: "--no-git", desc: "Ignore working-tree git changes when ranking files" },
        ]}
      >
        <pre className="ctx-scroll overflow-x-auto rounded-lg border border-line bg-surface p-4 font-mono text-[13px] leading-6 text-ink">
{`ctx context "add Google OAuth"
ctx context "fix the payment retry bug" --include-bodies
ctx context "add rate limiting" --max-tokens 12000
ctx context "refactor the CLI" --no-git`}
        </pre>
        <Warn>
          Ranking is <em>name-based</em>, not semantic. It scores symbol names,
          paths, hub centrality, recency, and git activity. Words in the task
          that do not appear in any symbol or path will not match anything.
        </Warn>
      </Command>

      <Command
        name="changed"
        usage={`ctx changed [OPTIONS]`}
        summary="Show symbols changed in the working tree or between refs."
        flags={[
          { flag: "--ref <REF>", desc: "Git ref to diff against (default: working tree vs HEAD)" },
          { flag: "--sync", desc: "Update the graph with current files before comparing" },
        ]}
      >
        <pre className="ctx-scroll overflow-x-auto rounded-lg border border-line bg-surface p-4 font-mono text-[13px] leading-6 text-ink">
{`ctx changed               # working tree vs HEAD
ctx changed --ref main     # diff against main
ctx changed --sync         # update the graph before comparing`}
        </pre>
      </Command>

      <Command
        name="diff"
        usage={`ctx diff [OPTIONS] [BASE] [HEAD]`}
        summary="Semantic diff of symbols between two git refs."
        flags={[
          { flag: "[BASE]", desc: "Base ref (default: HEAD; a single base is resolved to its merge-base with HEAD)" },
          { flag: "[HEAD]", desc: "Head ref (default: working tree when omitted)" },
        ]}
      >
        <pre className="ctx-scroll overflow-x-auto rounded-lg border border-line bg-surface p-4 font-mono text-[13px] leading-6 text-ink">
{`ctx diff HEAD~3 HEAD
ctx diff main`}
        </pre>
        <P>
          The base defaults to the merge-base with HEAD, so{" "}
          <Code>ctx diff main</Code> shows everything your branch changed, not
          everything main did since the fork. Reports <Code>Added</Code>,{" "}
          <Code>Removed</Code>, and <Code>Modified</Code> symbol-level entries —
          not just file status.
        </P>
      </Command>

      <Command
        name="skeleton"
        usage={`ctx skeleton [OPTIONS] <PATH>`}
        summary="Show a body-less structural skeleton of a source file."
        flags={[
          { flag: "<PATH>", desc: "Path to the file (project-relative or absolute)" },
          { flag: "--stats", desc: "Include sizes and symbol counts" },
        ]}
      >
        <pre className="ctx-scroll overflow-x-auto rounded-lg border border-line bg-surface p-4 font-mono text-[13px] leading-6 text-ink">
{`ctx skeleton src/models.py
ctx skeleton src/models.py --stats`}
        </pre>
        <P>
          Bodies are elided but signatures, types, and exports are preserved.
          Malformed code yields a bounded declaration-only skeleton and never
          leaks body lines.
        </P>
      </Command>

      <Command
        name="mcp"
        usage={`ctx mcp [OPTIONS]`}
        summary="Run the Model Context Protocol server over stdio."
      >
        <pre className="ctx-scroll overflow-x-auto rounded-lg border border-line bg-surface p-4 font-mono text-[13px] leading-6 text-ink">
{`ctx mcp -R /path/to/project`}
        </pre>
      </Command>

      <Command
        name="doctor"
        usage={`ctx doctor [OPTIONS]`}
        summary="Inspect the project and report the health of the ctx index."
      >
        <pre className="ctx-scroll overflow-x-auto rounded-lg border border-line bg-surface p-4 font-mono text-[13px] leading-6 text-ink">
{`ctx doctor
ctx doctor --json`}
        </pre>
        <P>
          Reports stale, missing, or corrupt indexes, invalid config, and other
          problems without crashing. Exits non-zero when the index is unhealthy
          — useful in CI.
        </P>
      </Command>

      <Command
        name="stats"
        usage={`ctx stats [OPTIONS]`}
        summary="Show index statistics (files, symbols, dependencies, db size)."
      >
        <pre className="ctx-scroll overflow-x-auto rounded-lg border border-line bg-surface p-4 font-mono text-[13px] leading-6 text-ink">
{`ctx stats
ctx stats --json`}
        </pre>
      </Command>

      <Command
        name="version"
        usage={`ctx version [OPTIONS]`}
        summary="Print version information."
      >
        <pre className="ctx-scroll overflow-x-auto rounded-lg border border-line bg-surface p-4 font-mono text-[13px] leading-6 text-ink">
{`ctx version
ctx version --json`}
        </pre>
      </Command>

      <Command
        name="schema"
        usage={`ctx schema [OPTIONS]`}
        summary="Print the SQLite graph schema."
      >
        <pre className="ctx-scroll overflow-x-auto rounded-lg border border-line bg-surface p-4 font-mono text-[13px] leading-6 text-ink">
{`ctx schema`}
        </pre>
      </Command>

      <Command
        name="benchmark"
        usage={`ctx benchmark [OPTIONS]`}
        summary="Re-run an index pass and print incremental timing."
      >
        <pre className="ctx-scroll overflow-x-auto rounded-lg border border-line bg-surface p-4 font-mono text-[13px] leading-6 text-ink">
{`ctx benchmark`}
        </pre>
      </Command>

      <Command
        name="watch"
        usage={`ctx watch [OPTIONS]`}
        summary="Watch the project and keep the graph in sync."
      >
        <pre className="ctx-scroll overflow-x-auto rounded-lg border border-line bg-surface p-4 font-mono text-[13px] leading-6 text-ink">
{`ctx watch`}
        </pre>
        <P>
          Re-indexes changed files as they are edited. Debounce and on/off are
          controlled by the <Code>[watch]</Code> config section.
        </P>
      </Command>

      <Note>
        Running <Code>ctx</Code> with no arguments prints a quick overview of the
        tool.
      </Note>
    </DocsShell>
  );
}