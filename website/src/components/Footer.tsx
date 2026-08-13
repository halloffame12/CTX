import Link from "next/link";

export default function Footer() {
  return (
    <footer className="border-t border-line px-4 py-10 sm:px-6">
      <div className="mx-auto flex max-w-5xl flex-col items-center justify-between gap-4 text-sm text-ink-faint sm:flex-row">
        <div className="flex items-center gap-2 font-mono">
          <span className="grid size-6 place-items-center rounded bg-accent text-xs font-bold text-white">
            &gt;_
          </span>
          <span>ctx</span>
        </div>
        <div className="flex flex-wrap items-center justify-center gap-x-5 gap-y-2">
          <Link href="/docs" className="transition-colors hover:text-ink">
            Documentation
          </Link>
          <a
            href="https://github.com/halloffame12/CTX"
            target="_blank"
            rel="noreferrer"
            className="transition-colors hover:text-ink"
          >
            GitHub
          </a>
          <a
            href="https://github.com/halloffame12/CTX/releases"
            target="_blank"
            rel="noreferrer"
            className="transition-colors hover:text-ink"
          >
            Releases
          </a>
        </div>
        <p className="font-mono text-xs">MIT licensed · no telemetry</p>
      </div>
    </footer>
  );
}