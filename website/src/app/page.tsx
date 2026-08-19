import type { Metadata } from "next";
import Link from "next/link";
import Navbar from "@/components/Navbar";
import Footer from "@/components/Footer";
import TerminalDemo from "@/components/TerminalDemo";
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
        <section className="px-4 pt-16 pb-12 sm:px-6 sm:pt-20">
          <div className="mx-auto grid max-w-5xl items-center gap-12 lg:grid-cols-[1fr_1fr]">
            <div>
              <h1 className="text-4xl font-bold leading-[1.1] tracking-tight text-ink sm:text-5xl">
                Codebase context for AI coding agents
              </h1>
              <p className="mt-5 max-w-xl text-lg leading-7 text-ink-soft">
                <span className="font-mono text-accent-deep">ctx</span> indexes a
                repository into a local, deterministic code graph and answers the
                questions agents actually ask: where does this symbol live, what
                would break if I change it, and which files does this task need?
              </p>

              <div className="mt-8 flex flex-col gap-3 sm:flex-row sm:items-center">
                <Link
                  href="/docs"
                  className="inline-flex items-center justify-center rounded-lg bg-ink px-5 py-2.5 text-sm font-semibold text-white transition-colors hover:bg-ink/90"
                >
                  Read the docs
                </Link>
                <a
                  href="https://github.com/halloffame12/CTX"
                  target="_blank"
                  rel="noreferrer"
                  className="inline-flex items-center justify-center rounded-lg border border-line bg-surface px-5 py-2.5 text-sm font-semibold text-ink transition-colors hover:border-ink/30"
                >
                  View on GitHub
                </a>
              </div>

              <p className="mt-8 font-mono text-xs text-ink-faint">
                Measured on this project&apos;s own repo: 75 files · 873 symbols · 452 edges indexed in ~90 ms.
              </p>
            </div>

            <div>
              <TerminalDemo />
            </div>
          </div>
        </section>

        {/* ---------------- INSTALL STRIP ---------------- */}
        <section id="install" className="scroll-mt-20 border-y border-line bg-surface px-4 py-8 sm:px-6">
          <div className="mx-auto grid max-w-5xl gap-6 sm:grid-cols-2">
            <div>
              <h2 className="mb-3 text-sm font-semibold text-ink">Install</h2>
              <CodeBlock
                title="npm — no Rust toolchain needed"
                code={`npm install -g ctxai-cli
ctx --version`}
              />
            </div>
            <div>
              <h2 className="mb-3 text-sm font-semibold text-ink">Or via cargo</h2>
              <CodeBlock
                title="cargo — from source"
                code={`cargo install ctxai-cli --locked
ctx --version`}
              />
            </div>
          </div>
        </section>

        {/* ---------------- WHY ---------------- */}
        <section className="px-4 py-16 sm:px-6 sm:py-24">
          <div className="mx-auto max-w-3xl">
            <h2 className="text-3xl font-bold tracking-tight text-ink sm:text-4xl">
              Agents guess. ctx doesn&apos;t.
            </h2>
            <div className="mt-6 space-y-4 text-base leading-7 text-ink-soft">
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
        <section className="border-y border-line bg-surface px-4 py-16 sm:px-6 sm:py-24">
          <div className="mx-auto max-w-5xl">
            <h2 className="mb-12 text-center text-3xl font-bold tracking-tight text-ink sm:text-4xl">
              What it does
            </h2>
            <div className="grid gap-px overflow-hidden rounded-lg border border-line bg-line sm:grid-cols-2 lg:grid-cols-3">
              {FEATURES.map((f) => (
                <div key={f.title} className="bg-surface p-6">
                  <h3 className="mb-2 text-base font-semibold text-ink">{f.title}</h3>
                  <p className="text-sm leading-6 text-ink-soft">{f.body}</p>
                </div>
              ))}
            </div>
          </div>
        </section>

        {/* ---------------- LANGUAGES ---------------- */}
        <section className="px-4 py-16 sm:px-6 sm:py-24">
          <div className="mx-auto max-w-5xl">
            <h2 className="mb-3 text-center text-3xl font-bold tracking-tight text-ink sm:text-4xl">
              Supported languages
            </h2>
            <p className="mb-10 text-center text-ink-soft">
              Parsed with tree-sitter, not regex.
            </p>
            <div className="grid gap-4 md:grid-cols-2">
              {LANGS.map((l) => (
                <div key={l.name} className="rounded-lg border border-line bg-surface p-6">
                  <h3 className="mb-1.5 text-base font-semibold text-ink">{l.name}</h3>
                  <p className="text-sm text-ink-soft">{l.detail}</p>
                  <p className="mt-2 font-mono text-xs text-ink-faint">{l.note}</p>
                </div>
              ))}
            </div>
          </div>
        </section>

        {/* ---------------- COMMANDS ---------------- */}
        <section className="border-y border-line bg-surface px-4 py-16 sm:px-6 sm:py-24">
          <div className="mx-auto max-w-5xl">
            <h2 className="mb-3 text-center text-3xl font-bold tracking-tight text-ink sm:text-4xl">
              Commands
            </h2>
            <p className="mb-10 text-center text-ink-soft">
              One tool, every question.
            </p>
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
            <p className="mt-8 text-center">
              <Link href="/docs/commands" className="text-sm font-semibold text-accent-deep hover:underline">
                Full command reference →
              </Link>
            </p>
          </div>
        </section>

        {/* ---------------- MCP ---------------- */}
        <section className="px-4 py-16 sm:px-6 sm:py-24">
          <div className="mx-auto grid max-w-5xl items-start gap-10 lg:grid-cols-2">
            <div>
              <h2 className="text-3xl font-bold tracking-tight text-ink sm:text-4xl">
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
            <div className="space-y-4">
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
        <section className="border-t border-line bg-surface px-4 py-16 sm:px-6 sm:py-20">
          <div className="mx-auto max-w-3xl text-center">
            <h2 className="text-3xl font-bold tracking-tight text-ink sm:text-4xl">
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
            <div className="mt-8 flex flex-col items-center justify-center gap-3 sm:flex-row">
              <Link
                href="/docs"
                className="inline-flex w-full items-center justify-center rounded-lg bg-ink px-6 py-3 text-sm font-semibold text-white transition-colors hover:bg-ink/90 sm:w-auto"
              >
                Documentation
              </Link>
              <a
                href="https://github.com/halloffame12/CTX/releases"
                target="_blank"
                rel="noreferrer"
                className="inline-flex w-full items-center justify-center rounded-lg border border-line px-6 py-3 text-sm font-semibold text-ink transition-colors hover:border-ink/30 sm:w-auto"
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