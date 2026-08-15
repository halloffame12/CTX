import type { ReactNode } from "react";
import Link from "next/link";
import Navbar from "@/components/Navbar";
import Footer from "@/components/Footer";

const NAV = [
  { href: "/docs", label: "Overview", match: /^\/docs$/, exact: true },
  { href: "/docs/install", label: "Installation" },
  { href: "/docs/commands", label: "Command reference" },
  { href: "/docs/mcp", label: "MCP server" },
  { href: "/docs/architecture", label: "How it works" },
  { href: "/docs/faq", label: "FAQ" },
];

export default function DocsLayout({ children }: { children: ReactNode }) {
  return (
    <>
      <Navbar />
      <nav
        aria-label="Docs sections"
        className="border-b border-line bg-surface px-4 py-3 md:hidden"
      >
        <div className="ctx-scroll flex gap-2 overflow-x-auto">
          {NAV.map((item) => (
            <Link
              key={item.href}
              href={item.href}
              className="whitespace-nowrap rounded-full border border-line px-3 py-1 text-xs font-medium text-ink-soft transition-colors hover:border-ink/30 hover:text-ink"
            >
              {item.label}
            </Link>
          ))}
        </div>
      </nav>
      <div className="mx-auto flex max-w-5xl gap-10 px-4 py-8 sm:px-6 md:py-12">
        <aside className="hidden w-52 shrink-0 md:block">
          <nav className="sticky top-20 space-y-1">
            <p className="mb-3 px-2 font-mono text-xs uppercase tracking-widest text-ink-faint">
              Docs
            </p>
            {NAV.map((item) => (
              <Link
                key={item.href}
                href={item.href}
                className="block rounded-md px-2 py-1.5 text-sm text-ink-soft transition-colors hover:bg-line/60 hover:text-ink"
              >
                {item.label}
              </Link>
            ))}
          </nav>
        </aside>
        <div className="min-w-0 flex-1">{children}</div>
      </div>
      <Footer />
    </>
  );
}