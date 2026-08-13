import type { Metadata } from "next";
import { H1, P, DocsShell, H2 } from "@/components/Sections";

export const metadata: Metadata = {
  title: "How it works",
  description:
    "How ctx builds its code graph: the SQLite index, tree-sitter parsing, dependency resolution, and ranking.",
};

export default function ArchitecturePage() {
  return (
    <DocsShell>
      <H1>How it works</H1>
      <P>
        ctx is three pieces: an incremental indexer, a resolver, and a ranker.
        Everything runs locally and everything is deterministic — the same
        repository and the same queries produce the same answers.
      </P>

      <H2>The index</H2>
      <P>
        <span className="font-mono text-sm text-ink">ctx init</span> walks the
        project and writes a SQLite database into{" "}
        <span className="font-mono text-sm text-ink">.ctx/</span>. The database
        holds files, symbols (kind, name, range, signature), and dependency
        edges between files. Parsing is done with tree-sitter grammars — not
        regex — so definitions, scopes, and boundaries are exact.
      </P>
      <P>
        Re-indexing is incremental and content-hash based. Files that have not
        changed are left untouched; only changed files are re-parsed and their
        edges rebuilt, transactionally. A typical repository re-indexes in
        tens of milliseconds.
      </P>

      <H2>Dependency resolution</H2>
      <P>
        ctx resolves imports to real files per language. Relative and alias
        imports in TypeScript/JavaScript, dotted and from-imports in Python,
        <span className="font-mono text-sm text-ink"> use</span> paths in Rust,
        and module-relative paths in Go. Anything it cannot resolve is recorded
        as an external/unknown edge rather than a guess — so impact analysis
        never silently assumes a false dependency.
      </P>

      <H2>Impact analysis</H2>
      <P>
        <span className="font-mono text-sm text-ink">ctx impact</span> runs a
        cycle-safe breadth-first search over the inverted graph starting from a
        symbol or file. It groups results into direct dependents, indirect
        dependents, test files, and unknown (unresolved) edges, with depth
        clamped to keep traversals bounded.
      </P>

      <H2>Context ranking</H2>
      <P>
        <span className="font-mono text-sm text-ink">ctx context</span> scores
        files against a task description using five signals: keyword match on
        path and symbol names, hub centrality (how many files depend on this
        one), recency, path depth, and working-tree git activity. The combined
        score is normalized and files are selected up to a token budget — and
        every selection carries the reason it was chosen, so the output is
        explainable rather than a black box.
      </P>

      <H2>Git awareness</H2>
      <P>
        <span className="font-mono text-sm text-ink">ctx changed</span> and{" "}
        <span className="font-mono text-sm text-ink">ctx diff</span> compare
        symbol sets between the working tree, a ref, or two refs — reporting
        which definitions were added, removed, or modified, not just which files
        changed status.
      </P>

      <H2>Storage & privacy</H2>
      <P>
        The index lives entirely inside{" "}
        <span className="font-mono text-sm text-ink">.ctx/</span> in your
        repository. ctx makes no network connections, sends no telemetry, and
        never executes project code. What it reads stays on your machine.
      </P>
    </DocsShell>
  );
}