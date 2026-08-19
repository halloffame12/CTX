import type { Metadata } from "next";
import Link from "next/link";
import Navbar from "@/components/Navbar";
import Footer from "@/components/Footer";
import { CodeBlock } from "@/components/Sections";

export const metadata: Metadata = {
  title: "ctx — codebase context for AI coding agents",
  description:
    "A local code graph for AI agents: find where a symbol lives, see what would break if you change it, and get the files a task actually needs. Runs on stdio, keeps source local.",
  openGraph: {
    title: "ctx — codebase context for AI coding agents",
    description:
      "A local code graph for AI agents: find where a symbol lives, see what would break if you change it, and get the files a task actually needs.",
    url: "https://ctx.sumitchauhan.me",
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

const STATS = [
  { label: "files", value: "75" },
  { label: "symbols", value: "873" },
  { label: "edges", value: "452" },
  { label: "index time", value: "~90 ms" },
];

const FEATURES = [
  {
    title: "Incremental code graph",
    body: "A SQLite graph of files, symbols, and dependency edges. Only changed files re-index — content-hash based, transactional.",
  },
  {
    title: "Symbol & file search",
    body: "Deterministic, rank-ordered search. Substring, prefix, and kind-aware matching with no embeddings and no API calls.",
  },
  {
    title: "Honest dependency resolution",
    body: "Relative imports, Python dotted modules, Rust use paths, and Go imports resolve to real files. Anything unresolved is flagged external — never guessed.",
  },
  {
    title: "Impact analysis",
    body: "Cycle-safe breadth-first search over the inverted graph, grouped into direct, indirect, test-file, and unknown buckets.",
  },
  {
    title: "Skeletons, not dumps",
    body: "Structural context — signatures, types, exports — without bodies. An agent sees the shape of a codebase in a fraction of the tokens.",
  },
  {
    title: "Explainable ranking",
    body: "Keyword, hub, recency, path, and git scoring pick the files that matter. Every selection tells you why it was chosen.",
  },
  {
    title: "Git-aware diffs",
    body: "Symbol-level diffs between refs — which definitions were added, removed, or modified — not just file status.",
  },
  {
    title: "Filesystem watching",
    body: "Debounced incremental re-indexing on file events. The graph stays in sync while you edit.",
  },
  {
    title: "MCP over stdio",
    body: "A Model Context Protocol server for Claude, Cursor, opencode, and any MCP client. No daemon, no open port, no data leaving the machine.",
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
        <section className="ctx-container pt-12 pb-16 sm:pt-24 sm:pb-20">
          <div className="max-w-4xl">
            <h1 className="text-[clamp(2rem,4.5vw+0.5rem,5rem)] font-bold leading-[1.08] tracking-tight text-ink">
              Codebase context for AI coding agents
            </h1>
            <p className="mt-6 max-w-2xl text-lg leading-8 text-ink-soft sm:text-xl sm:leading-8">
              <span className="font-mono text-accent-deep">ctx</span> indexes a
              repository into a local, deterministic code graph and answers the
              questions agents actually ask: where does this symbol live, what
              would break if I change it, and which files does this task need?
            </p>

            <div className="mt-8 flex flex-col gap-3 sm:flex-row sm:items-center">
              <Link href="/docs" className={`${PRIMARY_BTN} w-full sm:w-auto`}>
                Read the docs
              </Link>
              <a
                href="https://github.com/halloffame12/CTX"
                target="_blank"
                rel="noreferrer"
                className={`${SECONDARY_BTN} w-full sm:w-auto`}
              >
                View on GitHub
              </a>
            </div>

            <dl className="mt-10 grid max-w-2xl grid-cols-2 gap-px overflow-hidden rounded-lg border border-line bg-line sm:grid-cols-4">
              {STATS.map((s) => (
                <div key={s.label} className="bg-paper p-4">
                  <dt className="font-mono text-xs text-ink-faint">{s.label}</dt>
                  <dd className="mt-1 text-2xl font-semibold text-ink">{s.value}</dd>
                </div>
              ))}
            </dl>
          </div>

          <div className="mx-auto mt-12 max-w-5xl">
            <video
              className="aspect-video w-full overflow-hidden rounded-lg border border-line bg-ink shadow-sm"
              autoPlay
              muted
              loop
              playsInline
              preload="auto"
              aria-label="Cinematic product reveal for ctx"
            >
              <source src="/A_second_cinematic_product.mp4" type="video/mp4" />
            </video>
            <div className="mt-3 flex flex-wrap items-center justify-between gap-2 font-mono text-xs text-ink-faint">
              <span>ctx — product demo</span>
              <span>v0.1.2 · 10s loop · 1280×720</span>
            </div>
          </div>
        </section>

        {/* ---------------- INSTALL ---------------- */}
        <section id="install" className="scroll-mt-20 border-y border-line bg-surface">
          <div className="ctx-container grid gap-10 py-12 sm:py-14 lg:grid-cols-3">
            <div className="min-w-0 lg:pr-8">
              <h2 className="text-sm font-semibold text-ink">Install</h2>
              <p className="mt-2 max-w-xs text-sm leading-6 text-ink-soft">
                One binary, every package manager. The npm build ships without a
                Rust toolchain.
              </p>
              <p className="mt-4 font-mono text-xs text-ink-faint">
                npm · cargo · homebrew · scoop · winget
              </p>
            </div>
            <div className="min-w-0">
              <CodeBlock
                title="npm — no Rust toolchain needed"
                code={`npm install -g ctxai-cli
ctx --version`}
              />
            </div>
            <div className="min-w-0">
              <CodeBlock
                title="cargo — from source"
                code={`cargo install ctxai-cli --locked
ctx --version`}
              />
            </div>
          </div>
        </section>

        {/* ---------------- WHY ---------------- */}
        <section className="ctx-container py-20 sm:py-28">
          <div className="grid gap-10 lg:grid-cols-[minmax(0,5fr)_minmax(0,7fr)] lg:gap-20">
            <div className="lg:sticky lg:top-24 lg:self-start">
              <p className="font-mono text-xs uppercase tracking-widest text-accent-deep">
                Why ctx
              </p>
              <h2 className="mt-4 text-3xl font-bold tracking-tight text-ink sm:text-4xl lg:text-5xl">
                Agents guess. ctx doesn&apos;t.
              </h2>
            </div>
            <div className="space-y-5 text-base leading-7 text-ink-soft sm:text-lg sm:leading-8">
              <p>
                Coding agents fail at codebase navigation in predictable ways.
                They hallucinate file paths, dump entire directories into
                context, miss ripple effects, and burn the token window on noise.
              </p>
              <p>
                ctx fixes that at the source. Every path it returns comes from a
                real indexed graph, so there is nothing to hallucinate. Skeletons
                carry structure without bodies, impact analysis runs a real
                traversal of the dependency graph, and each ranked file explains
                why it was selected.
              </p>
            </div>
          </div>
        </section>

        {/* ---------------- FEATURES ---------------- */}
        <section className="border-y border-line bg-surface">
          <div className="ctx-container py-20 sm:py-28">
            <div className="mb-12 flex flex-col gap-2 sm:flex-row sm:items-end sm:justify-between">
              <h2 className="text-3xl font-bold tracking-tight text-ink sm:text-4xl">
                What it does
              </h2>
              <p className="font-mono text-xs text-ink-faint">
                nine capabilities · zero embeddings
              </p>
            </div>
            <div className="grid gap-px overflow-hidden rounded-lg border border-line bg-line sm:grid-cols-2 lg:grid-cols-3">
              {FEATURES.map((f) => (
                <div
                  key={f.title}
                  className="bg-surface p-6 transition-colors hover:bg-paper/70"
                >
                  <h3 className="mb-2 text-base font-semibold text-ink">{f.title}</h3>
                  <p className="text-sm leading-6 text-ink-soft">{f.body}</p>
                </div>
              ))}
            </div>
          </div>
        </section>

        {/* ---------------- LANGUAGES ---------------- */}
        <section className="ctx-container py-20 sm:py-28">
          <div className="mb-12 flex flex-col gap-2 sm:flex-row sm:items-end sm:justify-between">
            <h2 className="text-3xl font-bold tracking-tight text-ink sm:text-4xl">
              Supported languages
            </h2>
            <p className="font-mono text-xs text-ink-faint">
              parsed with tree-sitter, not regex
            </p>
          </div>
          <div className="grid gap-4 md:grid-cols-2">
            {LANGS.map((l) => (
              <div
                key={l.name}
                className="rounded-lg border border-line bg-surface p-6 transition-colors hover:border-ink/20"
              >
                <h3 className="mb-1.5 text-base font-semibold text-ink">{l.name}</h3>
                <p className="text-sm text-ink-soft">{l.detail}</p>
                <p className="mt-2 font-mono text-xs text-ink-faint">{l.note}</p>
              </div>
            ))}
          </div>
        </section>

        {/* ---------------- COMMANDS ---------------- */}
        <section className="border-y border-line bg-surface">
          <div className="ctx-container py-20 sm:py-28">
            <div className="mb-12 flex flex-col gap-2 sm:flex-row sm:items-end sm:justify-between">
              <h2 className="text-3xl font-bold tracking-tight text-ink sm:text-4xl">
                Commands
              </h2>
              <p className="font-mono text-xs text-ink-faint">
                one tool, every question
              </p>
            </div>
            <div className="mx-auto mb-12 max-w-2xl">
              <video
                className="aspect-video w-full overflow-hidden rounded-lg border border-line bg-ink shadow-sm"
                autoPlay
                muted
                loop
                playsInline
                preload="metadata"
                aria-label="Tutorial showing ctx install and first command"
              >
                <source src="/s_screen_style_tutorial_Spl.mp4" type="video/mp4" />
              </video>
            </div>
            <div className="grid gap-5 md:grid-cols-2">
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
                title="ctx skeleton"
                code={`$ ctx skeleton src/models.py
def create_user(name: str, **kwargs) -> User
  ...
def find_by_email(email: str) -> User | None
  ...
class User(BaseModel)
  fields: id, email, name, created_at`}
              />
              <CodeBlock
                title="ctx diff"
                code={`$ ctx diff HEAD~3 HEAD
ADDED    UserService.updatePhoto  src/user/service.rs
REMOVED  create_avatar           src/user/avatar.rs
MODIFIED UserService.updateName  src/user/service.rs`}
              />
            </div>
            <p className="mt-10 text-center">
              <Link href="/docs/commands" className="text-sm font-semibold text-accent-deep hover:underline">
                Full command reference →
              </Link>
            </p>
          </div>
        </section>

        {/* ---------------- MCP ---------------- */}
        <section className="ctx-container py-20 sm:py-28">
          <div className="grid items-start gap-10 lg:grid-cols-2">
            <div className="min-w-0">
              <p className="font-mono text-xs uppercase tracking-widest text-accent-deep">
                MCP over stdio
              </p>
              <h2 className="mt-4 text-3xl font-bold tracking-tight text-ink sm:text-4xl">
                Plug into any MCP client
              </h2>
              <p className="mt-5 leading-7 text-ink-soft">
                ctx ships a Model Context Protocol server over stdio. Wire it into
                Claude Desktop, Cursor, opencode, VS Code, or anything that speaks
                MCP — no daemon, no port, no source leaving your machine.
              </p>
              <p className="mt-4 leading-7 text-ink-soft">
                The server exposes eleven tools: <span className="font-mono text-sm text-ink">ctx_project</span>,{" "}
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
        <section className="border-t border-line bg-surface">
          <div className="ctx-container py-20 sm:py-28">
            <div className="mx-auto max-w-3xl text-center">
              <p className="font-mono text-xs uppercase tracking-widest text-accent-deep">
                Get started
              </p>
              <h2 className="mt-4 text-3xl font-bold tracking-tight text-ink sm:text-4xl">
                Try it on your own repository
              </h2>
              <div className="mt-8 text-left">
                <CodeBlock
                  title="get started"
                  code={`ctx init
ctx doctor        # verify the index is healthy
ctx context "what does the payment module do?"
ctx mcp           # expose the code graph to your agent`}
                />
              </div>
              <div className="mx-auto mt-8 max-w-md">
                <video
                  className="aspect-video w-full overflow-hidden rounded-lg border border-line bg-ink shadow-sm"
                  autoPlay
                  muted
                  loop
                  playsInline
                  preload="metadata"
                  aria-label="ctx release teaser"
                >
                  <source src="/s_punchy_launch_teaser_Black.mp4" type="video/mp4" />
                </video>
              </div>
              <div className="mt-8 flex flex-col items-center justify-center gap-3 sm:flex-row">
                <Link href="/docs" className={`${PRIMARY_BTN} w-full sm:w-auto`}>
                  Documentation
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
          </div>
        </section>
      </main>
      <Footer />
    </>
  );
}