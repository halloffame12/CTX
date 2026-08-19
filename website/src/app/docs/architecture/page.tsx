import type { Metadata } from "next";
import { H1, P, DocsShell, H2, Code, Ul } from "@/components/Sections";

export const metadata: Metadata = {
  title: "How it works",
  description:
    "How ctx builds its code graph: the SQLite index, tree-sitter parsing, per-language dependency resolution, and the ranking signals.",
};

export default function ArchitecturePage() {
  return (
    <DocsShell>
      <H1>How it works</H1>
      <P>
        ctx is three pieces: an incremental indexer, a per-language resolver, and
        a ranker. Everything runs locally and everything is deterministic — the
        same repository and the same queries produce the same answers.
      </P>

      <H2 id="index">The index</H2>
      <P>
        <Code>ctx init</Code> walks the project and writes a SQLite database into{" "}
        <Code>.ctx/</Code>. The database holds files, symbols (kind, name, range,
        signature), and dependency edges between files. Parsing is done with
        tree-sitter grammars — not regex — so definitions, scopes, and
        boundaries are exact.
      </P>
      <P>
        Re-indexing is incremental and content-hash based. Files that have not
        changed are left untouched; only changed files are re-parsed and their
        edges rebuilt, transactionally. The scanner skips the directories in{" "}
        <Code>[index].exclude</Code>, files over 2 MB, and symlinks.
      </P>

      <H2 id="resolution">Dependency resolution</H2>
      <P>
        ctx resolves imports to real files per language. Anything it cannot
        resolve is recorded as an external/unknown edge rather than a guess — so
        impact analysis never silently assumes a false dependency.
      </P>
      <CommandTable
        rows={[
          { lang: "TypeScript / JavaScript", detail: "Functions, methods, classes, interfaces, enums, types, constants, fields", imports: "import / require / dynamic import() · ./ ../ @/ aliases · index files · workspace packages" },
          { lang: "Python", detail: "Functions, methods, classes, constants", imports: "Dotted, relative & from-import resolution" },
          { lang: "Rust", detail: "Fns, methods, structs, traits, impls, enums, constants, modules", imports: "use / crate:: / self:: / super:: / mod probing" },
          { lang: "Go", detail: "Functions, methods, structs, interfaces", imports: "Import paths, module-relative resolution" },
        ]}
      />

      <H2 id="impact">Impact analysis</H2>
      <P>
        <Code>ctx impact</Code> runs a cycle-safe breadth-first search over the
        inverted graph starting from a symbol or file. It groups results into
        direct dependents, indirect dependents, test files, and unknown
        (unresolved) edges. Depth defaults to 3 and is bounded. When a name
        exists as both a production symbol and a test double, the production
        definition is preferred.
      </P>

      <H2 id="ranking">Context ranking</H2>
      <P>
        <Code>ctx context</Code> scores files against a task description. The
        signals, honestly stated:
      </P>
      <Ul>
        <li>
          <strong>Keyword match</strong> on symbol names, signatures, and paths,
          weighted by an inverse-document-frequency term so common words matter
          less. A small synonym vocabulary maps common task words.
        </li>
        <li>
          <strong>Hub centrality</strong> — files many others depend on get a
          bonus, but dependents of hubs are capped so they do not flood the
          package.
        </li>
        <li>
          <strong>Recency and path depth</strong> — surface-level files and
          recently modified files score slightly higher.
        </li>
        <li>
          <strong>Git activity</strong> — files changed in the working tree get a
          bump unless <Code>--no-git</Code> is passed.
        </li>
      </Ul>
      <P>
        Files are selected up to a token budget (default 12,000, ≤ 25 files), and
        every selection carries the reason it was chosen, so the output is
        explainable rather than a black box.
      </P>

      <H2 id="git">Git awareness</H2>
      <P>
        <Code>ctx changed</Code> and <Code>ctx diff</Code> compare symbol sets
        between the working tree, a ref, or two refs — reporting which
        definitions were added, removed, or modified, not just which files
        changed status. <Code>ctx diff</Code> uses merge-base resolution so a
        single ref compares your branch against where it forked.
      </P>

      <H2 id="privacy">Storage & privacy</H2>
      <P>
        The index lives entirely inside <Code>.ctx/</Code> in your repository.
        ctx makes no network connections, sends no telemetry, and never executes
        project code. What it reads stays on your machine.
      </P>
    </DocsShell>
  );
}

function CommandTable({
  rows,
}: {
  rows: { lang: string; detail: string; imports: string }[];
}) {
  return (
    <div className="my-6 overflow-x-auto rounded-lg border border-line">
      <table className="w-full border-collapse text-sm">
        <thead>
          <tr className="border-b border-line bg-paper text-left">
            <th className="px-4 py-2.5 font-mono text-xs font-semibold text-ink">Language</th>
            <th className="px-4 py-2.5 text-xs font-semibold text-ink">What is indexed</th>
            <th className="px-4 py-2.5 text-xs font-semibold text-ink">Import resolution</th>
          </tr>
        </thead>
        <tbody>
          {rows.map((r) => (
            <tr key={r.lang} className="border-b border-line last:border-0">
              <td className="px-4 py-2.5 align-top font-semibold text-ink">{r.lang}</td>
              <td className="px-4 py-2.5 align-top leading-6 text-ink-soft">{r.detail}</td>
              <td className="px-4 py-2.5 align-top leading-6 text-ink-soft">{r.imports}</td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}