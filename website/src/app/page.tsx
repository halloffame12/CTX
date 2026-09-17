import type { Metadata } from "next";
import Link from "next/link";
import Navbar from "@/components/Navbar";
import Footer from "@/components/Footer";
import Terminal from "@/components/Terminal";
import CodeGraph from "@/components/CodeGraph";
import Reveal from "@/components/Reveal";
import VideoPlayer from "@/components/VideoPlayer";
import { CodeBlock } from "@/components/Sections";

export const metadata: Metadata = {
  title: "ctx — the local, deterministic code graph for AI coding agents",
  description:
    "ctx indexes a repository into a local, deterministic code graph and answers an agent's questions with real file paths: where a symbol lives, what would break if it changed, and which files a task actually needs. No embeddings, no network, no telemetry.",
  openGraph: {
    title: "ctx — the local, deterministic code graph for AI coding agents",
    description:
      "A local code graph for AI agents: find where a symbol lives, see what would break if you change it, and get the files a task actually needs. No embeddings. Nothing leaves your machine.",
    url: "https://ctx.sumitchauhan.me",
    type: "website",
  },
};

const jsonLd = {
  "@context": "https://schema.org",
  "@type": "SoftwareApplication",
  name: "ctx",
  applicationCategory: "DeveloperApplication",
  operatingSystem: "macOS, Linux, Windows",
  description:
    "A local, deterministic code graph for AI coding agents: symbol search, impact analysis, and ranked context packages over stdio (MCP).",
  url: "https://ctx.sumitchauhan.me",
  installUrl: "https://www.npmjs.com/package/ctxai-cli",
  offers: { "@type": "Offer", price: "0", priceCurrency: "USD" },
  license: "https://github.com/halloffame12/CTX/blob/main/LICENSE",
  author: { "@type": "Person", name: "Sumit Chauhan", url: "https://github.com/halloffame12" },
  softwareVersion: "0.1.6",
  codeRepository: "https://github.com/halloffame12/CTX",
};

const STATS = [
  { value: "206", label: "files indexed", mono: "FILES" },
  { value: "1,721", label: "symbols", mono: "SYMBOLS" },
  { value: "594", label: "dependency edges", mono: "EDGES" },
  { value: "11", label: "MCP tools", mono: "MCP TOOLS" },
  { value: "111", label: "tests passing", mono: "TESTS" },
];

const PROBLEMS = [
  {
    n: "01",
    title: "Hallucinated paths",
    body: "The agent writes imports that look right but don't exist. Plausible fiction, the highest-cost kind of error.",
  },
  {
    n: "02",
    title: "Context flooding",
    body: "It dumps whole directories into the token window to be safe, and the signal drowns in the noise.",
  },
  {
    n: "03",
    title: "Missed ripple effects",
    body: "It changes a symbol and breaks callers it never saw, because nobody told it what imports what.",
  },
];

const PIPELINE = [
  { n: "01", title: "Scan", body: "gitignore-aware crawler walks the tree, hashes every raw file with SHA-256." },
  { n: "02", title: "Parse", body: "Tree-sitter extractors read real syntax — symbols, kinds, signatures, scopes. No regex." },
  { n: "03", title: "Resolve", body: "Imports resolve to real files: relative, dotted, use paths, module-relative. Unresolved becomes external — never guessed." },
  { n: "04", title: "Graph", body: "Files, symbols, and dependency edges land in a transactional SQLite database." },
  { n: "05", title: "Query", body: "Search, impact, and context read the graph and answer with real file paths." },
];

const LANG_DETAILS = [
  {
    name: "TypeScript / JavaScript",
    detail: "functions, methods, classes, interfaces, enums, types, constants, fields",
    note: "import / require / dynamic import() · ./ ../ @/ aliases · index files",
  },
  {
    name: "Python",
    detail: "functions, methods, classes, constants",
    note: "dotted, relative & from-import resolution",
  },
  {
    name: "Rust",
    detail: "fns, methods, structs, traits, impls, enums, constants, modules",
    note: "use / crate:: / super:: / self:: / mod probing",
  },
  {
    name: "Go",
    detail: "functions, methods, structs, interfaces",
    note: "import paths, module-relative resolution",
  },
];

