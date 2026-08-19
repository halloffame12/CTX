import type { Metadata } from "next";
import { H1, P, DocsShell, H2, Code, Note } from "@/components/Sections";

export const metadata: Metadata = {
  title: "Configuration",
  description:
    "The .ctx/config.toml file that ctx writes and reads: exclude lists, file size limit, context budget, and watch settings.",
};

export default function ConfigPage() {
  return (
    <DocsShell>
      <H1>Configuration</H1>
      <P>
        <Code>ctx init</Code> writes a default{" "}
        <Code>.ctx/config.toml</Code> at your project root. Editing it changes
        how the index is built and how <Code>ctx context</Code> behaves. If the
        file is missing, ctx uses the same defaults shown here.
      </P>

      <H2 id="default-config">The default config</H2>
      <P>This is exactly what <Code>ctx init</Code> writes:</P>
      <pre className="ctx-scroll overflow-x-auto rounded-lg border border-line bg-surface p-4 font-mono text-[13px] leading-6 text-ink">
{`# ctx configuration
[index]
exclude = [
    "node_modules",
    "target",
    ".git",
    ".ctx",
    "dist",
    "build",
    ".cache",
    "vendor",
    "coverage",
    "__pycache__",
    ".next",
    ".nuxt",
    ".venv",
    "venv",
    ".venvs",
    "env",
    ".env",
    "Pods",
    ".output",
]

[context]
max_tokens = 12000
max_files = 25
include_bodies = false

[watch]
enabled = true
debounce_ms = 200`}
      </pre>

      <H2 id="index">[index] — what gets scanned</H2>
      <P>
        <Code>exclude</Code> is a list of directory names that are skipped during
        scanning. Add any generated or vendored directory here so it is not
        parsed. Directories are matched by name at any depth.
      </P>
      <P>
        Two other limits are baked in and not currently configurable via TOML: a
        file larger than <Code>2 MB</Code> is skipped (oversized files are not
        upserted), and symlinks are not followed.
      </P>

      <H2 id="context">[context] — the context budget</H2>
      <P>
        These control <Code>ctx context</Code>:
      </P>
      <ul className="my-4 space-y-2 pl-5 leading-7 text-ink-soft [&>li]:list-disc">
        <li>
          <Code>max_tokens = 12000</Code> — the token budget for the suggested
          context package. Overridable per run with{" "}
          <Code>ctx context --max-tokens N</Code>.
        </li>
        <li>
          <Code>max_files = 25</Code> — the maximum number of files included.
        </li>
        <li>
          <Code>include_bodies = false</Code> — whether files are emitted as full
          source or as skeletons. Overridable per run with{" "}
          <Code>--include-bodies</Code>.
        </li>
      </ul>

      <H2 id="watch">[watch] — filesystem watching</H2>
      <P>
        <Code>ctx watch</Code> listens for file events and re-indexes changed
        files. <Code>enabled = true</Code> turns the watcher on, and{" "}
        <Code>debounce_ms = 200</Code> is how long a burst of edits is coalesced
        before re-indexing.
      </P>

      <Note>
        <Code>.ctx/</Code> is automatically added to your{" "}
        <Code>.gitignore</Code> on <Code>ctx init</Code> (if you are in a git
        repo) so the index is never committed.
      </Note>
    </DocsShell>
  );
}