import TerminalDemo from "@/components/TerminalDemo";
import Navbar from "@/components/Navbar";
import { CodeBlock, Eyebrow, FeatureCard, Section, SectionTitle } from "@/components/Sections";

const CMDS: { title: string; code: string }[] = [
  {
    title: "context",
    code: `# relevance-ranked package for a task
ctx context "add Google OAuth authentication"

Suggested files:
  src/auth/oauth.ts  (score 0.91, ~180 tokens)
      + path matches keyword \`auth\`
      + imported by 4 files (hub)
      + modified in working tree`,
  },
  {
    title: "impact",
    code: `# who would break if I change this?
ctx impact UserService.updateUser --depth 5 --json

Direct dependents     2 files, 4 symbols
Indirect dependents   1 file,  2 symbols
Tests                 1 file
UNKNOWN               1 unresolved import in affected files`,
  },
  {
    title: "skeleton",
    code: `# body-less structure of a file
ctx skeleton src/models.py

def create_user(name: str, **kwargs) -> User
  ...
def find_by_email(email: str) -> User | None
  ...
class User(BaseModel)
  fields: id, email, name, created_at
  methods: save, delete, to_dict`,
  },
  {
    title: "diff",
    code: `# symbol diff between git refs
ctx diff HEAD~3 HEAD

ADDED    UserService.updatePhoto  src/user/service.rs
REMOVED  create_avatar           src/user/avatar.rs
MODIFIED UserService.updateName  src/user/service.rs`,
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

const STATS = [
  { value: "52", label: "files indexed" },
  { value: "468", label: "symbols" },
  { value: "359", label: "dependency edges" },
  { value: "24 ms", label: "incremental re-index" },
];

export default function Home() {
  return (
    <>
      <Navbar />
      <main id="top">
        {/* ---------------- HERO ---------------- */}
        <section className="relative overflow-hidden px-4 pb-16 pt-14 sm:px-6 sm:pt-20">
          {/* backdrop glows */}
          <div
            aria-hidden
            className="pointer-events-none absolute inset-0 -z-10"
          >
            <div className="absolute left-1/2 top-[-220px] h-[420px] w-[720px] -translate-x-1/2 rounded-full bg-accent-500/15 blur-[120px]" />
            <div className="absolute right-[-120px] top-40 h-[260px] w-[260px] rounded-full bg-sky-500/10 blur-[100px]" />
            <div
              className="absolute inset-0 opacity-[0.35]"
              style={{
                backgroundImage:
                  "linear-gradient(rgba(148,163,184,0.06) 1px, transparent 1px), linear-gradient(90deg, rgba(148,163,184,0.06) 1px, transparent 1px)",
                backgroundSize: "44px 44px",
                maskImage: "radial-gradient(ellipse 90% 60% at 50% 0%, black, transparent)",
                WebkitMaskImage: "radial-gradient(ellipse 90% 60% at 50% 0%, black, transparent)",
              }}
            />
          </div>

          <div className="mx-auto grid max-w-6xl items-center gap-12 lg:grid-cols-[1.05fr_0.95fr]">
            <div>
              <div className="ctx-fade-up inline-flex items-center gap-2 rounded-full border border-accent-500/30 bg-accent-500/10 px-3.5 py-1.5 text-xs font-medium text-accent-300 sm:text-sm">
                <span className="relative flex size-2">
                  <span className="absolute inline-flex size-full animate-ping rounded-full bg-emerald-400 opacity-60" />
                  <span className="relative inline-flex size-2 rounded-full bg-emerald-400" />
                </span>
                Local · Private · Offline · Deterministic
              </div>

              <h1 className="ctx-fade-up ctx-fade-up-1 mt-6 text-4xl font-extrabold leading-[1.05] tracking-tight text-white sm:text-6xl">
                Codebase intelligence
                <br />
                <span className="ctx-gradient-text">for AI coding agents.</span>
              </h1>

              <p className="ctx-fade-up ctx-fade-up-2 mt-6 max-w-xl text-base leading-7 text-slate-400 sm:text-lg">
                <span className="font-mono text-accent-300">ctx</span> indexes your
                codebase into a searchable graph and produces relevance-ranked
                context packages for agents — the right files, the first time, in a
                fraction of the tokens.
              </p>

              <div className="ctx-fade-up ctx-fade-up-3 mt-8 flex flex-col gap-3 sm:flex-row sm:items-center">
                <a
                  href="#install"
                  className="inline-flex items-center justify-center gap-2 rounded-xl bg-gradient-to-r from-accent-400 to-sky-500 px-6 py-3 text-sm font-semibold text-ink-950 transition-transform hover:scale-[1.02]"
                >
                  <svg className="size-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.2" strokeLinecap="round" strokeLinejoin="round" aria-hidden>
                    <path d="M12 3v10m0 0 3.5-3.5M12 13 8.5 9.5" />
                    <path d="M4 17v2a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2v-2" />
                  </svg>
                  Install
                </a>
                <a
                  href="#features"
                  className="inline-flex items-center justify-center gap-2 rounded-xl border border-white/15 bg-white/5 px-6 py-3 text-sm font-semibold text-white transition-colors hover:border-white/30 hover:bg-white/10"
                >
                  Explore docs
                </a>
              </div>

              <p className="ctx-fade-up ctx-fade-up-4 mt-8 font-mono text-xs text-slate-600">
                # ctx context &quot;add Google Auth&quot; → the 4 files that matter
              </p>
            </div>

            <div className="ctx-fade-up ctx-fade-up-2">
              <TerminalDemo />
            </div>
          </div>
        </section>

        {/* ---------------- STATS BAND ---------------- */}
        <section className="border-y border-white/10 bg-ink-900/40">
          <div className="mx-auto grid max-w-6xl grid-cols-2 gap-6 px-4 py-8 sm:px-6 md:grid-cols-4">
            {STATS.map((s) => (
              <div key={s.label} className="text-center">
                <div className="font-mono text-2xl font-bold text-white sm:text-3xl">
                  {s.value}
                </div>
                <div className="mt-1 text-xs text-slate-500 sm:text-sm">{s.label}</div>
              </div>
            ))}
          </div>
          <p className="mx-auto max-w-6xl px-4 pb-6 text-center font-mono text-[11px] text-slate-600 sm:px-6">
            measured on this repository · 52 supported files · 24 ms incremental
            re-index
          </p>
        </section>

        {/* ---------------- WHY ---------------- */}
        <Section id="why">
          <div className="grid gap-12 lg:grid-cols-2 lg:items-center">
            <div>
              <Eyebrow>Why ctx?</Eyebrow>
              <SectionTitle>
                Agents read the <span className="ctx-gradient-text">right code</span>,
                not every line.
              </SectionTitle>
              <p className="mt-4 text-base leading-7 text-slate-400">
                Coding agents get the codebase wrong in predictable ways: they
                hallucinate file paths, dump whole directories into context, miss
                ripple effects, and burn the window on noise. <span className="font-mono text-accent-300">ctx</span> fixes
                that at the source.
              </p>
              <ul className="mt-6 space-y-3 text-sm text-slate-300">
                {[
                  "No hallucinated paths — paths come from a real indexed graph.",
                  "Skeletons over dumps: structure & signatures, not 12,000 lines.",
                  "Impact computed by BFS, not guessing.",
                  "Explainable ranking — every selected file tells you why.",
                ].map((t) => (
                  <li key={t} className="flex gap-3">
                    <span className="mt-0.5 text-accent-400">✓</span>
                    {t}
                  </li>
                ))}
              </ul>
            </div>
            <div className="space-y-4">
              <div className="flex items-center gap-2">
                <span className="rounded-md bg-red-400/10 px-2.5 py-1 font-mono text-[11px] uppercase tracking-wide text-red-300">
                  Without ctx
                </span>
                <span className="font-mono text-xs text-slate-600">≈ 12,000 lines · ~100k tokens</span>
              </div>
              <CodeBlock
                title="ai/human reading dump"
                code={`# everything. at once.
cat src/ -R  ->  300 files
12,000 lines  ->  ~100,000 tokens
...and the agent still can't find user_factory
(because nobody imported it directly)`}
              />
              <div className="mt-2 flex items-center gap-2">
                <span className="rounded-md bg-emerald-400/10 px-2.5 py-1 font-mono text-[11px] uppercase tracking-wide text-emerald-300">
                  With ctx
                </span>
                <span className="font-mono text-xs text-slate-600">4 files · ~2,000 tokens</span>
              </div>
              <CodeBlock
                title="ctx context &quot;where's create_user&quot;"
                code={`Relevant architecture:      src/
Relevant symbols:           create_user  src/db/users.py:41
Suggested files:            src/db/users.py
                            src/models/user.py
                            src/api/users.py
Context budget: 1,942 / 8,000 tokens (estimate)`}
              />
            </div>
          </div>
        </Section>

        {/* ---------------- FEATURES ---------------- */}
        <Section id="features" className="bg-ink-900/30">
          <div className="mb-12 text-center">
            <Eyebrow>Features</Eyebrow>
            <SectionTitle>Everything an agent needs to understand a codebase</SectionTitle>
          </div>
          <div className="grid gap-5 sm:grid-cols-2 lg:grid-cols-3">
            <FeatureCard
              icon={<IconIndex />}
              title="Incremental code graph"
            >
              A SQLite graph of files, symbols and dependency edges, updated only
              where files changed — content-hash based, transactional.
            </FeatureCard>
            <FeatureCard
              icon={<IconSearch />}
              title="Search that finds things"
            >
              Symbol and file search with deterministic ranking — substring, prefix
              and kind-aware matching.
            </FeatureCard>
            <FeatureCard
              icon={<IconGraph />}
              title="Honest dependencies"
            >
              Relative imports, Python dotted modules, Rust <span className="font-mono">use</span> and Go
              imports resolve to real files — or are marked external, never guessed.
            </FeatureCard>
            <FeatureCard
              icon={<IconImpact />}
              title="Impact analysis"
            >
              Cycle-safe BFS over the inverted graph, grouped into direct,
              indirect, test-file and UNKNOWN buckets.
            </FeatureCard>
            <FeatureCard
              icon={<IconSkeleton />}
              title="Skeletons over dumps"
            >
              Structural context — signatures, types, exports — without bodies, so
              an agent sees the shape of the codebase in a fraction of the tokens.
            </FeatureCard>
            <FeatureCard
              icon={<IconContext />}
              title="Explainable ranking"
            >
              Keyword + hub + recency + path + git scoring picks the files that
              matter — and tells you <em>why</em>, every time.
            </FeatureCard>
            <FeatureCard
              icon={<IconGit />}
              title="Git-aware changes"
            >
              Symbol-level diffs between refs, not just file status — know exactly
              which definitions were added, removed or modified.
            </FeatureCard>
            <FeatureCard
              icon={<IconWatch />}
              title="Watch & re-index"
            >
              Debounced incremental re-indexing on filesystem events. Keep the
              graph in sync while you edit.
            </FeatureCard>
            <FeatureCard
              icon={<IconMCP />}
              title="MCP over stdio"
            >
              A Model Context Protocol server, ready for Claude, Cursor, Copilot,
              opencode and any MCP client. No daemon, no port.
            </FeatureCard>
          </div>
        </Section>

        {/* ---------------- LANGUAGES ---------------- */}
        <Section id="languages">
          <div className="mb-12 text-center">
            <Eyebrow>Supported languages</Eyebrow>
            <SectionTitle>Parsed with tree-sitter, not regex</SectionTitle>
          </div>
          <div className="grid gap-5 md:grid-cols-2">
            {LANGS.map((l) => (
              <div
                key={l.name}
                className="rounded-2xl border border-white/10 bg-ink-900/60 p-6"
              >
                <div className="mb-2 flex items-center gap-3">
                  <span className="grid size-9 place-items-center rounded-lg border border-white/10 bg-white/5 font-mono text-sm font-bold text-accent-300">
                    {l.name.match(/^\S/)?.[0]}
                  </span>
                  <h3 className="text-base font-semibold text-white">{l.name}</h3>
                </div>
                <p className="text-sm text-slate-400">{l.detail}</p>
                <p className="mt-3 font-mono text-xs text-slate-500">{l.note}</p>
              </div>
            ))}
          </div>
        </Section>

        {/* ---------------- COMMANDS ---------------- */}
        <Section id="commands" className="bg-ink-900/30">
          <div className="mb-12 text-center">
            <Eyebrow>Commands</Eyebrow>
            <SectionTitle>One tool, every question</SectionTitle>
          </div>
          <div className="grid gap-5 md:grid-cols-2">
            {CMDS.map((c) => (
              <CodeBlock key={c.title} title={c.title} code={c.code} />
            ))}
          </div>
        </Section>

        {/* ---------------- MCP ---------------- */}
        <Section id="mcp">
          <div className="grid gap-12 lg:grid-cols-2 lg:items-center">
            <div className="order-2 space-y-5 lg:order-1">
              <CodeBlock
                title="opencode / Claude"
                code={`{
  "mcpServers": {
    "ctx": {
      "command": "ctx",
      "args": ["mcp", "-R", "/path/to/project"]
    }
  }
}`}
              />
              <ul className="space-y-2 text-sm text-slate-400">
                <li>· JSON-RPC 2.0 over stdio, one message per line</li>
                <li>· 10 tools: search, symbol, skeleton, deps, impact, context, changed, diff, …</li>
                <li>· all logs on stderr — stdout is protocol only</li>
                <li>· tool failures surface as <span className="font-mono">isError: true</span></li>
              </ul>
            </div>
            <div className="order-1 lg:order-2">
              <Eyebrow>MCP server</Eyebrow>
              <SectionTitle>
                Plug <span className="ctx-gradient-text">ctx</span> into any coding agent
              </SectionTitle>
              <p className="mt-4 text-base leading-7 text-slate-400">
                Run <span className="font-mono text-accent-300">ctx mcp</span> once and
                your agent gets first-class access to the code graph — ranked context,
                impact analysis and symbol-level diffs, without ever sending source
                code over the network.
              </p>
            </div>
          </div>
        </Section>

        {/* ---------------- INSTALL / CTA ---------------- */}
        <Section id="install" className="bg-ink-900/30">
          <div className="mx-auto max-w-3xl text-center">
            <Eyebrow>Installation</Eyebrow>
            <SectionTitle>Up and running in one command</SectionTitle>
            <div className="mt-8 text-left">
              <CodeBlock
                title="cargo / from source"
                code={`cargo install --path . --locked   # requires Rust 1.85+

# or grab a prebuilt binary from GitHub Releases
# linux x86_64 · linux arm64 · macos · windows

ctx init
ctx doctor        # verify the index is healthy & current
ctx context "add Google OAuth"
ctx mcp           # expose to your agent`}
              />
            </div>
            <div className="mt-8 flex flex-col items-center justify-center gap-3 sm:flex-row">
              <a
                href="https://github.com/halloffame12/CTX"
                target="_blank"
                rel="noreferrer"
                className="inline-flex w-full items-center justify-center gap-2 rounded-xl bg-gradient-to-r from-accent-400 to-sky-500 px-6 py-3 text-sm font-semibold text-ink-950 transition-transform hover:scale-[1.02] sm:w-auto"
              >
                View on GitHub
              </a>
              <a
                href="https://github.com/halloffame12/CTX/releases"
                target="_blank"
                rel="noreferrer"
                className="inline-flex w-full items-center justify-center gap-2 rounded-xl border border-white/15 bg-white/5 px-6 py-3 text-sm font-semibold text-white transition-colors hover:border-white/30 hover:bg-white/10 sm:w-auto"
              >
                Download binaries
              </a>
            </div>
            <p className="mt-8 font-mono text-xs leading-6 text-slate-600">
              MIT licensed · no telemetry · no network · never executes project code
            </p>
          </div>
        </Section>
      </main>

      <footer className="border-t border-white/10 px-4 py-10 sm:px-6">
        <div className="mx-auto flex max-w-6xl flex-col items-center justify-between gap-4 text-sm text-slate-500 sm:flex-row">
          <div className="flex items-center gap-2 font-mono">
            <span className="grid size-6 place-items-center rounded bg-gradient-to-br from-accent-400 to-sky-500 text-xs font-bold text-ink-950">
              &gt;_
            </span>
            <span>ctx</span>
          </div>
          <p className="font-mono text-xs">
            <a
              href="https://github.com/halloffame12/CTX"
              target="_blank"
              rel="noreferrer"
              className="transition-colors hover:text-slate-300"
            >
              github.com/halloffame12/CTX
            </a>
            {" · "}MIT Licensed
          </p>
        </div>
      </footer>
    </>
  );
}

/* ---- icon set (inline, stroke-based) ---- */
function IconIndex() {
  return (
    <svg className="size-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.8" strokeLinecap="round" strokeLinejoin="round" aria-hidden>
      <path d="M4 5h16M4 12h16M4 19h16M9 5v14" />
    </svg>
  );
}
function IconSearch() {
  return (
    <svg className="size-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.8" strokeLinecap="round" strokeLinejoin="round" aria-hidden>
      <circle cx="11" cy="11" r="7" />
      <path d="m20 20-3.5-3.5" />
    </svg>
  );
}
function IconGraph() {
  return (
    <svg className="size-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.8" strokeLinecap="round" strokeLinejoin="round" aria-hidden>
      <circle cx="5" cy="6" r="2" />
      <circle cx="19" cy="6" r="2" />
      <circle cx="12" cy="18" r="2" />
      <path d="M7 6h12M5 8v8a2 2 0 0 0 2 2h3M19 8v3a2 2 0 0 1-2 2h-3" />
    </svg>
  );
}
function IconImpact() {
  return (
    <svg className="size-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.8" strokeLinecap="round" strokeLinejoin="round" aria-hidden>
      <circle cx="18" cy="18" r="3" />
      <path d="M5 8 14 17M13 17l1 1M8 3a4 4 0 1 0 3 7M8 3a4 4 0 0 1 3 7" />
    </svg>
  );
}
function IconSkeleton() {
  return (
    <svg className="size-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.8" strokeLinecap="round" strokeLinejoin="round" aria-hidden>
      <path d="M4 6h16M4 10h10M4 14h16M4 18h7" />
    </svg>
  );
}
function IconContext() {
  return (
    <svg className="size-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.8" strokeLinecap="round" strokeLinejoin="round" aria-hidden>
      <path d="M3 7a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2v10a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2Z" />
      <path d="m4 8 8 5 8-5" />
    </svg>
  );
}
function IconGit() {
  return (
    <svg className="size-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.8" strokeLinecap="round" strokeLinejoin="round" aria-hidden>
      <circle cx="6" cy="6" r="2.5" />
      <circle cx="6" cy="18" r="2.5" />
      <circle cx="18" cy="6" r="2.5" />
      <path d="M6 8.5v7M8.5 6H15a3 3 0 0 1 2.6 1.5l.9 1.5" />
    </svg>
  );
}
function IconWatch() {
  return (
    <svg className="size-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.8" strokeLinecap="round" strokeLinejoin="round" aria-hidden>
      <circle cx="12" cy="12" r="8" />
      <path d="M12 8v4l3 2" />
    </svg>
  );
}
function IconMCP() {
  return (
    <svg className="size-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.8" strokeLinecap="round" strokeLinejoin="round" aria-hidden>
      <rect x="2.5" y="7" width="6" height="10" rx="1.5" />
      <rect x="15.5" y="7" width="6" height="10" rx="1.5" />
      <rect x="9" y="4" width="6" height="16" rx="1.5" />
    </svg>
  );
}