const MCP_TOOLS = [
  "ctx_project",
  "ctx_search",
  "ctx_skeleton",
  "ctx_symbol",
  "ctx_dependencies",
  "ctx_dependents",
  "ctx_impact",
  "ctx_context",
  "ctx_changed",
  "ctx_diff",
  "ctx_stats",
];

const CLI_HELP = [
  { prompt: true, text: "ctx --help" },
  { dim: true, text: "" },
  { text: "local deterministic code graph for AI coding agents", dim: true },
  { dim: true, text: "" },
  { text: "Usage: ctx <command> [OPTIONS]", dim: true },
  { dim: true, text: "" },
  { text: "Commands:" },
  { text: "  init        create .ctx, write config, index the project", dim: true },
  { text: "  search      search the graph for symbols or files", dim: true },
  { text: "  symbol      details about a symbol (definition / references / deps)", dim: true },
  { text: "  deps        what a file imports and what imports it", dim: true },
  { text: "  impact      analyze impact of changing a symbol or file", dim: true },
  { text: "  context     build a relevance-ranked context package for a task", dim: true },
  { text: "  changed     symbols changed in the working tree or between refs", dim: true },
  { text: "  diff        semantic diff of symbols between git refs", dim: true },
  { text: "  skeleton    body-less structural skeleton of a source file", dim: true },
  { text: "  schema      print the SQLite graph schema", dim: true },
  { text: "  benchmark   re-run an index pass with timing", dim: true },
  { text: "  watch       watch the project and keep the graph in sync", dim: true },
  { text: "  mcp         run the Model Context Protocol server (stdio)", dim: true },
  { text: "  doctor      inspect project and report index health", dim: true },
  { text: "  stats       show index statistics (files / symbols / deps)", dim: true },
  { text: "  version     print version information", dim: true },
];

const ARCHITECTURE = [
  { n: "01", name: "Scanner", path: "src/indexing/", body: "gitignore-aware walk, SHA-256 hashing, incremental reindex only on change." },
  { n: "02", name: "Parsers", path: "src/parser/", body: "tree-sitter extractors for TS, JS, Python, Rust, Go — bounded on malformed input." },
  { n: "03", name: "Graph", path: "src/graph/", body: "symbols, dependencies, and cycle-safe impact BFS over the inverted graph." },
  { n: "04", name: "Context", path: "src/context/", body: "deterministic token scoring, IDF damping, hub bonuses. Same query → same answer." },
  { n: "05", name: "Git", path: "src/git/", body: "symbol-level diff through the git binary. Never executes project code." },
  { n: "06", name: "MCP", path: "src/mcp/", body: "JSON-RPC 2.0 over stdio. Eleven read-only tools, no daemon, no port." },
];

