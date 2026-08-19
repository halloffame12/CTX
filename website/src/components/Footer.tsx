import Link from "next/link";

export default function Footer() {
  return (
    <footer className="border-t border-line">
      <div className="ctx-container flex flex-col gap-10 py-12 md:flex-row md:items-start md:justify-between">
        <div>
          <div className="flex items-center gap-2 font-mono">
            <span className="grid size-6 place-items-center rounded bg-accent text-xs font-bold text-white">
              &gt;_
            </span>
            <span>ctx</span>
          </div>
          <p className="mt-3 max-w-xs text-sm leading-6 text-ink-faint">
            Codebase context for AI coding agents. MCP over stdio — no daemon, no
            port, no telemetry.
          </p>
        </div>
        <div className="grid grid-cols-2 gap-10 sm:grid-cols-3">
          <div>
            <p className="font-mono text-xs uppercase tracking-widest text-ink-faint">Product</p>
            <ul className="mt-3 space-y-2 text-sm">
              <li><Link href="/docs" className="text-ink-soft transition-colors hover:text-ink">Documentation</Link></li>
              <li><Link href="/#install" className="text-ink-soft transition-colors hover:text-ink">Install</Link></li>
              <li><Link href="/docs/commands" className="text-ink-soft transition-colors hover:text-ink">Commands</Link></li>
            </ul>
          </div>
          <div>
            <p className="font-mono text-xs uppercase tracking-widest text-ink-faint">Links</p>
            <ul className="mt-3 space-y-2 text-sm">
              <li><a href="https://github.com/halloffame12/CTX" target="_blank" rel="noreferrer" className="text-ink-soft transition-colors hover:text-ink">GitHub</a></li>
              <li><a href="https://github.com/halloffame12/CTX/releases" target="_blank" rel="noreferrer" className="text-ink-soft transition-colors hover:text-ink">Releases</a></li>
              <li><a href="https://www.npmjs.com/package/ctxai-cli" target="_blank" rel="noreferrer" className="text-ink-soft transition-colors hover:text-ink">npm</a></li>
            </ul>
          </div>
          <div>
            <p className="font-mono text-xs uppercase tracking-widest text-ink-faint">Status</p>
            <ul className="mt-3 space-y-2 text-sm">
              <li><span className="text-ink-soft">MIT licensed</span></li>
              <li><span className="text-ink-soft">v0.1.2</span></li>
              <li><span className="text-ink-soft">no telemetry</span></li>
            </ul>
          </div>
        </div>
      </div>
    </footer>
  );
}