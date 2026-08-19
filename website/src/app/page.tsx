import type { Metadata } from "next";
import Link from "next/link";
import Navbar from "@/components/Navbar";
import Footer from "@/components/Footer";
import VideoPlayer from "@/components/VideoPlayer";
import { CodeBlock } from "@/components/Sections";

export const metadata: Metadata = {
  title: "ctx — eliminate AI hallucinations with a deterministic code graph",
  description:
    "ctx gives AI coding agents a local, deterministic code graph over stdio (MCP): real file paths, real dependency edges, zero embeddings, zero telemetry. Watch the 5-minute architecture film.",
  openGraph: {
    title: "ctx — eliminate AI hallucinations with a deterministic code graph",
    description:
      "A local code graph for AI agents: find where a symbol lives, see what would break if you change it, and get the files a task actually needs. No embeddings, no API calls.",
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
  softwareVersion: "0.1.2",
  codeRepository: "https://github.com/halloffame12/CTX",
};

const HERO_STATS = [
  { label: "files indexed", value: "75" },
  { label: "symbols", value: "873" },
  { label: "dependency edges", value: "452" },
  { label: "index time", value: "~90 ms" },
];

const FACTS = [
  { value: "109", label: "tests passing" },
  { value: "4", label: "languages parsed" },
  { value: "11", label: "MCP tools" },
  { value: "0", label: "telemetry · embeddings" },
];

const FEATURES = [
  {
    title: "Incremental code graph",
    body: "A SQLite graph of files, symbols, and dependency edges. Only changed files re-index — content-hash based, transactional.",
    icon: "M4 6h16M4 12h16M4 18h16",
  },
  {
    title: "Symbol & file search",
    body: "Deterministic, rank-ordered search. Substring, prefix, and kind-aware matching with no embeddings and no API calls.",
    icon: "M21 21l-4.3-4.3M17 10a7 7 0 11-14 0 7 7 0 0114 0z",
  },
  {
    title: "Honest dependency resolution",
    body: "Relative imports, Python dotted modules, Rust use paths, and Go imports resolve to real files. Anything unresolved is flagged external — never guessed.",
    icon: "M12 3v18M3 12h18M12 3l3.5 3.5M12 3L8.5 6.5M12 21l3.5-3.5M12 21l-3.5-3.5",
  },
  {
    title: "Impact analysis",
    body: "Cycle-safe breadth-first search over the inverted graph, grouped into direct, indirect, test-file, and unknown buckets.",
    icon: "M13 2L3 14h7l-1 8 10-12h-7l1-8z",
  },
  {
    title: "Skeletons, not dumps",
    body: "Structural context — signatures, types, exports — without bodies. An agent sees the shape of a codebase in a fraction of the tokens.",
    icon: "M4 4h16v16H4zM4 9h16M9 4v5M15 4v5",
  },
  {
    title: "Explainable ranking",
    body: "Keyword, hub, recency, path, and git scoring pick the files that matter. Every selection tells you why it was chosen.",
    icon: "M12 15a3 3 0 100-6 3 3 0 000 6zM2.5 12s3-6 9.5-6 9.5 6 9.5 6-3 6-9.5 6-9.5-6-9.5-6z",
  },
  {
    title: "Git-aware diffs",
    body: "Symbol-level diffs between refs — which definitions were added, removed, or modified — not just file status.",
    icon: "M4 6h9a4 4 0 010 8H4M13 14h3a4 4 0 010 8H4M4 6v16",
  },
  {
    title: "Filesystem watching",
    body: "Debounced incremental re-indexing on file events. The graph stays in sync while you edit.",
    icon: "M12 6V2M12 6a4 4 0 014 4v4M4 12a8 8 0 0116 0v4a4 4 0 01-4 4h-1V14h4",
  },
  {
    title: "MCP over stdio",
    body: "A Model Context Protocol server for Claude, Cursor, opencode, and any MCP client. No daemon, no open port, no data leaving the machine.",
    icon: "M8 9l-4 3 4 3M16 9l4 3-4 3M13 5l-2 14",
  },
];

const LANGS = [
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

const PRIMARY_BTN =
  "inline-flex min-h-11 items-center justify-center rounded-lg bg-ink px-6 py-3 text-sm font-semibold text-white transition-colors hover:bg-ink/90 active:scale-[0.98]";
const SECONDARY_BTN =
  "inline-flex min-h-11 items-center justify-center rounded-lg border border-line bg-surface px-6 py-3 text-sm font-semibold text-ink transition-colors hover:border-ink/30 active:scale-[0.98]";

function SectionHead({
  eyebrow,
  title,
  sub,
}: {
  eyebrow: string;
  title: string;
  sub?: string;
}) {
  return (
    <div className="mb-12 max-w-2xl">
      <p className="font-mono text-xs uppercase tracking-widest text-accent-deep">{eyebrow}</p>
      <h2 className="mt-4 text-3xl font-bold tracking-tight text-ink sm:text-4xl">{title}</h2>
      {sub && <p className="mt-4 text-lg leading-8 text-ink-soft">{sub}</p>}
    </div>
  );
}

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
        <section id="film" className="relative overflow-hidden">
          <div
            className="pointer-events-none absolute inset-0 bg-[radial-gradient(60%_50%_at_50%_0%,rgba(13,148,136,0.12),transparent_65%)]"
            aria-hidden
          />
          <div className="ctx-container relative grid gap-14 py-16 sm:py-24 lg:grid-cols-[minmax(0,11fr)_minmax(0,10fr)] lg:items-center lg:gap-16">
            <div>
              <p className="inline-flex items-center gap-2 rounded-full border border-line bg-surface px-3 py-1 font-mono text-xs text-ink-soft">
                <span className="size-1.5 rounded-full bg-accent" />
                v0.1.2 · open source · MIT · no telemetry
              </p>
              <h1 className="mt-6 text-[clamp(2.25rem,4.5vw+0.5rem,4.25rem)] font-bold leading-[1.06] tracking-tight text-ink">
                AI agents hallucinate code.{" "}
                <span className="text-accent-deep">ctx</span> gives them the real
                thing.
              </h1>
              <p className="mt-6 max-w-xl text-lg leading-8 text-ink-soft sm:text-xl sm:leading-8">
                <span className="font-mono text-accent-deep">ctx</span> indexes
                your repository into a local, deterministic code graph and answers
                the questions agents actually ask — where does this symbol live,
                what would break if I change it, which files does this task need?
                No embeddings. No API calls. Nothing leaves your machine.
              </p>
              <div className="mt-8 flex flex-col gap-3 sm:flex-row sm:items-center">
                <Link href="/docs" className={`${PRIMARY_BTN} w-full sm:w-auto`}>
                  Get started free
                </Link>
                <a
                  href="#film"
                  className={`${SECONDARY_BTN} w-full sm:w-auto`}
                >
                  ▶ Watch the 5-min film
                </a>
                <a
                  href="https://github.com/halloffame12/CTX"
                  target="_blank"
                  rel="noreferrer"
                  className={`${SECONDARY_BTN} w-full sm:w-auto`}
                >
                  View on GitHub
                </a>
              </div>

              <dl className="mt-12 grid max-w-2xl grid-cols-2 gap-px overflow-hidden rounded-lg border border-line bg-line sm:grid-cols-4">
                {HERO_STATS.map((s) => (
                  <div key={s.label} className="bg-paper p-4">
                    <dt className="font-mono text-[11px] text-ink-faint">{s.label}</dt>
                    <dd className="mt-1 text-2xl font-semibold text-ink">{s.value}</dd>
                  </div>
                ))}
              </dl>
              <p className="mt-3 font-mono text-xs text-ink-faint">
                measured on ctx&apos;s own repository
              </p>
            </div>

            <div>
              <p className="mb-3 flex items-center gap-2 font-mono text-xs uppercase tracking-widest text-accent-deep">
                <span className="inline-block size-1.5 rounded-full bg-accent" />
                the film · 5 min
              </p>
              <VideoPlayer
                src="/Context_Architecture__Eliminating_AI_Hallucinations.mp4"
                title="Eliminating AI Hallucinations"
                caption="The ctx architecture — a 5-minute deep dive into why agents guess, and how a deterministic code graph stops them."
                eyebrow="The film · 5 min"
              />
            </div>
          </div>
        </section>

        {/* ---------------- FACT STRIP ---------------- */}
        <section className="border-b border-line bg-surface">
          <div className="ctx-container grid grid-cols-2 gap-px overflow-hidden py-0 sm:grid-cols-4">
            {FACTS.map((f) => (
              <div key={f.label} className="flex flex-col items-center gap-1 py-8 text-center">
                <span className="text-3xl font-bold tracking-tight text-ink">{f.value}</span>
                <span className="font-mono text-xs text-ink-faint">{f.label}</span>
              </div>
            ))}
          </div>
        </section>

        {/* ---------------- PROBLEM ---------------- */}
        <section className="ctx-container py-20 sm:py-28">
          <div className="grid gap-10 lg:grid-cols-[minmax(0,5fr)_minmax(0,7fr)] lg:gap-20">
            <div className="lg:sticky lg:top-24 lg:self-start">
              <p className="font-mono text-xs uppercase tracking-widest text-accent-deep">
                The problem
              </p>
              <h2 className="mt-4 text-3xl font-bold tracking-tight text-ink sm:text-4xl lg:text-5xl">
                Agents guess. ctx doesn&apos;t.
              </h2>
            </div>
            <div className="space-y-5 text-base leading-7 text-ink-soft sm:text-lg sm:leading-8">
              <p>
                Coding agents fail at codebase navigation in predictable ways.
                They hallucinate file paths that look plausible but don&apos;t
                exist. They dump entire directories into context and burn the
                token window on noise. They miss ripple effects because nobody
                told them what imports what.
              </p>
              <p>
                These failures have one root cause: the agent is guessing. It
                doesn&apos;t have a map.
              </p>
              <p>
                ctx fixes that at the source. Every path it returns comes from a
                real indexed graph, so there is nothing to hallucinate. Skeletons
                carry structure without bodies, impact analysis runs a real
                traversal of the dependency graph, and each ranked file explains
                why it was selected — so an agent spends its context on actual
                code, not on confident fiction.
              </p>
            </div>
          </div>
        </section>

        {/* ---------------- FEATURES ---------------- */}
        <section className="border-y border-line bg-surface">
          <div className="ctx-container py-20 sm:py-28">
            <div className="mb-12 flex flex-col gap-4 sm:flex-row sm:items-end sm:justify-between">
              <SectionHead
                eyebrow="What it does"
                title="The map, built once, queried forever"
                sub="Nine capabilities. Zero embeddings."
              />
            </div>
            <div className="grid gap-px overflow-hidden rounded-xl border border-line bg-line sm:grid-cols-2 lg:grid-cols-3">
              {FEATURES.map((f) => (
                <div
                  key={f.title}
                  className="group bg-surface p-7 transition-colors hover:bg-paper/70"
                >
                  <div className="mb-4 grid size-10 place-items-center rounded-lg border border-line bg-paper">
                    <svg
                      className="size-5 text-accent-deep"
                      viewBox="0 0 24 24"
                      fill="none"
                      stroke="currentColor"
                      strokeWidth="1.8"
                      strokeLinecap="round"
                      strokeLinejoin="round"
                      aria-hidden
                    >
                      <path d={f.icon} />
                    </svg>
                  </div>
                  <h3 className="mb-2 text-base font-semibold text-ink">{f.title}</h3>
                  <p className="text-sm leading-6 text-ink-soft">{f.body}</p>
                </div>
              ))}
            </div>
          </div>
        </section>

        {/* ---------------- SEE IT RUN ---------------- */}
        <section className="ctx-container py-20 sm:py-28">
          <SectionHead
            eyebrow="See it run"
            title="One tool, every question"
            sub="A command-line tool and an MCP server — the same graph answers symbol lookups, dependency questions, impact analysis, and task context."
          />
          <div className="grid items-start gap-10 lg:grid-cols-2">
            <div className="min-w-0 space-y-5">
              <CodeBlock
                title="ctx context"
                code={`$ ctx context "add Google OAuth"
Suggested files:
  src/auth/oauth.ts   (score 0.91)
      + path matches keyword \`auth\`
      + imported by 4 files (hub)
      + modified in working tree
Context budget: 1,842 / 12,000 tokens`}
              />
              <CodeBlock
                title="ctx impact"
                code={`$ ctx impact UserService.updateUser --depth 5
Direct dependents    2 files, 4 symbols
Indirect dependents  1 file,  2 symbols
Tests                1 file
UNKNOWN              1 unresolved import`}
              />
              <CodeBlock
                title="ctx diff"
                code={`$ ctx diff HEAD~3 HEAD
ADDED    UserService.updatePhoto  src/user/service.rs
REMOVED  create_avatar           src/user/avatar.rs
MODIFIED UserService.updateName  src/user/service.rs`}
              />
            </div>
            <div className="min-w-0 space-y-5">
              <VideoPlayer
                src="/s_screen_style_tutorial_Spl.mp4"
                title="From install to first context package"
                caption="Install, index, and query — the full loop."
                eyebrow="Tutorial"
              />
              <CodeBlock
                title="ctx skeleton"
                code={`$ ctx skeleton src/models.py
def create_user(name: str, **kwargs) -> User
  ...
def find_by_email(email: str) -> User | None
  ...
class User(BaseModel)
  fields: id, email, name, created_at`}
              />
              <p className="text-right">
                <Link
                  href="/docs/commands"
                  className="text-sm font-semibold text-accent-deep hover:underline"
                >
                  Full command reference →
                </Link>
              </p>
            </div>
          </div>
        </section>

        {/* ---------------- LANGUAGES ---------------- */}
        <section className="border-y border-line bg-surface">
          <div className="ctx-container py-20 sm:py-28">
            <SectionHead
              eyebrow="Supported languages"
              title="Parsed with tree-sitter, not regex"
            />
            <div className="grid gap-4 md:grid-cols-2">
              {LANGS.map((l) => (
                <div
                  key={l.name}
                  className="rounded-xl border border-line bg-surface p-6 transition-colors hover:border-ink/20"
                >
                  <h3 className="mb-1.5 text-base font-semibold text-ink">{l.name}</h3>
                  <p className="text-sm text-ink-soft">{l.detail}</p>
                  <p className="mt-2 font-mono text-xs text-ink-faint">{l.note}</p>
                </div>
              ))}
            </div>
          </div>
        </section>

        {/* ---------------- INSTALL ---------------- */}
        <section id="install" className="scroll-mt-20">
          <div className="ctx-container py-20 sm:py-28">
            <div className="grid gap-12 lg:grid-cols-[minmax(0,5fr)_minmax(0,7fr)] lg:gap-16">
              <div>
                <SectionHead
                  eyebrow="Install"
                  title="One binary, every package manager"
                  sub="The npm build ships without a Rust toolchain. Homebrew, Scoop, and Winget manifests are maintained in the repository."
                />
                <div className="mt-8">
                  <video
                    className="aspect-video w-full overflow-hidden rounded-xl border border-line bg-ink shadow-sm"
                    autoPlay
                    muted
                    loop
                    playsInline
                    preload="auto"
                    aria-label="ctx product cinematic"
                  >
                    <source src="/A_second_cinematic_product.mp4" type="video/mp4" />
                  </video>
                </div>
              </div>
              <div className="min-w-0 space-y-4">
                <CodeBlock
                  title="npm — no Rust toolchain needed"
                  code={`npm install -g ctxai-cli
ctx --version`}
                />
                <CodeBlock
                  title="cargo — from source"
                  code={`cargo install ctxai-cli --locked
ctx --version`}
                />
                <CodeBlock
                  title="binaries — every GitHub release"
                  code={`curl -LO https://github.com/halloffame12/CTX/releases/download/v0.1.2/ctx-linux-x86_64
chmod +x ctx-linux-x86_64
sudo mv ctx-linux-x86_64 /usr/local/bin/ctx`}
                />
              </div>
            </div>
          </div>
        </section>

        {/* ---------------- MCP ---------------- */}
        <section className="border-y border-line bg-surface">
          <div className="ctx-container grid items-start gap-12 py-20 sm:py-28 lg:grid-cols-2 lg:gap-16">
            <div className="min-w-0">
              <SectionHead
                eyebrow="MCP over stdio"
                title="Plug into any MCP client"
                sub="Wire ctx into Claude Desktop, Cursor, opencode, VS Code, or anything that speaks MCP — no daemon, no port, no source leaving your machine."
              />
              <p className="mt-5 leading-7 text-ink-soft">
                The server exposes eleven read-only tools:{" "}
                <span className="font-mono text-sm text-ink">ctx_project</span>,{" "}
                <span className="font-mono text-sm text-ink">ctx_search</span>,{" "}
                <span className="font-mono text-sm text-ink">ctx_skeleton</span>,{" "}
                <span className="font-mono text-sm text-ink">ctx_symbol</span>,{" "}
                <span className="font-mono text-sm text-ink">ctx_dependencies</span>,{" "}
                <span className="font-mono text-sm text-ink">ctx_dependents</span>,{" "}
                <span className="font-mono text-sm text-ink">ctx_impact</span>,{" "}
                <span className="font-mono text-sm text-ink">ctx_context</span>,{" "}
                <span className="font-mono text-sm text-ink">ctx_changed</span>,{" "}
                <span className="font-mono text-sm text-ink">ctx_diff</span>, and{" "}
                <span className="font-mono text-sm text-ink">ctx_stats</span>.
              </p>
              <p className="mt-4 leading-7 text-ink-soft">
                Every path an agent receives comes from the graph. Nothing it
                reads is invented.
              </p>
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
            </div>
          </div>
        </section>

        {/* ---------------- CLOSING ---------------- */}
        <section className="ctx-container py-20 sm:py-28">
          <div className="mx-auto max-w-3xl text-center">
            <p className="font-mono text-xs uppercase tracking-widest text-accent-deep">
              Get started
            </p>
            <h2 className="mt-4 text-3xl font-bold tracking-tight text-ink sm:text-4xl">
              Stop guessing. Give your agent a map.
            </h2>
            <div className="mx-auto mt-8 max-w-md">
              <video
                className="aspect-video w-full overflow-hidden rounded-xl border border-line bg-ink shadow-sm"
                autoPlay
                muted
                loop
                playsInline
                preload="auto"
                aria-label="ctx release teaser"
              >
                <source src="/s_punchy_launch_teaser_Black.mp4" type="video/mp4" />
              </video>
            </div>
            <div className="mx-auto mt-8 max-w-md text-left">
              <CodeBlock
                title="get started"
                code={`ctx init
ctx doctor        # verify the index is healthy
ctx context "what does the payment module do?"
ctx mcp           # expose the code graph to your agent`}
              />
            </div>
            <div className="mt-8 flex flex-col items-center justify-center gap-3 sm:flex-row">
              <Link href="/docs" className={`${PRIMARY_BTN} w-full sm:w-auto`}>
                Read the documentation
              </Link>
              <a
                href="https://github.com/halloffame12/CTX/releases"
                target="_blank"
                rel="noreferrer"
                className={`${SECONDARY_BTN} w-full sm:w-auto`}
              >
                Download binaries
              </a>
            </div>
          </div>
        </section>
      </main>
      <Footer />
    </>
  );
}