export default function Home() {
  return (
    <>
      <script
        type="application/ld+json"
        dangerouslySetInnerHTML={{ __html: JSON.stringify(jsonLd) }}
      />
      <Navbar />
      <main>
        {/* ---------------- HERO ---------------- */}
        <section className="relative overflow-hidden border-b border-line">
          <div className="ctx-rule-grid pointer-events-none absolute inset-0" aria-hidden />
          <div className="ctx-container relative py-16 sm:py-20 lg:py-24">
            <div className="grid items-center gap-12 lg:grid-cols-[minmax(0,7fr)_minmax(0,6fr)] lg:gap-16">
              <div>
                <p className="inline-flex items-center gap-2 rounded border border-line bg-surface px-3 py-1 font-mono text-[11px] text-ink-soft">
                  <span className="size-1.5 rounded-full bg-accent" aria-hidden />
                  local code graph / ai context engine · v0.1.6 · MIT
                </p>
                <h1 className="mt-6 text-[clamp(2.25rem,5vw+0.5rem,4.5rem)] font-bold leading-[1.05] tracking-tight text-ink">
                  AI agents hallucinate code.{" "}
                  <span className="text-accent-deep">ctx</span> gives them the real
                  thing.
                </h1>
                <p className="mt-6 max-w-2xl text-lg leading-8 text-ink-soft sm:text-xl sm:leading-9">
                  ctx indexes your repository into a local, deterministic code
                  graph and answers the questions agents actually ask — where does
                  this symbol live, what would break if I change it, which files
                  does this task need. No embeddings. No API calls. Nothing leaves
                  your machine.
                </p>
                <div className="mt-8 flex flex-col gap-3 sm:flex-row sm:items-center">
                  <Link
                    href="/docs/install"
                    className="inline-flex min-h-11 items-center justify-center rounded-md bg-ink px-6 py-3 text-sm font-semibold text-white transition-colors hover:bg-ink/85 active:scale-[0.98]"
                  >
                    Get started
                  </Link>
                  <a
                    href="https://github.com/halloffame12/CTX"
                    target="_blank"
                    rel="noreferrer"
                    className="inline-flex min-h-11 items-center justify-center rounded-md border border-line-strong bg-surface px-6 py-3 text-sm font-semibold text-ink transition-colors hover:border-ink/30 active:scale-[0.98]"
                  >
                    View on GitHub
                  </a>
                </div>
              </div>

              <div>
                <VideoPlayer
                  src="/Context_Architecture__Eliminating_AI_Hallucinations.mp4"
                  title="Eliminating AI Hallucinations"
                  eyebrow="the film · 5 min"
                  caption="Why agents guess, and how a deterministic code graph stops them — the full ctx architecture."
                />
              </div>
            </div>

            <div className="mt-14 grid gap-6 lg:grid-cols-[minmax(0,5fr)_minmax(0,7fr)] lg:items-center lg:gap-10">
              <div className="min-w-0 space-y-4">
                <p className="font-mono text-[11px] uppercase tracking-widest text-ink-faint">
                  one command · one local graph
                </p>
                <Terminal
                  title="ctx index"
                  lines={[
                    { prompt: true, text: "ctx index" },
                    { dim: true, text: "" },
                    { dim: true, text: "Indexing codebase..." },
                    { dim: true, text: "" },
                    { text: "206 files", accent: true },
                    { text: "1,721 symbols", accent: true },
                    { text: "594 edges", accent: true },
                    { dim: true, text: "" },
                    { text: "index complete", dim: true },
                    { text: "~76 ms · sqlite://.ctx/index.db", dim: true },
                  ]}
                />
              </div>
              <dl className="min-w-0 grid gap-px overflow-hidden rounded-lg border border-line bg-line-strong">
                {STATS.map((s) => (
                  <div
                    key={s.label}
                    className="flex items-baseline justify-between gap-6 bg-paper px-6 py-5"
                  >
                    <dt className="order-2 font-mono text-[11px] uppercase tracking-widest text-ink-faint">
                      {s.mono}
                    </dt>
                    <dd className="order-1 text-2xl font-semibold tracking-tight text-ink sm:text-3xl">
                      {s.value}
                      <span className="sr-only">{s.label}</span>
                    </dd>
                  </div>
                ))}
              </dl>
            </div>
            <p className="mt-3 font-mono text-[11px] text-ink-faint">
              measured on ctx&apos;s own repository ·{" "}
              <Link href="/docs/commands" className="underline decoration-line-strong underline-offset-2 hover:text-ink">
                full command reference
              </Link>
            </p>
          </div>
        </section>

        {/* ---------------- THE PROBLEM ---------------- */}
        <section className="ctx-container py-20 sm:py-28">
          <div className="grid gap-10 lg:grid-cols-[minmax(0,5fr)_minmax(0,7fr)] lg:gap-20">
            <div className="lg:sticky lg:top-24 lg:self-start">
              <p className="font-mono text-[11px] uppercase tracking-widest text-accent-deep">
                01 / the problem
              </p>
              <h2 className="mt-4 text-3xl font-bold tracking-tight text-ink sm:text-4xl lg:text-5xl">
                Agents guess. ctx doesn&apos;t.
              </h2>
            </div>
            <div className="grid gap-px overflow-hidden rounded-lg border border-line bg-line sm:grid-cols-3">
              {PROBLEMS.map((p) => (
                <div key={p.n} className="bg-surface p-6 sm:p-7">
                  <p className="font-mono text-xs text-ink-faint">{p.n}</p>
                  <h3 className="mt-4 text-base font-semibold text-ink">{p.title}</h3>
                  <p className="mt-2 text-sm leading-6 text-ink-soft">{p.body}</p>
                </div>
              ))}
            </div>
          </div>
        </section>

        {/* ---------------- THE SOLUTION --------------- */}
        <section className="border-y border-line bg-surface">
          <div className="ctx-container py-20 sm:py-28">
            <Reveal className="max-w-2xl">
              <p className="font-mono text-[11px] uppercase tracking-widest text-accent-deep">
                02 / the solution
              </p>
              <h2 className="mt-4 text-3xl font-bold tracking-tight text-ink sm:text-4xl">
                A real index. Not a guess.
              </h2>
              <p className="mt-4 text-lg leading-8 text-ink-soft">
                ctx replaces confident estimates with a graph of what actually
                exists. Five steps, zero fabrication.
              </p>
            </Reveal>
            <div className="mt-12 grid gap-px overflow-hidden rounded-lg border border-line bg-line sm:grid-cols-5">
              {PIPELINE.map((s, i) => (
                <Reveal key={s.n} delay={i * 60} className="bg-surface">
                  <div className="flex h-full flex-col p-6">
                    <p className="font-mono text-xs text-accent-deep">{s.n}</p>
                    <h3 className="mt-4 text-base font-semibold text-ink">{s.title}</h3>
                    <p className="mt-2 text-[13px] leading-6 text-ink-soft">{s.body}</p>
                  </div>
                </Reveal>
              ))}
            </div>
          </div>
        </section>

        {/* ---------------- THE CODE GRAPH ---------------- */}
        <section className="ctx-container py-20 sm:py-28">
          <div className="grid items-start gap-10 lg:grid-cols-2 lg:gap-16">
            <div className="lg:sticky lg:top-24">
              <p className="font-mono text-[11px] uppercase tracking-widest text-accent-deep">
                03 / the code graph
              </p>
              <h2 className="mt-4 text-3xl font-bold tracking-tight text-ink sm:text-4xl">
                The map is the product.
              </h2>
              <p className="mt-4 text-lg leading-8 text-ink-soft">
                Every file is a node. Every import, reference, and requirement is
                an edge. Hover a file and the graph shows you exactly what depends
                on it — in both directions, computed from real source.
              </p>
              <ul className="mt-6 space-y-3 text-sm leading-6 text-ink-faint">
                <li>— same rows are byte-identical for the same codebase</li>
                <li>— incremental reindex: only changed files are re-parsed</li>
                <li>— the whole thing is a portable SQLite file</li>
              </ul>
            </div>
            <CodeGraph
              title="graph · files → symbols → dependencies"
              caption="Hover a node — connected edges are real import/reference relationships extracted by tree-sitter."
              nodes={[
                { id: "src/auth/service.ts", label: "auth/service.ts", x: 320, y: 150, target: true },
                { id: "src/api/user.ts", label: "api/user.ts", x: 140, y: 70 },
                { id: "src/auth/token.rs", label: "auth/token.rs", x: 150, y: 300 },
                { id: "src/billing/invoice.ts", label: "billing/invoice.ts", x: 520, y: 70 },
                { id: "src/billing/pay.go", label: "billing/pay.go", x: 540, y: 300 },
                { id: "src/db/index.ts", label: "db/index.ts", x: 560, y: 185 },
              ]}
              edges={[
                { from: "src/api/user.ts", to: "src/auth/service.ts", label: "imports" },
                { from: "src/auth/token.rs", to: "src/auth/service.ts", label: "imports" },
                { from: "src/billing/invoice.ts", to: "src/auth/service.ts", label: "imports" },
                { from: "src/billing/pay.go", to: "src/auth/service.ts", label: "imports" },
                { from: "src/db/index.ts", to: "src/api/user.ts", label: "extends" },
                { from: "src/db/index.ts", to: "src/billing/invoice.ts", label: "imports" },
                { from: "src/billing/pay.go", to: "src/db/index.ts", label: "reads" },
              ]}
            />
          </div>
        </section>

        {/* ---------------- DETERMINISTIC CONTEXT ---------------- */}
        <section className="border-y border-line bg-surface">
          <div className="ctx-container grid items-start gap-12 py-20 sm:py-28 lg:grid-cols-2 lg:gap-16">
            <div className="min-w-0 lg:sticky lg:top-24">
              <p className="font-mono text-[11px] uppercase tracking-widest text-accent-deep">
                04 / deterministic context
              </p>
              <h2 className="mt-4 text-3xl font-bold tracking-tight text-ink sm:text-4xl">
                Same codebase, same task, same answer.
              </h2>
              <p className="mt-4 text-lg leading-8 text-ink-soft">
                Context ranking is name-based and deterministic. Token scoring,
                IDF damping, and hub bonuses pick the files a task needs — and
                every suggestion carries a machine-checkable reason.
              </p>
              <ul className="mt-6 space-y-3 text-sm leading-6 text-ink-faint">
                <li>— no embeddings, no LLM calls, no randomness</li>
                <li>— keyword, hub, recency, path, and git scoring</li>
                <li>— every file explains why it was chosen</li>
              </ul>
            </div>
            <div className="min-w-0 space-y-4">
              <CodeBlock
                title="ctx context “add Google OAuth”"
                code={`$ ctx context "add Google OAuth"
Suggested files:
  src/auth/oauth.ts        (score 0.91)
      + path matches keyword \`auth\`
      + imported by 4 files (hub)
      + modified in working tree
  src/auth/service.ts      (score 0.87)
      + path matches keyword \`auth\`
  src/config.ts            (score 0.72)
      + path matches keyword \`config\`
Context budget: 1,842 / 12,000 tokens`}
              />
              <CodeBlock
                title="reasons are machine-checkable"
                code={`{
  "file": "src/auth/oauth.ts",
  "score": 0.91,
  "reasons": [
    "path matches keyword auth",
    "imported by 4 files (hub)",
    "modified in working tree"
  ]
}`}
              />
            </div>
          </div>
        </section>

        {/* ---------------- IMPACT ANALYSIS ---------------- */}
        <section className="ctx-container py-20 sm:py-28">
          <div className="grid items-start gap-10 lg:grid-cols-2 lg:gap-16">
            <div className="order-2 lg:order-1">
              <CodeGraph
                title="impact · change src/auth/service.ts"
                caption="Cycle-safe BFS over the inverted graph. Direct dependents first, then indirect — grouped, never guessed."
                nodes={[
                  { id: "src/auth/service.ts", label: "auth/service.ts", x: 320, y: 180, target: true },
                  { id: "src/api/user.ts", label: "api/user.ts", x: 130, y: 80 },
                  { id: "src/auth/token.rs", label: "auth/token.rs", x: 130, y: 280 },
                  { id: "src/billing/invoice.ts", label: "billing/invoice.ts", x: 510, y: 80 },
                  { id: "src/billing/pay.go", label: "billing/pay.go", x: 510, y: 280 },
                ]}
                edges={[
                  { from: "src/api/user.ts", to: "src/auth/service.ts", label: "direct" },
                  { from: "src/auth/token.rs", to: "src/auth/service.ts", label: "direct" },
                  { from: "src/billing/invoice.ts", to: "src/auth/service.ts", label: "direct" },
                  { from: "src/billing/pay.go", to: "src/auth/service.ts", label: "direct" },
                ]}
              />
              <p className="mt-3 font-mono text-[11px] text-ink-faint">
                depth arg · default 3 · buckets: direct / indirect / test files / unknown
              </p>
            </div>
            <div className="order-1 lg:order-2 lg:sticky lg:top-24">
              <p className="font-mono text-[11px] uppercase tracking-widest text-accent-deep">
                05 / impact analysis
              </p>
              <h2 className="mt-4 text-3xl font-bold tracking-tight text-ink sm:text-4xl">
                Know the blast radius before you edit.
              </h2>
              <p className="mt-4 text-lg leading-8 text-ink-soft">
                Change a symbol and ctx walks every transitive dependent, in both
                directions, cycle-safe by construction. Direct callers, indirect
                callers, tests — and a visible UNKNOWN bucket for whatever truly
                can&apos;t be resolved.
              </p>
              <div className="mt-6 grid gap-px overflow-hidden rounded-lg border border-line bg-line sm:grid-cols-2">
                {[
                  ["direct dependents", "src/api/user.ts · src/auth/token.rs"],
                  ["indirect", "src/billing/invoice.ts · src/billing/pay.go"],
                  ["test files", "tests/billing.test.ts"],
                  ["unknown", "0 — everything resolved"],
                ].map(([k, v]) => (
                  <div key={k} className="bg-surface p-5">
                    <p className="font-mono text-[11px] uppercase tracking-widest text-ink-faint">{k}</p>
                    <p className="mt-2 font-mono text-[12px] leading-5 text-ink">{v}</p>
                  </div>
                ))}
              </div>
            </div>
          </div>
        </section>

        {/* ---------------- MCP ---------------- */}
        <section className="border-y border-line bg-surface">
          <div className="ctx-container grid items-start gap-12 py-20 sm:py-28 lg:grid-cols-2 lg:gap-16">
            <div className="min-w-0 lg:sticky lg:top-24">
              <p className="font-mono text-[11px] uppercase tracking-widest text-accent-deep">
                06 / MCP over stdio
              </p>
              <h2 className="mt-4 text-3xl font-bold tracking-tight text-ink sm:text-4xl">
                Plug into any MCP client.
              </h2>
              <p className="mt-4 text-lg leading-8 text-ink-soft">
                A Model Context Protocol server that speaks JSON-RPC 2.0 over
                standard I/O. Wire it into Claude Desktop, Cursor, opencode, VS
                Code — anything that speaks MCP. No daemon, no port, no source
                leaving your machine.
              </p>
              <div className="mt-6 flex flex-wrap gap-2">
                {MCP_TOOLS.map((t) => (
                  <span
                    key={t}
                    className="rounded border border-line bg-paper px-2.5 py-1 font-mono text-[11px] text-ink-soft"
                  >
                    {t}
                  </span>
                ))}
              </div>
            </div>
            <div className="min-w-0 space-y-4">
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
              <CodeBlock
                title="without a global install"
                code={`{
  "mcpServers": {
    "ctx": {
      "command": "npx",
      "args": ["-y", "ctxai-cli", "mcp", "-R", "/path/to/project"]
    }
  }
}`}
              />
              <p className="text-sm leading-6 text-ink-faint">
                Eleven read-only tools: project, search, skeleton, symbol,
                dependencies, dependents, impact, context, changed, diff, stats.
                Every path an agent receives comes from the graph.
              </p>
            </div>
          </div>
        </section>

        {/* ---------------- CLI ---------------- */}
        <section className="ctx-container py-20 sm:py-28">
          <div className="grid items-start gap-10 lg:grid-cols-[minmax(0,5fr)_minmax(0,7fr)] lg:gap-16">
            <div className="lg:sticky lg:top-24">
              <p className="font-mono text-[11px] uppercase tracking-widest text-accent-deep">
                07 / the CLI
              </p>
              <h2 className="mt-4 text-3xl font-bold tracking-tight text-ink sm:text-4xl">
                Sixteen commands. One graph.
              </h2>
              <p className="mt-4 text-lg leading-8 text-ink-soft">
                Every question an agent asks maps to a command. No submenus, no
                hidden state — `ctx &lt;command&gt; --help` is the whole story.
              </p>
            </div>
            <div className="min-w-0">
              <Terminal
                title="ctx --help"
                startDelay={200}
                charMs={6}
                lines={CLI_HELP}
              />
            </div>
          </div>
        </section>

        {/* ---------------- ARCHITECTURE ---------------- */}
        <section className="border-y border-line bg-surface">
          <div className="ctx-container py-20 sm:py-28">
            <Reveal className="max-w-2xl">
              <p className="font-mono text-[11px] uppercase tracking-widest text-accent-deep">
                08 / architecture
              </p>
              <h2 className="mt-4 text-3xl font-bold tracking-tight text-ink sm:text-4xl">
                Six modules. Zero mystery.
              </h2>
              <p className="mt-4 text-lg leading-8 text-ink-soft">
                The whole system is open source and readable. These are the
                actual modules in the repository.
              </p>
            </Reveal>
            <div className="mt-12 grid gap-px overflow-hidden rounded-lg border border-line bg-line md:grid-cols-3">
              {ARCHITECTURE.map((a, i) => (
                <Reveal key={a.n} delay={i * 50} className="bg-surface">
                  <div className="flex h-full flex-col p-6">
                    <div className="flex items-baseline justify-between gap-4">
                      <p className="font-mono text-xs text-accent-deep">{a.n}</p>
                      <code className="font-mono text-[11px] text-ink-faint">{a.path}</code>
                    </div>
                    <h3 className="mt-4 text-base font-semibold text-ink">{a.name}</h3>
                    <p className="mt-2 text-[13px] leading-6 text-ink-soft">{a.body}</p>
                  </div>
                </Reveal>
              ))}
            </div>
          </div>
        </section>

        {/* ---------------- LOCAL FIRST ---------------- */}
        <section className="ctx-container py-20 sm:py-28">
          <div className="grid gap-10 lg:grid-cols-[minmax(0,5fr)_minmax(0,7fr)] lg:gap-20">
            <div className="lg:sticky lg:top-24 lg:self-start">
              <p className="font-mono text-[11px] uppercase tracking-widest text-accent-deep">
                09 / local first
              </p>
              <h2 className="mt-4 text-3xl font-bold tracking-tight text-ink sm:text-4xl">
                Your code goes where agents can read it. Nowhere else.
              </h2>
            </div>
            <div className="grid gap-px overflow-hidden rounded-lg border border-line bg-line sm:grid-cols-3">
              {[
                ["No network", "the binary opens no connections, anywhere"],
                ["No telemetry", "nothing is reported, ever"],
                ["No embeddings", "no model download, no API key"],
                ["No exec", "project code is never executed"],
                ["Local index", "one SQLite file in .ctx/"],
                ["Portable", "the .ctx folder is a self-contained graph"],
              ].map(([k, v]) => (
                <div key={k} className="bg-surface p-6">
                  <p className="text-base font-semibold text-ink">{k}</p>
                  <p className="mt-2 text-[13px] leading-6 text-ink-soft">{v}</p>
                </div>
              ))}
            </div>
          </div>
        </section>

        {/* ---------------- LANGUAGES ---------------- */}
        <section className="border-y border-line bg-surface">
          <div className="ctx-container py-20 sm:py-28">
            <div className="mb-12 grid gap-6 lg:grid-cols-[minmax(0,5fr)_minmax(0,7fr)]">
              <p className="font-mono text-[11px] uppercase tracking-widest text-accent-deep">
                10 / supported languages
              </p>
              <div>
                <h2 className="text-3xl font-bold tracking-tight text-ink sm:text-4xl">
                  Five parsers. Real syntax trees.
                </h2>
                <p className="mt-4 text-lg leading-8 text-ink-soft">
                  Tree-sitter, not regex. Everything else is invisible to the
                  index by design.
                </p>
              </div>
            </div>
            <div className="grid gap-4 md:grid-cols-2">
              {LANG_DETAILS.map((l) => (
                <div
                  key={l.name}
                  className="rounded-lg border border-line bg-surface p-6 transition-colors hover:border-ink/25"
                >
                  <h3 className="mb-1.5 text-base font-semibold text-ink">{l.name}</h3>
                  <p className="text-sm text-ink-soft">{l.detail}</p>
                  <p className="mt-2 font-mono text-xs text-ink-faint">{l.note}</p>
                </div>
              ))}
            </div>
          </div>
        </section>

        {/* ---------------- GET STARTED ---------------- */}
        <section className="ctx-container py-20 sm:py-28">
          <div className="mx-auto max-w-3xl">
            <p className="text-center font-mono text-[11px] uppercase tracking-widest text-accent-deep">
              11 / get started
            </p>
            <h2 className="mt-4 text-center text-3xl font-bold tracking-tight text-ink sm:text-4xl">
              Up in three commands.
            </h2>
            <div className="mt-10 space-y-4">
              <CodeBlock
                title="1 · install"
                code={`npm install -g ctxai-cli
ctx --version   # → 0.1.6`}
              />
              <CodeBlock
                title="2 · index"
                code={`cd /path/to/project
ctx init        # create .ctx, write config, index
ctx doctor      # verify the index is healthy`}
              />
              <CodeBlock
                title="3 · ask"
                code={`ctx context "what does the payment module do?"
ctx impact BillingService.charge
ctx mcp         # expose the code graph to your agent`}
              />
            </div>
            <p className="mt-8 text-center text-sm leading-6 text-ink-faint">
              Prefer from source?{" "}
              <code className="rounded bg-paper px-1.5 py-0.5 font-mono text-[0.85em] text-ink">cargo install</code>{" "}
              is available for devs, plus per-platform binaries on every release.
            </p>
          </div>
        </section>
      </main>
      <Footer />
    </>
  );
}