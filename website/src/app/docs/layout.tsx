"use client";

import { useState } from "react";
import type { ReactNode } from "react";
import Link from "next/link";
import { usePathname } from "next/navigation";
import Navbar from "@/components/Navbar";
import Footer from "@/components/Footer";
import DocsToc from "@/components/DocsToc";
import { DOC_NAV } from "@/components/docs-nav";

function activeItem(href: string) {
  return DOC_NAV.findIndex((item) => item.href === href);
}

export default function DocsLayout({ children }: { children: ReactNode }) {
  const pathname = usePathname();
  const [open, setOpen] = useState(false);
  const current = activeItem(pathname);
  const prev = current > 0 ? DOC_NAV[current - 1] : null;
  const next = current >= 0 && current < DOC_NAV.length - 1 ? DOC_NAV[current + 1] : null;

  const isActive = (href: string) =>
    href === "/docs" ? pathname === "/docs" : pathname.startsWith(href);

  return (
    <>
      <Navbar />
      <div className="mx-auto flex w-full max-w-6xl gap-8 px-4 py-6 sm:px-6 md:py-10">
        {/* Desktop sidebar */}
        <aside className="hidden w-56 shrink-0 lg:block">
          <nav aria-label="Docs sections" className="sticky top-20 max-h-[calc(100vh-6rem)] overflow-y-auto">
            <p className="mb-3 px-2 font-mono text-xs uppercase tracking-widest text-ink-faint">Docs</p>
            <ul className="space-y-0.5">
              {DOC_NAV.map((item) => (
                <li key={item.href}>
                  <Link
                    href={item.href}
                    className={`block rounded-md px-2 py-1.5 text-sm transition-colors ${
                      isActive(item.href)
                        ? "bg-line/60 font-semibold text-ink"
                        : "text-ink-soft hover:bg-line/40 hover:text-ink"
                    }`}
                  >
                    {item.label}
                  </Link>
                </li>
              ))}
            </ul>
            <a
              href="https://github.com/halloffame12/CTX"
              target="_blank"
              rel="noreferrer"
              className="mt-6 block rounded-md border border-line px-2 py-1.5 text-sm text-ink-soft transition-colors hover:border-ink/30 hover:text-ink"
            >
              View source on GitHub ↗
            </a>
          </nav>
        </aside>

        {/* Mobile nav toggle */}
        <div className="min-w-0 flex-1">
          <button
            type="button"
            onClick={() => setOpen((o) => !o)}
            aria-expanded={open}
            className="mb-4 flex w-full items-center justify-between rounded-lg border border-line bg-surface px-4 py-2.5 text-sm font-medium text-ink lg:hidden"
          >
            <span>{DOC_NAV.find((i) => i.href === pathname)?.label ?? "Docs"}</span>
            <svg
              className={`size-4 transition-transform ${open ? "rotate-180" : ""}`}
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              strokeWidth="2"
              strokeLinecap="round"
              aria-hidden
            >
              <path d="M6 9l6 6 6-6" />
            </svg>
          </button>
          {open && (
            <nav
              aria-label="Docs sections"
              className="mb-6 rounded-lg border border-line bg-surface p-2 lg:hidden"
            >
              <ul className="space-y-0.5">
                {DOC_NAV.map((item) => (
                  <li key={item.href}>
                    <Link
                      href={item.href}
                      onClick={() => setOpen(false)}
                      className={`block rounded-md px-3 py-2 text-sm transition-colors ${
                        isActive(item.href)
                          ? "bg-line/60 font-semibold text-ink"
                          : "text-ink-soft hover:bg-line/40 hover:text-ink"
                      }`}
                    >
                      {item.label}
                    </Link>
                  </li>
                ))}
              </ul>
            </nav>
          )}

          <article data-doc-article className="min-w-0 max-w-3xl">
            {children}
          </article>

          {/* Prev / next */}
          {(prev || next) && (
            <nav
              aria-label="Document navigation"
              className="mt-12 grid gap-3 border-t border-line pt-6 sm:grid-cols-2"
            >
              {prev ? (
                <Link
                  href={prev.href}
                  className="group rounded-lg border border-line bg-surface p-4 transition-colors hover:border-ink/30"
                >
                  <span className="block font-mono text-xs text-ink-faint">← Previous</span>
                  <span className="mt-1 block text-sm font-semibold text-ink">{prev.label}</span>
                </Link>
              ) : (
                <span />
              )}
              {next ? (
                <Link
                  href={next.href}
                  className="group rounded-lg border border-line bg-surface p-4 text-right transition-colors hover:border-ink/30"
                >
                  <span className="block font-mono text-xs text-ink-faint">Next →</span>
                  <span className="mt-1 block text-sm font-semibold text-ink">{next.label}</span>
                </Link>
              ) : null}
            </nav>
          )}
        </div>

        {/* Right TOC */}
        <aside className="hidden w-52 shrink-0 xl:block">
          <DocsToc />
        </aside>
      </div>
      <Footer />
    </>
  );
}